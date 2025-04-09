use hyperswitch_interfaces::api::{ConnectorCommon, payments_v2::PaymentAuthorizeV2, payments_v2::PaymentSyncV2};

#[derive(Clone, Debug)]
pub enum ConnectorEnum {
    Adyen,
    Razorpay,
}

pub trait ConnectorServiceTrait:
    ConnectorCommon
    + PaymentAuthorizeV2
    + PaymentSyncV2
{
}

pub type BoxedConnector = Box<&'static (dyn ConnectorServiceTrait + Sync)>;

#[derive(Clone)]
pub struct ConnectorData {
    pub connector: BoxedConnector,
    pub connector_name: ConnectorEnum,
}

impl ConnectorData {
    pub fn get_connector_by_name(
        connector_name: &ConnectorEnum,
    ) -> Self {
        let connector = Self::convert_connector(connector_name.clone());
        Self {
            connector,
            connector_name: connector_name.clone(),
        }
    }

    fn convert_connector(
        _connector_name: ConnectorEnum,
    ) -> BoxedConnector {
        todo!()
        // match connector_name {
        //     ConnectorEnum::Adyen => Ok(Box::new(Adyen::new())),
        //     ConnectorEnum::Razorpay => Ok(Box::new(Razorpay::new())),
        // }
    }
}