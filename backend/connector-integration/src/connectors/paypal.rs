use domain_types::{
    connector_flow::{Authorize, Capture, CreateOrder, PSync, RSync, Refund, Void},
    connector_types::{
        ConnectorServiceTrait, PaymentCreateOrderData, PaymentCreateOrderResponse, PaymentFlowData,
        PaymentVoidData, PaymentsAuthorizeData, PaymentsCaptureData, PaymentsResponseData,
        PaymentsSyncData, RefundFlowData, RefundSyncData, RefundsData, RefundsResponseData,
        PaymentAuthorizeV2, PaymentCapture as PaymentCaptureTrait, PaymentOrderCreate as PaymentOrderCreateTrait, 
        PaymentSyncV2, PaymentVoidV2, RefundSyncV2 as RefundSyncV2Trait, RefundV2 as RefundV2Trait,
        ValidationTrait, IncomingWebhook,
    },
    types::Connectors as DomainConnectors, 
};
use error_stack::{ResultExt, report};
use hyperswitch_api_models::enums;
use hyperswitch_common_utils::{
    errors::CustomResult,
    request::{RequestContent as HyperRequestContent},
    types::{AmountConvertor, StringMajorUnit, StringMajorUnitForConnector},
    fp_utils, 
};
use hyperswitch_domain_models::{
    router_data::{ConnectorAuthType, ErrorResponse as RouterErrorResponse, AccessToken},
    router_data_v2::RouterDataV2,
    router_request_types::{ResponseId},
    router_response_types::{RedirectForm}, 
};
use hyperswitch_interfaces::{
    api::ConnectorCommon,
    connector_integration_v2::ConnectorIntegrationV2,
    consts::{NO_ERROR_CODE, NO_ERROR_MESSAGE},
    errors as connector_errors,
    events::connector_api_logs::ConnectorEvent,
    types::Response,
    configs::Connectors as InterfaceConnectors, // Used in base_url
};
use hyperswitch_masking::{Maskable, PeekInterface, Secret, Mask};
use serde_json;
use uuid;

pub mod transformers;
use self::transformers::{PaypalAuthType, PaypalErrorResponse, PaypalRouterData};

pub(crate) mod headers {
    pub(crate) const CONTENT_TYPE: &str = "Content-Type";
    pub(crate) const AUTHORIZATION: &str = "Authorization";
}

pub struct Paypal {
    amount_converter: &'static (dyn AmountConvertor<Output = StringMajorUnit> + Sync),
}

impl Paypal {
    pub fn new() -> Self {
        Self {
            amount_converter: &StringMajorUnitForConnector,
        }
    }
}

impl ConnectorCommon for Paypal {
    fn id(&self) -> &'static str {
        "paypal"
    }

    fn common_get_content_type(&self) -> &'static str {
        "application/json"
    }

    fn base_url<'a>(&self, connectors: &'a InterfaceConnectors) -> &'a str {
        &connectors.paypal.base_url
    }
}

impl ValidationTrait for Paypal {}
impl ConnectorServiceTrait for Paypal {}
impl PaymentAuthorizeV2 for Paypal {}
impl PaymentSyncV2 for Paypal {}
impl PaymentOrderCreateTrait for Paypal {}
impl PaymentVoidV2 for Paypal {}
impl RefundSyncV2Trait for Paypal {}
impl RefundV2Trait for Paypal {}
impl PaymentCaptureTrait for Paypal {}
impl IncomingWebhook for Paypal {}

impl
    ConnectorIntegrationV2<
        Authorize,
        PaymentFlowData,
        PaymentsAuthorizeData,
        PaymentsResponseData,
    > for Paypal
{
    fn get_headers(
        &self,
        req: &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
    ) -> CustomResult<Vec<(String, Maskable<String>)>, connector_errors::ConnectorError> {
        let mut headers_vec = vec![(
            headers::CONTENT_TYPE.to_string(),
            Self::common_get_content_type(self).to_string().into(),
        )];

        if let Some(access_token_obj) = req.access_token.as_ref() {
            let secret_token: Secret<String> = Secret::new(access_token_obj.clone());
            let token_value: &String = secret_token.peek();
            headers_vec.push((
                headers::AUTHORIZATION.to_string(),
                format!("Bearer {}", token_value).into_masked()
            ));
        } else {
            return Err(report!(connector_errors::ConnectorError::FailedToObtainAuthType)
                .attach_printable("Paypal access token not found for Authorize call"));
        }

        headers_vec.push((
            "Paypal-Request-Id".to_string(),
            uuid::Uuid::new_v4().to_string().into(),
        ));
        Ok(headers_vec)
    }

    fn get_url(
        &self,
        req: &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
    ) -> CustomResult<String, connector_errors::ConnectorError> {
        Ok(format!("{}/v2/checkout/orders", req.resource_common_data.connectors.paypal.base_url))
    }

    fn get_request_body(
        &self,
        req: &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
    ) -> CustomResult<Option<HyperRequestContent>, connector_errors::ConnectorError> {
        let paypal_req = transformers::PaypalPaymentRequest::try_from(&req.request)
            .change_context(connector_errors::ConnectorError::RequestEncodingFailed)?;
        Ok(Some(HyperRequestContent::Json(Box::new(paypal_req))))
    }

    fn handle_response_v2(
        &self,
        data: &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
        _event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<
        RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
        connector_errors::ConnectorError,
    > {
        let paypal_res: transformers::PaypalPaymentResponse = serde_json::from_slice(&res.response)
            .change_context(connector_errors::ConnectorError::ResponseDeserializationFailed)?;
        let payments_response = domain_types::connector_types::PaymentsResponseData::try_from(&paypal_res)
            .change_context(connector_errors::ConnectorError::ResponseHandlingFailed)?;
        let mut new_data = data.clone();
        new_data.response = Ok(payments_response);
        Ok(new_data)
    }

    fn get_error_response_v2(
        &self,
        res: Response, 
        _event_builder: Option<&mut ConnectorEvent>
    ) -> CustomResult<RouterErrorResponse, connector_errors::ConnectorError> {
        let error_response: Result<PaypalErrorResponse, _> = serde_json::from_slice(&res.response)
            .change_context(connector_errors::ConnectorError::ResponseDeserializationFailed)
            .attach_printable("Failed to deserialize Paypal error response");

        match error_response {
            Ok(paypal_error) => Ok(RouterErrorResponse {
                code: paypal_error.name, 
                message: paypal_error.message,
                reason: paypal_error.debug_id, 
                status_code: res.status_code,
                attempt_status: None, 
                connector_transaction_id: None,
            }),
            Err(_e) => {
                 let response_body_str = String::from_utf8_lossy(&res.response);
                 let reason_str = format!("Failed to deserialize specific PaypalErrorResponse. Raw response: {}", response_body_str);
                 Ok(RouterErrorResponse {
                    code: NO_ERROR_CODE.to_string(),
                    message: NO_ERROR_MESSAGE.to_string(),
                    reason: Some(reason_str),
                    status_code: res.status_code,
                    attempt_status: None, 
                    connector_transaction_id: None,
                })
            }
        }
    }

    fn get_5xx_error_response(
        &self,
        res: Response, 
        _event_builder: Option<&mut ConnectorEvent>
    ) -> CustomResult<RouterErrorResponse, connector_errors::ConnectorError> {
        <Self as ConnectorIntegrationV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>>::get_error_response_v2(self, res, _event_builder) 
    }
}

impl ConnectorIntegrationV2<PSync, PaymentFlowData, PaymentsSyncData, PaymentsResponseData> for Paypal {}
impl ConnectorIntegrationV2<CreateOrder, PaymentFlowData, PaymentCreateOrderData, PaymentCreateOrderResponse> for Paypal {}
impl ConnectorIntegrationV2<RSync, RefundFlowData, RefundSyncData, RefundsResponseData> for Paypal {}
impl ConnectorIntegrationV2<Void, PaymentFlowData, PaymentVoidData, PaymentsResponseData> for Paypal {}
impl ConnectorIntegrationV2<Refund, RefundFlowData, RefundsData, RefundsResponseData> for Paypal {}
impl ConnectorIntegrationV2<Capture, PaymentFlowData, PaymentsCaptureData, PaymentsResponseData> for Paypal {} 