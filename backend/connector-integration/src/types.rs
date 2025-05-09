use domain_types::connector_types::{BoxedConnector, ConnectorEnum};

use crate::connectors::{Adyen, Razorpay, Checkout, Paypal, Jpmorgan, Airwallex};

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
            ConnectorEnum::Checkout => Box::new(Checkout::new()),
            ConnectorEnum::Paypal => Box::new(Paypal::new()),
            ConnectorEnum::Jpmorgan => Box::new(Jpmorgan::new()),
            ConnectorEnum::Airwallex => Box::new(Airwallex::new()),
        }
    }
}
