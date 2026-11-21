//! F1: short codes are base62 of a counter (D-codes).

#[test]
fn f1_ac4_base62_digits() {
    // F1-ac4
    let cases = [
        (1, "1"),
        (10, "a"),
        (35, "z"),
        (36, "A"),
        (61, "Z"),
        (62, "10"),
    ];
    for (n, want) in cases {
        assert_eq!(shortlink::store::base62(n), want, "F1-ac4: base62({n})");
    }
}
