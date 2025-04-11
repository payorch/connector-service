use crate::{configs::Config, domain_types::generate_payment_sync_response, utils::ForeignTryFrom};
use connector_integration::{
    self as connector_integration_service,
    flow::CreateOrder,
    types::{ConnectorData, PaymentCreateOrderData, PaymentCreateOrderResponse},
};
use external_services;
use grpc_api_types::{
    payments::payment_service_server::PaymentService,
    payments::{
        PaymentsAuthorizeRequest, PaymentsAuthorizeResponse, PaymentsSyncRequest,
        PaymentsSyncResponse,
    },
};
use hyperswitch_domain_models::{
    router_data::{ConnectorAuthType, ErrorResponse},
    router_data_v2::{PaymentFlowData, RouterDataV2},
    router_flow_types::{Authorize, PSync},
    router_request_types::{PaymentsAuthorizeData, PaymentsSyncData},
    router_response_types::PaymentsResponseData,
};
use hyperswitch_interfaces::connector_integration_v2::BoxedConnectorIntegrationV2;
use tracing::info;

pub struct Payments {
    pub config: Config,
}

impl Payments {
    async fn handle_order_creation(
        &self,
        connector_data: ConnectorData,
        payment_flow_data: &mut PaymentFlowData,
        connector_auth_details: ConnectorAuthType,
        payload: &PaymentsAuthorizeRequest,
    ) -> Result<(), tonic::Status> {
        // Get connector integration
        let connector_integration: BoxedConnectorIntegrationV2<
            '_,
            CreateOrder,
            PaymentFlowData,
            PaymentCreateOrderData,
            PaymentCreateOrderResponse,
        > = connector_data.connector.get_connector_integration_v2();

        let currency =
            match hyperswitch_common_enums::Currency::foreign_try_from(payload.currency()) {
                Ok(currency) => currency,
                Err(e) => {
                    return Err(tonic::Status::invalid_argument(format!(
                        "Invalid currency: {}",
                        e
                    )))
                }
            };

        let order_create_data = PaymentCreateOrderData {
            amount: hyperswitch_common_utils::types::MinorUnit::new(payload.minor_amount),
            currency,
        };

        let order_router_data = RouterDataV2::<
            CreateOrder,
            PaymentFlowData,
            PaymentCreateOrderData,
            PaymentCreateOrderResponse,
        > {
            flow: std::marker::PhantomData,
            resource_common_data: payment_flow_data.clone(),
            connector_auth_type: connector_auth_details,
            request: order_create_data,
            response: Err(ErrorResponse::default()),
        };

        // Execute connector processing
        let response = match external_services::service::execute_connector_processing_step(
            &self.config.proxy,
            connector_integration,
            order_router_data,
        )
        .await
        {
            Ok(resp) => resp,
            Err(e) => {
                return Err(tonic::Status::internal(format!(
                    "Connector processing error: {}",
                    e
                )))
            }
        };

        match response.response {
            Ok(PaymentCreateOrderResponse { order_id, .. }) => {
                payment_flow_data.reference_id = Some(order_id);
                Ok(())
            }
            Err(ErrorResponse { message, .. }) => Err(tonic::Status::internal(format!(
                "Order creation error: {}",
                message
            ))),
        }
    }
}

#[tonic::async_trait]
impl PaymentService for Payments {
    async fn payment_authorize(
        &self,
        request: tonic::Request<PaymentsAuthorizeRequest>,
    ) -> Result<tonic::Response<PaymentsAuthorizeResponse>, tonic::Status> {
        info!("PAYMENT_AUTHORIZE_FLOW: initiated");

        let payload = request.into_inner();

        // Convert connector enum from the request
        let connector = match connector_integration_service::types::ConnectorEnum::foreign_try_from(
            payload.connector,
        ) {
            Ok(connector) => connector,
            Err(e) => {
                return Err(tonic::Status::invalid_argument(format!(
                    "Invalid connector: {}",
                    e
                )))
            }
        };

        //get connector data
        let connector_data =
            ConnectorData::get_connector_by_name(&connector);

        // Get connector integration
        let connector_integration: BoxedConnectorIntegrationV2<
            '_,
            Authorize,
            PaymentFlowData,
            PaymentsAuthorizeData,
            PaymentsResponseData,
        > = connector_data.connector.get_connector_integration_v2();

        // Create common request data
        let mut payment_flow_data = match PaymentFlowData::foreign_try_from(payload.clone()) {
            Ok(data) => data,
            Err(e) => {
                return Err(tonic::Status::invalid_argument(format!(
                    "Invalid request data: {}",
                    e
                )))
            }
        };

        // Extract auth credentials
        let auth_creds = payload.auth_creds.clone();

        let auth_creds = match auth_creds {
            Some(auth_creds) => auth_creds,
            None => {
                return Err(tonic::Status::invalid_argument(format!(
                    "Missing auth_creds in request",
                )))
            }
        };

        let connector_auth_details = match ConnectorAuthType::foreign_try_from(auth_creds) {
            Ok(auth_type) => auth_type,
            Err(e) => {
                return Err(tonic::Status::invalid_argument(format!(
                    "Invalid auth_creds in request: {}",
                    e
                )))
            }
        };

        let should_do_order_create = connector_data.connector.should_do_order_create();

        if should_do_order_create {
            self.handle_order_creation(
                connector_data.clone(),
                &mut payment_flow_data,
                connector_auth_details.clone(),
                &payload,
            )
            .await?;
        }

        // Create connector request data
        let payment_authorize_data = match PaymentsAuthorizeData::foreign_try_from(payload.clone())
        {
            Ok(data) => data,
            Err(e) => {
                return Err(tonic::Status::invalid_argument(format!(
                    "Invalid request data: {}",
                    e
                )))
            }
        };

        // Construct router data
        let router_data = RouterDataV2::<
            Authorize,
            PaymentFlowData,
            PaymentsAuthorizeData,
            PaymentsResponseData,
        > {
            flow: std::marker::PhantomData,
            resource_common_data: payment_flow_data,
            connector_auth_type: connector_auth_details,
            request: payment_authorize_data,
            response: Err(ErrorResponse::default()),
        };

        // Execute connector processing
        let response = match external_services::service::execute_connector_processing_step(
            &self.config.proxy,
            connector_integration,
            router_data,
        )
        .await
        {
            Ok(resp) => resp,
            Err(e) => {
                return Err(tonic::Status::internal(format!(
                    "Connector processing error: {}",
                    e
                )))
            }
        };

        // Generate response
        let authorize_response =
            match crate::domain_types::generate_payment_authorize_response(response) {
                Ok(resp) => resp,
                Err(e) => {
                    return Err(tonic::Status::internal(format!(
                        "Response generation error: {}",
                        e
                    )))
                }
            };

        Ok(tonic::Response::new(authorize_response))
    }

    async fn payment_sync(
        &self,
        request: tonic::Request<PaymentsSyncRequest>,
    ) -> Result<tonic::Response<PaymentsSyncResponse>, tonic::Status> {
        info!("PAYMENT_SYNC_FLOW: initiated");

        let payload = request.into_inner();

        // Convert connector enum from the request
        let connector = match connector_integration_service::types::ConnectorEnum::foreign_try_from(
            payload.connector,
        ) {
            Ok(connector) => connector,
            Err(e) => {
                return Err(tonic::Status::invalid_argument(format!(
                    "Invalid connector: {}",
                    e
                )))
            }
        };

        // Get connector data
        let connector_data =
            ConnectorData::get_connector_by_name(&connector);

        // Get connector integration
        let connector_integration: BoxedConnectorIntegrationV2<
            '_,
            PSync,
            PaymentFlowData,
            PaymentsSyncData,
            PaymentsResponseData,
        > = connector_data.connector.get_connector_integration_v2();

        // Extract auth credentials
        let auth_creds = payload.auth_creds.clone();

        // Create connector request data
        let payment_sync_data = match PaymentsSyncData::foreign_try_from(payload.clone()) {
            Ok(data) => data,
            Err(e) => {
                return Err(tonic::Status::invalid_argument(format!(
                    "Invalid request data: {}",
                    e
                )))
            }
        };

        // Create common request data
        let payment_flow_data = match PaymentFlowData::foreign_try_from(payload.clone()) {
            Ok(data) => data,
            Err(e) => {
                return Err(tonic::Status::invalid_argument(format!(
                    "Invalid flow data: {}",
                    e
                )))
            }
        };

        let auth_creds = match auth_creds {
            Some(auth_creds) => auth_creds,
            None => {
                return Err(tonic::Status::invalid_argument(format!(
                    "Missing auth_creds in request",
                )))
            }
        };

        let connector_auth_details = match ConnectorAuthType::foreign_try_from(auth_creds) {
            Ok(auth_type) => auth_type,
            Err(e) => {
                return Err(tonic::Status::invalid_argument(format!(
                    "Invalid auth_creds in request: {}",
                    e
                )))
            }
        };

        // Create router data
        let router_data = RouterDataV2 {
            flow: std::marker::PhantomData,
            resource_common_data: payment_flow_data,
            connector_auth_type: connector_auth_details,
            request: payment_sync_data,
            response: Err(ErrorResponse::default()),
        };

        // Execute connector processing
        let response = match external_services::service::execute_connector_processing_step(
            &self.config.proxy,
            connector_integration,
            router_data,
        )
        .await
        {
            Ok(resp) => resp,
            Err(e) => {
                return Err(tonic::Status::internal(format!(
                    "Connector processing error: {}",
                    e
                )))
            }
        };

        // Generate response
        let sync_response = match generate_payment_sync_response(response) {
            Ok(resp) => resp,
            Err(e) => {
                return Err(tonic::Status::internal(format!(
                    "Response generation error: {}",
                    e
                )))
            }
        };

        Ok(tonic::Response::new(sync_response))
    }
}
