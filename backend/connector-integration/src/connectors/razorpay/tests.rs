#[cfg(test)]
mod tests {

    use domain_types::connector_types::{
            BoxedConnector, ConnectorServiceTrait, PaymentFlowData, PaymentsAuthorizeData,
        };
    use hyperswitch_cards::CardNumber;
    use hyperswitch_common_enums::{AttemptStatus, AuthenticationType, PaymentMethod};
    use hyperswitch_domain_models::{
        payment_method_data::{Card, PaymentMethodData},
        router_request_types::BrowserInformation,
    };
    use hyperswitch_interfaces::{
        connector_integration_v2::ConnectorIntegrationV2, types::Response,
    };
    use serde_json::{json, to_value};

    use crate::connectors::Razorpay;

    mod authorize {
        use std::str::FromStr;

        use domain_types::{
            connector_types::{
                BoxedConnector, ConnectorServiceTrait, PaymentFlowData, PaymentsAuthorizeData,
            },
            types::{ConnectorParams, Connectors},
        };
        use hyperswitch_api_models::payments::{Address, PhoneDetails};
        use hyperswitch_cards::CardNumber;
        use hyperswitch_common_enums::{
            AttemptStatus, AuthenticationType, Currency, PaymentMethod, PaymentMethodType,
        };
        use hyperswitch_common_utils::{
            id_type::MerchantId, pii::Email, request::RequestContent, types::MinorUnit,
        };
        use hyperswitch_domain_models::{
            payment_address::PaymentAddress,
            payment_method_data::{Card, PaymentMethodData},
            router_data::{ConnectorAuthType, ErrorResponse},
            router_data_v2::RouterDataV2,
            router_request_types::BrowserInformation,
        };
        use hyperswitch_interfaces::{
            connector_integration_v2::ConnectorIntegrationV2, types::Response,
        };
        use serde_json::{json, to_value, Value};

        use crate::connectors::Razorpay;

        #[test]
        fn test_build_request_valid() {
            let email = Email::try_from("testuser@gmail.com".to_string()).unwrap();

            let test_router_data = RouterDataV2 {
                flow: std::marker::PhantomData,
                resource_common_data: PaymentFlowData {
                    merchant_id: MerchantId::default(),
                    customer_id: None,
                    connector_customer: None,
                    payment_id: "IRRELEVANT_PAYMENT_ID".to_string(),
                    attempt_id: "IRRELEVANT_ATTEMPT_ID".to_string(),
                    status: AttemptStatus::Pending,
                    payment_method: PaymentMethod::Card,
                    description: None,
                    return_url: None,
                    address: PaymentAddress::new(
                        None,
                        Some(Address {
                            address: None,
                            phone: Some(PhoneDetails {
                                number: Some("1234567890".to_string().into()),
                                country_code: Some("+1".to_string()),
                            }),
                            email: Some(email.clone()),
                        }),
                        None,
                        None,
                    ),
                    auth_type: AuthenticationType::NoThreeDs,
                    connector_meta_data: None,
                    amount_captured: None,
                    minor_amount_captured: None,
                    access_token: None,
                    session_token: None,
                    reference_id: Some("order_QMSVrXxHS9sBmu".to_string()),
                    payment_method_token: None,
                    preprocessing_id: None,
                    connector_api_version: None,
                    connector_request_reference_id: "ref_12345".to_string(),
                    test_mode: None,
                    connector_http_status_code: None,
                    external_latency: None,
                    connectors: Connectors {
                        adyen: ConnectorParams {
                            base_url: "https://checkout-test.adyen.com/".to_string(),
                        },
                        razorpay: ConnectorParams {
                            base_url: "https://api.razorpay.com/".to_string(),
                        },
                    },
                },
                connector_auth_type: ConnectorAuthType::BodyKey {
                    api_key: "dummy_api_key".to_string().into(),
                    key1: "dummy_key1".to_string().into(),
                },
                request: PaymentsAuthorizeData {
                    payment_method_data: PaymentMethodData::Card(Card {
                        card_number: CardNumber::from_str("5123456789012346").unwrap(),
                        card_exp_month: "12".to_string().into(),
                        card_exp_year: "2026".to_string().into(),
                        card_cvc: "123".to_string().into(),
                        card_issuer: None,
                        card_network: None,
                        card_type: None,
                        card_issuing_country: None,
                        bank_code: None,
                        nick_name: None,
                    }),
                    amount: 1000,
                    order_tax_amount: None,
                    email: Some(email.clone()),
                    customer_name: None,
                    currency: Currency::USD,
                    confirm: true,
                    statement_descriptor_suffix: None,
                    statement_descriptor: None,
                    capture_method: None,
                    router_return_url: None,
                    webhook_url: None,
                    complete_authorize_url: None,
                    mandate_id: None,
                    setup_future_usage: None,
                    off_session: None,
                    browser_info: Some(BrowserInformation {
                        color_depth: None,
                        java_enabled: Some(false),
                        java_script_enabled: None,
                        language: Some("en-US".to_string()),
                        screen_height: Some(1080),
                        screen_width: Some(1920),
                        time_zone: None,
                        ip_address: None,
                        accept_header: Some(
                            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
                                .to_string(),
                        ),
                        user_agent: Some(
                            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)".to_string(),
                        ),
                    }),
                    order_category: None,
                    session_token: None,
                    enrolled_for_3ds: false,
                    related_transaction_id: None,
                    payment_experience: None,
                    payment_method_type: Some(PaymentMethodType::Credit),
                    customer_id: None,
                    request_incremental_authorization: false,
                    metadata: None,
                    minor_amount: MinorUnit::new(1000),
                    merchant_order_reference_id: None,
                    shipping_cost: None,
                    merchant_account_id: None,
                    merchant_config_currency: None,
                },
                response: Err(ErrorResponse {
                    code: "HE_00".to_string(),
                    message: "Something went wrong".to_string(),
                    reason: None,
                    status_code: 500,
                    attempt_status: None,
                    connector_transaction_id: None,
                }),
            };

            let connector: BoxedConnector = Box::new(Razorpay::new());
            let result = connector.get_request_body(&test_router_data);
            let request_content = result.unwrap();
           
            let actual_json: Value = match request_content {
                Some(RequestContent::Json(payload)) => {
                    to_value(&payload).expect("Failed to serialize payload to JSON")
                }
                _ => panic!("Expected JSON payload"),
            };
            let expected_json: Value = json!({
                "amount": 1000,
                "currency": "USD",
                "contact": "1234567890",
                "email": "testuser@gmail.com",
                "order_id": "order_QMSVrXxHS9sBmu",
                "method": "card",
                "card": {
                    "number": "5123456789012346",
                    "expiry_month": "12",
                    "expiry_year": "2026",
                    "cvv": "123"
                },
                "authentication": {
                    "authentication_channel": "browser"
                },
                "browser": {
                    "java_enabled": false,
                    "language": "en-US",
                    "screen_height": 1080,
                    "screen_width": 1920
                },
                "ip": "",
                "referer": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                "user_agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"
            });
            assert_eq!(actual_json, expected_json);
        }

        #[test]
        fn test_build_request_missing() {
            let test_router_data = RouterDataV2 {
                flow: std::marker::PhantomData,
                resource_common_data: PaymentFlowData {
                    merchant_id: MerchantId::default(),
                    customer_id: None,
                    connector_customer: None,
                    payment_id: "MISSING_EMAIL_ID".to_string(),
                    attempt_id: "MISSING_CARD_ID".to_string(),
                    status: AttemptStatus::Pending,
                    payment_method: PaymentMethod::Card,
                    description: None,
                    return_url: None,
                    address: PaymentAddress::new(None, None, None, None),
                    auth_type: AuthenticationType::NoThreeDs,
                    connector_meta_data: None,
                    amount_captured: None,
                    minor_amount_captured: None,
                    access_token: None,
                    session_token: None,
                    reference_id: Some("order_missing".to_string()),
                    payment_method_token: None,
                    preprocessing_id: None,
                    connector_api_version: None,
                    connector_request_reference_id: "ref_missing".to_string(),
                    test_mode: None,
                    connector_http_status_code: None,
                    external_latency: None,
                    connectors: Connectors {
                        adyen: ConnectorParams {
                            base_url: "".to_string(),
                        },
                        razorpay: ConnectorParams {
                            base_url: "".to_string(),
                        },
                    },
                },
                connector_auth_type: ConnectorAuthType::BodyKey {
                    api_key: "dummy_api_key".to_string().into(),
                    key1: "dummy_key1".to_string().into(),
                },
                request: PaymentsAuthorizeData {
                    payment_method_data: PaymentMethodData::Card(Card {
                        card_number: CardNumber::from_str("").unwrap_or_default(),
                        card_exp_month: "".to_string().into(),
                        card_exp_year: "".to_string().into(),
                        card_cvc: "".to_string().into(),
                        card_issuer: None,
                        card_network: None,
                        card_type: None,
                        card_issuing_country: None,
                        bank_code: None,
                        nick_name: None,
                    }),
                    amount: 1000,
                    order_tax_amount: None,
                    email: None,
                    customer_name: None,
                    currency: Currency::USD,
                    confirm: true,
                    statement_descriptor_suffix: None,
                    statement_descriptor: None,
                    capture_method: None,
                    router_return_url: None,
                    webhook_url: None,
                    complete_authorize_url: None,
                    mandate_id: None,
                    setup_future_usage: None,
                    off_session: None,
                    browser_info: None,
                    order_category: None,
                    session_token: None,
                    enrolled_for_3ds: false,
                    related_transaction_id: None,
                    payment_experience: None,
                    payment_method_type: Some(PaymentMethodType::Credit),
                    customer_id: None,
                    request_incremental_authorization: false,
                    metadata: None,
                    minor_amount: MinorUnit::new(1000),
                    merchant_order_reference_id: None,
                    shipping_cost: None,
                    merchant_account_id: None,
                    merchant_config_currency: None,
                },
                response: Err(ErrorResponse {
                    code: "HE_01".to_string(),
                    message: "Missing required fields".to_string(),
                    reason: None,
                    status_code: 400,
                    attempt_status: None,
                    connector_transaction_id: None,
                }),
            };

            let connector: BoxedConnector = Box::new(Razorpay::new());
            let result = connector.get_request_body(&test_router_data);

            assert!(
                result.is_err(),
                "Expected error for missing required fields, but got: {:?}",
                result
            );
        }

        #[test]
        fn test_build_request_invalid() {
            use hyperswitch_common_utils::pii::Email;

            let email = Email::try_from("invalid-email@nowhere.com".to_string()).unwrap();

            let test_router_data = RouterDataV2 {
                flow: std::marker::PhantomData,
                resource_common_data: PaymentFlowData {
                    merchant_id: MerchantId::default(),
                    customer_id: None,
                    connector_customer: None,
                    payment_id: "INVALID_PAYMENT".to_string(),
                    attempt_id: "INVALID_ATTEMPT".to_string(),
                    status: AttemptStatus::Pending,
                    payment_method: PaymentMethod::Card,
                    description: None,
                    return_url: None,
                    address: PaymentAddress::new(None, None, None, None),
                    auth_type: AuthenticationType::NoThreeDs,
                    connector_meta_data: None,
                    amount_captured: None,
                    minor_amount_captured: None,
                    access_token: None,
                    session_token: None,
                    reference_id: Some("invalid_id".to_string()),
                    payment_method_token: None,
                    preprocessing_id: None,
                    connector_api_version: None,
                    connector_request_reference_id: "ref_invalid".to_string(),
                    test_mode: None,
                    connector_http_status_code: None,
                    external_latency: None,
                    connectors: Connectors {
                        adyen: ConnectorParams {
                            base_url: "".to_string(),
                        },
                        razorpay: ConnectorParams {
                            base_url: "".to_string(),
                        },
                    },
                },
                connector_auth_type: ConnectorAuthType::BodyKey {
                    api_key: "dummy_api_key".to_string().into(),
                    key1: "dummy_key1".to_string().into(),
                },
                request: PaymentsAuthorizeData {
                    payment_method_data: PaymentMethodData::Card(Card {
                        card_number: CardNumber::from_str("123").unwrap_or_default(),
                        card_exp_month: "99".to_string().into(),
                        card_exp_year: "1999".to_string().into(),
                        card_cvc: "1".to_string().into(),
                        card_issuer: None,
                        card_network: None,
                        card_type: None,
                        card_issuing_country: None,
                        bank_code: None,
                        nick_name: None,
                    }),
                    amount: 1000,
                    order_tax_amount: None,
                    email: Some(email),
                    customer_name: None,
                    currency: Currency::USD,
                    confirm: true,
                    statement_descriptor_suffix: None,
                    statement_descriptor: None,
                    capture_method: None,
                    router_return_url: None,
                    webhook_url: None,
                    complete_authorize_url: None,
                    mandate_id: None,
                    setup_future_usage: None,
                    off_session: None,
                    browser_info: None,
                    order_category: None,
                    session_token: None,
                    enrolled_for_3ds: false,
                    related_transaction_id: None,
                    payment_experience: None,
                    payment_method_type: Some(PaymentMethodType::Credit),
                    customer_id: None,
                    request_incremental_authorization: false,
                    metadata: None,
                    minor_amount: MinorUnit::new(1000),
                    merchant_order_reference_id: None,
                    shipping_cost: None,
                    merchant_account_id: None,
                    merchant_config_currency: None,
                },
                response: Err(ErrorResponse {
                    code: "HE_02".to_string(),
                    message: "Invalid format".to_string(),
                    reason: None,
                    status_code: 422,
                    attempt_status: None,
                    connector_transaction_id: None,
                }),
            };

            let connector: BoxedConnector = Box::new(Razorpay::new());
            let result = connector.get_request_body(&test_router_data);

            assert!(
                result.is_err(),
                "Expected error for invalid field values, but got: {:?}",
                result
            );
        }

        #[test]
        fn test_handle_response_v2_valid_authorize_response() {
            use domain_types::connector_types::{
                BoxedConnector, PaymentFlowData,
            };
            use domain_types::types::{ConnectorParams, Connectors};
            use hyperswitch_api_models::payments::{Address, PhoneDetails};
            use hyperswitch_common_enums::Currency;
            use hyperswitch_common_utils::pii::Email;
            use hyperswitch_common_utils::{id_type::MerchantId, types::MinorUnit};
            use hyperswitch_domain_models::{
                payment_address::PaymentAddress,
                router_data::{ConnectorAuthType, ErrorResponse},
                router_data_v2::RouterDataV2,
            };
            use std::str::FromStr;
            let connector: BoxedConnector = Box::new(Razorpay::new());
            let email = Email::try_from("testuser@gmail.com".to_string()).unwrap();

            let data = RouterDataV2 {
                flow: std::marker::PhantomData,
                resource_common_data: PaymentFlowData {
                    merchant_id: MerchantId::default(),
                    customer_id: None,
                    connector_customer: None,
                    payment_id: "IRRELEVANT_PAYMENT_ID".to_string(),
                    attempt_id: "IRRELEVANT_ATTEMPT_ID".to_string(),
                    status: AttemptStatus::Pending,
                    payment_method: PaymentMethod::Card,
                    description: None,
                    return_url: None,
                    address: PaymentAddress::new(
                        None,
                        Some(Address {
                            address: None,
                            phone: Some(PhoneDetails {
                                number: Some("1234567890".to_string().into()),
                                country_code: Some("+1".to_string()),
                            }),
                            email: Some(email.clone()),
                        }),
                        None,
                        None,
                    ),
                    auth_type: AuthenticationType::NoThreeDs,
                    connector_meta_data: None,
                    amount_captured: None,
                    minor_amount_captured: None,
                    access_token: None,
                    session_token: None,
                    reference_id: Some("order_QMsUrrLPdwNxPG".to_string()),
                    payment_method_token: None,
                    preprocessing_id: None,
                    connector_api_version: None,
                    connector_request_reference_id: "ref_12345".to_string(),
                    test_mode: None,
                    connector_http_status_code: None,
                    external_latency: None,
                    connectors: Connectors {
                        adyen: ConnectorParams {
                            base_url: "https://checkout-test.adyen.com/".to_string(),
                        },
                        razorpay: ConnectorParams {
                            base_url: "https://api.razorpay.com/".to_string(),
                        },
                    },
                },
                connector_auth_type: ConnectorAuthType::BodyKey {
                    api_key: "dummy_api_key".to_string().into(),
                    key1: "dummy_key1".to_string().into(),
                },
                request: PaymentsAuthorizeData {
                    payment_method_data: PaymentMethodData::Card(Card {
                        card_number: CardNumber::from_str("5123450000000008").unwrap(),
                        card_exp_month: "12".to_string().into(),
                        card_exp_year: "2025".to_string().into(),
                        card_cvc: "123".to_string().into(),
                        card_issuer: None,
                        card_network: None,
                        card_type: None,
                        card_issuing_country: None,
                        bank_code: None,
                        nick_name: None,
                    }),
                    amount: 1000,
                    order_tax_amount: None,
                    email: Some(email),
                    customer_name: None,
                    currency: Currency::USD,
                    confirm: true,
                    statement_descriptor_suffix: None,
                    statement_descriptor: None,
                    capture_method: None,
                    router_return_url: None,
                    webhook_url: None,
                    complete_authorize_url: None,
                    mandate_id: None,
                    setup_future_usage: None,
                    off_session: None,
                    browser_info: Some(BrowserInformation {
                        color_depth: None,
                        java_enabled: Some(false),
                        java_script_enabled: None,
                        language: Some("en-US".to_string()),
                        screen_height: Some(1080),
                        screen_width: Some(1920),
                        time_zone: None,
                        ip_address: None,
                        accept_header: Some(
                            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
                                .to_string(),
                        ),
                        user_agent: Some(
                            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)".to_string(),
                        ),
                    }),
                    order_category: None,
                    session_token: None,
                    enrolled_for_3ds: false,
                    related_transaction_id: None,
                    payment_experience: None,
                    payment_method_type: Some(hyperswitch_common_enums::PaymentMethodType::Credit),
                    customer_id: None,
                    request_incremental_authorization: false,
                    metadata: None,
                    minor_amount: MinorUnit::new(1000),
                    merchant_order_reference_id: None,
                    shipping_cost: None,
                    merchant_account_id: None,
                    merchant_config_currency: None,
                },
                response: Err(ErrorResponse {
                    code: "HE_00".to_string(),
                    message: "Something went wrong".to_string(),
                    reason: None,
                    status_code: 500,
                    attempt_status: None,
                    connector_transaction_id: None,
                }),
            };

            let http_response = Response {
                headers: None,
                response: br#"{
            "razorpay_payment_id":"pay_QMsUsXCDy9sX3b",
            "next":[
                {
                    "action":"redirect",
                    "url":"https://api.razorpay.com/v1/payments/QMsUsXCDy9sX3b/authenticate"
                }
            ]
        }"#
                .to_vec()
                .into(),
                status_code: 200,
            };

            let result = connector
                .handle_response_v2(&data, None, http_response)
                .unwrap();

            assert!(matches!(
                result.resource_common_data.status,
                AttemptStatus::AuthenticationPending
            ));
        }

        #[test]
        fn test_handle_authorize_error_response() {
            use domain_types::connector_flow::Authorize;
            use domain_types::connector_types::{
                BoxedConnector, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData,
            };

            let http_response = Response {
                headers: None,
                response: br#"{
                    "error": {
                        "code": "BAD_REQUEST_ERROR",
                        "description": "The id provided does not exist",
                        "source": "internal",
                        "step": "payment_initiation",
                        "reason": "input_validation_failed",
                        "metadata": {}
                    }
                }"#
                .to_vec()
                .into(),
                status_code: 400,
            };

            let connector: BoxedConnector = Box::new(Razorpay::new());

            let result = <dyn ConnectorServiceTrait + Sync as ConnectorIntegrationV2<
                Authorize,
                PaymentFlowData,
                PaymentsAuthorizeData,
                PaymentsResponseData,
            >>::get_error_response_v2(&**connector, http_response, None)
            .unwrap();

            let actual_json = to_value(&result).unwrap();

            let expected_json = json!({
                "code": "BAD_REQUEST_ERROR",
                "message": "The id provided does not exist",
                "reason": "input_validation_failed",
                "status_code": 400,
                "attempt_status": null,
                "connector_transaction_id": null
            });

            assert_eq!(actual_json, expected_json);
        }

        #[test]
        fn test_handle_authorize_missing_required_fields() {
            use domain_types::connector_flow::Authorize;
            use domain_types::connector_types::{
                BoxedConnector, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData,
            };

            let http_response = Response {
                headers: None,
                response: br#"{
                    "error": {
                        "description": "Missing required fields",
                        "step": "payment_initiation",
                        "reason": "input_validation_failed"
                    }
                }"#
                .to_vec()
                .into(),
                status_code: 400,
            };

            let connector: BoxedConnector = Box::new(Razorpay::new());

            let result = <dyn ConnectorServiceTrait + Sync as ConnectorIntegrationV2<
                Authorize,
                PaymentFlowData,
                PaymentsAuthorizeData,
                PaymentsResponseData,
            >>::get_error_response_v2(&**connector, http_response, None);

            assert!(
                result.is_err(),
                "Expected panic due to missing required fields",
            );
        }
    
    }

    #[test]
    fn test_handle_authorize_invalid_error_fields() {
        use domain_types::connector_flow::Authorize;
        use domain_types::connector_types::{
            BoxedConnector, PaymentFlowData, PaymentsAuthorizeData, PaymentsResponseData,
        };

        let http_response = Response {
            headers: None,
            response: br#"{
            "error": {
                "code": 500,
                "description": "Card number is invalid.",
                "step": "payment_authorization",
                "reason": "input_validation_failed",
                "source": "business",
                "metadata": {}
            }
        }"#
            .to_vec()
            .into(),
            status_code: 400,
        };

        let connector: BoxedConnector = Box::new(Razorpay::new());

        let result = <dyn ConnectorServiceTrait + Sync as ConnectorIntegrationV2<
            Authorize,
            PaymentFlowData,
            PaymentsAuthorizeData,
            PaymentsResponseData,
        >>::get_error_response_v2(&**connector, http_response, None);

        assert!(
            result.is_err(),
            "Expected panic due to missing required fields"
        );
    }

    #[test]
    fn test_handle_response_v2_missing_fields_authorize_response() {
        use domain_types::connector_types::{BoxedConnector, PaymentFlowData};
        use domain_types::types::{ConnectorParams, Connectors};
        use hyperswitch_api_models::payments::{Address, PhoneDetails};
        use hyperswitch_common_enums::Currency;
        use hyperswitch_common_utils::pii::Email;
        use hyperswitch_common_utils::{id_type::MerchantId, types::MinorUnit};
        use hyperswitch_domain_models::{
            payment_address::PaymentAddress,
            router_data::{ConnectorAuthType, ErrorResponse},
            router_data_v2::RouterDataV2,
        };
        use std::str::FromStr;

        let connector: BoxedConnector = Box::new(Razorpay::new());
        let email = Email::try_from("testuser@gmail.com".to_string()).unwrap();

        let data = RouterDataV2 {
            flow: std::marker::PhantomData,
            resource_common_data: PaymentFlowData {
                merchant_id: MerchantId::default(),
                customer_id: None,
                connector_customer: None,
                payment_id: "IRRELEVANT_PAYMENT_ID".to_string(),
                attempt_id: "IRRELEVANT_ATTEMPT_ID".to_string(),
                status: AttemptStatus::Pending,
                payment_method: PaymentMethod::Card,
                description: None,
                return_url: None,
                address: PaymentAddress::new(
                    None,
                    Some(Address {
                        address: None,
                        phone: Some(PhoneDetails {
                            number: Some("1234567890".to_string().into()),
                            country_code: Some("+1".to_string()),
                        }),
                        email: Some(email.clone()),
                    }),
                    None,
                    None,
                ),
                auth_type: AuthenticationType::NoThreeDs,
                connector_meta_data: None,
                amount_captured: None,
                minor_amount_captured: None,
                access_token: None,
                session_token: None,
                reference_id: Some("order_QMsUrrLPdwNxPG".to_string()),
                payment_method_token: None,
                preprocessing_id: None,
                connector_api_version: None,
                connector_request_reference_id: "ref_12345".to_string(),
                test_mode: None,
                connector_http_status_code: None,
                external_latency: None,
                connectors: Connectors {
                    adyen: ConnectorParams {
                        base_url: "https://checkout-test.adyen.com/".to_string(),
                    },
                    razorpay: ConnectorParams {
                        base_url: "https://api.razorpay.com/".to_string(),
                    },
                },
            },
            connector_auth_type: ConnectorAuthType::BodyKey {
                api_key: "dummy_api_key".to_string().into(),
                key1: "dummy_key1".to_string().into(),
            },
            request: PaymentsAuthorizeData {
                payment_method_data: PaymentMethodData::Card(Card {
                    card_number: CardNumber::from_str("5123450000000008").unwrap(),
                    card_exp_month: "12".to_string().into(),
                    card_exp_year: "2025".to_string().into(),
                    card_cvc: "123".to_string().into(),
                    card_issuer: None,
                    card_network: None,
                    card_type: None,
                    card_issuing_country: None,
                    bank_code: None,
                    nick_name: None,
                }),
                amount: 1000,
                order_tax_amount: None,
                email: Some(email),
                customer_name: None,
                currency: Currency::USD,
                confirm: true,
                statement_descriptor_suffix: None,
                statement_descriptor: None,
                capture_method: None,
                router_return_url: None,
                webhook_url: None,
                complete_authorize_url: None,
                mandate_id: None,
                setup_future_usage: None,
                off_session: None,
                browser_info: Some(BrowserInformation {
                    color_depth: None,
                    java_enabled: Some(false),
                    java_script_enabled: None,
                    language: Some("en-US".to_string()),
                    screen_height: Some(1080),
                    screen_width: Some(1920),
                    time_zone: None,
                    ip_address: None,
                    accept_header: Some(
                        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
                            .to_string(),
                    ),
                    user_agent: Some("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)".to_string()),
                }),
                order_category: None,
                session_token: None,
                enrolled_for_3ds: false,
                related_transaction_id: None,
                payment_experience: None,
                payment_method_type: Some(hyperswitch_common_enums::PaymentMethodType::Credit),
                customer_id: None,
                request_incremental_authorization: false,
                metadata: None,
                minor_amount: MinorUnit::new(1000),
                merchant_order_reference_id: None,
                shipping_cost: None,
                merchant_account_id: None,
                merchant_config_currency: None,
            },
            response: Err(ErrorResponse {
                code: "HE_00".to_string(),
                message: "Something went wrong".to_string(),
                reason: None,
                status_code: 500,
                attempt_status: None,
                connector_transaction_id: None,
            }),
        };

        let http_response = Response {
            headers: None,
            response: br#"{
            "next":[
                {
                    "action":"redirect",
                    "url":"https://api.razorpay.com/v1/payments/QMsUsXCDy9sX3b/authenticate"
                }
            ]
        }"#
            .to_vec()
            .into(),
            status_code: 200,
        };

        let result = connector.handle_response_v2(&data, None, http_response);

        assert!(
            result.is_err(),
            "Expected error due to missing razorpay_payment_id, but got success."
        );
    }

    #[test]
    fn test_handle_response_v2_invalid_json_authorize_response() {
        use domain_types::connector_types::{BoxedConnector, PaymentFlowData};
        use domain_types::types::{ConnectorParams, Connectors};
        use hyperswitch_api_models::payments::{Address, PhoneDetails};
        use hyperswitch_common_enums::Currency;
        use hyperswitch_common_utils::pii::Email;
        use hyperswitch_common_utils::{id_type::MerchantId, types::MinorUnit};
        use hyperswitch_domain_models::{
            payment_address::PaymentAddress,
            router_data::{ConnectorAuthType, ErrorResponse},
            router_data_v2::RouterDataV2,
        };
        use std::str::FromStr;

        let connector: BoxedConnector = Box::new(Razorpay::new());
        let email = Email::try_from("testuser@gmail.com".to_string()).unwrap();

        let data = RouterDataV2 {
            flow: std::marker::PhantomData,
            resource_common_data: PaymentFlowData {
                merchant_id: MerchantId::default(),
                customer_id: None,
                connector_customer: None,
                payment_id: "IRRELEVANT_PAYMENT_ID".to_string(),
                attempt_id: "IRRELEVANT_ATTEMPT_ID".to_string(),
                status: AttemptStatus::Pending,
                payment_method: PaymentMethod::Card,
                description: None,
                return_url: None,
                address: PaymentAddress::new(
                    None,
                    Some(Address {
                        address: None,
                        phone: Some(PhoneDetails {
                            number: Some("1234567890".to_string().into()),
                            country_code: Some("+1".to_string()),
                        }),
                        email: Some(email.clone()),
                    }),
                    None,
                    None,
                ),
                auth_type: AuthenticationType::NoThreeDs,
                connector_meta_data: None,
                amount_captured: None,
                minor_amount_captured: None,
                access_token: None,
                session_token: None,
                reference_id: Some("order_QMsUrrLPdwNxPG".to_string()),
                payment_method_token: None,
                preprocessing_id: None,
                connector_api_version: None,
                connector_request_reference_id: "ref_12345".to_string(),
                test_mode: None,
                connector_http_status_code: None,
                external_latency: None,
                connectors: Connectors {
                    adyen: ConnectorParams {
                        base_url: "https://checkout-test.adyen.com/".to_string(),
                    },
                    razorpay: ConnectorParams {
                        base_url: "https://api.razorpay.com/".to_string(),
                    },
                },
            },
            connector_auth_type: ConnectorAuthType::BodyKey {
                api_key: "dummy_api_key".to_string().into(),
                key1: "dummy_key1".to_string().into(),
            },
            request: PaymentsAuthorizeData {
                payment_method_data: PaymentMethodData::Card(Card {
                    card_number: CardNumber::from_str("5123450000000008").unwrap(),
                    card_exp_month: "12".to_string().into(),
                    card_exp_year: "2025".to_string().into(),
                    card_cvc: "123".to_string().into(),
                    card_issuer: None,
                    card_network: None,
                    card_type: None,
                    card_issuing_country: None,
                    bank_code: None,
                    nick_name: None,
                }),
                amount: 1000,
                order_tax_amount: None,
                email: Some(email),
                customer_name: None,
                currency: Currency::USD,
                confirm: true,
                statement_descriptor_suffix: None,
                statement_descriptor: None,
                capture_method: None,
                router_return_url: None,
                webhook_url: None,
                complete_authorize_url: None,
                mandate_id: None,
                setup_future_usage: None,
                off_session: None,
                browser_info: Some(BrowserInformation {
                    color_depth: None,
                    java_enabled: Some(false),
                    java_script_enabled: None,
                    language: Some("en-US".to_string()),
                    screen_height: Some(1080),
                    screen_width: Some(1920),
                    time_zone: None,
                    ip_address: None,
                    accept_header: Some(
                        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
                            .to_string(),
                    ),
                    user_agent: Some("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)".to_string()),
                }),
                order_category: None,
                session_token: None,
                enrolled_for_3ds: false,
                related_transaction_id: None,
                payment_experience: None,
                payment_method_type: Some(hyperswitch_common_enums::PaymentMethodType::Credit),
                customer_id: None,
                request_incremental_authorization: false,
                metadata: None,
                minor_amount: MinorUnit::new(1000),
                merchant_order_reference_id: None,
                shipping_cost: None,
                merchant_account_id: None,
                merchant_config_currency: None,
            },
            response: Err(ErrorResponse {
                code: "HE_00".to_string(),
                message: "Something went wrong".to_string(),
                reason: None,
                status_code: 500,
                attempt_status: None,
                connector_transaction_id: None,
            }),
        };

        let http_response = Response {
        headers: None,
        response: br#"{"razorpay_payment_id": "pay_xyz", "next": [ { "action": "redirect" "url": "https://api.razorpay.com/v1/payments/xyz/authenticate" } ]"#.to_vec().into(),
        status_code: 200,
    };

        let result = connector.handle_response_v2(&data, None, http_response);

        assert!(
            result.is_err(),
            "Expected error due to missing razorpay_payment_id, but got success."
        );
    }

    mod order {
        use domain_types::types::{ConnectorParams, Connectors};
        use hyperswitch_api_models::payments::{Address, PhoneDetails};
        use hyperswitch_common_utils::{pii::Email, request::RequestContent};
        use hyperswitch_domain_models::router_data::ConnectorAuthType;
        use serde_json::{to_value, Value};

        use crate::connectors::Razorpay;

        #[test]
        fn test_build_request_valid_order() {
            use hyperswitch_common_enums::Currency;
            use hyperswitch_common_utils::{
                id_type::MerchantId, request::RequestContent, types::MinorUnit,
            };
            use hyperswitch_domain_models::{
                payment_address::PaymentAddress,
                router_data::{ConnectorAuthType, ErrorResponse},
                router_data_v2::RouterDataV2,
            };
            use serde_json::{to_value, Value};

            
            use domain_types::connector_types::{BoxedConnector, PaymentCreateOrderData};

            let email = Email::try_from("testuser@gmail.com".to_string()).unwrap();

            let test_router_data = RouterDataV2 {
                flow: std::marker::PhantomData,
                resource_common_data: domain_types::connector_types::PaymentFlowData {
                    merchant_id: MerchantId::default(),
                    customer_id: None,
                    connector_customer: None,
                    payment_id: "IRRELEVANT_PAYMENT_ID".to_string(),
                    attempt_id: "IRRELEVANT_ATTEMPT_ID".to_string(),
                    status: hyperswitch_common_enums::AttemptStatus::Pending,
                    payment_method: hyperswitch_common_enums::PaymentMethod::Card,
                    description: None,
                    return_url: None,
                    address: PaymentAddress::new(
                        None,
                        Some(Address {
                            address: None,
                            phone: Some(PhoneDetails {
                                number: Some("1234567890".to_string().into()),
                                country_code: Some("+1".to_string()),
                            }),
                            email: Some(email.clone()),
                        }),
                        None,
                        None,
                    ),
                    auth_type: hyperswitch_common_enums::AuthenticationType::NoThreeDs,
                    connector_meta_data: None,
                    amount_captured: None,
                    minor_amount_captured: None,
                    access_token: None,
                    session_token: None,
                    reference_id: None,
                    payment_method_token: None,
                    preprocessing_id: None,
                    connector_api_version: None,
                    connector_request_reference_id: "ref_12345".to_string(),
                    test_mode: None,
                    connector_http_status_code: None,
                    external_latency: None,
                    connectors: domain_types::types::Connectors {
                        adyen: domain_types::types::ConnectorParams {
                            base_url: "https://checkout-test.adyen.com/".to_string(),
                        },
                        razorpay: domain_types::types::ConnectorParams {
                            base_url: "https://api.razorpay.com/".to_string(),
                        },
                    },
                },
                connector_auth_type: ConnectorAuthType::BodyKey {
                    api_key: "dummy_api_key".to_string().into(),
                    key1: "dummy_key1".to_string().into(),
                },
                request: PaymentCreateOrderData {
                    amount: MinorUnit::new(1000),
                    currency: Currency::USD,
                },
                response: Err(ErrorResponse {
                    code: "HE_00".to_string(),
                    message: "Something went wrong".to_string(),
                    reason: None,
                    status_code: 500,
                    attempt_status: None,
                    connector_transaction_id: None,
                }),
            };

            let connector: BoxedConnector = Box::new(Razorpay::new());
            let result = connector.get_request_body(&test_router_data).unwrap();

            let actual_json: Value = match result {
                Some(RequestContent::Json(payload)) => {
                    to_value(&payload).expect("Failed to serialize payload")
                }
                _ => panic!("Expected JSON payload"),
            };

            assert_eq!(actual_json["amount"], 1000);
            assert_eq!(actual_json["currency"], "USD");

            let receipt_value = &actual_json["receipt"];
            assert!(
                receipt_value.is_string(),
                "Expected receipt to be a string, got: {:?}",
                receipt_value
            );
            let receipt_str = receipt_value.as_str().unwrap();
            assert!(!receipt_str.is_empty(), "Expected non-empty receipt string");

          
        }

        #[test]
        fn test_build_request_missing() {
            use hyperswitch_common_enums::Currency;
            use hyperswitch_common_utils::{id_type::MerchantId, types::MinorUnit};
            use hyperswitch_domain_models::{
                payment_address::PaymentAddress,
                router_data::{ConnectorAuthType, ErrorResponse},
                router_data_v2::RouterDataV2,
            };

            use crate::connectors::Razorpay;
            use domain_types::connector_types::{BoxedConnector, PaymentCreateOrderData};

            let test_router_data = RouterDataV2 {
                flow: std::marker::PhantomData,
                resource_common_data: domain_types::connector_types::PaymentFlowData {
                    merchant_id: MerchantId::default(),
                    customer_id: None,
                    connector_customer: None,
                    payment_id: "".to_string(),
                    attempt_id: "".to_string(),
                    status: hyperswitch_common_enums::AttemptStatus::Pending,
                    payment_method: hyperswitch_common_enums::PaymentMethod::Card,
                    description: None,
                    return_url: None,
                    address: PaymentAddress::default(),
                    auth_type: hyperswitch_common_enums::AuthenticationType::NoThreeDs,
                    connector_meta_data: None,
                    amount_captured: None,
                    minor_amount_captured: None,
                    access_token: None,
                    session_token: None,
                    reference_id: None,
                    payment_method_token: None,
                    preprocessing_id: None,
                    connector_api_version: None,
                    connector_request_reference_id: "".to_string(),
                    test_mode: None,
                    connector_http_status_code: None,
                    external_latency: None,
                    connectors: Connectors {
                        adyen: ConnectorParams {
                            base_url: "https://checkout-test.adyen.com/".to_string(),
                        },
                        razorpay: ConnectorParams {
                            base_url: "https://api.razorpay.com/".to_string(),
                        },
                    },
                },
                connector_auth_type: ConnectorAuthType::BodyKey {
                    api_key: "dummy_api_key".to_string().into(),
                    key1: "dummy_key1".to_string().into(),
                },
                request: PaymentCreateOrderData {
                    amount: MinorUnit::new(0),
                    currency: Currency::default(),
                },
                response: Err(ErrorResponse {
                    code: "HE_01".to_string(),
                    message: "Missing required fields".to_string(),
                    reason: None,
                    status_code: 400,
                    attempt_status: None,
                    connector_transaction_id: None,
                }),
            };

            let connector: BoxedConnector = Box::new(Razorpay::new());
            let result = connector.get_request_body(&test_router_data);
            let req = result.unwrap();

            let actual_json: Value = match req {
                Some(RequestContent::Json(payload)) => {
                    to_value(&payload).expect("Failed to serialize payload")
                }
                _ => panic!("Expected JSON payload"),
            };

            assert_eq!(actual_json["amount"], 0);
            assert_eq!(actual_json["currency"], "USD");

            let receipt_value = &actual_json["receipt"];
            assert!(
                receipt_value.is_string(),
                "Expected receipt to be a string, got: {:?}",
                receipt_value
            );
            let receipt_str = receipt_value.as_str().unwrap();
            assert!(!receipt_str.is_empty(), "Expected non-empty receipt string");

        }

        #[test]
        fn test_build_request_invalid() {
            use crate::connectors::Razorpay;
            use domain_types::connector_types::{BoxedConnector, PaymentFlowData, PaymentsAuthorizeData};
            use domain_types::types::{ConnectorParams, Connectors};
            use hyperswitch_common_enums::{
                AttemptStatus, AuthenticationType, Currency, PaymentMethod, PaymentMethodType,
            };
            use hyperswitch_common_utils::{id_type::MerchantId, types::MinorUnit};
            use hyperswitch_domain_models::{
                payment_address::PaymentAddress,
                payment_method_data::{Card, PaymentMethodData},
                router_data::ErrorResponse,
                router_data_v2::RouterDataV2,
            };
        
            let test_router_data = RouterDataV2 {
                flow: std::marker::PhantomData,
                resource_common_data: PaymentFlowData {
                    merchant_id: MerchantId::default(),
                    customer_id: None,
                    connector_customer: None,
                    payment_id: "invalid_payment_id".to_string(),
                    attempt_id: "invalid_attempt_id".to_string(),
                    status: AttemptStatus::Pending,
                    payment_method: PaymentMethod::Card,
                    description: None,
                    return_url: None,
                    address: PaymentAddress::new(None, None, None, None),
                    auth_type: AuthenticationType::NoThreeDs,
                    connector_meta_data: None,
                    amount_captured: None,
                    minor_amount_captured: None,
                    access_token: None,
                    session_token: None,
                    reference_id: Some("order_invalid".to_string()),
                    payment_method_token: None,
                    preprocessing_id: None,
                    connector_api_version: None,
                    connector_request_reference_id: "ref_invalid".to_string(),
                    test_mode: None,
                    connector_http_status_code: None,
                    external_latency: None,
                    connectors: Connectors {
                        adyen: ConnectorParams {
                            base_url: "".to_string(),
                        },
                        razorpay: ConnectorParams {
                            base_url: "".to_string(),
                        },
                    },
                },
                connector_auth_type: ConnectorAuthType::BodyKey {
                    api_key: "invalid_key".to_string().into(),
                    key1: "invalid_key1".to_string().into(),
                },
                request: PaymentsAuthorizeData {
                    payment_method_data: PaymentMethodData::Card(Card {
                        card_number: Default::default(),
                        card_exp_month: "".to_string().into(),
                        card_exp_year: "".to_string().into(),
                        card_cvc: "".to_string().into(),
                        card_issuer: None,
                        card_network: None,
                        card_type: None,
                        card_issuing_country: None,
                        bank_code: None,
                        nick_name: None,
                    }),
                    amount: 1000,
                    order_tax_amount: None,
                    email: None,
                    customer_name: None,
                    currency: Currency::USD,
                    confirm: true,
                    statement_descriptor_suffix: None,
                    statement_descriptor: None,
                    capture_method: None,
                    router_return_url: None,
                    webhook_url: None,
                    complete_authorize_url: None,
                    mandate_id: None,
                    setup_future_usage: None,
                    off_session: None,
                    browser_info: None,
                    order_category: None,
                    session_token: None,
                    enrolled_for_3ds: false,
                    related_transaction_id: None,
                    payment_experience: None,
                    payment_method_type: Some(PaymentMethodType::Credit),
                    customer_id: None,
                    request_incremental_authorization: false,
                    metadata: None,
                    minor_amount: MinorUnit::new(1000),
                    merchant_order_reference_id: None,
                    shipping_cost: None,
                    merchant_account_id: None,
                    merchant_config_currency: None,
                },
                response: Err(ErrorResponse {
                    code: "HE_INVALID".to_string(),
                    message: "Invalid request body".to_string(),
                    reason: None,
                    status_code: 422,
                    attempt_status: None,
                    connector_transaction_id: None,
                }),
            };
        
            let connector: BoxedConnector = Box::new(Razorpay::new());
            let result = connector.get_request_body(&test_router_data);
        
            assert!(
                result.is_err(),
                "Expected error for invalid request data, but got: {:?}",
                result
            );
        }
        
    }

    #[test]
    fn test_handle_response_v2_valid_order_response() {
        use domain_types::connector_types::{
            BoxedConnector, PaymentCreateOrderData, PaymentFlowData,
        };
        use domain_types::types::{ConnectorParams, Connectors};
        use hyperswitch_api_models::payments::{Address, PhoneDetails};
        use hyperswitch_common_enums::Currency;
        use hyperswitch_common_utils::pii::Email;
        use hyperswitch_common_utils::{id_type::MerchantId, types::MinorUnit};
        use hyperswitch_domain_models::{
            payment_address::PaymentAddress,
            router_data::{ConnectorAuthType, ErrorResponse},
            router_data_v2::RouterDataV2,
        };
        let email = Email::try_from("testuser@gmail.com".to_string()).unwrap();
        let connector: BoxedConnector = Box::new(Razorpay::new());

        let data = RouterDataV2 {
            flow: std::marker::PhantomData,
            resource_common_data: PaymentFlowData {
                merchant_id: MerchantId::default(),
                customer_id: None,
                connector_customer: None,
                payment_id: "IRRELEVANT_PAYMENT_ID".to_string(),
                attempt_id: "IRRELEVANT_ATTEMPT_ID".to_string(),
                status: hyperswitch_common_enums::AttemptStatus::Pending,
                payment_method: hyperswitch_common_enums::PaymentMethod::Card,
                description: None,
                return_url: None,
                address: PaymentAddress::new(
                    None,
                    Some(Address {
                        address: None,
                        phone: Some(PhoneDetails {
                            number: Some("1234567890".to_string().into()),
                            country_code: Some("+1".to_string()),
                        }),
                        email: Some(email.clone()),
                    }),
                    None,
                    None,
                ),
                auth_type: hyperswitch_common_enums::AuthenticationType::NoThreeDs,
                connector_meta_data: None,
                amount_captured: None,
                minor_amount_captured: None,
                access_token: None,
                session_token: None,
                reference_id: None,
                payment_method_token: None,
                preprocessing_id: None,
                connector_api_version: None,
                connector_request_reference_id: "ref_12345".to_string(),
                test_mode: None,
                connector_http_status_code: None,
                external_latency: None,
                connectors: Connectors {
                    adyen: ConnectorParams {
                        base_url: "https://checkout-test.adyen.com/".to_string(),
                    },
                    razorpay: ConnectorParams {
                        base_url: "https://api.razorpay.com/".to_string(),
                    },
                },
            },
            connector_auth_type: ConnectorAuthType::BodyKey {
                api_key: "dummy_api_key".to_string().into(),
                key1: "dummy_key1".to_string().into(),
            },
            request: PaymentCreateOrderData {
                amount: MinorUnit::new(1000),
                currency: Currency::USD,
            },
            response: Err(ErrorResponse {
                code: "HE_00".to_string(),
                message: "Something went wrong".to_string(),
                reason: None,
                status_code: 500,
                attempt_status: None,
                connector_transaction_id: None,
            }),
        };


        let http_response = Response {
            headers: None,
            response: br#"{
                "amount":1000,
                "amount_due":1000,
                "amount_paid":0,
                "attempts":0,
                "created_at":1745490447,
                "currency":"USD",
                "entity":"order",
                "id":"order_QMrTOdLWvEHsXz",
                "notes":[],
                "offer_id":null,
                "receipt":"141674f6-30d3-4a17-b904-27fe6ca085c7",
                "status":"created"
            }"#
            .to_vec()
            .into(),
            status_code: 200,
        };

        let result = connector
            .handle_response_v2(&data, None, http_response)
            .unwrap();

        assert_eq!(
            result.response.unwrap().order_id,
            "order_QMrTOdLWvEHsXz".to_string()
        );

    }

    #[test]
    fn test_handle_response_missing() {
        use domain_types::connector_types::{BoxedConnector, PaymentCreateOrderData};
        use domain_types::types::{ConnectorParams, Connectors};
        use hyperswitch_api_models::payments::{Address, PhoneDetails};
        use hyperswitch_common_enums::Currency;
        use hyperswitch_common_utils::pii::Email;
        use hyperswitch_common_utils::{id_type::MerchantId, types::MinorUnit};
        use hyperswitch_domain_models::{
            payment_address::PaymentAddress,
            router_data::{ConnectorAuthType, ErrorResponse},
            router_data_v2::RouterDataV2,
        };

        let email = Email::try_from("testuser@gmail.com".to_string()).unwrap();
        let connector: BoxedConnector = Box::new(Razorpay::new());

        let data = RouterDataV2 {
            flow: std::marker::PhantomData,
            resource_common_data: PaymentFlowData {
                merchant_id: MerchantId::default(),
                customer_id: None,
                connector_customer: None,
                payment_id: "IRRELEVANT_PAYMENT_ID".to_string(),
                attempt_id: "IRRELEVANT_ATTEMPT_ID".to_string(),
                status: hyperswitch_common_enums::AttemptStatus::Pending,
                payment_method: hyperswitch_common_enums::PaymentMethod::Card,
                description: None,
                return_url: None,
                address: PaymentAddress::new(
                    None,
                    Some(Address {
                        address: None,
                        phone: Some(PhoneDetails {
                            number: Some("1234567890".to_string().into()),
                            country_code: Some("+1".to_string()),
                        }),
                        email: Some(email.clone()),
                    }),
                    None,
                    None,
                ),
                auth_type: hyperswitch_common_enums::AuthenticationType::NoThreeDs,
                connector_meta_data: None,
                amount_captured: None,
                minor_amount_captured: None,
                access_token: None,
                session_token: None,
                reference_id: None,
                payment_method_token: None,
                preprocessing_id: None,
                connector_api_version: None,
                connector_request_reference_id: "ref_12345".to_string(),
                test_mode: None,
                connector_http_status_code: None,
                external_latency: None,
                connectors: Connectors {
                    adyen: ConnectorParams {
                        base_url: "https://checkout-test.adyen.com/".to_string(),
                    },
                    razorpay: ConnectorParams {
                        base_url: "https://api.razorpay.com/".to_string(),
                    },
                },
            },
            connector_auth_type: ConnectorAuthType::BodyKey {
                api_key: "dummy_api_key".to_string().into(),
                key1: "dummy_key1".to_string().into(),
            },
            request: PaymentCreateOrderData {
                amount: MinorUnit::new(1000),
                currency: Currency::USD,
            },
            response: Err(ErrorResponse {
                code: "HE_00".to_string(),
                message: "Something went wrong".to_string(),
                reason: None,
                status_code: 500,
                attempt_status: None,
                connector_transaction_id: None,
            }),
        };

        let http_response = Response {
            headers: None,
            response: br#"{
            "amount":1000,
            "currency":"USD",
            "status":"created"
        }"#
            .to_vec()
            .into(),
            status_code: 200,
        };

        let result = connector.handle_response_v2(&data, None, http_response);

        assert!(
            result.is_err(),
            "Expected error due to missing order_id or receipt"
        );
    }

    #[test]
    fn test_handle_response_invalid() {
        use domain_types::connector_types::{BoxedConnector, PaymentCreateOrderData};
        use domain_types::types::{ConnectorParams, Connectors};
        use hyperswitch_api_models::payments::{Address, PhoneDetails};
        use hyperswitch_common_enums::Currency;
        use hyperswitch_common_utils::pii::Email;
        use hyperswitch_common_utils::{id_type::MerchantId, types::MinorUnit};
        use hyperswitch_domain_models::{
            payment_address::PaymentAddress,
            router_data::{ConnectorAuthType, ErrorResponse},
            router_data_v2::RouterDataV2,
        };

        let email = Email::try_from("testuser@gmail.com".to_string()).unwrap();
        let connector: BoxedConnector = Box::new(Razorpay::new());

        let data = RouterDataV2 {
            flow: std::marker::PhantomData,
            resource_common_data: PaymentFlowData {
                merchant_id: MerchantId::default(),
                customer_id: None,
                connector_customer: None,
                payment_id: "IRRELEVANT_PAYMENT_ID".to_string(),
                attempt_id: "IRRELEVANT_ATTEMPT_ID".to_string(),
                status: hyperswitch_common_enums::AttemptStatus::Pending,
                payment_method: hyperswitch_common_enums::PaymentMethod::Card,
                description: None,
                return_url: None,
                address: PaymentAddress::new(
                    None,
                    Some(Address {
                        address: None,
                        phone: Some(PhoneDetails {
                            number: Some("1234567890".to_string().into()),
                            country_code: Some("+1".to_string()),
                        }),
                        email: Some(email.clone()),
                    }),
                    None,
                    None,
                ),
                auth_type: hyperswitch_common_enums::AuthenticationType::NoThreeDs,
                connector_meta_data: None,
                amount_captured: None,
                minor_amount_captured: None,
                access_token: None,
                session_token: None,
                reference_id: None,
                payment_method_token: None,
                preprocessing_id: None,
                connector_api_version: None,
                connector_request_reference_id: "ref_12345".to_string(),
                test_mode: None,
                connector_http_status_code: None,
                external_latency: None,
                connectors: Connectors {
                    adyen: ConnectorParams {
                        base_url: "https://checkout-test.adyen.com/".to_string(),
                    },
                    razorpay: ConnectorParams {
                        base_url: "https://api.razorpay.com/".to_string(),
                    },
                },
            },
            connector_auth_type: ConnectorAuthType::BodyKey {
                api_key: "dummy_api_key".to_string().into(),
                key1: "dummy_key1".to_string().into(),
            },
            request: PaymentCreateOrderData {
                amount: MinorUnit::new(1000),
                currency: Currency::USD,
            },
            response: Err(ErrorResponse {
                code: "HE_00".to_string(),
                message: "Something went wrong".to_string(),
                reason: None,
                status_code: 500,
                attempt_status: None,
                connector_transaction_id: None,
            }),
        };

        let http_response = Response {
            headers: None,
            response: br#"{
            "amount":1000,
            "currency":"USD",
            "status":"created"
        }"#
            .to_vec()
            .into(),
            status_code: 500,
        };

        let result = connector.handle_response_v2(&data, None, http_response);

        assert!(
            result.is_err(),
            "Expected error due to invalid response format"
        );
    }

    #[test]
    fn test_handle_error_response_valid() {
        use domain_types::connector_types::BoxedConnector;

        let http_response = Response {
            headers: None,
            response: br#"{
                "error": {
                    "code": "BAD_REQUEST_ERROR",
                    "description": "Order receipt should be unique.",
                    "step": "payment_initiation",
                    "reason": "input_validation_failed",
                    "source": "business",
                    "metadata": {
                        "order_id": "order_OL0t841dI8F9NV"
                    }
                }
            }"#
            .to_vec()
            .into(),
            status_code: 400,
        };
        let connector: BoxedConnector = Box::new(Razorpay::new());

        let result = <dyn ConnectorServiceTrait + Sync as ConnectorIntegrationV2<
            domain_types::connector_flow::CreateOrder,
            domain_types::connector_types::PaymentFlowData,
            domain_types::connector_types::PaymentCreateOrderData,
            domain_types::connector_types::PaymentCreateOrderResponse,
        >>::get_error_response_v2(&**connector, http_response, None)
        .unwrap();

        let actual_json = to_value(&result).unwrap();
        let expected_json = json!({
            "code": "BAD_REQUEST_ERROR",
            "message": "Order receipt should be unique.",
            "reason": "input_validation_failed",
            "status_code": 400,
            "attempt_status": null,
            "connector_transaction_id": null
        });
        assert_eq!(actual_json, expected_json);
       
    }

    #[test]
    fn test_handle_error_response_invalid_json() {
     
        let http_response = Response {
            headers: None,
            response: br#"{ "error": { "code": "BAD_REQUEST_ERROR" "#.to_vec().into(),
            status_code: 400,
        };

        let connector: BoxedConnector = Box::new(Razorpay::new());

        let result = <dyn ConnectorServiceTrait + Sync as ConnectorIntegrationV2<
            domain_types::connector_flow::CreateOrder,
            domain_types::connector_types::PaymentFlowData,
            domain_types::connector_types::PaymentCreateOrderData,
            domain_types::connector_types::PaymentCreateOrderResponse,
        >>::get_error_response_v2(&**connector, http_response, None);

        assert!(result.is_err(), "Expected error for invalid JSON");
    }
    #[test]
    fn test_handle_error_response_missing_error_field() {
       
        let http_response = Response {
            headers: None,
            response: br#"{
            "message": "Some generic message",
            "status": "failed"
        }"#
            .to_vec()
            .into(),
            status_code: 400,
        };

        let connector: BoxedConnector = Box::new(Razorpay::new());

        let result = <dyn ConnectorServiceTrait + Sync as ConnectorIntegrationV2<
            domain_types::connector_flow::CreateOrder,
            domain_types::connector_types::PaymentFlowData,
            domain_types::connector_types::PaymentCreateOrderData,
            domain_types::connector_types::PaymentCreateOrderResponse,
        >>::get_error_response_v2(&**connector, http_response, None);

        assert!(result.is_err(), "Expected error for missing 'error' field");
    }
}
