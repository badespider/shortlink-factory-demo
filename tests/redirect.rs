//! F3: GET /{code} follows a short code (D-redirect, D-routes).
mod common;

use axum::http::{header, StatusCode};

#[tokio::test]
async fn f3_ac1_get_code_redirects_302_to_the_stored_url() {
    // F3-ac1
    let app = shortlink::app();
    let created = common::create(&app, r#"{"url":"https://example.com/a"}"#).await;
    assert_eq!(created.status, StatusCode::CREATED, "F3-ac1: create status");
    assert_eq!(created.json()["code"], "1", "F3-ac1: created code");

    let r = common::send(&app, "GET", "/1", None, None).await;
    assert_eq!(r.status, StatusCode::FOUND, "F3-ac1: GET /1 status");
    let location = r
        .headers
        .get(header::LOCATION)
        .map(|v| v.to_str().unwrap_or("").to_string());
    assert_eq!(
        location.as_deref(),
        Some("https://example.com/a"),
        "F3-ac1: Location header"
    );
    assert!(r.body.is_empty(), "F3-ac1: redirect body is not empty");
}

#[tokio::test]
async fn f3_ac2_unknown_code_is_404_with_empty_body() {
    // F3-ac2
    let app = shortlink::app();
    let r = common::send(&app, "GET", "/zzz", None, None).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND, "F3-ac2: GET /zzz status");
    assert!(
        r.body.is_empty(),
        "F3-ac2: 404 body is not empty: {:?}",
        String::from_utf8_lossy(&r.body)
    );
}

#[tokio::test]
async fn f3_ac3_multi_segment_path_is_404() {
    // F3-ac3
    let app = shortlink::app();
    let created = common::create(&app, r#"{"url":"https://example.com/a"}"#).await;
    assert_eq!(created.json()["code"], "1", "F3-ac3: created code");
    let r = common::send(&app, "GET", "/a/b", None, None).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND, "F3-ac3: GET /a/b status");
}
