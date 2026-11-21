//! F2: POST /links rejects bad input with 400 and stores nothing.
mod common;

use axum::http::StatusCode;

/// Asserts a 400 whose JSON body has a non-empty string `error` (D-errors).
fn assert_rejected(tag: &str, input: &str, r: &common::Reply) {
    assert_eq!(
        r.status,
        StatusCode::BAD_REQUEST,
        "{tag}: status for body {input:?}"
    );
    let body: serde_json::Value = serde_json::from_slice(&r.body).unwrap_or_else(|e| {
        panic!(
            "{tag}: body for {input:?} is not JSON ({e}): {:?}",
            String::from_utf8_lossy(&r.body)
        )
    });
    let error = body.get("error").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        !error.is_empty(),
        "{tag}: no non-empty string `error` for {input:?}, body was {body}"
    );
}

#[tokio::test]
async fn f2_ac1_not_json_answers_400_with_json_error() {
    // F2-ac1
    let app = shortlink::app();
    let r = common::create(&app, "not json").await;
    assert_rejected("F2-ac1", "not json", &r);
    assert!(
        r.content_type().starts_with("application/json"),
        "F2-ac1: Content-Type was {:?}",
        r.content_type()
    );
}

#[tokio::test]
async fn f2_ac2_missing_or_non_string_url_answers_400() {
    // F2-ac2
    for input in ["{}", r#"{"url": 5}"#] {
        let app = shortlink::app();
        let r = common::create(&app, input).await;
        assert_rejected("F2-ac2", input, &r);
    }
}

#[tokio::test]
async fn f2_ac3_non_http_urls_answer_400() {
    // F2-ac3
    for input in [
        r#"{"url":"ftp://x"}"#,
        r#"{"url":"https://"}"#,
        r#"{"url":"https://a b"}"#,
    ] {
        let app = shortlink::app();
        let r = common::create(&app, input).await;
        assert_rejected("F2-ac3", input, &r);
    }
}

#[tokio::test]
async fn f2_ac4_uppercase_scheme_is_accepted_and_stored_as_given() {
    // F2-ac4
    let app = shortlink::app();
    let r = common::create(&app, r#"{"url":"HTTP://example.com"}"#).await;
    assert_eq!(r.status, StatusCode::CREATED, "F2-ac4: status");
    assert_eq!(
        r.json()["url"].as_str(),
        Some("HTTP://example.com"),
        "F2-ac4: url field"
    );
}

#[tokio::test]
async fn f2_ac5_rejected_post_stores_nothing() {
    // F2-ac5
    let app = shortlink::app();
    let bad = common::create(&app, r#"{"url":"ftp://x"}"#).await;
    assert_rejected("F2-ac5", r#"{"url":"ftp://x"}"#, &bad);
    let good = common::create(&app, r#"{"url":"https://example.com/a"}"#).await;
    assert_eq!(good.status, StatusCode::CREATED, "F2-ac5: status of valid post");
    assert_eq!(
        good.json()["code"].as_str(),
        Some("1"),
        "F2-ac5: code of the next valid post"
    );
}
