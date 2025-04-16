use hyperswitch_api_models::enums::{self, AttemptStatus};

use hyperswitch_common_utils::{errors::CustomResult, request::Method, types::MinorUnit};

use hyperswitch_domain_models::{
    payment_method_data::{Card, PaymentMethodData},
    router_data::{ConnectorAuthType, ErrorResponse, RouterData},
    router_data_v2::{PaymentFlowData, RouterDataV2},
    router_flow_types::Authorize,
    router_request_types::{PaymentsAuthorizeData, ResponseId},
    router_response_types::{MandateReference, PaymentsResponseData, RedirectForm},
};
use hyperswitch_interfaces::{
    consts::{NO_ERROR_CODE, NO_ERROR_MESSAGE},
    errors,
};
use hyperswitch_masking::{ExposeInterface, Secret};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use url::Url;

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

type Error = error_stack::Report<hyperswitch_interfaces::errors::ConnectorError>;

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
#[serde(rename_all = "camelCase")]
pub struct AdyenCard {
    number: hyperswitch_cards::CardNumber,
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
struct AdyenMpiData {
    directory_response: String,
    authentication_response: String,
    token_authentication_verification_value: Secret<String>,
    eci: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub enum AdyenShopperInteraction {
    #[default]
    Ecommerce,
    #[serde(rename = "ContAuth")]
    ContinuedAuthentication,
    Moto,
    #[serde(rename = "POS")]
    Pos,
}

impl From<&RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>>
    for AdyenShopperInteraction
{
    fn from(
        item: &RouterDataV2<
            Authorize,
            PaymentFlowData,
            PaymentsAuthorizeData,
            PaymentsResponseData,
        >,
    ) -> Self {
        match item.request.off_session {
            Some(true) => Self::ContinuedAuthentication,
            _ => Self::Ecommerce,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AdyenRecurringModel {
    UnscheduledCardOnFile,
    CardOnFile,
}

#[serde_with::skip_serializing_none]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalData {
    authorisation_type: Option<AuthType>,
    manual_capture: Option<String>,
    execute_three_d: Option<String>,
    pub recurring_processing_model: Option<AdyenRecurringModel>,
    /// Enable recurring details in dashboard to receive this ID, https://docs.adyen.com/online-payments/tokenization/create-and-use-tokens#test-and-go-live
    #[serde(rename = "recurring.recurringDetailReference")]
    recurring_detail_reference: Option<Secret<String>>,
    #[serde(rename = "recurring.shopperReference")]
    recurring_shopper_reference: Option<String>,
    network_tx_reference: Option<Secret<String>>,
    funds_availability: Option<String>,
    refusal_reason_raw: Option<String>,
    refusal_code_raw: Option<String>,
    merchant_advice_code: Option<String>,
    #[serde(flatten)]
    riskdata: Option<RiskData>,
}

#[serde_with::skip_serializing_none]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskData {
    #[serde(rename = "riskdata.basket.item1.itemID")]
    item_i_d: Option<String>,
    #[serde(rename = "riskdata.basket.item1.productTitle")]
    product_title: Option<String>,
    #[serde(rename = "riskdata.basket.item1.amountPerItem")]
    amount_per_item: Option<String>,
    #[serde(rename = "riskdata.basket.item1.currency")]
    currency: Option<String>,
    #[serde(rename = "riskdata.basket.item1.upc")]
    upc: Option<String>,
    #[serde(rename = "riskdata.basket.item1.brand")]
    brand: Option<String>,
    #[serde(rename = "riskdata.basket.item1.manufacturer")]
    manufacturer: Option<String>,
    #[serde(rename = "riskdata.basket.item1.category")]
    category: Option<String>,
    #[serde(rename = "riskdata.basket.item1.quantity")]
    quantity: Option<String>,
    #[serde(rename = "riskdata.basket.item1.color")]
    color: Option<String>,
    #[serde(rename = "riskdata.basket.item1.size")]
    size: Option<String>,
    #[serde(rename = "riskdata.deviceCountry")]
    device_country: Option<String>,
    #[serde(rename = "riskdata.houseNumberorName")]
    house_numberor_name: Option<String>,
    #[serde(rename = "riskdata.accountCreationDate")]
    account_creation_date: Option<String>,
    #[serde(rename = "riskdata.affiliateChannel")]
    affiliate_channel: Option<String>,
    #[serde(rename = "riskdata.avgOrderValue")]
    avg_order_value: Option<String>,
    #[serde(rename = "riskdata.deliveryMethod")]
    delivery_method: Option<String>,
    #[serde(rename = "riskdata.emailName")]
    email_name: Option<String>,
    #[serde(rename = "riskdata.emailDomain")]
    email_domain: Option<String>,
    #[serde(rename = "riskdata.lastOrderDate")]
    last_order_date: Option<String>,
    #[serde(rename = "riskdata.merchantReference")]
    merchant_reference: Option<String>,
    #[serde(rename = "riskdata.paymentMethod")]
    payment_method: Option<String>,
    #[serde(rename = "riskdata.promotionName")]
    promotion_name: Option<String>,
    #[serde(rename = "riskdata.secondaryPhoneNumber")]
    secondary_phone_number: Option<String>,
    #[serde(rename = "riskdata.timefromLogintoOrder")]
    timefrom_loginto_order: Option<String>,
    #[serde(rename = "riskdata.totalSessionTime")]
    total_session_time: Option<String>,
    #[serde(rename = "riskdata.totalAuthorizedAmountInLast30Days")]
    total_authorized_amount_in_last30_days: Option<String>,
    #[serde(rename = "riskdata.totalOrderQuantity")]
    total_order_quantity: Option<String>,
    #[serde(rename = "riskdata.totalLifetimeValue")]
    total_lifetime_value: Option<String>,
    #[serde(rename = "riskdata.visitsMonth")]
    visits_month: Option<String>,
    #[serde(rename = "riskdata.visitsWeek")]
    visits_week: Option<String>,
    #[serde(rename = "riskdata.visitsYear")]
    visits_year: Option<String>,
    #[serde(rename = "riskdata.shipToName")]
    ship_to_name: Option<String>,
    #[serde(rename = "riskdata.first8charactersofAddressLine1Zip")]
    first8charactersof_address_line1_zip: Option<String>,
    #[serde(rename = "riskdata.affiliateOrder")]
    affiliate_order: Option<bool>,
}

#[serde_with::skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShopperName {
    first_name: Option<Secret<String>>,
    last_name: Option<Secret<String>>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineItem {
    amount_excluding_tax: Option<MinorUnit>,
    amount_including_tax: Option<MinorUnit>,
    description: Option<String>,
    id: Option<String>,
    tax_amount: Option<MinorUnit>,
    quantity: Option<u16>,
}

#[derive(Debug, Clone, Serialize)]
pub enum Channel {
    Web,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdyenSplitData {
    amount: Option<Amount>,
    #[serde(rename = "type")]
    split_type: AdyenSplitType,
    account: Option<String>,
    reference: String,
    description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentType {
    Affirm,
    Afterpaytouch,
    Alipay,
    #[serde(rename = "alipay_hk")]
    AlipayHk,
    #[serde(rename = "doku_alfamart")]
    Alfamart,
    Alma,
    Applepay,
    Bizum,
    Atome,
    Blik,
    #[serde(rename = "boletobancario")]
    BoletoBancario,
    ClearPay,
    Dana,
    Eps,
    Gcash,
    Googlepay,
    #[serde(rename = "gopay_wallet")]
    GoPay,
    Ideal,
    #[serde(rename = "doku_indomaret")]
    Indomaret,
    Klarna,
    Kakaopay,
    Mbway,
    MobilePay,
    #[serde(rename = "momo_wallet")]
    Momo,
    #[serde(rename = "momo_atm")]
    MomoAtm,
    #[serde(rename = "onlineBanking_CZ")]
    OnlineBankingCzechRepublic,
    #[serde(rename = "ebanking_FI")]
    OnlineBankingFinland,
    #[serde(rename = "onlineBanking_PL")]
    OnlineBankingPoland,
    #[serde(rename = "onlineBanking_SK")]
    OnlineBankingSlovakia,
    #[serde(rename = "molpay_ebanking_fpx_MY")]
    OnlineBankingFpx,
    #[serde(rename = "molpay_ebanking_TH")]
    OnlineBankingThailand,
    #[serde(rename = "paybybank")]
    OpenBankingUK,
    #[serde(rename = "oxxo")]
    Oxxo,
    #[serde(rename = "paysafecard")]
    PaySafeCard,
    PayBright,
    Paypal,
    Scheme,
    #[serde(rename = "networkToken")]
    NetworkToken,
    #[serde(rename = "trustly")]
    Trustly,
    #[serde(rename = "touchngo")]
    TouchNGo,
    Walley,
    #[serde(rename = "wechatpayWeb")]
    WeChatPayWeb,
    #[serde(rename = "ach")]
    AchDirectDebit,
    SepaDirectDebit,
    #[serde(rename = "directdebit_GB")]
    BacsDirectDebit,
    Samsungpay,
    Twint,
    Vipps,
    Giftcard,
    Knet,
    Benefit,
    Swish,
    #[serde(rename = "doku_permata_lite_atm")]
    PermataBankTransfer,
    #[serde(rename = "doku_bca_va")]
    BcaBankTransfer,
    #[serde(rename = "doku_bni_va")]
    BniVa,
    #[serde(rename = "doku_bri_va")]
    BriVa,
    #[serde(rename = "doku_cimb_va")]
    CimbVa,
    #[serde(rename = "doku_danamon_va")]
    DanamonVa,
    #[serde(rename = "doku_mandiri_va")]
    MandiriVa,
    #[serde(rename = "econtext_seven_eleven")]
    SevenEleven,
    #[serde(rename = "econtext_stores")]
    Lawson,
}

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
    strum::Display,
    strum::EnumString,
)]
#[strum(serialize_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
pub enum AdyenSplitType {
    /// Books split amount to the specified account.
    BalanceAccount,
    /// The aggregated amount of the interchange and scheme fees.
    AcquiringFees,
    /// The aggregated amount of all transaction fees.
    PaymentFee,
    /// The aggregated amount of Adyen's commission and markup fees.
    AdyenFees,
    ///  The transaction fees due to Adyen under blended rates.
    AdyenCommission,
    /// The transaction fees due to Adyen under Interchange ++ pricing.
    AdyenMarkup,
    ///  The fees paid to the issuer for each payment made with the card network.
    Interchange,
    ///  The fees paid to the card scheme for using their network.
    SchemeFee,
    /// Your platform's commission on the payment (specified in amount), booked to your liable balance account.
    Commission,
    /// Allows you and your users to top up balance accounts using direct debit, card payments, or other payment methods.
    TopUp,
    /// The value-added tax charged on the payment, booked to your platforms liable balance account.
    Vat,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdyenPaymentRequest {
    amount: Amount,
    merchant_account: Secret<String>,
    payment_method: PaymentMethod,
    mpi_data: Option<AdyenMpiData>,
    reference: String,
    return_url: String,
    browser_info: Option<AdyenBrowserInfo>,
    shopper_interaction: AdyenShopperInteraction,
    recurring_processing_model: Option<AdyenRecurringModel>,
    additional_data: Option<AdditionalData>,
    shopper_reference: Option<String>,
    store_payment_method: Option<bool>,
    shopper_name: Option<ShopperName>,
    #[serde(rename = "shopperIP")]
    shopper_ip: Option<Secret<String, hyperswitch_common_utils::pii::IpAddress>>,
    shopper_locale: Option<String>,
    shopper_email: Option<hyperswitch_common_utils::pii::Email>,
    shopper_statement: Option<String>,
    social_security_number: Option<Secret<String>>,
    telephone_number: Option<Secret<String>>,
    billing_address: Option<Address>,
    delivery_address: Option<Address>,
    country_code: Option<enums::CountryAlpha2>,
    line_items: Option<Vec<LineItem>>,
    channel: Option<Channel>,
    merchant_order_reference: Option<String>,
    splits: Option<Vec<AdyenSplitData>>,
    store: Option<String>,
    device_fingerprint: Option<Secret<String>>,
}

#[derive(Debug, Serialize)]
pub struct AdyenRouterData<T> {
    pub amount: MinorUnit,
    pub router_data: T,
}

impl<T> TryFrom<(MinorUnit, T)> for AdyenRouterData<T> {
    type Error = hyperswitch_interfaces::errors::ConnectorError;
    fn try_from((amount, item): (MinorUnit, T)) -> Result<Self, Self::Error> {
        Ok(Self {
            amount,
            router_data: item,
        })
    }
}

fn get_amount_data(
    item: &AdyenRouterData<
        &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
    >,
) -> Amount {
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
    type Error = hyperswitch_interfaces::errors::ConnectorError;
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
            _ => Err(hyperswitch_interfaces::errors::ConnectorError::FailedToObtainAuthType),
        }
    }
}

impl TryFrom<(&Card, Option<Secret<String>>)> for AdyenPaymentMethod {
    type Error = hyperswitch_interfaces::errors::ConnectorError;
    fn try_from(
        (card, card_holder_name): (&Card, Option<Secret<String>>),
    ) -> Result<Self, Self::Error> {
        let adyen_card = AdyenCard {
            number: card.card_number.clone(),
            expiry_month: card.card_exp_month.clone(),
            expiry_year: card.card_exp_year.clone(),
            cvc: Some(card.card_cvc.clone()),
            holder_name: card_holder_name,
            brand: Some(CardBrand::Visa),
            network_payment_reference: None,
        };
        Ok(AdyenPaymentMethod::AdyenCard(Box::new(adyen_card)))
    }
}

impl
    TryFrom<(
        &AdyenRouterData<
            &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
        >,
        &Card,
    )> for AdyenPaymentRequest
{
    type Error = Error;
    fn try_from(
        value: (
            &AdyenRouterData<
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
        let amount = get_amount_data(item);
        let auth_type = AdyenAuthType::try_from(&item.router_data.connector_auth_type)?;
        let shopper_interaction = AdyenShopperInteraction::from(item.router_data);
        let shopper_reference = build_shopper_reference(
            &item.router_data.customer_id,
            item.router_data.merchant_id.clone(),
        );
        let (recurring_processing_model, store_payment_method, _) =
            get_recurring_processing_model(item.router_data)?;

        let return_url = item
            .router_data
            .request
            .router_return_url
            .clone()
            .ok_or_else(Box::new(move || {
                errors::ConnectorError::MissingRequiredField {
                    field_name: "return_url",
                }
            }))?;

        let billing_address = get_address_info(
            item.router_data
                .resource_common_data
                .address
                .get_payment_billing(),
        )
        .and_then(Result::ok);

        let card_holder_name = item.router_data.request.customer_name.clone();

        let additional_data = get_additional_data(item.router_data);

        let payment_method = PaymentMethod::AdyenPaymentMethod(Box::new(
            AdyenPaymentMethod::try_from((card_data, card_holder_name))?,
        ));

        Ok(AdyenPaymentRequest {
            amount,
            merchant_account: auth_type.merchant_account,
            payment_method,
            reference: item.router_data.connector_request_reference_id.clone(),
            return_url,
            shopper_interaction,
            recurring_processing_model,
            browser_info: None,
            additional_data,
            mpi_data: None,
            telephone_number: None,
            shopper_name: None,
            shopper_email: None,
            shopper_locale: None,
            social_security_number: None,
            billing_address,
            delivery_address: None,
            country_code: None,
            line_items: None,
            shopper_reference,
            store_payment_method,
            channel: None,
            shopper_statement: item.router_data.request.statement_descriptor.clone(),
            shopper_ip: None,
            merchant_order_reference: item.router_data.request.merchant_order_reference_id.clone(),
            store: None,
            splits: None,
            device_fingerprint: None,
        })
    }
}

impl
    TryFrom<
        &AdyenRouterData<
            &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
        >,
    > for AdyenPaymentRequest
{
    type Error = Error;
    fn try_from(
        item: &AdyenRouterData<
            &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
        >,
    ) -> Result<Self, Self::Error> {
        match item
            .router_data
            .request
            .mandate_id
            .to_owned()
            .and_then(|mandate_ids| mandate_ids.mandate_reference_id)
        {
            Some(_mandate_ref) => Err(
                hyperswitch_interfaces::errors::ConnectorError::NotImplemented(
                    "payment_method".into(),
                ),
            )?,
            None => match item.router_data.request.payment_method_data {
                PaymentMethodData::Card(ref card) => AdyenPaymentRequest::try_from((item, card)),
                PaymentMethodData::Wallet(_)
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
                | PaymentMethodData::CardToken(_) => Err(
                    hyperswitch_interfaces::errors::ConnectorError::NotImplemented(
                        "payment method".into(),
                    ),
                )?,
            },
        }
    }
}

pub struct ResponseRouterData<Flow, R, Request, Response> {
    pub response: R,
    pub data: RouterData<Flow, Request, Response>,
    pub http_code: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AdyenPaymentResponse {
    Response(Box<AdyenResponse>),
    RedirectionResponse(Box<RedirectionResponse>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdyenResponse {
    psp_reference: String,
    result_code: AdyenStatus,
    amount: Option<Amount>,
    merchant_reference: String,
    refusal_reason: Option<String>,
    refusal_reason_code: Option<String>,
    additional_data: Option<AdditionalData>,
    splits: Option<Vec<AdyenSplitData>>,
    store: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RedirectionResponse {
    result_code: AdyenStatus,
    action: AdyenRedirectAction,
    refusal_reason: Option<String>,
    refusal_reason_code: Option<String>,
    psp_reference: Option<String>,
    merchant_reference: Option<String>,
    store: Option<String>,
    splits: Option<Vec<AdyenSplitData>>,
    additional_data: Option<AdditionalData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdyenRedirectAction {
    payment_method_type: PaymentType,
    url: Option<Url>,
    method: Option<hyperswitch_common_utils::request::Method>,
    #[serde(rename = "type")]
    type_of_response: ActionType,
    data: Option<std::collections::HashMap<String, String>>,
    payment_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    Redirect,
    Await,
    #[serde(rename = "qrCode")]
    QrCode,
    Voucher,
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
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethodType {
    Credit,
}

pub trait ForeignTryFrom<F>: Sized {
    type Error;

    fn foreign_try_from(from: F) -> Result<Self, Self::Error>;
}

fn get_adyen_payment_status(
    is_manual_capture: bool,
    adyen_status: AdyenStatus,
    _pmt: Option<hyperswitch_api_models::enums::PaymentMethodType>,
) -> AttemptStatus {
    match adyen_status {
        AdyenStatus::AuthenticationFinished => AttemptStatus::AuthenticationSuccessful,
        AdyenStatus::AuthenticationNotRequired | AdyenStatus::Received => AttemptStatus::Pending,
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
    }
}

impl<F, Req>
    ForeignTryFrom<(
        AdyenPaymentResponse,
        RouterDataV2<F, PaymentFlowData, Req, PaymentsResponseData>,
        u16,
        Option<hyperswitch_api_models::enums::CaptureMethod>,
        bool,
        Option<hyperswitch_api_models::enums::PaymentMethodType>,
    )> for RouterDataV2<F, PaymentFlowData, Req, PaymentsResponseData>
{
    type Error = Error;
    fn foreign_try_from(
        (response, data, http_code, _capture_method, _is_multiple_capture_psync_flow, pmt): (
            AdyenPaymentResponse,
            RouterDataV2<F, PaymentFlowData, Req, PaymentsResponseData>,
            u16,
            Option<hyperswitch_api_models::enums::CaptureMethod>,
            bool,
            Option<hyperswitch_api_models::enums::PaymentMethodType>,
        ),
    ) -> Result<Self, Self::Error> {
        let is_manual_capture = false;
        let (_status, error, payment_response_data) = match response {
            AdyenPaymentResponse::Response(response) => {
                get_adyen_response(*response, is_manual_capture, http_code, pmt)?
            }
            AdyenPaymentResponse::RedirectionResponse(response) => {
                get_redirection_response(*response, is_manual_capture, http_code, pmt)?
            }
        };

        Ok(Self {
            response: error.map_or_else(|| Ok(payment_response_data), Err),
            ..data
        })
    }
}

pub fn get_adyen_response(
    response: AdyenResponse,
    is_capture_manual: bool,
    status_code: u16,
    pmt: Option<hyperswitch_common_enums::enums::PaymentMethodType>,
) -> CustomResult<
    (
        hyperswitch_common_enums::enums::AttemptStatus,
        Option<hyperswitch_domain_models::router_data::ErrorResponse>,
        PaymentsResponseData,
    ),
    hyperswitch_interfaces::errors::ConnectorError,
> {
    let status = get_adyen_payment_status(is_capture_manual, response.result_code, pmt);
    let error = if response.refusal_reason.is_some()
        || response.refusal_reason_code.is_some()
        || status == hyperswitch_common_enums::enums::AttemptStatus::Failure
    {
        Some(hyperswitch_domain_models::router_data::ErrorResponse {
            code: response
                .refusal_reason_code
                .unwrap_or_else(|| NO_ERROR_CODE.to_string()),
            message: response
                .refusal_reason
                .clone()
                .unwrap_or_else(|| hyperswitch_interfaces::consts::NO_ERROR_MESSAGE.to_string()),
            reason: response.refusal_reason,
            status_code,
            attempt_status: None,
            connector_transaction_id: Some(response.psp_reference.clone()),
        })
    } else {
        None
    };
    let mandate_reference = response
        .additional_data
        .as_ref()
        .and_then(|data| data.recurring_detail_reference.to_owned())
        .map(|mandate_id| MandateReference {
            connector_mandate_id: Some(mandate_id.expose()),
            payment_method_id: None,
        });
    let network_txn_id = response.additional_data.and_then(|additional_data| {
        additional_data
            .network_tx_reference
            .map(|network_tx_id| network_tx_id.expose())
    });

    let payments_response_data = PaymentsResponseData::TransactionResponse {
        resource_id: ResponseId::ConnectorTransactionId(response.psp_reference),
        redirection_data: None,
        mandate_reference: mandate_reference,
        connector_metadata: None,
        network_txn_id,
        connector_response_reference_id: Some(response.merchant_reference),
        incremental_authorization_allowed: None,
        charge_id: None,
    };
    Ok((status, error, payments_response_data))
}

pub fn get_redirection_response(
    response: RedirectionResponse,
    is_manual_capture: bool,
    status_code: u16,
    pmt: Option<hyperswitch_common_enums::enums::PaymentMethodType>,
) -> CustomResult<
    (
        hyperswitch_common_enums::enums::AttemptStatus,
        Option<ErrorResponse>,
        PaymentsResponseData,
    ),
    hyperswitch_interfaces::errors::ConnectorError,
> {
    let status = get_adyen_payment_status(is_manual_capture, response.result_code.clone(), pmt);
    let error = if response.refusal_reason.is_some()
        || response.refusal_reason_code.is_some()
        || status == hyperswitch_common_enums::enums::AttemptStatus::Failure
    {
        Some(ErrorResponse {
            code: response
                .refusal_reason_code
                .clone()
                .unwrap_or_else(|| NO_ERROR_CODE.to_string()),
            message: response
                .refusal_reason
                .clone()
                .unwrap_or_else(|| NO_ERROR_MESSAGE.to_string()),
            reason: response.refusal_reason.to_owned(),
            status_code,
            attempt_status: None,
            connector_transaction_id: response.psp_reference.clone(),
        })
    } else {
        None
    };

    let redirection_data = response.action.url.clone().map(|url| {
        let form_fields = response.action.data.clone().unwrap_or_else(|| {
            std::collections::HashMap::from_iter(
                url.query_pairs()
                    .map(|(key, value)| (key.to_string(), value.to_string())),
            )
        });
        RedirectForm::Form {
            endpoint: url.to_string(),
            method: response.action.method.unwrap_or(Method::Get),
            form_fields,
        }
    });

    let connector_metadata = get_wait_screen_metadata(&response)?;

    let payments_response_data = PaymentsResponseData::TransactionResponse {
        resource_id: match response.psp_reference.as_ref() {
            Some(psp) => ResponseId::ConnectorTransactionId(psp.to_string()),
            None => ResponseId::NoResponseId,
        },
        redirection_data: redirection_data,
        mandate_reference: None,
        connector_metadata,
        network_txn_id: None,
        connector_response_reference_id: response
            .merchant_reference
            .clone()
            .or(response.psp_reference),
        incremental_authorization_allowed: None,
        charge_id: None,
    };
    Ok((status, error, payments_response_data))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitScreenData {
    display_from_timestamp: i128,
    display_to_timestamp: Option<i128>,
}

pub fn get_wait_screen_metadata(
    next_action: &RedirectionResponse,
) -> CustomResult<Option<serde_json::Value>, hyperswitch_interfaces::errors::ConnectorError> {
    match next_action.action.payment_method_type {
        PaymentType::Blik => {
            let current_time = OffsetDateTime::now_utc().unix_timestamp_nanos();
            Ok(Some(serde_json::json!(WaitScreenData {
                display_from_timestamp: current_time,
                display_to_timestamp: Some(current_time + Duration::minutes(1).whole_nanoseconds())
            })))
        }
        PaymentType::Mbway => {
            let current_time = OffsetDateTime::now_utc().unix_timestamp_nanos();
            Ok(Some(serde_json::json!(WaitScreenData {
                display_from_timestamp: current_time,
                display_to_timestamp: None
            })))
        }
        PaymentType::Affirm
        | PaymentType::Oxxo
        | PaymentType::Afterpaytouch
        | PaymentType::Alipay
        | PaymentType::AlipayHk
        | PaymentType::Alfamart
        | PaymentType::Alma
        | PaymentType::Applepay
        | PaymentType::Bizum
        | PaymentType::Atome
        | PaymentType::BoletoBancario
        | PaymentType::ClearPay
        | PaymentType::Dana
        | PaymentType::Eps
        | PaymentType::Gcash
        | PaymentType::Googlepay
        | PaymentType::GoPay
        | PaymentType::Ideal
        | PaymentType::Indomaret
        | PaymentType::Klarna
        | PaymentType::Kakaopay
        | PaymentType::MobilePay
        | PaymentType::Momo
        | PaymentType::MomoAtm
        | PaymentType::OnlineBankingCzechRepublic
        | PaymentType::OnlineBankingFinland
        | PaymentType::OnlineBankingPoland
        | PaymentType::OnlineBankingSlovakia
        | PaymentType::OnlineBankingFpx
        | PaymentType::OnlineBankingThailand
        | PaymentType::OpenBankingUK
        | PaymentType::PayBright
        | PaymentType::Paypal
        | PaymentType::Scheme
        | PaymentType::NetworkToken
        | PaymentType::Trustly
        | PaymentType::TouchNGo
        | PaymentType::Walley
        | PaymentType::WeChatPayWeb
        | PaymentType::AchDirectDebit
        | PaymentType::SepaDirectDebit
        | PaymentType::BacsDirectDebit
        | PaymentType::Samsungpay
        | PaymentType::Twint
        | PaymentType::Vipps
        | PaymentType::Swish
        | PaymentType::Knet
        | PaymentType::Benefit
        | PaymentType::PermataBankTransfer
        | PaymentType::BcaBankTransfer
        | PaymentType::BniVa
        | PaymentType::BriVa
        | PaymentType::CimbVa
        | PaymentType::DanamonVa
        | PaymentType::Giftcard
        | PaymentType::MandiriVa
        | PaymentType::PaySafeCard
        | PaymentType::SevenEleven
        | PaymentType::Lawson => Ok(None),
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdyenErrorResponse {
    pub status: i32,
    pub error_code: String,
    pub message: String,
    pub error_type: String,
    pub psp_reference: Option<String>,
}

fn build_shopper_reference(
    customer_id: &Option<hyperswitch_common_utils::id_type::CustomerId>,
    merchant_id: hyperswitch_common_utils::id_type::MerchantId,
) -> Option<String> {
    customer_id.clone().map(|c_id| {
        format!(
            "{}_{}",
            merchant_id.get_string_repr(),
            c_id.get_string_repr()
        )
    })
}

type RecurringDetails = (Option<AdyenRecurringModel>, Option<bool>, Option<String>);

fn get_recurring_processing_model(
    item: &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
) -> Result<RecurringDetails, Error> {
    let customer_id = item
        .request
        .customer_id
        .clone()
        .ok_or_else(Box::new(move || {
            errors::ConnectorError::MissingRequiredField {
                field_name: "customer_id",
            }
        }))?;

    match (item.request.setup_future_usage, item.request.off_session) {
        (Some(hyperswitch_common_enums::enums::FutureUsage::OffSession), _) => {
            let shopper_reference = format!(
                "{}_{}",
                item.merchant_id.get_string_repr(),
                customer_id.get_string_repr()
            );
            let store_payment_method = is_mandate_payment(item);
            Ok((
                Some(AdyenRecurringModel::UnscheduledCardOnFile),
                Some(store_payment_method),
                Some(shopper_reference),
            ))
        }
        (_, Some(true)) => Ok((
            Some(AdyenRecurringModel::UnscheduledCardOnFile),
            None,
            Some(format!(
                "{}_{}",
                item.merchant_id.get_string_repr(),
                customer_id.get_string_repr()
            )),
        )),
        _ => Ok((None, None, None)),
    }
}

fn is_mandate_payment(
    item: &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
) -> bool {
    ((item.request.customer_acceptance.is_some() || item.request.setup_mandate_details.is_some())
        && (item.request.setup_future_usage
            == Some(hyperswitch_common_enums::enums::FutureUsage::OffSession)))
        || item
            .request
            .mandate_id
            .as_ref()
            .and_then(|mandate_ids| mandate_ids.mandate_reference_id.as_ref())
            .is_some()
}

pub fn get_address_info(
    address: Option<&hyperswitch_api_models::payments::Address>,
) -> Option<Result<Address, error_stack::Report<hyperswitch_interfaces::errors::ConnectorError>>> {
    address.and_then(|add| {
        add.address.as_ref().map(
            |a| -> Result<
                Address,
                error_stack::Report<hyperswitch_interfaces::errors::ConnectorError>,
            > {
                Ok(Address {
                    city: a.city.clone().unwrap(),
                    country: a.country.unwrap(),
                    house_number_or_name: a.line1.clone().unwrap(),
                    postal_code: a.zip.clone().unwrap(),
                    state_or_province: a.state.clone(),
                    street: a.line2.clone(),
                })
            },
        )
    })
}

fn get_additional_data(
    item: &RouterDataV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
) -> Option<AdditionalData> {
    let (authorisation_type, manual_capture) = match item.request.capture_method {
        Some(hyperswitch_common_enums::enums::CaptureMethod::Manual)
        | Some(enums::CaptureMethod::ManualMultiple) => {
            (Some(AuthType::PreAuth), Some("true".to_string()))
        }
        _ => (None, None),
    };
    let riskdata = item.request.metadata.clone().and_then(get_risk_data);

    let execute_three_d = if matches!(
        item.resource_common_data.auth_type,
        hyperswitch_common_enums::enums::AuthenticationType::ThreeDs
    ) {
        Some("true".to_string())
    } else {
        None
    };

    if authorisation_type.is_none()
        && manual_capture.is_none()
        && execute_three_d.is_none()
        && riskdata.is_none()
    {
        //without this if-condition when the above 3 values are None, additionalData will be serialized to JSON like this -> additionalData: {}
        //returning None, ensures that additionalData key will not be present in the serialized JSON
        None
    } else {
        Some(AdditionalData {
            authorisation_type,
            manual_capture,
            execute_three_d,
            network_tx_reference: None,
            recurring_detail_reference: None,
            recurring_shopper_reference: None,
            recurring_processing_model: None,
            riskdata,
            ..AdditionalData::default()
        })
    }
}

pub fn get_risk_data(metadata: serde_json::Value) -> Option<RiskData> {
    let item_i_d = get_str("riskdata.basket.item1.itemID", &metadata);
    let product_title = get_str("riskdata.basket.item1.productTitle", &metadata);
    let amount_per_item = get_str("riskdata.basket.item1.amountPerItem", &metadata);
    let currency = get_str("riskdata.basket.item1.currency", &metadata);
    let upc = get_str("riskdata.basket.item1.upc", &metadata);
    let brand = get_str("riskdata.basket.item1.brand", &metadata);
    let manufacturer = get_str("riskdata.basket.item1.manufacturer", &metadata);
    let category = get_str("riskdata.basket.item1.category", &metadata);
    let quantity = get_str("riskdata.basket.item1.quantity", &metadata);
    let color = get_str("riskdata.basket.item1.color", &metadata);
    let size = get_str("riskdata.basket.item1.size", &metadata);

    let device_country = get_str("riskdata.deviceCountry", &metadata);
    let house_numberor_name = get_str("riskdata.houseNumberorName", &metadata);
    let account_creation_date = get_str("riskdata.accountCreationDate", &metadata);
    let affiliate_channel = get_str("riskdata.affiliateChannel", &metadata);
    let avg_order_value = get_str("riskdata.avgOrderValue", &metadata);
    let delivery_method = get_str("riskdata.deliveryMethod", &metadata);
    let email_name = get_str("riskdata.emailName", &metadata);
    let email_domain = get_str("riskdata.emailDomain", &metadata);
    let last_order_date = get_str("riskdata.lastOrderDate", &metadata);
    let merchant_reference = get_str("riskdata.merchantReference", &metadata);
    let payment_method = get_str("riskdata.paymentMethod", &metadata);
    let promotion_name = get_str("riskdata.promotionName", &metadata);
    let secondary_phone_number = get_str("riskdata.secondaryPhoneNumber", &metadata);
    let timefrom_loginto_order = get_str("riskdata.timefromLogintoOrder", &metadata);
    let total_session_time = get_str("riskdata.totalSessionTime", &metadata);
    let total_authorized_amount_in_last30_days =
        get_str("riskdata.totalAuthorizedAmountInLast30Days", &metadata);
    let total_order_quantity = get_str("riskdata.totalOrderQuantity", &metadata);
    let total_lifetime_value = get_str("riskdata.totalLifetimeValue", &metadata);
    let visits_month = get_str("riskdata.visitsMonth", &metadata);
    let visits_week = get_str("riskdata.visitsWeek", &metadata);
    let visits_year = get_str("riskdata.visitsYear", &metadata);
    let ship_to_name = get_str("riskdata.shipToName", &metadata);
    let first8charactersof_address_line1_zip =
        get_str("riskdata.first8charactersofAddressLine1Zip", &metadata);
    let affiliate_order = get_bool("riskdata.affiliateOrder", &metadata);

    Some(RiskData {
        item_i_d,
        product_title,
        amount_per_item,
        currency,
        upc,
        brand,
        manufacturer,
        category,
        quantity,
        color,
        size,
        device_country,
        house_numberor_name,
        account_creation_date,
        affiliate_channel,
        avg_order_value,
        delivery_method,
        email_name,
        email_domain,
        last_order_date,
        merchant_reference,
        payment_method,
        promotion_name,
        secondary_phone_number,
        timefrom_loginto_order,
        total_session_time,
        total_authorized_amount_in_last30_days,
        total_order_quantity,
        total_lifetime_value,
        visits_month,
        visits_week,
        visits_year,
        ship_to_name,
        first8charactersof_address_line1_zip,
        affiliate_order,
    })
}

fn get_str(key: &str, riskdata: &serde_json::Value) -> Option<String> {
    riskdata
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn get_bool(key: &str, riskdata: &serde_json::Value) -> Option<bool> {
    riskdata.get(key).and_then(|v| v.as_bool())
}
