use crate::connectors::{Adyen, Razorpay};
use crate::flow;
use domain_types::errors::{ApiError, ApplicationErrorResponse};
use domain_types::utils::ForeignTryFrom;
use hyperswitch_api_models::enums::Currency;
use hyperswitch_common_utils::types::MinorUnit;
use hyperswitch_domain_models::router_data_v2::PaymentFlowData;
use hyperswitch_interfaces::{
    api::{
        payments_v2::{PaymentAuthorizeV2, PaymentSyncV2},
        ConnectorCommon,
    },
    connector_integration_v2::ConnectorIntegrationV2,
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
    ConnectorCommon + ValidationTrait + PaymentAuthorizeV2 + PaymentSyncV2 + PaymentOrderCreate
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
    flow::CreateOrder,
    PaymentFlowData,
    PaymentCreateOrderData,
    PaymentCreateOrderResponse,
>
{
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

#[derive(Clone)]
pub struct ConnectorData {
    pub connector: BoxedConnector,
    pub connector_name: ConnectorEnum,
}

impl ConnectorData {
    pub fn get_connector_by_name(connector_name: &ConnectorEnum) -> Self {
        let connector = Self::convert_connector(connector_name.clone());
        Self {
            connector,
            connector_name: connector_name.clone(),
        }
    }

    fn convert_connector(connector_name: ConnectorEnum) -> BoxedConnector {
        match connector_name {
            ConnectorEnum::Adyen => Box::new(Adyen::new()),
            ConnectorEnum::Razorpay => Box::new(Razorpay::new()),
        }
    }
}
