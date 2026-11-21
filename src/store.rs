//! The in-memory link store (D-store, D-codes). No HTTP types here.

use std::sync::Mutex;

use serde::Serialize;

/// One short link; also the exact JSON shape `{"code","url","hits"}` of the API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Link {
    pub code: String,
    pub url: String,
    pub hits: u64,
}

/// Links in creation order plus the never-reused code counter, behind one lock.
#[derive(Debug)]
pub struct Store {
    inner: Mutex<Inner>,
}

#[derive(Debug)]
struct Inner {
    links: Vec<Link>,
    next: u64,
}

impl Default for Store {
    fn default() -> Self {
        Store {
            inner: Mutex::new(Inner {
                links: Vec::new(),
                next: 1,
            }),
        }
    }
}

/// A code that would collide with the `/links` route, so it is never handed out.
const RESERVED: &str = "links";

impl Store {
    /// Stores `url` under the next free code and returns the new link.
    pub fn create(&self, url: String) -> Link {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let code = loop {
            let code = base62(inner.next);
            inner.next += 1;
            if code != RESERVED {
                break code;
            }
        };
        let link = Link { code, url, hits: 0 };
        inner.links.push(link.clone());
        link
    }
}

const DIGITS: &[u8; 62] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// `n` written in base62 with the digits `0-9a-zA-Z`; `base62(0)` is `"0"`.
pub fn base62(mut n: u64) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut out = Vec::new();
    while n > 0 {
        out.push(DIGITS[(n % 62) as usize]);
        n /= 62;
    }
    out.reverse();
    String::from_utf8(out).expect("base62 digits are ASCII")
}
