pub mod transformers;

use hyperswitch_common_utils::{
    errors::CustomResult,
    ext_traits::{ByteSliceExt, OptionExt},
    request::RequestContent,
    types::{AmountConvertor, MinorUnit},
};

use crate::{with_error_response_body, with_response_body};

use hyperswitch_domain_models::{
    router_data::{ConnectorAuthType, ErrorResponse},
    router_data_v2::RouterDataV2,
    router_request_types::SyncRequestType,
};

use error_stack::ResultExt;
use hyperswitch_interfaces::{
    api::{self, CaptureSyncMethod, ConnectorCommon},
    configs::Connectors,
    connector_integration_v2::ConnectorIntegrationV2,
    errors,
    events::connector_api_logs::ConnectorEvent,
    types::Response,
};
use hyperswitch_masking::{Mask, Maskable};

use domain_types::{
    connector_flow::{Authorize, PSync},
    connector_types::{
        ConnectorServiceTrait, PaymentAuthorizeV2, PaymentCreateOrderData,
        PaymentCreateOrderResponse, PaymentFlowData, PaymentOrderCreate, PaymentSyncV2,
        PaymentsAuthorizeData, PaymentsResponseData, PaymentsSyncData, ValidationTrait,
    },
};
use transformers::{self as adyen, ForeignTryFrom};

use domain_types::connector_flow::CreateOrder;

pub(crate) mod headers {
    pub(crate) const CONTENT_TYPE: &str = "Content-Type";
    pub(crate) const X_API_KEY: &str = "X-Api-Key";
}

impl ConnectorServiceTrait for Adyen {}
impl PaymentAuthorizeV2 for Adyen {}
impl PaymentSyncV2 for Adyen {}

#[derive(Clone)]
pub struct Adyen {
    #[allow(dead_code)]
    pub(crate) amount_converter: &'static (dyn AmountConvertor<Output = MinorUnit> + Sync),
}

impl Adyen {
    pub const fn new() -> &'static Self {
        &Self {
            amount_converter: &hyperswitch_common_utils::types::MinorUnitForConnector,
        }
    }
}

impl ConnectorCommon for Adyen {
    fn id(&self) -> &'static str {
        "adyen"
    }
    fn get_currency_unit(&self) -> api::CurrencyUnit {
        api::CurrencyUnit::Minor
    }
    fn get_auth_header(
        &self,
        auth_type: &ConnectorAuthType,
    ) -> CustomResult<Vec<(String, Maskable<String>)>, errors::ConnectorError> {
        let auth = adyen::AdyenAuthType::try_from(auth_type)
            .map_err(|_| errors::ConnectorError::FailedToObtainAuthType)?;
        Ok(vec![(
            headers::X_API_KEY.to_string(),
            auth.api_key.into_masked(),
        )])
    }
    fn base_url<'a>(&self, connectors: &'a Connectors) -> &'a str {
        connectors.adyen.base_url.as_ref()
    }

    fn build_error_response(
        &self,
        res: Response,
        event_builder: Option<&mut ConnectorEvent>,
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        let response: adyen::AdyenErrorResponse = res
            .response
            .parse_struct("ErrorResponse")
            .map_err(|_| errors::ConnectorError::ResponseDeserializationFailed)?;

        with_error_response_body!(event_builder, response);

        Ok(ErrorResponse {
            status_code: res.status_code,
            code: response.error_code,
            message: response.message.to_owned(),
            reason: Some(response.message),
            attempt_status: None,
            connector_transaction_id: response.psp_reference,
        })
    }
}

const ADYEN_API_VERSION: &str = "v68";

impl ConnectorIntegrationV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>
    for Adyen
{
    fn get_headers(
        &self,
        req: &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
    ) -> CustomResult<Vec<(String, Maskable<String>)>, errors::ConnectorError>
    where
        Self: ConnectorIntegrationV2<
            Authorize,
            PaymentFlowData,
            PaymentsAuthorizeData,
            PaymentsResponseData,
        >,
    {
        let mut header = vec![(
            headers::CONTENT_TYPE.to_string(),
            "application/json".to_string().into(),
        )];
        let mut api_key = self.get_auth_header(&req.connector_auth_type)?;
        header.append(&mut api_key);
        Ok(header)
    }

    fn get_url(
        &self,
        req: &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
    ) -> CustomResult<String, errors::ConnectorError> {
        Ok(format!(
            "{}{}/payments",
            req.resource_common_data.connectors.adyen.base_url, ADYEN_API_VERSION
        ))
    }

    fn get_request_body(
        &self,
        req: &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
    ) -> CustomResult<Option<RequestContent>, errors::ConnectorError> {
        let connector_router_data =
            adyen::AdyenRouterData::try_from((req.request.minor_amount, req))?;
        let connector_req = adyen::AdyenPaymentRequest::try_from(&connector_router_data)?;
        Ok(Some(RequestContent::Json(Box::new(connector_req))))
    }

    fn handle_response_v2(
        &self,
        data: &RouterDataV2<
            Authorize,
            PaymentFlowData,
            PaymentsAuthorizeData,
            PaymentsResponseData,
        >,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<
        RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
        errors::ConnectorError,
    > {
        let response: adyen::AdyenPaymentResponse = res
            .response
            .parse_struct("AdyenPaymentResponse")
            .map_err(|_| errors::ConnectorError::ResponseDeserializationFailed)?;

        with_response_body!(event_builder, response);

        RouterDataV2::foreign_try_from((
            response,
            data.clone(),
            res.status_code,
            data.request.capture_method,
            false,
            data.request.payment_method_type,
        ))
        .change_context(errors::ConnectorError::ResponseHandlingFailed)
    }

    fn get_error_response_v2(
        &self,
        res: Response,
        event_builder: Option<&mut ConnectorEvent>,
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }

    fn get_5xx_error_response(
        &self,
        res: Response,
        event_builder: Option<&mut ConnectorEvent>,
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }
}

impl ConnectorIntegrationV2<PSync, PaymentFlowData, PaymentsSyncData, PaymentsResponseData>
    for Adyen
{
    fn get_headers(
        &self,
        req: &RouterDataV2<PSync, PaymentFlowData, PaymentsSyncData, PaymentsResponseData>,
    ) -> CustomResult<Vec<(String, Maskable<String>)>, errors::ConnectorError>
    where
        Self: ConnectorIntegrationV2<
            Authorize,
            PaymentFlowData,
            PaymentsAuthorizeData,
            PaymentsResponseData,
        >,
    {
        let mut header = vec![(
            headers::CONTENT_TYPE.to_string(),
            "application/json".to_string().into(),
        )];
        let mut api_key = self.get_auth_header(&req.connector_auth_type)?;
        header.append(&mut api_key);
        Ok(header)
    }

    fn get_url(
        &self,
        _req: &RouterDataV2<PSync, PaymentFlowData, PaymentsSyncData, PaymentsResponseData>,
    ) -> CustomResult<String, errors::ConnectorError> {
        Ok(format!(
            "{}{}/payments/details",
            "https://checkout-test.adyen.com/", ADYEN_API_VERSION
        ))
    }

    fn get_request_body(
        &self,
        req: &RouterDataV2<PSync, PaymentFlowData, PaymentsSyncData, PaymentsResponseData>,
    ) -> CustomResult<Option<RequestContent>, errors::ConnectorError> {
        let encoded_data = req
            .request
            .encoded_data
            .clone()
            .get_required_value("encoded_data")
            .change_context(errors::ConnectorError::RequestEncodingFailed)?;
        let adyen_redirection_type = serde_urlencoded::from_str::<
            transformers::AdyenRedirectRequestTypes,
        >(encoded_data.as_str())
        .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;

        let connector_req = match adyen_redirection_type {
            adyen::AdyenRedirectRequestTypes::AdyenRedirection(req) => {
                adyen::AdyenRedirectRequest {
                    details: adyen::AdyenRedirectRequestTypes::AdyenRedirection(
                        adyen::AdyenRedirection {
                            redirect_result: req.redirect_result,
                            type_of_redirection_result: None,
                            result_code: None,
                        },
                    ),
                }
            }
            adyen::AdyenRedirectRequestTypes::AdyenThreeDS(req) => adyen::AdyenRedirectRequest {
                details: adyen::AdyenRedirectRequestTypes::AdyenThreeDS(adyen::AdyenThreeDS {
                    three_ds_result: req.three_ds_result,
                    type_of_redirection_result: None,
                    result_code: None,
                }),
            },
            adyen::AdyenRedirectRequestTypes::AdyenRefusal(req) => adyen::AdyenRedirectRequest {
                details: adyen::AdyenRedirectRequestTypes::AdyenRefusal(adyen::AdyenRefusal {
                    payload: req.payload,
                    type_of_redirection_result: None,
                    result_code: None,
                }),
            },
        };

        Ok(Some(RequestContent::Json(Box::new(connector_req))))
    }

    fn handle_response_v2(
        &self,
        data: &RouterDataV2<PSync, PaymentFlowData, PaymentsSyncData, PaymentsResponseData>,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<
        RouterDataV2<PSync, PaymentFlowData, PaymentsSyncData, PaymentsResponseData>,
        errors::ConnectorError,
    > {
        let response: adyen::AdyenPaymentResponse = res
            .response
            .parse_struct("AdyenPaymentResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;

        with_response_body!(event_builder, response);

        let is_multiple_capture_sync = match data.request.sync_type {
            SyncRequestType::MultipleCaptureSync(_) => true,
            SyncRequestType::SinglePaymentSync => false,
        };
        RouterDataV2::foreign_try_from((
            response,
            data.clone(),
            res.status_code,
            data.request.capture_method,
            is_multiple_capture_sync,
            data.request.payment_method_type,
        ))
        .change_context(errors::ConnectorError::ResponseHandlingFailed)
    }

    fn get_error_response_v2(
        &self,
        res: Response,
        event_builder: Option<&mut ConnectorEvent>,
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }

    fn get_multiple_capture_sync_method(
        &self,
    ) -> CustomResult<CaptureSyncMethod, errors::ConnectorError> {
        Ok(CaptureSyncMethod::Individual)
    }
    fn get_5xx_error_response(
        &self,
        res: Response,
        event_builder: Option<&mut ConnectorEvent>,
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }
}

impl ValidationTrait for Adyen {}

impl PaymentOrderCreate for Adyen {}

impl
    ConnectorIntegrationV2<
        CreateOrder,
        PaymentFlowData,
        PaymentCreateOrderData,
        PaymentCreateOrderResponse,
    > for Adyen
{
}
