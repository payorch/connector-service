# Tech Context: Connector Service

## 1. Core Technologies

*   **Programming Language: Rust**
    *   Indicated by `Cargo.toml`, `Cargo.lock` files, and `.rs` source files throughout the `backend/` directory.
    *   Rust's focus on performance, safety, and concurrency makes it suitable for a backend service like this.
*   **API Protocol: gRPC**
    *   `backend/grpc-api-types/proto/` contains `.proto` files (e.g., `payment.proto`, `health_check.proto`).
    *   `backend/grpc-server/` implements the gRPC server logic.
    *   Build scripts (`build.rs` in `grpc-api-types` and `grpc-server`) likely handle protobuf code generation.
*   **Build System/Package Manager: Cargo**
    *   Standard for Rust projects. `Cargo.toml` files define dependencies and project structure for each crate.
*   **Configuration Format: TOML**
    *   `config/development.toml` is used for application configuration.

## 2. Development Setup & Tooling (Inferred)

*   **Makefile:** A `Makefile` exists at the root, likely containing common development commands (e.g., build, test, run, lint).
*   **Docker:** `Dockerfile` and `.dockerignore` suggest containerization capabilities, useful for deployment and consistent environments.
*   **Typos Linter:** `.typos.toml` indicates the use of a tool to check for typos in the codebase.
*   **Git:** Standard version control, with `.gitignore` and `.gitattributes`.
*   **Example Clients/SDKs:**
    *   **Node.js (TypeScript):** `sdk/node-grpc-client/` and `examples/example-js/` (uses TypeScript, `tsconfig.json`).
    *   **Python:** `sdk/python-grpc-client/` and `examples/example-py/`. Uses `pyproject.toml` (likely with Poetry or a similar tool) and `uv.lock`.
    *   **Rust:** `sdk/rust-grpc-client/` and `examples/example-rs/`.
    *   **Haskell:** `examples/example-hs/` and `examples/example-hs-grpc/` (uses Cabal).
    *   **CLI Example:** `examples/example-cli/` provides a command-line interface for interacting with the service.
    *   **TUI Example:** `examples/example-tui/` suggests a Text User Interface example.
    *   **MCP Example:** `examples/example-mcp/` (Model Context Protocol) with Python scripts.

## 3. Key Dependencies (To be confirmed by inspecting Cargo.toml files)

*   **gRPC Libraries:** `tonic` (popular Rust gRPC library) is highly probable for `grpc-server`.
*   **Serialization/Deserialization:** `serde` is standard in the Rust ecosystem for handling data formats like JSON, TOML.
*   **Async Runtime:** `tokio` is the most common async runtime for Rust and is often used with `tonic`.
*   **Logging:** A logging framework (e.g., `tracing`, `log`) would be essential. `backend/grpc-server/src/logger/` confirms custom logging setup.
*   **Error Handling:** Libraries like `thiserror` or `anyhow` for robust error management.
*   **HTTP Client:** For `connector-integration` to communicate with external payment gateway APIs (e.g., `reqwest`).

## 4. Technical Constraints & Considerations

*   **Performance:** As a payment service, low latency and high throughput are likely important.
*   **Reliability & Fault Tolerance:** The system must be resilient to failures in external gateways or internal components.
*   **Security:**
    *   Secure handling of API keys and sensitive payment data is paramount.
    *   Protection against common web vulnerabilities.
    *   Secure communication (TLS for gRPC and external API calls).
*   **Scalability:** The architecture should support scaling to handle increasing load.
*   **Maintainability:** Clear code structure, good documentation, and comprehensive tests are crucial. The modular structure (crates for different concerns) aids this.
*   **Interoperability:** gRPC and providing SDKs in multiple languages address this.

## 5. Testing

*   **Unit Tests:** Likely co-located with source code (e.g., `test.rs` files like `backend/connector-integration/src/connectors/adyen/test.rs`).
*   **Integration Tests:** The `backend/grpc-server/tests/` directory suggests integration tests for the gRPC server.
*   **End-to-End Tests:** The example clients could be used as a basis for end-to-end testing.

*(This context is based on file structure and common Rust/gRPC practices. Specific libraries and versions will be confirmed by inspecting `Cargo.toml` files.)*
