//! F6: the listen address comes from SHORTLINK_ADDR (D-addr, D-tests).
use std::net::SocketAddr;

#[test]
fn f6_ac1_unset_or_empty_defaults_to_localhost_3000() {
    // F6-ac1
    let want: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    assert_eq!(
        shortlink::listen_addr(None),
        Ok(want),
        "F6-ac1: listen_addr(None)"
    );
    assert_eq!(
        shortlink::listen_addr(Some("")),
        Ok(want),
        "F6-ac1: listen_addr(Some(\"\"))"
    );
}

#[test]
fn f6_ac2_valid_address_is_used() {
    // F6-ac2
    let want: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    assert_eq!(
        shortlink::listen_addr(Some("0.0.0.0:8080")),
        Ok(want),
        "F6-ac2: listen_addr(Some(\"0.0.0.0:8080\"))"
    );
}

#[test]
fn f6_ac3_bad_address_is_an_error_with_a_message() {
    // F6-ac3
    match shortlink::listen_addr(Some("not an addr")) {
        Ok(a) => panic!("F6-ac3: expected Err, got Ok({a})"),
        Err(msg) => assert!(!msg.trim().is_empty(), "F6-ac3: error message is empty"),
    }
}
