# Component Patterns: External Services (`backend/external-services`)

## 1. Component-Specific Design Patterns

*   **Template Method Pattern:** The `execute_connector_processing_step` function acts as a Template Method. It defines a skeleton of an algorithm (the steps to interact with a connector) and defers some steps to subclasses (or in this Rust context, to implementations of the `ConnectorIntegrationV2` trait).
    *   **Abstract Steps (delegated to trait implementations):**
        *   `build_request_v2()`: Constructing the connector-specific request.
        *   `handle_response_v2()`: Processing a successful connector response.
        *   `get_error_response_v2()` / `get_5xx_error_response()`: Processing an error response from the connector.
    *   **Concrete Steps (handled by `execute_connector_processing_step`):**
        *   Making the HTTP call using `call_connector_api`.
        *   Basic parsing of HTTP status codes to differentiate success/error.
        *   Logging and timing.
*   **Generic Programming:** The `execute_connector_processing_step` function is highly generic, using type parameters `F` (Flow), `ResourceCommonData`, `Req` (Request), and `Resp` (Response). This allows it to be reused for any payment flow and any connector that conforms to the `ConnectorIntegrationV2` trait.
*   **Centralized HTTP Client Management:** The crate manages `reqwest::Client` instances, including support for HTTP/HTTPS proxies and connection pooling (`NON_PROXIED_CLIENT` and `PROXIED_CLIENT` static `OnceCell`s).
*   **Structured Logging:** Uses `tracing` spans to provide detailed, structured logs for outgoing API calls, including masked request/response bodies, headers, status codes, and latency.

## 2. Internal Architecture

The crate is relatively small and focused:

*   **`service.rs`:**
    *   Contains the primary function `execute_connector_processing_step`.
    *   Contains `call_connector_api` for making HTTP requests via `reqwest`.
    *   Contains `create_client` and `get_base_client` for `reqwest::Client` instantiation and proxy configuration.
    *   Contains `handle_response` for initial processing of `reqwest::Response`.
    *   Includes helper traits (`HeaderExt`, `RequestBuilderExt`) and logging functions (`debug_log`, `info_log`, etc.).
*   **`lib.rs`:**
    *   The crate root, likely just re-exporting items from `service.rs`.

## 3. Key Algorithms and Approaches

*   **Connector Interaction Lifecycle:**
    1.  Build request (`connector.build_request_v2`).
    2.  If request exists, make HTTP call (`call_connector_api`).
        *   Parse URL.
        *   Determine proxy usage.
        *   Get/create `reqwest::Client`.
        *   Construct headers.
        *   Send request.
        *   Handle `reqwest::Response` (check status code, read body).
    3.  If HTTP call successful, handle connector response (`connector.handle_response_v2`).
    4.  If HTTP call failed (at network level or connector error status), handle connector error (`connector.get_error_response_v2` or `get_5xx_error_response`).
    5.  Return updated `RouterDataV2`.
*   **Proxy Handling:** The `create_client` and `get_base_client` functions check proxy configuration (`Proxy` struct from `domain_types`) and `bypass_proxy_urls` to determine if a proxied `reqwest::Client` should be used.
*   **Error Handling:** Uses `error-stack` for structured error reporting. Distinguishes between network-level API client errors (`ApiClientError`) and connector-specific errors (`ConnectorError`).

## 4. Data Flows Within the Component

1.  **Input:** `execute_connector_processing_step` receives:
    *   `proxy` configuration.
    *   A `BoxedConnectorIntegrationV2` trait object (the specific connector implementation for a given flow).
    *   `RouterDataV2` containing the request data and other context.
2.  **Request Building:** Calls `build_request_v2()` on the trait object, which returns an `Option<Request>`.
3.  **HTTP Call:** The `Request` (URL, method, headers, body) is passed to `call_connector_api`.
4.  **HTTP Response:** `call_connector_api` returns a `Result<Response, Response>` (where `Response` is from `hyperswitch_interfaces`).
5.  **Response Handling:** This `Response` is passed to `handle_response_v2()` or error handling methods on the trait object.
6.  **Output:** `execute_connector_processing_step` returns the modified `RouterDataV2` containing the outcome of the operation (either a success response or an error).

## 5. State Management Strategies

*   **Stateless Operations:** The `execute_connector_processing_step` function itself is stateless regarding individual requests.
*   **Shared HTTP Clients:** `reqwest::Client` instances (proxied and non-proxied) are managed as static `OnceCell` variables, allowing them to be initialized once and reused across calls, which is good for performance (connection pooling).

*(This is an initial draft based on recent analysis.)*
