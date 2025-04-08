pub mod transformers;

// use base64::Engine;
// use common_enums::enums::{self, PaymentMethodType};
use hyperswitch_common_utils::{
    consts,
    errors::CustomResult,
    ext_traits::{ByteSliceExt, OptionExt},
    request::{Method, Request, RequestBuilder, RequestContent},
    types::{AmountConvertor, MinorUnit, MinorUnitForConnector},
};
// use error_stack::{report, ResultExt};
use hyperswitch_domain_models::{
    api::ApplicationResponse,
    payment_method_data::PaymentMethodData,
    router_data::{AccessToken, ConnectorAuthType, ErrorResponse, RouterData},
    router_flow_types::{
        access_token_auth::AccessTokenAuth,
        payments::{
            Authorize, Capture, PSync, PaymentMethodToken, PreProcessing, Session, SetupMandate,
            Void,
        },
        refunds::{Execute, RSync},
        Accept, Defend, Evidence, Retrieve, Upload,
    },
    router_request_types::{
        AcceptDisputeRequestData, AccessTokenRequestData, DefendDisputeRequestData,
        PaymentMethodTokenizationData, PaymentsAuthorizeData, PaymentsCancelData,
        PaymentsCaptureData, PaymentsPreProcessingData, PaymentsSessionData, PaymentsSyncData,
        RefundsData, RetrieveFileRequestData, SetupMandateRequestData, SubmitEvidenceRequestData,
        SyncRequestType, UploadFileRequestData,
    },
    router_data_v2::flow_common_types:: PaymentFlowData,
    router_response_types::{
        AcceptDisputeResponse, DefendDisputeResponse, PaymentsResponseData, RefundsResponseData,
        RetrieveFileResponse, SubmitEvidenceResponse, UploadFileResponse,
    },
    types::{
        PaymentsAuthorizeRouterData, PaymentsCancelRouterData, PaymentsCaptureRouterData,
        PaymentsSyncRouterData, RefundsRouterData,
        SetupMandateRouterData,
    },
};
#[cfg(feature = "payouts")]
use hyperswitch_domain_models::{
    router_flow_types::payouts::{PoCancel, PoCreate, PoEligibility, PoFulfill},
    router_response_types::PayoutsResponseData,
    types::{PayoutsData, PayoutsRouterData},
};
#[cfg(feature = "payouts")]
use hyperswitch_interfaces::types::{
    PayoutCancelType, PayoutCreateType, PayoutEligibilityType, PayoutFulfillType,
};
use hyperswitch_interfaces::{
    api::{
        self,
        disputes::{AcceptDispute, DefendDispute, Dispute, SubmitEvidence},
        files::{FilePurpose, FileUpload, RetrieveFile, UploadFile},
        CaptureSyncMethod, ConnectorCommon, ConnectorIntegration,
        ConnectorValidation,
    },
    configs::Connectors,
    consts::{NO_ERROR_CODE, NO_ERROR_MESSAGE},
    disputes, errors,
    events::connector_api_logs::ConnectorEvent,
    types::{
        AcceptDisputeType, DefendDisputeType, PaymentsAuthorizeType, PaymentsCaptureType,
        PaymentsPreProcessingType, PaymentsSyncType, PaymentsVoidType, RefundExecuteType, Response,
        SetupMandateType, SubmitEvidenceType,
    },
    connector_integration_v2::ConnectorIntegrationV2,
};
use hyperswitch_masking::{ExposeInterface, Mask, Maskable, Secret};

use transformers as adyen;

#[cfg(feature = "payouts")]
use crate::utils::PayoutsData as UtilsPayoutData;
// use crate::{
//     capture_method_not_supported,
//     constants::{self, headers},
//     types::{
//         AcceptDisputeRouterData, DefendDisputeRouterData, ResponseRouterData,
//         SubmitEvidenceRouterData,
//     },
//     utils::{
//         self as connector_utils, convert_payment_authorize_router_response,
//         convert_setup_mandate_router_data_to_authorize_router_data, is_mandate_supported,
//         ForeignTryFrom, PaymentMethodDataType,
//     },
// };

pub(crate) mod headers {
    pub(crate) const ACCEPT: &str = "Accept";
    pub(crate) const API_KEY: &str = "API-KEY";
    pub(crate) const APIKEY: &str = "apikey";
    pub(crate) const API_TOKEN: &str = "Api-Token";
    pub(crate) const AUTHORIZATION: &str = "Authorization";
    pub(crate) const CONTENT_TYPE: &str = "Content-Type";
    pub(crate) const DATE: &str = "Date";
    pub(crate) const IDEMPOTENCY_KEY: &str = "Idempotency-Key";
    pub(crate) const MESSAGE_SIGNATURE: &str = "Message-Signature";
    pub(crate) const MERCHANT_ID: &str = "Merchant-ID";
    pub(crate) const REQUEST_ID: &str = "request-id";
    pub(crate) const NONCE: &str = "nonce";
    pub(crate) const TIMESTAMP: &str = "Timestamp";
    pub(crate) const TOKEN: &str = "token";
    pub(crate) const X_ACCEPT_VERSION: &str = "X-Accept-Version";
    pub(crate) const X_CC_API_KEY: &str = "X-CC-Api-Key";
    pub(crate) const X_CC_VERSION: &str = "X-CC-Version";
    pub(crate) const X_DATE: &str = "X-Date";
    pub(crate) const X_LOGIN: &str = "X-Login";
    pub(crate) const X_NN_ACCESS_KEY: &str = "X-NN-Access-Key";
    pub(crate) const X_TRANS_KEY: &str = "X-Trans-Key";
    pub(crate) const X_RANDOM_VALUE: &str = "X-RandomValue";
    pub(crate) const X_REQUEST_DATE: &str = "X-RequestDate";
    pub(crate) const X_VERSION: &str = "X-Version";
    pub(crate) const X_API_KEY: &str = "X-Api-Key";
    pub(crate) const CORRELATION_ID: &str = "Correlation-Id";
    pub(crate) const WP_API_VERSION: &str = "WP-Api-Version";
    pub(crate) const SOURCE: &str = "Source";
    pub(crate) const USER_AGENT: &str = "User-Agent";
    pub(crate) const KEY: &str = "key";
    pub(crate) const X_SIGNATURE: &str = "X-Signature";
    pub(crate) const SOAP_ACTION: &str = "SOAPAction";
}

#[derive(Clone)]
pub struct Adyen {
    amount_converter: &'static (dyn AmountConvertor<Output = MinorUnit> + Sync),
}

impl ConnectorIntegrationV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData> for Adyen {
    fn get_headers(
        &self,
        req: &PaymentsAuthorizeRouterData,
        _connectors: &Connectors,
    ) -> CustomResult<Vec<(String, Maskable<String>)>, errors::ConnectorError>
    where
        Self: ConnectorIntegrationV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
    {
        let mut header = vec![(
            headers::CONTENT_TYPE.to_string(),
            PaymentsAuthorizeType::get_content_type(self)
                .to_string()
                .into(),
        )];
        let mut api_key = self.get_auth_header(&req.connector_auth_type)?;
        header.append(&mut api_key);
        Ok(header)
    }

    fn get_url(
        &self,
        req: &PaymentsAuthorizeRouterData,
        connectors: &Connectors,
    ) -> CustomResult<String, errors::ConnectorError> {
        Ok(format!("{}{}/payments", "endpoint", "ADYEN_API_VERSION"))
    }

    fn get_request_body(
        &self,
        req: &PaymentsAuthorizeRouterData,
        _connectors: &Connectors,
    ) -> CustomResult<RequestContent, errors::ConnectorError> {
        let connector_router_data = adyen::AdyenRouterData::try_from((req.request.minor_amount, req))?;
        let connector_req = adyen::AdyenPaymentRequest::try_from(&connector_router_data)?;
        Ok(RequestContent::Json(Box::new(connector_req)))
    }

    fn build_request_v2(
        &self,
        req: &PaymentsAuthorizeRouterData,
        connectors: &Connectors,
    ) -> CustomResult<Option<Request>, errors::ConnectorError> {
        Ok(Some(
            RequestBuilder::new()
                .method(Method::Post)
                .url(&PaymentsAuthorizeType::get_url(self, req, connectors)?)
                .attach_default_headers()
                .headers(PaymentsAuthorizeType::get_headers(self, req, connectors)?)
                .set_body(PaymentsAuthorizeType::get_request_body(
                    self, req, connectors,
                )?)
                .build(),
        ))
    }

    fn handle_response_v2(
        &self,
        data: &PaymentsAuthorizeRouterData,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<PaymentsAuthorizeRouterData, errors::ConnectorError> {
        let response: adyen::AdyenPaymentResponse = res
            .response
            .parse_struct("AdyenPaymentResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        event_builder.map(|i| i.set_response_body(&response));
        router_env::logger::info!(connector_response=?response);
        RouterData::foreign_try_from((
            ResponseRouterData {
                response,
                data: data.clone(),
                http_code: res.status_code,
            },
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