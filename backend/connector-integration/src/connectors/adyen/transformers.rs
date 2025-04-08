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
    payment_method_data::PaymentMethodData,
    router_data::RouterData,
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
pub struct Amount {
    pub currency: storage_enums::Currency,
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
pub struct AdyenPaymentRequest<'a> {
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


impl TryFrom<&AdyenRouterData<&PaymentsAuthorizeRouterData>> for AdyenPaymentRequest<'_> {
    type Error = Error;
    fn try_from(item: &AdyenRouterData<&PaymentsAuthorizeRouterData>) -> Result<Self, Self::Error> {
        match item
            .router_data
            .request
            .mandate_id
            .to_owned()
            .and_then(|mandate_ids| mandate_ids.mandate_reference_id)
        {
            Some(mandate_ref) => AdyenPaymentRequest::try_from((item, mandate_ref)),
            None => match item.router_data.request.payment_method_data {
                PaymentMethodData::Card(ref card) => AdyenPaymentRequest::try_from((item, card)),
                PaymentMethodData::Wallet(ref wallet) => {
                    AdyenPaymentRequest::try_from((item, wallet))
                }
                PaymentMethodData::PayLater(ref pay_later) => {
                    AdyenPaymentRequest::try_from((item, pay_later))
                }
                PaymentMethodData::BankRedirect(ref bank_redirect) => {
                    AdyenPaymentRequest::try_from((item, bank_redirect))
                }
                PaymentMethodData::BankDebit(ref bank_debit) => {
                    AdyenPaymentRequest::try_from((item, bank_debit))
                }
                PaymentMethodData::BankTransfer(ref bank_transfer) => {
                    AdyenPaymentRequest::try_from((item, bank_transfer.as_ref()))
                }
                PaymentMethodData::CardRedirect(ref card_redirect_data) => {
                    AdyenPaymentRequest::try_from((item, card_redirect_data))
                }
                PaymentMethodData::Voucher(ref voucher_data) => {
                    AdyenPaymentRequest::try_from((item, voucher_data))
                }
                PaymentMethodData::GiftCard(ref gift_card_data) => {
                    AdyenPaymentRequest::try_from((item, gift_card_data.as_ref()))
                }
                PaymentMethodData::NetworkToken(ref token_data) => {
                    AdyenPaymentRequest::try_from((item, token_data))
                }
                PaymentMethodData::Crypto(_)
                | PaymentMethodData::MandatePayment
                | PaymentMethodData::Reward
                | PaymentMethodData::RealTimePayment(_)
                | PaymentMethodData::MobilePayment(_)
                | PaymentMethodData::Upi(_)
                | PaymentMethodData::OpenBanking(_)
                | PaymentMethodData::CardToken(_)
                | PaymentMethodData::CardDetailsForNetworkTransactionId(_) => {
                    Err(errors::ConnectorError::NotImplemented(
                        utils::get_unimplemented_payment_method_error_message("Adyen"),
                    ))?
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

#[router_derive::diesel_enum(storage_type = "text")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PaymentMethodType {
    Ach,
    Affirm,
    AfterpayClearpay,
    Alfamart,
    AliPay,
    AliPayHk,
    Alma,
    AmazonPay,
    ApplePay,
    Atome,
    Bacs,
    BancontactCard,
    Becs,
    Benefit,
    Bizum,
    Blik,
    Boleto,
    BcaBankTransfer,
    BniVa,
    BriVa,
    CardRedirect,
    CimbVa,
    #[serde(rename = "classic")]
    ClassicReward,
    Credit,
    CryptoCurrency,
    Cashapp,
    Dana,
    DanamonVa,
    Debit,
    DuitNow,
    Efecty,
    Eft,
    Eps,
    Fps,
    Evoucher,
    Giropay,
    Givex,
    GooglePay,
    GoPay,
    Gcash,
    Ideal,
    Interac,
    Indomaret,
    Klarna,
    KakaoPay,
    LocalBankRedirect,
    MandiriVa,
    Knet,
    MbWay,
    MobilePay,
    Momo,
    MomoAtm,
    Multibanco,
    OnlineBankingThailand,
    OnlineBankingCzechRepublic,
    OnlineBankingFinland,
    OnlineBankingFpx,
    OnlineBankingPoland,
    OnlineBankingSlovakia,
    Oxxo,
    PagoEfectivo,
    PermataBankTransfer,
    OpenBankingUk,
    PayBright,
    Paypal,
    Paze,
    Pix,
    PaySafeCard,
    Przelewy24,
    PromptPay,
    Pse,
    RedCompra,
    RedPagos,
    SamsungPay,
    Sepa,
    SepaBankTransfer,
    Sofort,
    Swish,
    TouchNGo,
    Trustly,
    Twint,
    UpiCollect,
    UpiIntent,
    Vipps,
    VietQr,
    Venmo,
    Walley,
    WeChatPay,
    SevenEleven,
    Lawson,
    MiniStop,
    FamilyMart,
    Seicomart,
    PayEasy,
    LocalBankTransfer,
    Mifinity,
    #[serde(rename = "open_banking_pis")]
    OpenBankingPIS,
    DirectCarrierBilling,
    InstantBankTransfer,
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
        AdyenStatus::Pending => match pmt {
            Some(PaymentMethodType::Pix) => {
                AttemptStatus::AuthenticationPending
            }
            _ => AttemptStatus::Pending,
        },
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
    type Error = Error;
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