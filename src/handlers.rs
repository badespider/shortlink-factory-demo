//! One handler per endpoint, mapping store results to responses (D-modules).

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

use crate::store::Store;

/// `POST /links`: parses the raw body as JSON whatever the `Content-Type`
/// (D-json-input), stores the string `url` and answers 201 with the new link.
/// Extra fields are ignored. Full 400 handling and URL checks belong to F2.
pub async fn create_link(State(store): State<Arc<Store>>, body: Bytes) -> Response {
    let value: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return bad_request("invalid JSON body"),
    };
    let Some(url) = value.get("url").and_then(Value::as_str) else {
        return bad_request("missing or non-string \"url\"");
    };
    let link = store.create(url.to_string());
    (StatusCode::CREATED, Json(link)).into_response()
}

fn bad_request(message: &str) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": message }))).into_response()
}
