use std::collections::HashMap;

use hyperswitch_api_models::enums::{self, AttemptStatus};

use hyperswitch_cards::CardNumber;
use hyperswitch_common_utils::{request::Method, types::MinorUnit};

use hyperswitch_domain_models::{
    payment_method_data::{Card, PaymentMethodData},
    router_data::{ConnectorAuthType, RouterData},
    router_data_v2::{PaymentFlowData, RouterDataV2},
    router_flow_types::Authorize,
    router_request_types::{PaymentsAuthorizeData, ResponseId},
    router_response_types::{PaymentsResponseData, RedirectForm},
};
use hyperswitch_masking::Secret;
use serde::{Deserialize, Serialize};
use crate::{
    flow::CreateOrder,
    types::{
        PaymentCreateOrderData, PaymentCreateOrderResponse,
    },
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum Currency {
    #[default]
    USD,
    EUR,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Amount {
    pub currency: enums::Currency,
    pub value: MinorUnit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CardBrand {
    Visa,
}

#[derive(Debug, PartialEq)]
pub enum ConnectorError {
    ParsingFailed,
    NotImplemented,
    FailedToObtainAuthType,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RazorpayCard {
    number: CardNumber,
    expiry_month: Secret<String>,
    expiry_year: Secret<String>,
    cvc: Option<Secret<String>>,
    holder_name: Option<Secret<String>>,
    brand: Option<CardBrand>,
    network_payment_reference: Option<Secret<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum RazorpayPaymentMethod {
    #[serde(rename = "scheme")]
    RazorpayCard(Box<RazorpayCard>),
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum AuthType {
    #[default]
    PreAuth,
}

#[serde_with::skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Address {
    city: String,
    country: enums::CountryAlpha2,
    house_number_or_name: Secret<String>,
    postal_code: Secret<String>,
    state_or_province: Option<Secret<String>>,
    street: Option<Secret<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum PaymentMethod {
    RazorpayPaymentMethod(Box<RazorpayPaymentMethod>),
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CardDetails {
    pub number: CardNumber,
    pub name: String,
    pub expiry_month: Secret<String>,
    pub expiry_year: String,
    pub cvv: Option<Secret<String>>,
}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AuthenticationDetails {
    pub authentication_channel: String,
}

#[derive(Default, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BrowserInfo {
    pub java_enabled: bool,
    pub javascript_enabled: bool,
    pub timezone_offset: i32,
    pub color_depth: i32,
    pub screen_width: i32,
    pub screen_height: i32,
    pub language: String,
}

#[derive(Debug, Serialize)]
#[serde( rename_all = "snake_case")]
pub struct RazorpayPaymentRequest {
    pub amount: MinorUnit,
    pub currency: String,
    pub contact: String,
    pub email: String,
    pub order_id: String,
    // pub method: String,
    // #[serde(flatten)]
    pub card: PaymentMethodSpecificData,
    pub authentication: Option<AuthenticationDetails>,
    pub browser: BrowserInfo,
    pub ip: String,
    pub referer: String,
    pub user_agent: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum PaymentMethodSpecificData {
    Card(CardDetails),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentMethodType {
    Card,
    Wallet,
    Upi,
    Emi,
    Netbanking,
}

#[derive(Debug, Serialize)]
pub struct RazorpayRouterData<T> {
    pub amount: MinorUnit,
    pub router_data: T,
}

impl<T> TryFrom<(MinorUnit, T)> for RazorpayRouterData<T> {
    type Error = hyperswitch_interfaces::errors::ConnectorError;
    fn try_from((amount, item): (MinorUnit, T)) -> Result<Self, Self::Error> {
        Ok(Self {
            amount,
            router_data: item,
        })
    }
}

pub struct RazorpayAuthType {
    pub(super) key_id: Secret<String>,
    pub(super) secret_key: Secret<String>,
}

impl TryFrom<&ConnectorAuthType> for RazorpayAuthType {
    type Error = hyperswitch_interfaces::errors::ConnectorError;
    fn try_from(auth_type: &ConnectorAuthType) -> Result<Self, Self::Error> {
        match auth_type {
            ConnectorAuthType::BodyKey { api_key, key1 } => Ok(Self {
                key_id: api_key.to_owned(),
                secret_key: key1.to_owned(),
            }),
            _ => Err(hyperswitch_interfaces::errors::ConnectorError::FailedToObtainAuthType),
        }
    }
}

impl TryFrom<(&Card, Option<Secret<String>>)> for RazorpayPaymentMethod {
    type Error = ConnectorError;
    fn try_from(
        (card, card_holder_name): (&Card, Option<Secret<String>>),
    ) -> Result<Self, Self::Error> {
        let razorpay_card = RazorpayCard {
            number: card.card_number.clone(),
            expiry_month: card.card_exp_month.clone(),
            expiry_year: "2031".to_string().into(),
            cvc: Some(card.card_cvc.clone()),
            holder_name: card_holder_name,
            brand: Some(CardBrand::Visa),
            network_payment_reference: None,
        };
        Ok(RazorpayPaymentMethod::RazorpayCard(Box::new(razorpay_card)))
    }
}

impl
    TryFrom<(
        &RazorpayRouterData<
            &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
        >,
        &Card,
    )> for RazorpayPaymentRequest
{
    type Error = hyperswitch_interfaces::errors::ConnectorError;

    fn try_from(
        value: (
            &RazorpayRouterData<
                &RouterDataV2<
                    Authorize,
                    PaymentFlowData,
                    PaymentsAuthorizeData,
                    PaymentsResponseData,
                >,
            >,
            &Card,
        ),
    ) -> Result<Self, Self::Error> {
        let (item, card_data) = value;
println!("$$$ router data {:?}",item.router_data);
        let amount = item.amount;
        let currency = item.router_data.request.currency.to_string();

        let contact = "9900008989".to_string();

        // let email = item.router_data.request.email.clone().ok_or(
        //     hyperswitch_interfaces::errors::ConnectorError::MissingRequiredField {
        //         field_name: "email",
        //     },
        // )?;
        let email="sweta.sharma@juspay.in".to_string();

         let order_id = item.router_data.reference_id.clone().ok_or(
            hyperswitch_interfaces::errors::ConnectorError::MissingRequiredField {
                field_name: "ordeR_id",
            },
        )?;
        // let order_id = item.router_data.reference_id.clone() ;
        let method = "card".to_string();
        let card_holder_name = "Sweta Sharma".to_string().into();
        let card = PaymentMethodSpecificData::Card(CardDetails {
            number: card_data.card_number.clone(),
            name: card_holder_name,
            expiry_month: card_data.card_exp_month.clone(),
            expiry_year: "2030".to_string().into(),
            cvv: Some(card_data.card_cvc.clone()),
        });

        let authentication = Some(AuthenticationDetails {
            authentication_channel: "browser".to_string(),
        });

        let browser = BrowserInfo {
            java_enabled: false,
            javascript_enabled: false,
            timezone_offset: 0,
            color_depth: 24,
            screen_width: 1920,
            screen_height: 1080,
            language: "en-US".to_string(),
        };

        Ok(RazorpayPaymentRequest {
            amount,
            currency,
            contact,
            email,
            order_id,
            // method,
            card,
            authentication,
            browser,
            ip: "105.106.107.108".to_string(),
            referer: "https://merchansite.com/example/paybill".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
        })
    }
}

impl
    TryFrom<
        &RazorpayRouterData<
            &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
        >,
    > for RazorpayPaymentRequest
{
    type Error = hyperswitch_interfaces::errors::ConnectorError;

    fn try_from(
        item: &RazorpayRouterData<
            &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
        >,
    ) -> Result<Self, Self::Error> {
        match &item.router_data.request.payment_method_data {
            PaymentMethodData::Card(card) => RazorpayPaymentRequest::try_from((item, card)),
            _ => Err(
                hyperswitch_interfaces::errors::ConnectorError::NotImplemented(
                    "Only card payments are supported".into(),
                ),
            ),
        }
    }
}

pub struct ResponseRouterData<Flow, R, Request, Response> {
    pub response: R,
    pub data: RouterData<Flow, Request, Response>,
    pub http_code: u16,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RazorpayPaymentResponse {
    pub razorpay_payment_id: String,
    pub next: Option<Vec<NextAction>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NextAction {
    pub action: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum RazorpayResponse {
    PaymentResponse(RazorpayPaymentResponse),
    PsyncResponse(RazorpayPsyncResponse),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RazorpayPsyncResponse {
    pub id: String,
    // pub entity: String,
    // pub amount: i64,
    // pub currency: String,
    pub status: RazorpayStatus,
    // pub method: String,
    // pub order_id: Option<String>,
    // pub description: Option<String>,
    // pub international: bool,
    // pub refund_status: Option<String>,
    // pub amount_refunded: i64,
    // pub captured: bool,
    // pub email: String,
    // pub contact: String,
    // pub fee: i64,
    // pub tax: i64,
    // pub error_code: Option<String>,
    // pub error_description: Option<String>,
    // pub error_source: Option<String>,
    // pub error_step: Option<String>,
    // pub error_reason: Option<String>,
    // pub notes: Option<HashMap<String, String>>,
    // pub created_at: i64,
    // pub card_id: Option<String>,
    // pub card: Option<SyncCardDetails>,
    // pub upi: Option<SyncUPIDetails>,
    // pub acquirer_data: Option<Vec<AcquirerData>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncCardDetails {
    pub id: String,
    pub entity: String,
    pub name: String,
    pub last4: i32,
    pub network: String,
    pub r#type: String,
    pub issuer: Option<String>,
    pub emi: bool,
    pub sub_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncUPIDetails {
    pub payer_account_type: String,
    pub vpa: String,
    pub flow: String,
    pub bank: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AcquirerData {
    pub rrn: String,
    pub authentication_reference_number: String,
    pub bank_transaction_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]

pub enum RazorpayStatus {
    Created,
    Authorized,
    Captured,
    Refunded,
    Failed,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureMethod {
    #[default]
    Automatic,
    Manual,
    ManualMultiple,
    Scheduled,
    SequentialAutomatic,
}

pub trait ForeignTryFrom<F>: Sized {
    type Error;

    fn foreign_try_from(from: F) -> Result<Self, Self::Error>;
}

fn get_authorization_razorpay_payment_status_from_action(
    is_manual_capture: bool,
    has_next_action: bool,
) -> AttemptStatus {
    if has_next_action {
        AttemptStatus::AuthenticationPending
    } else if is_manual_capture {
        AttemptStatus::Authorized
    } else {
        AttemptStatus::Charged
    }
}

fn get_psync_razorpay_payment_status(
    is_manual_capture: bool,
    razorpay_status: RazorpayStatus,
) -> AttemptStatus {
    match razorpay_status {
        RazorpayStatus::Created => AttemptStatus::Pending,
        RazorpayStatus::Authorized => {
            if is_manual_capture {
                AttemptStatus::Authorized
            } else {
                AttemptStatus::Charged
            }
        }
        RazorpayStatus::Captured => AttemptStatus::Charged,
        RazorpayStatus::Refunded => AttemptStatus::AutoRefunded,
        RazorpayStatus::Failed => AttemptStatus::Failure,
    }
}

impl<F, Req>
    ForeignTryFrom<(
        RazorpayResponse,
        RouterDataV2<F, PaymentFlowData, Req, PaymentsResponseData>,
        u16,
        Option<hyperswitch_api_models::enums::CaptureMethod>,
        bool,
        Option<hyperswitch_api_models::enums::PaymentMethodType>,
    )> for RouterDataV2<F, PaymentFlowData, Req, PaymentsResponseData>
{
    type Error = hyperswitch_interfaces::errors::ConnectorError;

    fn foreign_try_from(
        (response, data, _http_code, _capture_method, _is_multiple_capture_psync_flow, _pmt): (
            RazorpayResponse,
            RouterDataV2<F, PaymentFlowData, Req, PaymentsResponseData>,
            u16,
            Option<hyperswitch_api_models::enums::CaptureMethod>,
            bool,
            Option<hyperswitch_api_models::enums::PaymentMethodType>,
        ),
    ) -> Result<Self, Self::Error> {
        let is_manual_capture = false;

        match response {
            RazorpayResponse::PaymentResponse(payment_response) => {
                let status =
                    get_authorization_razorpay_payment_status_from_action(is_manual_capture, true);
                let redirect_url = payment_response
                    .next
                    .as_ref()
                    .and_then(|next_actions| next_actions.first())
                    .map(|action| action.url.clone())
                    .ok_or_else(|| {
                        hyperswitch_interfaces::errors::ConnectorError::MissingRequiredField {
                            field_name: "next.url",
                        }
                    })?;

                let mut form_fields = HashMap::new();
                form_fields.insert(
                    "transaction_id".to_string(),
                    payment_response.razorpay_payment_id.clone(),
                );
                if let Some(next_action) = payment_response.next {
                    for action in next_action {
                        form_fields.insert("action".to_string(), action.action);
                        form_fields.insert("url".to_string(), action.url);
                    }
                }
                let payment_response_data = PaymentsResponseData::TransactionResponse {
                    resource_id: ResponseId::ConnectorTransactionId(
                        payment_response.razorpay_payment_id.clone(),
                    ),
                    redirection_data: *Box::new(Some(RedirectForm::Form {
                        endpoint: redirect_url,
                        method: Method::Get,
                        form_fields,
                    })),
                    mandate_reference: None,
                    connector_metadata: None,
                    network_txn_id: None,
                    connector_response_reference_id: None,
                    incremental_authorization_allowed: None,
                    charge_id: None,
                };
                let error = None;

                Ok(Self {
                    response: error.map_or_else(|| Ok(payment_response_data), Err),
                    resource_common_data: PaymentFlowData {
                        status,
                        ..data.resource_common_data
                    },
                    ..data
                })
            }
            RazorpayResponse::PsyncResponse(psync_response) => {
                let status =
                    get_psync_razorpay_payment_status(is_manual_capture, psync_response.status);
                let psync_response_data = PaymentsResponseData::TransactionResponse {
                    resource_id: ResponseId::ConnectorTransactionId(
                        psync_response.id,
                    ),
                    redirection_data: None,
                    mandate_reference: None,
                    connector_metadata: None,
                    network_txn_id: None,
                    connector_response_reference_id: None,
                    incremental_authorization_allowed: None,
                    charge_id: None,
                };
                let error = None;

                Ok(Self {
                    response: error.map_or_else(|| Ok(psync_response_data), Err),
                    resource_common_data: PaymentFlowData {
                        status,
                        ..data.resource_common_data
                    },
                    ..data
                })
            }
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RazorpayErrorResponse {
    pub status: i32,
    pub error_code: String,
    pub message: String,
    pub error_type: String,
    pub psp_reference: Option<String>,
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RazorpayOrderRequest {
    pub amount: MinorUnit,
    pub currency: String,
    pub receipt: String,
    // pub partial_payment: Option<bool>,
    // pub first_payment_min_amount: Option<MinorUnit>,
    // pub notes: Option<HashMap<String, String>>,
}

impl
    TryFrom<
        &RazorpayRouterData<
            &RouterDataV2<
                CreateOrder,
                PaymentFlowData,
                PaymentCreateOrderData,
                PaymentCreateOrderResponse,
            >,
        >,
    > for RazorpayOrderRequest
{
    type Error = hyperswitch_interfaces::errors::ConnectorError;

    fn try_from(
        item: &RazorpayRouterData<
            &RouterDataV2<
                CreateOrder,
                PaymentFlowData,
                PaymentCreateOrderData,
                PaymentCreateOrderResponse,
            >,
        >,
    ) -> Result<Self, Self::Error> {
        let request_data = &item.router_data.request;

        Ok(RazorpayOrderRequest {
            amount: item.amount,
            currency: request_data.currency.to_string(),
            receipt: uuid::Uuid::new_v4().to_string(),
            // partial_payment: Some(false),
            // first_payment_min_amount: None,
            // notes: None,
        })
    }
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RazorpayOrderResponse {
    pub id: String,
    // pub entity: String,
    // pub amount: MinorUnit,
    // pub amount_paid: MinorUnit,
    // pub amount_due: MinorUnit,
    // pub currency: String,
    // pub receipt: String,
    // pub status: String,
    // pub attempts: u32,
    // pub notes: Option<HashMap<String, String>>,
    // pub created_at: u64,
}

impl
    ForeignTryFrom<(
        RazorpayOrderResponse,
        RouterDataV2<
            CreateOrder,
            PaymentFlowData,
            PaymentCreateOrderData,
            PaymentCreateOrderResponse,
        >,
        u16,
        bool,
    )> for RouterDataV2<
        CreateOrder,
        PaymentFlowData,
        PaymentCreateOrderData,
        PaymentCreateOrderResponse,
    >
{
    type Error = hyperswitch_interfaces::errors::ConnectorError;

    fn foreign_try_from(
        (response, data, _status_code, _): (
            RazorpayOrderResponse,
            RouterDataV2<
                CreateOrder,
                PaymentFlowData,
                PaymentCreateOrderData,
                PaymentCreateOrderResponse,
            >,
            u16,
            bool,
        ),
    ) -> Result<Self, Self::Error> {
        let order_response = PaymentCreateOrderResponse {
            order_id: response.id,
        };

        Ok(Self {
            response: Ok(order_response),
            ..data
        })
    }
}


