// backend/connector-integration/src/connectors/bitpay/transformers.rs
// Will be populated later 

use hyperswitch_domain_models::{
    payment_method_data::PaymentMethodData,
    router_data::{ConnectorAuthType, RouterData},
    router_data_v2::RouterDataV2,
    router_response_types::{PaymentsResponseData, RedirectForm},
    router_request_types::ResponseId,
};
use hyperswitch_common_utils::{errors::CustomResult, ext_traits::ByteSliceExt, types::MinorUnit};
use hyperswitch_interfaces::errors;
use serde::{Deserialize, Serialize};

use domain_types::{
    connector_flow::Authorize,
    connector_types::{PaymentFlowData, PaymentsAuthorizeData},
};
use hyperswitch_masking::{ExposeInterface, Secret};


// Placeholder for Bitpay Card Payment Request
#[derive(Debug, Clone, Serialize)]
pub struct BitpayPaymentsRequest {
    token: Secret<String>, // Assuming a card token will be used, similar to other card payments
    amount: MinorUnit,
    currency: String,
    #[serde(rename = "orderId")]
    order_id: String,
    description: Option<String>,
    #[serde(rename = "redirectURL")]
    redirect_url: Option<String>,
    #[serde(rename = "notificationURL")]
    notification_url: Option<String>,
    // Other fields as potentially needed by Bitpay for card payments
}

// Placeholder for Bitpay Card Payment Response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BitpayPaymentsResponse {
    status: String, // e.g., "processing", "paid", "confirmed", "complete", "expired", "invalid"
    id: String,     // Connector's transaction ID
    url: Option<String>, // For redirection if any
    // Other fields from Bitpay's response
}

impl<F, T>
    TryFrom<(
        &RouterDataV2<F, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
        PaymentsAuthorizeData, // Directly pass PaymentsAuthorizeData
    )> for BitpayPaymentsRequest
where
    F: Clone,
    T: Clone,
{
    type Error = error_stack::Report<errors::ConnectorError>;

    fn try_from(
        (item, گھر_data): ( // Renamed to avoid conflict with item.request
            &RouterDataV2<F, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData>,
            PaymentsAuthorizeData,
        ),
    ) -> Result<Self, Self::Error> {
        let card_token = match گھر_data.payment_method_data {
            PaymentMethodData::Card(cc) => cc.card_number.expose(), // This is a simplification. Real tokenization needed.
            _ => return Err(errors::ConnectorError::NotImplemented("Only card payments are supported for now".to_string()).into()),
        };

        Ok(Self {
            token: card_token.into(), // Placeholder, real token needed
            amount: گھر_data.minor_amount,
            currency: گھر_data.currency.to_string(),
            order_id: item.resource_common_data.connector_request_reference_id.clone(),
            description: item.resource_common_data.description.clone(),
            redirect_url: گھر_data.router_return_url.clone(),
            notification_url: گھر_data.webhook_url.clone(),
        })
    }
}


impl TryFrom<BitpayPaymentsResponse>
    for PaymentsResponseData
{
    type Error = error_stack::Report<errors::ConnectorError>;

    fn try_from(data: BitpayPaymentsResponse) -> Result<Self, Self::Error> {
        let redirection_data = data.url.map(|redirect_url| {
            RedirectForm::Html {
                html_data: format!("<script>window.location.href = '{}';</script>", redirect_url), // Simplistic HTML redirect
            }
        });

        Ok(PaymentsResponseData::TransactionResponse {
            resource_id: ResponseId::ConnectorTransactionId(data.id),
            redirection_data: Box::new(redirection_data),
            connector_metadata: None,
            network_txn_id: None,
            connector_response_reference_id: None,
            incremental_authorization_allowed: None,
        })
    }
}

// For converting Hyperswitch status to Bitpay status (if needed)
// pub fn to_bitpay_status(status: hyperswitch_api_models::enums::AttemptStatus) -> &'static str { ... }

// For converting Bitpay status to Hyperswitch status
pub fn to_hyperswitch_status(bitpay_status: &str) -> hyperswitch_api_models::enums::AttemptStatus {
    match bitpay_status {
        "paid" | "confirmed" | "complete" => hyperswitch_api_models::enums::AttemptStatus::Charged, // Or Authorized if capture is manual
        "processing" => hyperswitch_api_models::enums::AttemptStatus::Pending,
        "expired" | "invalid" => hyperswitch_api_models::enums::AttemptStatus::Failure,
        _ => hyperswitch_api_models::enums::AttemptStatus::Unresolved, // Default or map specific errors
    }
}

// Helper for RouterDataV2 conversion (if complex mapping is needed)
// This is a basic shell, actual implementation might need more fields.
impl<F: Clone, Req: Clone, Resp: Clone>
    ForeignTryFrom<(
        BitpayPaymentsResponse,
        RouterDataV2<F, PaymentFlowData, Req, Resp>,
        Option<hyperswitch_api_models::enums::CaptureMethod>, // capture_method
        bool, // is_latency_enabled
        Option<hyperswitch_api_models::enums::PaymentMethodType>, // payment_method_type
    )> for RouterDataV2<F, PaymentFlowData, Req, Resp>
{
    type Error = error_stack::Report<errors::ConnectorError>;

    fn foreign_try_from(
        (payload, mut item, _capture_method, _is_latency_enabled, _payment_method_type): (
            BitpayPaymentsResponse,
            RouterDataV2<F, PaymentFlowData, Req, Resp>,
            Option<hyperswitch_api_models::enums::CaptureMethod>,
            bool,
            Option<hyperswitch_api_models::enums::PaymentMethodType>,
        ),
    ) -> Result<Self, Self::Error> {
        item.resource_common_data.status = to_hyperswitch_status(&payload.status);
        item.response = Ok(PaymentsResponseData::try_from(payload)?);
        // item.resource_common_data.connector_http_status_code = Some(status_code); // If you have status_code
        Ok(item)
    }
} 