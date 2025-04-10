use grpc_api_types::payments::{self, payment_service_client::PaymentServiceClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the URL from command line arguments
    let args = std::env::args().collect::<Vec<_>>();
    let url = args.get(1);
    let url = match url {
        Some(url) => url,
        None => {
            eprintln!("Usage: {} <url>", args[0]);
            std::process::exit(1);
        }
    };

    // Create a gRPC client
    let mut client = PaymentServiceClient::connect(url.clone()).await.unwrap();

    // Create a request
    let request = payments::PaymentsAuthorizeRequest {
        amount: 1000,
        currency: payments::Currency::Usd as i32,
        connector: payments::Connector::Adyen as i32,
        auth_creds: Some(payments::AuthType {
            auth_details: Some(payments::auth_type::AuthDetails::SignatureKey(
                payments::SignatureKey {
                    api_key: "".to_string(),
                    key1: "".to_string(),
                    api_secret: "".to_string()
                },
            )),
        }),
        payment_method: payments::PaymentMethod::Card as i32,
        payment_method_data: Some(payments::PaymentMethodData {
            data: Some(payments::payment_method_data::Data::Card(payments::Card {
                card_number: "4111111111111111".to_string(),
                card_exp_month: "03".to_string(),
                card_exp_year: "2030".to_string(),
                card_cvc: "737".to_string(),
                ..Default::default()
            })),
        }),
        address: Some(payments::PaymentAddress::default()),
        auth_type: payments::AuthenticationType::ThreeDs as i32,
        connector_request_reference_id: "ref_12345".to_string(),
        enrolled_for_3ds: true,
        request_incremental_authorization: false,
        minor_amount: 1000,
        ..Default::default()
    };

    // Send the request
    let response = client
        .payment_authorize(request)
        .await
        .expect("Failed to send request");

    // Print the response
    println!("Response: {:?}", response);
    Ok(())
}
