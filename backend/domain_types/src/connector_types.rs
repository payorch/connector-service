use crate::connector_flow::{self, Authorize, PSync, RSync};
use crate::errors::{ApiError, ApplicationErrorResponse};
use crate::types::Connectors;
use crate::utils::ForeignTryFrom;
use hyperswitch_api_models::enums::Currency;
use hyperswitch_common_utils::types::MinorUnit;
use hyperswitch_domain_models::router_request_types::{ResponseId, SyncRequestType};
use hyperswitch_interfaces::{
    api::ConnectorCommon, connector_integration_v2::ConnectorIntegrationV2,
};

#[derive(Clone, Debug)]
pub enum ConnectorEnum {
    Adyen,
    Razorpay,
}

impl ForeignTryFrom<i32> for ConnectorEnum {
    type Error = ApplicationErrorResponse;

    fn foreign_try_from(connector: i32) -> Result<Self, error_stack::Report<Self::Error>> {
        match connector {
            2 => Ok(Self::Adyen),
            68 => Ok(Self::Razorpay),
            _ => Err(ApplicationErrorResponse::BadRequest(ApiError {
                sub_code: "INVALID_CONNECTOR".to_owned(),
                error_identifier: 401,
                error_message: format!("Invalid value for authenticate_by: {}", connector),
                error_object: None,
            })
            .into()),
        }
    }
}

pub trait ConnectorServiceTrait:
    ConnectorCommon
    + ValidationTrait
    + PaymentAuthorizeV2
    + PaymentSyncV2
    + PaymentOrderCreate
    + RefundSyncV2
{
}

pub type BoxedConnector = Box<&'static (dyn ConnectorServiceTrait + Sync)>;

pub trait ValidationTrait {
    fn should_do_order_create(&self) -> bool {
        false
    }
}

pub trait PaymentOrderCreate:
    ConnectorIntegrationV2<
    connector_flow::CreateOrder,
    PaymentFlowData,
    PaymentCreateOrderData,
    PaymentCreateOrderResponse,
>
{
}

pub trait PaymentAuthorizeV2:
    ConnectorIntegrationV2<Authorize, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>
{
}

pub trait PaymentSyncV2:
    ConnectorIntegrationV2<PSync, PaymentFlowData, PaymentsSyncData, PaymentsResponseData>
{
}

#[derive(Debug, Clone)]
pub struct PaymentFlowData {
    pub merchant_id: hyperswitch_common_utils::id_type::MerchantId,
    pub customer_id: Option<hyperswitch_common_utils::id_type::CustomerId>,
    pub connector_customer: Option<String>,
    pub payment_id: String,
    pub attempt_id: String,
    pub status: hyperswitch_common_enums::AttemptStatus,
    pub payment_method: hyperswitch_common_enums::PaymentMethod,
    pub description: Option<String>,
    pub return_url: Option<String>,
    pub address: hyperswitch_domain_models::payment_address::PaymentAddress,
    pub auth_type: hyperswitch_common_enums::AuthenticationType,
    pub connector_meta_data: Option<hyperswitch_common_utils::pii::SecretSerdeValue>,
    pub amount_captured: Option<i64>,
    // minor amount for amount frameworka
    pub minor_amount_captured: Option<MinorUnit>,
    pub access_token: Option<String>,
    pub session_token: Option<String>,
    pub reference_id: Option<String>,
    pub payment_method_token: Option<String>,
    pub preprocessing_id: Option<String>,
    ///for switching between two different versions of the same connector
    pub connector_api_version: Option<String>,
    /// Contains a reference ID that should be sent in the connector request
    pub connector_request_reference_id: String,
    pub test_mode: Option<bool>,
    pub connector_http_status_code: Option<u16>,
    pub external_latency: Option<u128>,
    pub connectors: Connectors,
}

#[derive(Debug, Clone)]
pub struct PaymentsAuthorizeData {
    pub payment_method_data: hyperswitch_domain_models::payment_method_data::PaymentMethodData,
    /// total amount (original_amount + surcharge_amount + tax_on_surcharge_amount)
    /// If connector supports separate field for surcharge amount, consider using below functions defined on `PaymentsAuthorizeData` to fetch original amount and surcharge amount separately
    /// ```text
    /// get_original_amount()
    /// get_surcharge_amount()
    /// get_tax_on_surcharge_amount()
    /// get_total_surcharge_amount() // returns surcharge_amount + tax_on_surcharge_amount
    /// ```
    pub amount: i64,
    pub order_tax_amount: Option<MinorUnit>,
    pub email: Option<hyperswitch_common_utils::pii::Email>,
    pub customer_name: Option<String>,
    pub currency: hyperswitch_common_enums::Currency,
    pub confirm: bool,
    pub statement_descriptor_suffix: Option<String>,
    pub statement_descriptor: Option<String>,
    pub capture_method: Option<hyperswitch_common_enums::CaptureMethod>,
    pub router_return_url: Option<String>,
    pub webhook_url: Option<String>,
    pub complete_authorize_url: Option<String>,
    // Mandates
    pub mandate_id: Option<hyperswitch_api_models::payments::MandateIds>,
    pub setup_future_usage: Option<hyperswitch_common_enums::FutureUsage>,
    pub off_session: Option<bool>,
    pub browser_info: Option<hyperswitch_domain_models::router_request_types::BrowserInformation>,
    pub order_category: Option<String>,
    pub session_token: Option<String>,
    pub enrolled_for_3ds: bool,
    pub related_transaction_id: Option<String>,
    pub payment_experience: Option<hyperswitch_common_enums::PaymentExperience>,
    pub payment_method_type: Option<hyperswitch_common_enums::PaymentMethodType>,
    pub customer_id: Option<hyperswitch_common_utils::id_type::CustomerId>,
    pub request_incremental_authorization: bool,
    pub metadata: Option<serde_json::Value>,
    // New amount for amount frame work
    pub minor_amount: MinorUnit,
    /// Merchant's identifier for the payment/invoice. This will be sent to the connector
    /// if the connector provides support to accept multiple reference ids.
    /// In case the connector supports only one reference id, Hyperswitch's Payment ID will be sent as reference.
    pub merchant_order_reference_id: Option<String>,
    pub shipping_cost: Option<MinorUnit>,
    pub merchant_account_id: Option<String>,
    pub merchant_config_currency: Option<hyperswitch_common_enums::Currency>,
}

#[derive(Debug, Default, Clone)]
pub struct PaymentsSyncData {
    pub connector_transaction_id: ResponseId,
    pub encoded_data: Option<String>,
    pub capture_method: Option<hyperswitch_common_enums::CaptureMethod>,
    pub connector_meta: Option<serde_json::Value>,
    pub sync_type: SyncRequestType,
    pub mandate_id: Option<hyperswitch_api_models::payments::MandateIds>,
    pub payment_method_type: Option<hyperswitch_common_enums::PaymentMethodType>,
    pub currency: hyperswitch_common_enums::Currency,
    pub payment_experience: Option<hyperswitch_common_enums::PaymentExperience>,
    pub amount: MinorUnit,
}

#[derive(Debug, Clone)]
pub enum PaymentsResponseData {
    TransactionResponse {
        resource_id: hyperswitch_domain_models::router_request_types::ResponseId,
        redirection_data:
            Box<Option<hyperswitch_domain_models::router_response_types::RedirectForm>>,
        connector_metadata: Option<serde_json::Value>,
        network_txn_id: Option<String>,
        connector_response_reference_id: Option<String>,
        incremental_authorization_allowed: Option<bool>,
    },
    SessionResponse {
        session_token: hyperswitch_api_models::payments::SessionToken,
    },
}

#[derive(Debug, Clone)]
pub struct PaymentCreateOrderData {
    pub amount: MinorUnit,
    pub currency: Currency,
}

#[derive(Debug, Clone)]
pub struct PaymentCreateOrderResponse {
    pub order_id: String,
}

pub trait RefundSyncV2:
    ConnectorIntegrationV2<RSync, RefundFlowData, RefundSyncData, RefundsResponseData>
{
}

#[derive(Debug, Default, Clone)]
pub struct RefundSyncData {
    pub connector_transaction_id: String,
    pub connector_refund_id: Option<String>,
    pub reason: Option<String>,
    pub refund_connector_metadata: Option<hyperswitch_common_utils::pii::SecretSerdeValue>,
    pub refund_status: hyperswitch_common_enums::RefundStatus,
}

#[derive(Debug, Clone)]
pub struct RefundsResponseData {
    pub connector_refund_id: String,
    pub refund_status: hyperswitch_common_enums::RefundStatus,
}

#[derive(Debug, Clone)]
pub struct RefundFlowData {
    pub status: hyperswitch_common_enums::RefundStatus,
    pub payment_method: hyperswitch_common_enums::PaymentMethod,
    pub connector_meta_data: Option<hyperswitch_common_utils::pii::SecretSerdeValue>,
    pub amount_captured: Option<i64>,
    pub minor_amount_captured: Option<MinorUnit>,
    pub connector_request_reference_id: String,
    pub refund_id: String,
    pub connectors: Connectors,
}
