//! Startup only (D-modules): read SHORTLINK_ADDR, bind, serve (D-addr).
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    // Only an unset variable means the default; a value that is not valid
    // Unicode is a bad address, not a missing one (D-addr, issue #13).
    let var = match std::env::var("SHORTLINK_ADDR") {
        Ok(v) => Some(v),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(raw)) => {
            return fail(&format!(
                "invalid SHORTLINK_ADDR {raw:?}: not valid Unicode"
            ))
        }
    };
    let addr = match shortlink::listen_addr(var.as_deref()) {
        Ok(a) => a,
        Err(e) => return fail(&e),
    };
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => return fail(&format!("cannot bind {addr}: {e}")),
    };
    match axum::serve(listener, shortlink::app()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => fail(&format!("server error: {e}")),
    }
}

/// Prints one line to stderr and returns exit status 1.
fn fail(msg: &str) -> ExitCode {
    eprintln!("shortlink: {}", msg.replace(['\r', '\n'], " "));
    ExitCode::FAILURE
}
