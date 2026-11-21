//! F4: GET /links lists every link with its hit count, newest first (D-store, D-routes).
mod common;

use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn f4_ac1_empty_list_is_200_json_empty_array() {
    // F4-ac1
    let app = shortlink::app();
    let r = common::send(&app, "GET", "/links", None, None).await;
    assert_eq!(r.status, StatusCode::OK, "F4-ac1: GET /links status");
    assert!(
        r.content_type().starts_with("application/json"),
        "F4-ac1: Content-Type is {:?}, want application/json",
        r.content_type()
    );
    assert_eq!(r.json(), json!([]), "F4-ac1: body of an empty list");
}

#[tokio::test]
async fn f4_ac2_lists_links_newest_first() {
    // F4-ac2
    let app = shortlink::app();
    let a = common::create(&app, r#"{"url":"https://example.com/a"}"#).await;
    assert_eq!(a.status, StatusCode::CREATED, "F4-ac2: create a status");
    let b = common::create(&app, r#"{"url":"https://example.com/b"}"#).await;
    assert_eq!(b.status, StatusCode::CREATED, "F4-ac2: create b status");

    let r = common::send(&app, "GET", "/links", None, None).await;
    assert_eq!(r.status, StatusCode::OK, "F4-ac2: GET /links status");
    assert_eq!(
        r.json(),
        json!([
            {"code": "2", "url": "https://example.com/b", "hits": 0},
            {"code": "1", "url": "https://example.com/a", "hits": 0}
        ]),
        "F4-ac2: list body"
    );
}

#[tokio::test]
async fn f4_ac3_list_shows_counted_hits() {
    // F4-ac3
    let app = shortlink::app();
    let a = common::create(&app, r#"{"url":"https://example.com/a"}"#).await;
    assert_eq!(a.json()["code"], "1", "F4-ac3: created code");
    for _ in 0..2 {
        let r = common::send(&app, "GET", "/1", None, None).await;
        assert_eq!(r.status, StatusCode::FOUND, "F4-ac3: GET /1 status");
    }

    let r = common::send(&app, "GET", "/links", None, None).await;
    assert_eq!(r.status, StatusCode::OK, "F4-ac3: GET /links status");
    assert_eq!(
        r.json(),
        json!([{"code": "1", "url": "https://example.com/a", "hits": 2}]),
        "F4-ac3: list body after two visits"
    );
}
