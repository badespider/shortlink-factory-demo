//! shortlink: an in-memory URL shortener HTTP API. See factory/spec.md.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::{delete, get, post};
use axum::Router;

pub mod error;
pub mod handlers;
pub mod store;

/// The whole API, with fresh, empty state on each call (D-router-fn).
pub fn app() -> Router {
    let store = Arc::new(store::Store::default());
    Router::new()
        .route(
            "/links",
            post(handlers::create_link).get(handlers::list_links),
        )
        .route("/links/{code}", delete(handlers::delete_link))
        .route("/{code}", get(handlers::follow_link))
        .with_state(store)
}

/// The address used when `SHORTLINK_ADDR` is unset or empty (D-addr).
pub const DEFAULT_ADDR: &str = "127.0.0.1:3000";

/// Parses the `SHORTLINK_ADDR` value into a socket address (D-addr).
/// `None` or an empty string means [`DEFAULT_ADDR`].
pub fn listen_addr(var: Option<&str>) -> Result<SocketAddr, String> {
    let raw = match var {
        Some(v) if !v.is_empty() => v,
        _ => DEFAULT_ADDR,
    };
    raw.parse()
        .map_err(|e| format!("invalid SHORTLINK_ADDR {raw:?}: {e}"))
}
