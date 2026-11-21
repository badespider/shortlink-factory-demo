//! The one API error type (D-errors): every 400 is `{"error": "<message>"}`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// Why a request was rejected. Each variant answers 400 with a JSON body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiError {
    /// The body is not JSON (D-json-input).
    InvalidJson,
    /// The JSON is not an object with a string `url` (D-json-input).
    MissingUrl,
    /// The `url` fails the D-url-check rule.
    BadUrl,
}

impl ApiError {
    /// The short lowercase message of D-errors.
    pub fn message(self) -> &'static str {
        match self {
            ApiError::InvalidJson => "invalid JSON body",
            ApiError::MissingUrl => "missing or non-string \"url\"",
            ApiError::BadUrl => "url must start with http:// or https://",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": self.message() })),
        )
            .into_response()
    }
}
