//! One handler per endpoint, mapping store results to responses (D-modules).

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::Value;

use crate::error::ApiError;
use crate::store::Store;

/// `POST /links`: parses the raw body as JSON whatever the `Content-Type`
/// (D-json-input), checks the string `url` (D-url-check) and only then stores
/// it, answering 201 with the new link. Extra fields are ignored. A rejected
/// request answers 400 and never reaches the store, so no code is used up.
pub async fn create_link(State(store): State<Arc<Store>>, body: Bytes) -> Response {
    match parse_url(&body) {
        Ok(url) => (StatusCode::CREATED, Json(store.create(url))).into_response(),
        Err(e) => e.into_response(),
    }
}

/// `GET /links`: 200 with every link as `{"code","url","hits"}`, newest first,
/// and `[]` when there are none (D-store, D-routes).
pub async fn list_links(State(store): State<Arc<Store>>) -> Response {
    Json(store.list()).into_response()
}

/// `GET /{code}`: 302 with `Location` set to the stored URL and an empty body,
/// built by hand because `Redirect::to` answers 303 (D-redirect). The hit is
/// counted inside the store lock (D-store). An unknown code is 404 with an
/// empty body (D-routes).
pub async fn follow_link(State(store): State<Arc<Store>>, Path(code): Path<String>) -> Response {
    match store.follow(&code) {
        Some(url) => (StatusCode::FOUND, [(header::LOCATION, url)]).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// The valid `url` of a `POST /links` body, or why the body is rejected.
fn parse_url(body: &[u8]) -> Result<String, ApiError> {
    let value: Value = serde_json::from_slice(body).map_err(|_| ApiError::InvalidJson)?;
    let url = value
        .get("url")
        .and_then(Value::as_str)
        .ok_or(ApiError::MissingUrl)?;
    check_url(url)?;
    Ok(url.to_string())
}

/// D-url-check: an `http://` or `https://` prefix (scheme case-insensitive),
/// at least one character after the `//`, and no whitespace or control characters.
pub fn check_url(url: &str) -> Result<(), ApiError> {
    let rest = ["http://", "https://"]
        .iter()
        .find_map(|p| {
            url.get(..p.len())
                .filter(|head| head.eq_ignore_ascii_case(p))
                .map(|_| &url[p.len()..])
        })
        .ok_or(ApiError::BadUrl)?;
    if rest.is_empty() || url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(ApiError::BadUrl);
    }
    Ok(())
}
