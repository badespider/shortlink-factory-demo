//! F1: POST /links creates a short link.
mod common;

use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn f1_ac1_post_links_answers_201_with_the_new_link() {
    // F1-ac1
    let app = shortlink::app();
    let r = common::create(&app, r#"{"url":"https://example.com/a"}"#).await;
    assert_eq!(r.status, StatusCode::CREATED, "F1-ac1: status");
    assert!(
        r.content_type().starts_with("application/json"),
        "F1-ac1: Content-Type was {:?}",
        r.content_type()
    );
    assert_eq!(
        r.json(),
        json!({"code":"1","url":"https://example.com/a","hits":0}),
        "F1-ac1: body"
    );
}

#[tokio::test]
async fn f1_ac2_three_posts_answer_codes_1_2_3_in_order() {
    // F1-ac2
    let app = shortlink::app();
    let mut codes = Vec::new();
    for url in [
        "https://example.com/one",
        "http://example.com/two",
        "https://example.org/three",
    ] {
        let r = common::create(&app, &json!({ "url": url }).to_string()).await;
        assert_eq!(r.status, StatusCode::CREATED, "F1-ac2: status for {url}");
        codes.push(r.json()["code"].as_str().unwrap_or("<no code>").to_string());
    }
    assert_eq!(codes, ["1", "2", "3"], "F1-ac2: codes");
}

#[tokio::test]
async fn f1_ac3_extra_fields_and_no_content_type_still_create() {
    // F1-ac3
    let app = shortlink::app();
    let r = common::send(
        &app,
        "POST",
        "/links",
        Some(r#"{"url":"https://example.com/a","extra":true}"#),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "F1-ac3: status");
    let body = r.json();
    let obj = body.as_object().expect("F1-ac3: body is a JSON object");
    let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["code", "hits", "url"], "F1-ac3: body keys");
}
