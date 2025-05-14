# System Patterns: Connector Service

## 1. Overall Architecture

The system follows a **layered architecture** with a clear separation of concerns:

*   **API Layer (gRPC):** `backend/grpc-server/` and `backend/grpc-api-types/` provide a gRPC-based interface for external communication. This layer handles incoming requests, authenticates them, and translates them into internal operations.
*   **Core Logic/Domain Layer:** `backend/domain_types/` defines the core business objects, data structures, and traits used throughout the system. It provides a standardized representation of payments, refunds, and other entities, independent of specific connectors or API protocols.
*   **Connector Integration Layer:** `backend/connector-integration/` manages interactions with third-party payment gateways. It uses an Adapter pattern, with specific modules for each connector (e.g., Adyen, Razorpay). This layer implements the `ConnectorIntegrationV2` trait (from `hyperswitch_interfaces`) to provide a consistent interface for payment operations.
*   **External Services Layer:** `backend/external-services/` contains the `execute_connector_processing_step` function, which is the engine for making HTTP calls to connectors.
*   **SDKs/Client Libraries:** The `sdk/` directory provides client libraries for different languages to simplify integration with the gRPC API.
*   **Examples:** The `examples/` directory provides practical usage scenarios and sample code.

```mermaid
graph TD
    ClientApp[Client Applications] -->|gRPC API| APILayer[API Layer (grpc-server)]
    APILayer --> CoreLogic[Core Logic (domain_types)]
    CoreLogic --> ConnectorIntegration[Connector Integration Layer (connector-integration)]
    ConnectorIntegration --> Adyen[Adyen Connector]
    ConnectorIntegration --> Razorpay[Razorpay Connector]
    ConnectorIntegration --> OtherConnectors[...]
    
    APILayer --> ExternalServices[External Services Layer (external-services)]
    ExternalServices -- "execute_connector_processing_step" --> ConnectorIntegration

    subgraph SDKs
        NodeSDK[Node.js SDK]
        PythonSDK[Python SDK]
        RustSDK[Rust SDK]
    end
    
    ClientApp --> SDKs

    style APILayer fill:#f9f,stroke:#333,stroke-width:2px
    style CoreLogic fill:#ccf,stroke:#333,stroke-width:2px
    style ConnectorIntegration fill:#cfc,stroke:#333,stroke-width:2px
    style ExternalServices fill:#ffc,stroke:#333,stroke-width:2px
```

## 2. Key Design Patterns

*   **Adapter Pattern:** Each connector implementation within `backend/connector-integration/src/connectors/` (e.g., `adyen.rs`, `razorpay.rs`) acts as an adapter, translating the system's internal payment models and operations into the specific format and protocol required by the external payment gateway. The `ConnectorIntegrationV2` trait (from `hyperswitch_interfaces`) defines the common interface for these adapters.
*   **Strategy Pattern (Implicit):** The selection of a specific connector at runtime is achieved through a strategy pattern. The `grpc-server` selects the appropriate connector based on request parameters or merchant configuration.
*   **Data Transfer Objects (DTOs):** The `proto` files in `backend/grpc-api-types/proto/` define the structure of data exchanged over gRPC. These act as DTOs. Internal domain types (`backend/domain_types/`) are mapped to/from these DTOs at the API layer using `ForeignTryFrom` implementations.
*   **Template Method Pattern:** The `external-services/src/service.rs` crate's `execute_connector_processing_step` function implements a Template Method pattern. It defines the overall steps for processing a connector request (building the request, making the API call, handling the response), while delegating the connector-specific details (like building the request body and parsing the response) to the concrete connector implementations.
*   **Configuration Management:** The `config/development.toml` file provides a centralized approach to configuration. The `backend/grpc-server/src/configs.rs` handles loading and providing this configuration to the application.
*   **Macros for Code Generation/Boilerplate Reduction:** The `backend/connector-integration/src/connectors/macros.rs` file uses macros to reduce boilerplate code, especially for defining common connector functionalities or traits.

## 3. Component Relationships

*   `grpc-server` depends on `grpc-api-types` for request/response structures and service definitions.
*   `grpc-server` uses `domain_types` for its core business logic processing and data models.
*   `grpc-server` orchestrates calls to `connector-integration` to perform actions on external payment gateways, using the `ConnectorIntegrationV2` trait.
*   `connector-integration` uses `domain_types` to understand the data it needs to adapt for specific connectors.
*   `connector-integration` relies on `external-services` to execute the actual API calls to the connectors.
*   Individual connectors (e.g., `adyen.rs`) implement the `ConnectorIntegrationV2` trait defined in `hyperswitch_interfaces` and use data structures defined in `domain_types`.

## 4. Critical Implementation Paths

*   **Payment Authorization Flow:**
    1.  Client sends `PaymentsAuthorizeRequest` (gRPC) to `grpc-server`.
    2.  `grpc-server` receives, validates, and maps the request to internal domain models (`PaymentFlowData`, `PaymentsAuthorizeData`).
    3.  `grpc-server` selects the appropriate connector.
    4.  `external-services` calls `connector-integration` (specific connector adapter) to transform the request and call the external payment gateway.
    5.  The specific connector's `transformers.rs` module is used to map the request to the gateway's API format.
    6.  Response from the gateway is processed, transformed back to a standardized domain model, and then to a gRPC response.
*   **Webhook Handling Flow (Hypothetical):**
    1.  External payment gateway sends a webhook to a dedicated endpoint on the `grpc-server` (or a separate webhook ingestion service).
    2.  The server validates and parses the webhook.
    3.  The information is mapped to internal domain events.
    4.  Relevant business logic is triggered (e.g., update payment status).
    5.  Optionally, forward a notification to the original client application.

## 5. Modularity and Extensibility

*   The separation of `connector-integration` into individual connector modules (e.g., `adyen/`, `razorpay/`) promotes modularity and allows for independent development and maintenance of each connector.
*   The use of traits (like `ConnectorIntegrationV2`) enables a high degree of extensibility. New connectors can be added by implementing these traits, without requiring changes to the core system.
*   Adding a new connector involves:
    1.  Creating a new module within `backend/connector-integration/src/connectors/`.
    2.  Implementing the `ConnectorIntegrationV2` trait for all required payment flows.
    3.  Implementing the necessary adapter logic and data transformers to map between the system's domain models and the connector's specific API.
    4.  Registering the new connector with the core system (likely through configuration).
    5.  Adding the connector's authentication details to the configuration.

*(This is an updated interpretation based on code analysis.)*
