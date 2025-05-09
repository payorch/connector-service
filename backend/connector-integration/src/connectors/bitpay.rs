// backend/connector-integration/src/connectors/bitpay.rs
use domain_types::connector_types::{ConnectorServiceTrait, RefundSyncData};
use hyperswitch_common_utils::{errors::CustomResult, ext_traits::ByteSliceExt, request::RequestContent, types::FloatMajorUnit};
use hyperswitch_domain_models::{router_data::ErrorResponse, router_data_v2::RouterDataV2};
use hyperswitch_interfaces::{
    api::ConnectorCommon, connector_integration_v2::ConnectorIntegrationV2, errors::{self, ConnectorError}, events::connector_api_logs::ConnectorEvent, types::Response
};
use hyperswitch_masking::Maskable;

use domain_types::connector_flow::{Authorize, Capture, CreateOrder, PSync, RSync, Refund, Void};
use domain_types::connector_types::{
    IncomingWebhook, PaymentAuthorizeV2, PaymentCapture, PaymentCreateOrderData,
    PaymentCreateOrderResponse, PaymentFlowData, PaymentOrderCreate, PaymentSyncV2,
    PaymentVoidData, PaymentVoidV2, PaymentsAuthorizeData, PaymentsCaptureData,
    PaymentsResponseData, PaymentsSyncData, RefundFlowData, RefundSyncV2, RefundV2, RefundsData,
    RefundsResponseData, RequestDetails, ValidationTrait, WebhookDetailsResponse, EventType,
    ConnectorWebhookSecrets, RefundWebhookDetailsResponse
};
pub(crate) mod headers {
    pub(crate) const CONTENT_TYPE: &str = "Content-Type";
    pub(crate) const AUTHORIZATION: &str = "Authorization";
}

use hyperswitch_common_utils::{
    types::{AmountConvertor, MinorUnit},
};
use hyperswitch_domain_models::router_data::ConnectorAuthType;


pub struct Bitpay {
    #[allow(dead_code)]
    amount_converter: &'static (dyn AmountConvertor<Output = FloatMajorUnit> + Sync),
}

impl Bitpay {
    pub fn new() -> &'static Self {
        &Self {
            amount_converter: &FloatMajorUnitForConnector,
        }
    }
}

use hyperswitch_common_utils::types::FloatMajorUnitForConnector;
use hyperswitch_interfaces::configs::Connectors;

use crate::{connectors::bitpay, with_error_response_body};
impl ConnectorCommon for Bitpay {
    fn id(&self) -> &'static str {
        "bitpay"
    }
    fn base_url<'a>(&self, connectors: &'a Connectors) -> &'a str {
        connectors.bitpay.base_url.as_ref()
    }

    fn build_error_response(
        &self,
        res: Response,
        event_builder: Option<&mut ConnectorEvent>,
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        let response: bitpay::Bitpay = res
            .response
            .parse_struct("ErrorResponse")
            .map_err(|_| errors::ConnectorError::ResponseDeserializationFailed)?;

        with_error_response_body!(event_builder, response);

        Ok(ErrorResponse {
            status_code: res.status_code,
            code: response.error.code,
            message: response.error.description,
            reason: Some(response.error.reason),
            attempt_status: None,
            connector_transaction_id: None,
        })
    }

}

impl ConnectorServiceTrait for Bitpay {}
impl ValidationTrait for Bitpay {}
impl PaymentAuthorizeV2 for Bitpay {}
impl PaymentSyncV2 for Bitpay {}
impl PaymentOrderCreate for Bitpay {}
impl PaymentVoidV2 for Bitpay {}
impl RefundV2 for Bitpay {}
impl PaymentCapture for Bitpay {}
impl RefundSyncV2 for Bitpay {}
impl IncomingWebhook for Bitpay {}

impl ConnectorIntegrationV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>
    for Bitpay
{
    fn get_headers(
        &self,
        req: &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
    ) -> CustomResult<Vec<(String, Maskable<String>)>, errors::ConnectorError> {
        let mut header = vec![
            (
                headers::CONTENT_TYPE.to_string(),
                "application/json".to_string().into(),
            ),
        ];
        let mut api_key = self.get_auth_header(&req.connector_auth_type)?;

        let auth_header = hyperswitch_interfaces::api::get_auth_header(&req.connector_auth_type, Self::id())?;
        header.append(&mut api_key);
        Ok(header)
    }

    fn get_url(
        &self,
        req: &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
    ) -> CustomResult<String, errors::ConnectorError> {
        Ok(format!(
            "{}/invoices",
            req.resource_common_data.connectors.bitpay.base_url
        ))
    }

    fn get_request_body(
        &self,
        _req: &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
    ) -> CustomResult<Option<RequestContent>, errors::ConnectorError> {
        Err(errors::ConnectorError::NotImplemented("get_request_body for Authorize".to_string()).into())
    }

    fn handle_response_v2(
        &self,
        _data: &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
        _event_builder: Option<&mut hyperswitch_interfaces::events::connector_api_logs::ConnectorEvent>,
        _res: hyperswitch_interfaces::types::Response,
    ) -> CustomResult<RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>, errors::ConnectorError> {
        Err(errors::ConnectorError::NotImplemented("handle_response_v2 for Authorize".to_string()).into())
    }

    fn get_error_response_v2(
        &self,
        res: hyperswitch_interfaces::types::Response,
        event_builder: Option<&mut hyperswitch_interfaces::events::connector_api_logs::ConnectorEvent>,
    ) -> CustomResult<hyperswitch_domain_models::router_data::ErrorResponse, errors::ConnectorError> {
        hyperswitch_interfaces::api::build_error_response(res, event_builder, Self::id())
    }

    fn get_5xx_error_response(
        &self,
        res: hyperswitch_interfaces::types::Response,
        event_builder: Option<&mut hyperswitch_interfaces::events::connector_api_logs::ConnectorEvent>,
    ) -> CustomResult<hyperswitch_domain_models::router_data::ErrorResponse, errors::ConnectorError> {
        hyperswitch_interfaces::api::build_error_response(res, event_builder, Self::id())
    }
}

// Implement stubs for other ConnectorIntegrationV2 traits
macro_rules! impl_connector_integration_stubs {
    ($flow:ty, $req_data:ty, $res_data:ty, $flow_name:literal) => {
        impl ConnectorIntegrationV2<$flow, PaymentFlowData, $req_data, $res_data> for Bitpay {
            fn get_headers(&self, _req: &RouterDataV2<$flow, PaymentFlowData, $req_data, $res_data>) -> CustomResult<Vec<(String, Maskable<String>)>, errors::ConnectorError> { Err(errors::ConnectorError::NotImplemented(concat!("get_headers for ", $flow_name)).into()) }
            fn get_url(&self, _req: &RouterDataV2<$flow, PaymentFlowData, $req_data, $res_data>) -> CustomResult<String, errors::ConnectorError> { Err(errors::ConnectorError::NotImplemented(concat!("get_url for ", $flow_name)).into()) }
            fn get_request_body(&self, _req: &RouterDataV2<$flow, PaymentFlowData, $req_data, $res_data>) -> CustomResult<Option<RequestContent>, errors::ConnectorError> { Err(errors::ConnectorError::NotImplemented(concat!("get_request_body for ", $flow_name)).into()) }
            fn handle_response_v2(&self, _data: &RouterDataV2<$flow, PaymentFlowData, $req_data, $res_data>, _event_builder: Option<&mut hyperswitch_interfaces::events::connector_api_logs::ConnectorEvent>, _res: hyperswitch_interfaces::types::Response,) -> CustomResult<RouterDataV2<$flow, PaymentFlowData, $req_data, $res_data>, errors::ConnectorError> { Err(errors::ConnectorError::NotImplemented(concat!("handle_response_v2 for ", $flow_name)).into()) }
            fn get_error_response_v2(&self, res: hyperswitch_interfaces::types::Response, event_builder: Option<&mut hyperswitch_interfaces::events::connector_api_logs::ConnectorEvent>) -> CustomResult<hyperswitch_domain_models::router_data::ErrorResponse, errors::ConnectorError> { hyperswitch_interfaces::api::build_error_response(res, event_builder, Self::id()) }
            fn get_5xx_error_response(&self, res: hyperswitch_interfaces::types::Response, event_builder: Option<&mut hyperswitch_interfaces::events::connector_api_logs::ConnectorEvent>) -> CustomResult<hyperswitch_domain_models::router_data::ErrorResponse, errors::ConnectorError> { hyperswitch_interfaces::api::build_error_response(res, event_builder, Self::id()) }
        }
    };
    ($flow:ty, RefundFlowData, $req_data:ty, $res_data:ty, $flow_name:literal) => {
        impl ConnectorIntegrationV2<$flow, RefundFlowData, $req_data, $res_data> for Bitpay {
            fn get_headers(&self, _req: &RouterDataV2<$flow, RefundFlowData, $req_data, $res_data>) -> CustomResult<Vec<(String, Maskable<String>)>, errors::ConnectorError> { Err(errors::ConnectorError::NotImplemented(concat!("get_headers for ", $flow_name)).into()) }
            fn get_url(&self, _req: &RouterDataV2<$flow, RefundFlowData, $req_data, $res_data>) -> CustomResult<String, errors::ConnectorError> { Err(errors::ConnectorError::NotImplemented(concat!("get_url for ", $flow_name)).into()) }
            fn get_request_body(&self, _req: &RouterDataV2<$flow, RefundFlowData, $req_data, $res_data>) -> CustomResult<Option<RequestContent>, errors::ConnectorError> { Err(errors::ConnectorError::NotImplemented(concat!("get_request_body for ", $flow_name)).into()) }
            fn handle_response_v2(&self, _data: &RouterDataV2<$flow, RefundFlowData, $req_data, $res_data>, _event_builder: Option<&mut hyperswitch_interfaces::events::connector_api_logs::ConnectorEvent>, _res: hyperswitch_interfaces::types::Response,) -> CustomResult<RouterDataV2<$flow, RefundFlowData, $req_data, $res_data>, errors::ConnectorError> { Err(errors::ConnectorError::NotImplemented(concat!("handle_response_v2 for ", $flow_name)).into()) }
            fn get_error_response_v2(&self, res: hyperswitch_interfaces::types::Response, event_builder: Option<&mut hyperswitch_interfaces::events::connector_api_logs::ConnectorEvent>) -> CustomResult<hyperswitch_domain_models::router_data::ErrorResponse, errors::ConnectorError> { hyperswitch_interfaces::api::build_error_response(res, event_builder, Self::id()) }
            fn get_5xx_error_response(&self, res: hyperswitch_interfaces::types::Response, event_builder: Option<&mut hyperswitch_interfaces::events::connector_api_logs::ConnectorEvent>) -> CustomResult<hyperswitch_domain_models::router_data::ErrorResponse, errors::ConnectorError> { hyperswitch_interfaces::api::build_error_response(res, event_builder, Self::id()) }
        }
    };
}

impl ConnectorIntegrationV2<PSync, PaymentFlowData, PaymentsSyncData, PaymentsResponseData> for Bitpay {}
impl ConnectorIntegrationV2<CreateOrder, PaymentFlowData, PaymentCreateOrderData, PaymentCreateOrderResponse> for Bitpay {}
impl ConnectorIntegrationV2<RSync, RefundFlowData, RefundSyncData, RefundsResponseData> for Bitpay {}
// IncomingWebhook already impl separately
impl ConnectorIntegrationV2<Void, PaymentFlowData, PaymentVoidData, PaymentsResponseData> for Bitpay {}
impl ConnectorIntegrationV2<Refund, RefundFlowData, RefundsData, RefundsResponseData> for Bitpay {}
impl ConnectorIntegrationV2<Capture, PaymentFlowData, PaymentsCaptureData, PaymentsResponseData> for Bitpay {}
