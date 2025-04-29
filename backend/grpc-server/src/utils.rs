use std::str::FromStr;

use crate::consts;
use domain_types::connector_types;
use http::request::Request;
use tonic::metadata;

/// Record the header's fields in request's trace
pub fn record_fields_from_header<B: hyper::body::Body>(request: &Request<B>) -> tracing::Span {
    let url_path = request.uri().path();

    let span = tracing::debug_span!(
        "request",
        uri = %url_path,
        version = ?request.version(),
        tenant_id = tracing::field::Empty,
        request_id = tracing::field::Empty,
    );
    request
        .headers()
        .get(consts::X_TENANT_ID)
        .and_then(|value| value.to_str().ok())
        .map(|tenant_id| span.record("tenant_id", tenant_id));

    request
        .headers()
        .get(consts::X_REQUEST_ID)
        .and_then(|value| value.to_str().ok())
        .map(|request_id| span.record("request_id", request_id));

    span
}

pub fn connector_from_metadata(
    metadata: &metadata::MetadataMap,
) -> Result<connector_types::ConnectorEnum, tonic::Status> {
    metadata
        .get(consts::X_CONNECTOR)
        .ok_or(tonic::Status::invalid_argument(
            "Missing connector in request metadata".to_string(),
        ))
        .and_then(|value| {
            value.to_str().map_err(|e| {
                tonic::Status::invalid_argument(format!(
                    "Invalid connector in request metadata: {e}"
                ))
            })
        })
        .and_then(|inner| {
            connector_types::ConnectorEnum::from_str(inner)
                .map_err(|e| tonic::Status::invalid_argument(format!("Invalid connector: {e}")))
        })
}
