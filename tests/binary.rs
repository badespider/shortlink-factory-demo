//! F6: the shortlink binary refuses a bad SHORTLINK_ADDR (D-addr, D-tests).
//! Only an address that fails to parse is used, so no socket is opened.
use std::process::Command;

#[test]
fn f6_ac4_bad_addr_exits_1_with_one_stderr_line() {
    // F6-ac4
    let bin = option_env!("CARGO_BIN_EXE_shortlink")
        .expect("F6-ac4: the shortlink binary target is missing");
    let out = Command::new(bin)
        .env("SHORTLINK_ADDR", "not-an-addr")
        .output()
        .expect("F6-ac4: could not run the shortlink binary");
    assert_eq!(out.status.code(), Some(1), "F6-ac4: exit status");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(lines.len(), 1, "F6-ac4: stderr has {} lines", lines.len());
    assert!(!lines[0].trim().is_empty(), "F6-ac4: stderr line is empty");
}
