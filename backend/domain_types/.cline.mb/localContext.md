# Local Context: Domain Types Component (`backend/domain_types`)

## 1. Component Purpose and Responsibilities

This component (`backend/domain_types/`) is central to the `connector-service` architecture. Its primary responsibilities are:

*   **Defining Core Data Structures:** Specifies the primary Rust structs and enums that represent business objects like payments, refunds, customers, addresses, and payment methods. These types are used consistently across the `grpc-server`, `connector-integration`, and `external-services` layers.
*   **Modeling Payment Flows:** Defines marker types or enums (e.g., `Authorize`, `Capture`, `Refund` in `connector_flow.rs`) that represent different stages or types of payment operations.
*   **Defining Connector Abstractions:** Contains key traits that define the contract for payment connectors, most notably the `ConnectorServiceTrait` and its constituent traits (like `PaymentAuthorizeV2`, `RefundV2`, etc.), which often alias or build upon `ConnectorIntegrationV2` from `hyperswitch_interfaces`.
*   **Type Conversions:** Provides implementations (often using `ForeignTryFrom`) for converting data between gRPC types (from `grpc-api-types`) and the internal domain models. This is crucial for decoupling the API layer from the core business logic.
*   **Standardized Error Handling:** Defines common error types (`errors.rs`) used within the domain layer and potentially propagated or mapped by other components.

## 2. Relationship to Overall System

*   **Parent Context:** This Local Memory Bank inherits from and builds upon the Root Memory Bank (`/.cline.mb/`).
*   **Central Data Model:** Acts as the "source of truth" for many data structures and enum definitions used by other backend components.
*   **Used By:**
    *   `grpc-server`: For request/response data mapping and understanding internal data structures.
    *   `connector-integration`: For the data it needs to adapt for specific connectors and for implementing connector traits.
    *   `external-services`: For understanding the `RouterDataV2` structure it processes.
*   **Uses:**
    *   `grpc-api-types`: For the gRPC types it needs to convert from/to.
    *   `hyperswitch_interfaces`, `hyperswitch_domain_models`, `hyperswitch_common_enums`, `hyperswitch_common_utils`, `hyperswitch_api_models`, `hyperswitch_cards`: Leverages these foundational crates from the Hyperswitch ecosystem for base types, traits, and utilities.

## 3. Integration Points

*   **Internal:** This crate is primarily integrated with other backend Rust crates within the same workspace. Its types and traits are directly used.
*   **External:** None directly. It defines internal models.

## 4. Local Design Decisions (Confirmed)

*   **Trait-Based Abstraction for Connectors:** The use of `ConnectorIntegrationV2` and related traits is a core design decision for abstracting connector behavior.
*   **`RouterDataV2` as a Central Flow Carrier:** This generic struct is used to carry all necessary data (request, response, common resources, auth) through different processing stages for various flows.
*   **`ForeignTryFrom` for Conversions:** This custom trait is consistently used for type mapping between layers.
*   **Marker Structs for Flows:** Empty structs in `connector_flow.rs` (e.g., `Authorize`, `PSync`) are used as type parameters to specialize generic logic like `RouterDataV2` and `ConnectorIntegrationV2`.

## 5. Component-Specific Constraints

*   **Clarity and Stability:** As a core component defining shared types and traits, its API needs to be clear and relatively stable to avoid widespread changes in dependent crates.
*   **Performance of Conversions:** Type conversions should be efficient.

*(This is an initial draft based on recent analysis. It will be refined as the component's code is analyzed in more detail.)*
