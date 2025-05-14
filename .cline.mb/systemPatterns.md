# System Patterns: Connector Service

## 1. Overall Architecture (Inferred)

The system appears to follow a **layered architecture** with a clear separation of concerns:

*   **API Layer (gRPC):** `backend/grpc-server/` and `backend/grpc-api-types/` suggest a gRPC-based interface for external communication. This layer handles incoming requests and translates them into internal operations.
*   **Core Logic/Domain Layer:** `backend/domain_types/` likely defines the core business objects and logic, independent of specific connectors or API protocols. This layer would contain the standardized models for payments, refunds, etc.
*   **Connector Integration Layer:** `backend/connector-integration/` is dedicated to managing interactions with various third-party payment gateways. This layer contains the specific adapters for each connector (e.g., Adyen, Razorpay).
*   **External Services Layer:** `backend/external-services/` might be for interactions with other auxiliary services, though its exact role needs clarification.
*   **SDKs/Client Libraries:** The `sdk/` directory indicates the provision of client libraries for different languages to simplify integration with the gRPC API.
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

*   **Adapter Pattern:** Each connector implementation within `backend/connector-integration/src/connectors/` (e.g., `adyen.rs`, `razorpay.rs`) acts as an adapter, translating the system's internal payment models and operations into the specific format and protocol required by the external payment gateway.
*   **Strategy Pattern (Potential):** The selection of a specific connector at runtime could be implemented using the Strategy pattern, where a common interface (`ConnectorIntegration` trait perhaps) is defined, and concrete strategies (specific connectors) are chosen based on configuration or request parameters.
*   **Data Transfer Objects (DTOs):** The `proto` files in `backend/grpc-api-types/proto/` define the structure of data exchanged over gRPC. These act as DTOs. Internal domain types (`backend/domain_types/`) might be mapped to/from these DTOs at the API layer.
*   **Configuration Management:** The `config/development.toml` file suggests a centralized approach to configuration. The `backend/grpc-server/src/configs.rs` likely handles loading and providing this configuration to the application.
*   **Macros for Code Generation/Boilerplate Reduction:** The presence of `backend/connector-integration/src/connectors/macros.rs` suggests the use of macros to reduce boilerplate code, possibly for defining common connector functionalities or traits.

## 3. Component Relationships

*   `grpc-server` depends on `grpc-api-types` for request/response structures and service definitions.
*   `grpc-server` likely uses `domain_types` for its core business logic processing.
*   `grpc-server` orchestrates calls to `connector-integration` to perform actions on external payment gateways.
*   `connector-integration` uses `domain_types` to understand the data it needs to adapt for specific connectors.
*   Individual connectors (e.g., `adyen.rs`) implement common traits or interfaces defined within `connector-integration` or `domain_types`.

## 4. Critical Implementation Paths

*   **Payment Creation Flow:**
    1.  Client sends `CreatePaymentRequest` (gRPC).
    2.  `grpc-server` receives, validates, and maps to internal domain model.
    3.  `grpc-server` (or a dedicated service) selects the appropriate connector.
    4.  `connector-integration` (specific connector adapter) transforms the request and calls the external payment gateway.
    5.  Response from gateway is processed, transformed back to a standardized domain model, and then to a gRPC response.
*   **Webhook Handling Flow (Hypothetical):**
    1.  External payment gateway sends a webhook to a dedicated endpoint on the `grpc-server` (or a separate webhook ingestion service).
    2.  The server validates and parses the webhook.
    3.  The information is mapped to internal domain events.
    4.  Relevant business logic is triggered (e.g., update payment status).
    5.  Optionally, forward a notification to the original client application.

## 5. Modularity and Extensibility

*   The separation of `connector-integration` into individual connector modules (e.g., `adyen/`, `razorpay/`) promotes modularity.
*   Adding a new connector would likely involve:
    1.  Creating a new module within `backend/connector-integration/src/connectors/`.
    2.  Implementing the necessary adapter logic and data transformers.
    3.  Registering the new connector with the core system.
    4.  Updating configuration to include credentials/settings for the new connector.

*(This is an initial interpretation based on the file structure. It will be refined as code is examined.)*
