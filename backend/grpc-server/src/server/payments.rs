use crate::{configs::Config, utils::ForeignTryFrom};
use connector_integration as connector_integration_service;
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
    router_flow_types::Authorize,
    router_request_types::PaymentsAuthorizeData,
    router_response_types::PaymentsResponseData,
};
use hyperswitch_interfaces::connector_integration_v2::BoxedConnectorIntegrationV2;
use tracing::info;

pub struct Payments {
    pub config: Config,
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
            connector_integration_service::types::ConnectorData::get_connector_by_name(&connector);

        // Get connector integration
        let connector_integration: BoxedConnectorIntegrationV2<
            '_,
            Authorize,
            PaymentFlowData,
            PaymentsAuthorizeData,
            PaymentsResponseData,
        > = connector_data.connector.get_connector_integration_v2();

        // Extract auth credentials
        let auth_creds = payload.auth_creds.clone();

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

        // Create common request data
        let payment_flow_data = match PaymentFlowData::foreign_try_from(payload.clone()) {
            Ok(data) => data,
            Err(e) => {
                return Err(tonic::Status::invalid_argument(format!(
                    "Invalid request data: {}",
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
            Err(e) => return Err(tonic::Status::invalid_argument(format!(
                "Invalid auth_creds in request: {}",
                e
            )))
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
        let authorize_response = match crate::domain_types::generate_payment_authorize_response(response) {
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
        _request: tonic::Request<PaymentsSyncRequest>,
    ) -> Result<tonic::Response<PaymentsSyncResponse>, tonic::Status> {
        todo!()
    }
}
