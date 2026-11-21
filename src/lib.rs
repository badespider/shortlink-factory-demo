//! shortlink: an in-memory URL shortener HTTP API. See factory/spec.md.

use std::sync::Arc;

use axum::routing::post;
use axum::Router;

pub mod error;
pub mod handlers;
pub mod store;

/// The whole API, with fresh, empty state on each call (D-router-fn).
pub fn app() -> Router {
    let store = Arc::new(store::Store::default());
    Router::new()
        .route("/links", post(handlers::create_link))
        .with_state(store)
}
