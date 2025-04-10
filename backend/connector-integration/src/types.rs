use crate::connectors::Adyen;
use hyperswitch_interfaces::api::{
    payments_v2::{PaymentAuthorizeV2, PaymentSyncV2},
    ConnectorCommon,
};

#[derive(Clone, Debug)]
pub enum ConnectorEnum {
    Adyen,
    Razorpay,
}

pub trait ConnectorServiceTrait: ConnectorCommon + PaymentAuthorizeV2 + PaymentSyncV2 {}

pub type BoxedConnector = Box<&'static (dyn ConnectorServiceTrait + Sync)>;

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
            ConnectorEnum::Razorpay => todo!(),
        }
    }
}
