//! issue-13: a SHORTLINK_ADDR that is not valid Unicode is refused (D-addr),
//! not treated as unset. With the fix the binary exits before binding.
use std::ffi::OsString;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[cfg(unix)]
fn not_unicode() -> OsString {
    use std::os::unix::ffi::OsStringExt;
    OsString::from_vec(vec![0xED, 0xA0, 0x80])
}

#[cfg(windows)]
fn not_unicode() -> OsString {
    use std::os::windows::ffi::OsStringExt;
    OsString::from_wide(&[0xD800])
}

#[test]
fn issue_13_not_unicode_addr_exits_1_naming_the_variable() {
    let bin = env!("CARGO_BIN_EXE_shortlink");
    let mut child = Command::new(bin)
        .env("SHORTLINK_ADDR", not_unicode())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("issue-13: could not run the shortlink binary");
    let start = Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait().expect("issue-13: wait failed") {
            break Some(s);
        }
        if start.elapsed() > Duration::from_secs(5) {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let status = status.unwrap_or_else(|| {
        panic!("issue-13: a SHORTLINK_ADDR that is not valid Unicode was not refused: still serving after 5 s")
    });
    let mut stderr = String::new();
    std::io::Read::read_to_string(&mut child.stderr.take().unwrap(), &mut stderr).unwrap();
    assert_eq!(
        status.code(),
        Some(1),
        "issue-13: exit status, stderr [{stderr}]"
    );
    let lines: Vec<&str> = stderr.lines().collect();
    assert_eq!(lines.len(), 1, "issue-13: stderr has {} lines", lines.len());
    assert!(
        lines[0].contains("SHORTLINK_ADDR"),
        "issue-13: stderr does not name SHORTLINK_ADDR: [{}]",
        lines[0]
    );
}
