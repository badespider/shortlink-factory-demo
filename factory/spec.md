<!-- factory:spec -->
# shortlink

## Restated goal
A developer or script runs one small Rust binary and can then turn a long http(s) URL into a short code over a JSON HTTP API, follow the code to be redirected (each visit counted), list every link with its hit count, and delete links. You can tell it works when `cargo test` drives the in-process router through every endpoint, including the 400 and 404 cases, with no network.

## Goals
- G-1: `POST /links` with `{"url": "<http or https url>"}` answers 201 and `{"code": "<code>", "url": "<url>", "hits": 0}`, with a code that is short, URL-safe and unique among live and past links of this process.
- G-2: `GET /<code>` for a known code answers 302 with `Location: <stored url>` and adds one to that link's hits; an unknown code answers 404.
- G-3: `GET /links` answers 200 with a JSON array of every link as `{"code", "url", "hits"}`, newest first (`[]` when there are none).
- G-4: `DELETE /links/<code>` answers 204 with an empty body and the link is gone (its redirect then answers 404 and it leaves the list); an unknown code answers 404.
- G-5: Bad input to `POST /links` (a body that is not JSON, JSON without a string `url`, or a URL whose scheme is not http or https) answers 400 with `{"error": "<message>"}`, and nothing is stored.
- G-6: The binary listens on the address in `SHORTLINK_ADDR`, or `127.0.0.1:3000` when it is unset.

## Non-goals
- NG-1: Persistence. Links live in memory and vanish when the process stops; no database, file store or snapshot.
- NG-2: Custom or vanity codes chosen by the client, and link expiry.
- NG-3: Authentication, users, rate limiting or per-client quotas.
- NG-4: Any HTML page or UI; the surface is JSON plus redirects.
- NG-5: Editing a link's URL in place (`PUT`/`PATCH`), and per-hit analytics beyond the single counter.
- NG-6: Deduplicating URLs: posting the same URL twice creates two links.
- NG-7: TLS, CORS, metrics endpoints and structured logging.

## Users
- U-client: a developer, script or service that calls the API over HTTP to shorten URLs and manage links.
- U-visitor: anyone (usually a browser) who opens `/<code>` and should land on the stored URL.
- U-operator: whoever runs the binary on their machine or a server and picks the listen address.

## Flows
- FL-1 (U-client): `POST /links {"url":"https://example.com/a"}` -> 201 `{"code":"<c>","url":"https://example.com/a","hits":0}`.
- FL-2 (U-visitor): `GET /<c>` -> 302 `Location: https://example.com/a` -> the link's hits go up by one.
- FL-3 (U-client): `GET /links` -> 200 `[{"code","url","hits"}, ...]`, the most recently created first.
- FL-4 (U-client): `DELETE /links/<c>` -> 204 -> `GET /<c>` is 404 and `<c>` is missing from `GET /links`; a second delete is 404.
- FL-5 (U-client): `POST /links` with `not json`, `{}`, `{"url": 5}` or `{"url":"ftp://x"}` -> 400 `{"error":"..."}` -> `GET /links` is unchanged.
- FL-6 (U-operator): `SHORTLINK_ADDR=0.0.0.0:8080 cargo run` -> the server accepts requests on port 8080; without the variable it listens on `127.0.0.1:3000`.

## Design
- D-rust-axum: Rust 2021, one crate `shortlink` with a library (`src/lib.rs`) and a binary (`src/main.rs`). HTTP is axum 0.8 on tokio; JSON is serde + serde_json. Dependencies are exactly these four, plus `tower` (`util`) and `http-body-util` as dev-dependencies for tests. Adding a dependency is a protected-path change (`Cargo.toml`, `Cargo.lock`). Why: the idea fixes the stack and asks for few dependencies. Rejected: actix-web, hyper directly, a `url` or `rand` crate.
- D-router-fn: the library exposes `pub fn app() -> axum::Router`, which builds a router with fresh, empty state on each call. Tests call it and send requests with `tower::ServiceExt::oneshot`; `main.rs` only reads the address, binds a `tokio::net::TcpListener` and runs `axum::serve(listener, shortlink::app())`. Why: in-process tests with no network, each test isolated. Rejected: a global or `lazy_static` store.
- D-modules: `src/lib.rs` (the `app()` router and route table), `src/store.rs` (the link store, no HTTP types), `src/handlers.rs` (one handler per endpoint, mapping store results to responses), `src/error.rs` (the API error type). `src/main.rs` holds only startup. Why: features touch separate files where they can.
- D-store: `Store` holds a `std::sync::Mutex` around a `Vec<Link>` in creation order, a next-id counter, and is shared as `Arc<Store>` axum state. `Link { code: String, url: String, hits: u64 }` derives `Serialize` and is the exact JSON shape `{"code","url","hits"}` used by both `POST /links` and `GET /links`. Listing returns the vector reversed (newest first); redirect increments `hits` under the lock; delete removes the entry. Why: tiny data, O(n) lookups are fine, the order falls out of the vector. Rejected: `HashMap` plus a timestamp sort, `RwLock`, `dashmap`.
- D-codes: a code is the next value of a process-wide `u64` counter, starting at 1, written in base62 (`0-9a-zA-Z`), so the first codes are `1`, `2`, ... `z`, `10`. Counters are never reused, even after delete, so codes are unique, short and URL-safe. The literal code `links` is never handed out (the counter skips it) so it cannot collide with the `/links` route. Why: no `rand` dependency, deterministic tests. Rejected: random codes (needs `rand` or a hand-rolled RNG), hashing the URL (collides on duplicates). See Q-1.
- D-routes: `POST /links` and `GET /links` on `/links`, `DELETE /links/{code}`, `GET /{code}`. Any other method on these paths is axum's default 405; any other path is 404. A 404 has an empty body. Why: the surface the idea names, and nothing more.
- D-errors: every 400 has the body `{"error": "<message>"}` with `Content-Type: application/json`, built from one `ApiError` type implementing `IntoResponse`. Messages are short and lowercase: `invalid JSON body`, `missing or non-string "url"`, `url must start with http:// or https://`. Tests assert the status and that `error` is a non-empty string, never the exact wording. Why: axum's own `Json` extractor answers 415 without a JSON content type and 422 for a missing field, which the idea does not allow.
- D-json-input: `POST /links` takes the raw body (`axum::body::Bytes`) and parses it with `serde_json::from_slice` into `serde_json::Value`; the request's `Content-Type` is ignored. Any parse failure is 400 `invalid JSON body`; a value that is not an object with a string `url` is 400 `missing or non-string "url"`. Extra fields are ignored. Why: so every bad body is a 400 as G-5 asks (see D-errors). Rejected: `axum::Json<T>` with a custom rejection layer.
- D-url-check: a URL is valid when it starts with `http://` or `https://` (scheme matched case-insensitively), has at least one character after the `//`, and contains no whitespace or control characters. It is stored exactly as given. Why: the idea asks only for an http(s) check, and a `url` crate is a dependency it does not need. Rejected: full RFC 3986 parsing.
- D-redirect: `GET /{code}` answers `302 Found` with a `Location` header holding the stored URL and an empty body (axum's `Redirect::to` answers 303, so it is not used). Why: the idea says 302.
- D-addr: `main.rs` reads `SHORTLINK_ADDR`; unset or empty means `127.0.0.1:3000`. A value that is not a socket address, or a bind failure, prints one line to stderr and exits with status 1. Why: G-6; the binary is not exercised by tests beyond the address parsing, which lives in a small `pub fn listen_addr(var: Option<&str>) -> Result<SocketAddr, String>` in the library so tests can call it.
- D-tests: all tests are integration tests in `tests/*.rs` (one file per area, for example `tests/create.rs`, `tests/redirect.rs`), run by `cargo test`; shared helpers go in `tests/common/mod.rs`. Each test builds `shortlink::app()` and uses `oneshot`; bodies are read with `http_body_util::BodyExt::collect`. No test opens a socket. Why: the gate's red-green check runs test files by name (`cargo test --test <name>`). Rejected: `#[cfg(test)]` unit tests in `src/`, which the gate's test globs do not see.

## Open questions
- Q-1: Should codes be sequential (predictable: anyone can enumerate every link) or random?
  - Assumption: sequential base62 (D-codes). Links here are not secrets, and switching to random codes later changes only `store.rs`.
  - Status: assumed
- Q-2: Does `GET /<code>` count a hit for `HEAD` requests too?
  - Assumption: only `GET` is routed for `/{code}`; axum runs the `GET` handler for `HEAD` too, so a `HEAD` counts a hit. No test relies on it either way.
  - Status: assumed
- Q-3: What does the list look like when the array is large: is pagination needed?
  - Assumption: no pagination; the whole list is returned (in-memory demo, NG-1).
  - Status: assumed
- Q-4: Should 404 responses carry a JSON `{"error"}` body like 400s do?
  - Assumption: no; 404 has an empty body (D-routes). Tests assert only the status.
  - Status: assumed

## Spikes
- None. Every question above has a reversible default. The stack was checked by building it instead: with the committed `Cargo.toml` and `Cargo.lock` (axum 0.8.9, tokio 1.54.1), `cargo test` exited 0 on rustc 1.100.0.

## Gate
- `project.setup: cargo fetch --locked`: downloads the locked dependencies; fails if `Cargo.lock` is out of date.
- `gate.test: cargo test --locked --all-targets`: builds and runs every test; fails on a compile error or a failing test.
- `gate.test_files`: runs each changed top-level `tests/<name>.rs` with `cargo test --locked --test <name>`, skipping helper files under `tests/*/`; stops at the first failure.
- `gate.test_globs: tests/**`: every test and test helper lives there (D-tests).
- `gate.lint: cargo fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings`: formatting and clippy warnings fail the gate.
- `gate.typecheck: none`: `cargo test` already compiles every target.
- `merge.protected_paths`: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/**`, `build.rs`, `rustfmt.toml`, `clippy.toml`: dependencies, toolchain and the configs the gate commands read.
