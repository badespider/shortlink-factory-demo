//! Shared helpers for the integration tests (D-tests).
#![allow(dead_code)]

use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use tower::ServiceExt;

/// What a request answered: status, headers and the whole body.
pub struct Reply {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl Reply {
    /// The body parsed as JSON; panics with the raw body if it is not JSON.
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.body).unwrap_or_else(|e| {
            panic!(
                "body is not JSON ({e}): {:?}",
                String::from_utf8_lossy(&self.body)
            )
        })
    }

    /// The `Content-Type` header, or "" when it is absent.
    pub fn content_type(&self) -> String {
        self.headers
            .get(axum::http::header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap_or("").to_string())
            .unwrap_or_default()
    }
}

/// Sends one request to a clone of `app` with `oneshot` and collects the reply.
/// `content_type` of `None` sends no `Content-Type` header.
pub async fn send(
    app: &Router,
    method: &str,
    path: &str,
    body: Option<&str>,
    content_type: Option<&str>,
) -> Reply {
    let mut req = Request::builder().method(method).uri(path);
    if let Some(ct) = content_type {
        req = req.header(axum::http::header::CONTENT_TYPE, ct);
    }
    let req = req
        .body(match body {
            Some(b) => Body::from(b.to_string()),
            None => Body::empty(),
        })
        .expect("request builds");
    let res = app.clone().oneshot(req).await.expect("router answers");
    let status = res.status();
    let headers = res.headers().clone();
    let body = res
        .into_body()
        .collect()
        .await
        .expect("body collects")
        .to_bytes()
        .to_vec();
    Reply {
        status,
        headers,
        body,
    }
}

/// `POST /links` with a JSON body and `Content-Type: application/json`.
pub async fn create(app: &Router, body: &str) -> Reply {
    send(app, "POST", "/links", Some(body), Some("application/json")).await
}
