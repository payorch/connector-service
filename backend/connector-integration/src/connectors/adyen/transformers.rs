use hyperswitch_api_models::{
    enums,
};

use hyperswitch_common_utils::{
    errors,
    types::MinorUnit,
};
// use error_stack::{report, ResultExt};
use hyperswitch_domain_models::{
    // network_tokenization::NetworkTokenNumber,
    payment_method_data::{Card, PaymentMethodData},
    router_data::{ConnectorAuthType, RouterData},
    router_request_types::ResponseId,
    router_response_types::PaymentsResponseData,
    types::{
        PaymentsAuthorizeRouterData, PaymentsCancelRouterData, PaymentsCaptureRouterData,
        RefundsRouterData,
    },
};

use hyperswitch_interfaces::{
    consts,
};
use hyperswitch_masking::{ExposeInterface, PeekInterface, Secret};
use serde::{Deserialize, Serialize};
// use time::{Duration, OffsetDateTime, PrimitiveDateTime};
// use url::Url;


// use crate::{
//     types::{
//         AcceptDisputeRouterData, DefendDisputeRouterData, PaymentsCancelResponseRouterData,
//         PaymentsCaptureResponseRouterData, RefundsResponseRouterData, ResponseRouterData,
//         SubmitEvidenceRouterData,
//     },
//     utils::{
//         self, is_manual_capture, missing_field_err, AddressDetailsData, BrowserInformationData,
//         CardData, ForeignTryFrom, NetworkTokenData as UtilsNetworkTokenData,
//         PaymentsAuthorizeRequestData, PhoneDetailsData, RouterData as OtherRouterData,
//     },
// };

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
    MC,
    Amex,
    Argencard,
    Bcmc,
    Bijcard,
    Cabal,
    Cartebancaire,
    Codensa,
    Cup,
    Dankort,
    Diners,
    Discover,
    Electron,
    Elo,
    Forbrugsforeningen,
    Hiper,
    Hipercard,
    Jcb,
    Karenmillen,
    Laser,
    Maestro,
    Maestrouk,
    Mcalphabankbonus,
    Mir,
    Naranja,
    Oasis,
    Rupay,
    Shopping,
    Solo,
    Troy,
    Uatp,
    Visaalphabankbonus,
    Visadankort,
    Warehouse,
}

#[derive(Debug, PartialEq)]
pub enum ConnectorError {
    ParsingFailed,
    NotImplemented,
    FailedToObtainAuthType,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdyenCard {
    number: String,
    expiry_month: Secret<String>,
    expiry_year: Secret<String>,
    cvc: Option<Secret<String>>,
    holder_name: Option<Secret<String>>,
    brand: Option<CardBrand>, //Mandatory for mandate using network_txns_id
    network_payment_reference: Option<Secret<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum AdyenPaymentMethod {
    #[serde(rename = "scheme")]
    AdyenCard(Box<AdyenCard>),
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdyenBrowserInfo {
    user_agent: String,
    accept_header: String,
    language: String,
    color_depth: u8,
    screen_height: u32,
    screen_width: u32,
    time_zone_offset: i32,
    java_enabled: bool,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum AuthType {
    #[default]
    PreAuth,
}

#[serde_with::skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    AdyenPaymentMethod(Box<AdyenPaymentMethod>),
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdyenPaymentRequest {
    amount: Amount,
    merchant_account: Secret<String>,
    payment_method: PaymentMethod,
    reference: String,
    return_url: String,
    browser_info: Option<AdyenBrowserInfo>,
    billing_address: Option<Address>,
    country_code: Option<enums::CountryAlpha2>,
}

#[derive(Debug, Serialize)]
pub struct AdyenRouterData<T> {
    pub amount: MinorUnit,
    pub router_data: T,
}

impl<T> TryFrom<(MinorUnit, T)> for AdyenRouterData<T> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from((amount, item): (MinorUnit, T)) -> Result<Self, Self::Error> {
        Ok(Self {
            amount,
            router_data: item,
        })
    }
}

fn get_amount_data(item: &AdyenRouterData<&PaymentsAuthorizeRouterData>) -> Amount {
    Amount {
        currency: item.router_data.request.currency,
        value: item.amount.to_owned(),
    }
}

pub struct AdyenAuthType {
    pub(super) api_key: Secret<String>,
    pub(super) merchant_account: Secret<String>,
    #[allow(dead_code)]
    pub(super) review_key: Option<Secret<String>>,
}

impl TryFrom<&ConnectorAuthType> for AdyenAuthType {
    type Error = ConnectorError;
    fn try_from(auth_type: &ConnectorAuthType) -> Result<Self, Self::Error> {
        match auth_type {
            ConnectorAuthType::BodyKey { api_key, key1 } => Ok(Self {
                api_key: api_key.to_owned(),
                merchant_account: key1.to_owned(),
                review_key: None,
            }),
            ConnectorAuthType::SignatureKey {
                api_key,
                key1,
                api_secret,
            } => Ok(Self {
                api_key: api_key.to_owned(),
                merchant_account: key1.to_owned(),
                review_key: Some(api_secret.to_owned()),
            }),
            _ => Err(ConnectorError::FailedToObtainAuthType)?,
        }
    }
}

impl TryFrom<(&AdyenRouterData<&PaymentsAuthorizeRouterData>, &Card)> for AdyenPaymentRequest {
    type Error = ConnectorError;
    fn try_from(
        value: (&AdyenRouterData<&PaymentsAuthorizeRouterData>, &Card),
    ) -> Result<Self, Self::Error> {
        let (item, card_data) = value;
        let amount = get_amount_data(item);
        let auth_type = AdyenAuthType::try_from(&item.router_data.connector_auth_type)?;
        let browser_info = None;
        let billing_address = Address {
            city: "New York".to_string(),
            country: enums::CountryAlpha2::US,
            house_number_or_name: "1234".to_string().into(),
            postal_code: "123456".to_string().into(),
            state_or_province: Some("California".to_string().into()),
            street: Some(Secret<String>),
        }
        let country_code = get_country_code(item.router_data.get_optional_billing());
        let return_url = "www.google.com".to_string();
        let card_holder_name = "Sagnik Mitra";
        let payment_method = PaymentMethod::AdyenPaymentMethod(Box::new(
            AdyenPaymentMethod::try_from((card_data, card_holder_name))?,
        ));

        Ok(AdyenPaymentRequest {
            amount,
            merchant_account: auth_type.merchant_account,
            payment_method,
            reference: item.router_data.connector_request_reference_id.clone(),
            return_url,
            browser_info: None,
            billing_address,
            country_code,
        })
    }
}


impl TryFrom<&AdyenRouterData<&PaymentsAuthorizeRouterData>> for AdyenPaymentRequest {
    type Error = ConnectorError;
    fn try_from(item: &AdyenRouterData<&PaymentsAuthorizeRouterData>) -> Result<Self, Self::Error> {
        match item
            .router_data
            .request
            .mandate_id
            .to_owned()
            .and_then(|mandate_ids| mandate_ids.mandate_reference_id)
        {
            Some(mandate_ref) => Err(ConnectorError::NotImplemented)?,
            None => match item.router_data.request.payment_method_data {
                PaymentMethodData::Card(ref card) => AdyenPaymentRequest::try_from((item, card)),
                | PaymentMethodData::Wallet(_)
                | PaymentMethodData::PayLater(_)
                | PaymentMethodData::BankRedirect(_)
                | PaymentMethodData::BankDebit(_)
                | PaymentMethodData::BankTransfer(_)
                | PaymentMethodData::CardRedirect(_)
                | PaymentMethodData::Voucher(_)
                | PaymentMethodData::GiftCard(_)
                | PaymentMethodData::Crypto(_)
                | PaymentMethodData::MandatePayment
                | PaymentMethodData::Reward
                | PaymentMethodData::RealTimePayment(_)
                | PaymentMethodData::Upi(_)
                | PaymentMethodData::OpenBanking(_)
                | PaymentMethodData::CardToken(_) => {
                    Err(ConnectorError::NotImplemented)?
                }
            },
        }
    }
}

pub struct ResponseRouterData<Flow, R, Request, Response> {
    pub response: R,
    pub data: RouterData<Flow, Request, Response>,
    pub http_code: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdyenPaymentResponse {
    psp_reference: String,
    result_code: AdyenStatus,
    amount: Option<Amount>,
    merchant_reference: String,
    refusal_reason: Option<String>,
    refusal_reason_code: Option<String>,
    store: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdyenStatus {
    AuthenticationFinished,
    AuthenticationNotRequired,
    Authorised,
    Cancelled,
    ChallengeShopper,
    Error,
    Pending,
    Received,
    RedirectShopper,
    Refused,
    PresentToShopper,
    #[cfg(feature = "payouts")]
    #[serde(rename = "[payout-confirm-received]")]
    PayoutConfirmReceived,
    #[cfg(feature = "payouts")]
    #[serde(rename = "[payout-decline-received]")]
    PayoutDeclineReceived,
    #[cfg(feature = "payouts")]
    #[serde(rename = "[payout-submit-received]")]
    PayoutSubmitReceived,
}

#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CaptureMethod {
    /// Post the payment authorization, the capture will be executed on the full amount immediately
    #[default]
    Automatic,
    /// The capture will happen only if the merchant triggers a Capture API request
    Manual,
    /// The capture will happen only if the merchant triggers a Capture API request
    ManualMultiple,
    /// The capture can be scheduled to automatically get triggered at a specific date & time
    Scheduled,
    /// Handles separate auth and capture sequentially; same as `Automatic` for most connectors.
    SequentialAutomatic,
}

#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PaymentMethodType {
    Credit,
}

pub trait ForeignTryFrom<F>: Sized {
    type Error;

    fn foreign_try_from(from: F) -> Result<Self, Self::Error>;
}

#[router_derive::diesel_enum(storage_type = "db_enum")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AttemptStatus {
    Started,
    AuthenticationFailed,
    RouterDeclined,
    AuthenticationPending,
    AuthenticationSuccessful,
    Authorized,
    AuthorizationFailed,
    Charged,
    Authorizing,
    CodInitiated,
    Voided,
    VoidInitiated,
    CaptureInitiated,
    CaptureFailed,
    VoidFailed,
    AutoRefunded,
    PartialCharged,
    PartialChargedAndChargeable,
    Unresolved,
    #[default]
    Pending,
    Failure,
    PaymentMethodAwaited,
    ConfirmationAwaited,
    DeviceDataCollectionPending,
}

fn get_adyen_payment_status(
    is_manual_capture: bool,
    adyen_status: AdyenStatus,
    pmt: Option<PaymentMethodType>,
) -> AttemptStatus {
    match adyen_status {
        AdyenStatus::AuthenticationFinished => {
            AttemptStatus::AuthenticationSuccessful
        }
        AdyenStatus::AuthenticationNotRequired | AdyenStatus::Received => {
            AttemptStatus::Pending
        }
        AdyenStatus::Authorised => match is_manual_capture {
            true => AttemptStatus::Authorized,
            // In case of Automatic capture Authorized is the final status of the payment
            false => AttemptStatus::Charged,
        },
        AdyenStatus::Cancelled => AttemptStatus::Voided,
        AdyenStatus::ChallengeShopper
        | AdyenStatus::RedirectShopper
        | AdyenStatus::PresentToShopper => AttemptStatus::AuthenticationPending,
        AdyenStatus::Error | AdyenStatus::Refused => AttemptStatus::Failure,
        AdyenStatus::Pending => AttemptStatus::Pending,
        #[cfg(feature = "payouts")]
        AdyenStatus::PayoutConfirmReceived => AttemptStatus::Started,
        #[cfg(feature = "payouts")]
        AdyenStatus::PayoutSubmitReceived => AttemptStatus::Pending,
        #[cfg(feature = "payouts")]
        AdyenStatus::PayoutDeclineReceived => AttemptStatus::Voided,
    }
}

impl<F, Req>
    ForeignTryFrom<(
        ResponseRouterData<F, AdyenPaymentResponse, Req, PaymentsResponseData>,
        Option<CaptureMethod>,
        bool,
        Option<PaymentMethodType>,
    )> for RouterData<F, Req, PaymentsResponseData>
{
    type Error = ConnectorError;
    fn foreign_try_from(
        (item, capture_method, is_multiple_capture_psync_flow, pmt): (
            ResponseRouterData<F, AdyenPaymentResponse, Req, PaymentsResponseData>,
            Option<CaptureMethod>,
            bool,
            Option<PaymentMethodType>,
        ),
    ) -> Result<Self, Self::Error> {
        let is_manual_capture = false;
        let status = get_adyen_payment_status(is_manual_capture,item.response.result_code,pmt);
        let payment_response_data = PaymentsResponseData::TransactionResponse {
            resource_id: ResponseId::ConnectorTransactionId(item.response.psp_reference),
            redirection_data: Box::new(None),
            mandate_reference: Box::new(None),
            connector_metadata: None,
            network_txn_id: None,
            connector_response_reference_id: Some(item.response.merchant_reference),
            incremental_authorization_allowed: None,
            charges: None,
        };
        let error=None;

        Ok(Self {
            status,
            response: error.map_or_else(|| Ok(payment_response_data), Err),
            ..item.data
        })
    }
}