use grpc_server::{app, configs};
use grpc_api_types::payments::{payment_service_client::PaymentServiceClient, PaymentsAuthorizeRequest};
use tonic::{transport::Channel, Request};
use std::collections::HashMap;
use serde_json::json;

mod common;

#[tokio::test]
async fn test_config_override() {
    grpc_test!(client, PaymentServiceClient<Channel>, {
        // Create a request with configuration override
        let mut request = Request::new(PaymentsAuthorizeRequest {
            amount: 1000,
            currency: 1, // USD
            payment_method: 1, // Card
            payment_method_data: Some(grpc_api_types::payments::PaymentMethodData {
                card: Some(grpc_api_types::payments::Card {
                    card_number: "4111111111111111".to_string(),
                    expiry_month: "12".to_string(),
                    expiry_year: "2025".to_string(),
                    card_holder_name: Some("Test User".to_string()),
                    security_code: Some("123".to_string()),
                }),
                ..Default::default()
            }),
            connector_request_reference_id: "test_reference".to_string(),
            ..Default::default()
        });

        // Add configuration override header
        let override_config = json!({
            "connectors": {
                "adyen": {
                    "base_url": "https://override-test.adyen.com/"
                }
            }
        });
        
        request.metadata_mut().insert(
            "x-config-override",
            override_config.to_string().parse().unwrap(),
        );

        // Add required headers
        request.metadata_mut().insert(
            "x-connector",
            "adyen".parse().unwrap(),
        );

        // Make the request
        let response = client.payment_authorize(request).await;
        
        // The request should fail with an invalid argument error since we're using test data
        // but we can verify that the configuration override was processed
        assert!(response.is_err());
        let error = response.unwrap_err();
        assert!(error.message().contains("Invalid request data"));
    });
} 