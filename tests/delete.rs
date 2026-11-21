//! F5: DELETE /links/{code} removes a link (D-routes, D-store, D-codes).
mod common;

use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn f5_ac1_delete_answers_204_empty_body() {
    // F5-ac1
    let app = shortlink::app();
    let c = common::create(&app, r#"{"url":"https://example.com/a"}"#).await;
    assert_eq!(c.status, StatusCode::CREATED, "F5-ac1: create status");
    assert_eq!(c.json()["code"], json!("1"), "F5-ac1: created code");

    let r = common::send(&app, "DELETE", "/links/1", None, None).await;
    assert_eq!(r.status, StatusCode::NO_CONTENT, "F5-ac1: DELETE status");
    assert!(r.body.is_empty(), "F5-ac1: DELETE body is not empty");
}

#[tokio::test]
async fn f5_ac2_deleted_link_redirect_404_and_leaves_list() {
    // F5-ac2
    let app = shortlink::app();
    let c = common::create(&app, r#"{"url":"https://example.com/a"}"#).await;
    assert_eq!(c.status, StatusCode::CREATED, "F5-ac2: create status");

    let d = common::send(&app, "DELETE", "/links/1", None, None).await;
    assert_eq!(d.status, StatusCode::NO_CONTENT, "F5-ac2: DELETE status");

    let g = common::send(&app, "GET", "/1", None, None).await;
    assert_eq!(g.status, StatusCode::NOT_FOUND, "F5-ac2: GET /1 status");

    let l = common::send(&app, "GET", "/links", None, None).await;
    assert_eq!(l.status, StatusCode::OK, "F5-ac2: GET /links status");
    assert_eq!(l.json(), json!([]), "F5-ac2: list after delete");
}

#[tokio::test]
async fn f5_ac3_second_delete_and_unknown_code_are_404() {
    // F5-ac3
    let app = shortlink::app();
    let c = common::create(&app, r#"{"url":"https://example.com/a"}"#).await;
    assert_eq!(c.status, StatusCode::CREATED, "F5-ac3: create status");

    let first = common::send(&app, "DELETE", "/links/1", None, None).await;
    assert_eq!(
        first.status,
        StatusCode::NO_CONTENT,
        "F5-ac3: first DELETE status"
    );
    let second = common::send(&app, "DELETE", "/links/1", None, None).await;
    assert_eq!(
        second.status,
        StatusCode::NOT_FOUND,
        "F5-ac3: second DELETE status"
    );

    let fresh = shortlink::app();
    let u = common::send(&fresh, "DELETE", "/links/zzz", None, None).await;
    assert_eq!(
        u.status,
        StatusCode::NOT_FOUND,
        "F5-ac3: DELETE unknown code status"
    );
}

#[tokio::test]
async fn f5_ac4_codes_are_never_reused_after_delete() {
    // F5-ac4
    let app = shortlink::app();
    let c = common::create(&app, r#"{"url":"https://example.com/a"}"#).await;
    assert_eq!(c.status, StatusCode::CREATED, "F5-ac4: create status");
    assert_eq!(c.json()["code"], json!("1"), "F5-ac4: first code");

    let d = common::send(&app, "DELETE", "/links/1", None, None).await;
    assert_eq!(d.status, StatusCode::NO_CONTENT, "F5-ac4: DELETE status");

    let n = common::create(&app, r#"{"url":"https://example.com/b"}"#).await;
    assert_eq!(
        n.status,
        StatusCode::CREATED,
        "F5-ac4: second create status"
    );
    assert_eq!(n.json()["code"], json!("2"), "F5-ac4: code after delete");
}
