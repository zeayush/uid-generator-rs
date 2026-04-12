use std::collections::HashSet;
use std::str::FromStr;
use uid_generator_rs::{Ulid, UlidError, decode_base32_char};

const CROCKFORD_ALPHABET: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

#[test]
fn new_ulid_string_is_26_chars() {
    let ulid = Ulid::new().unwrap();
    let s = ulid.to_string();
    assert_eq!(s.len(), 26, "expected 26 chars, got {}: {s:?}", s.len());
}

#[test]
fn new_ulid_string_uses_crockford_alphabet() {
    let ulid = Ulid::new().unwrap();
    for c in ulid.to_string().chars() {
        assert!(
            CROCKFORD_ALPHABET.contains(c),
            "invalid character {c:?} in ULID string"
        );
    }
}

#[test]
fn timestamp_roundtrip() {
    let ms = 1_700_000_000_000u64; // 2023-11-14 22:13:20 UTC
    let ulid = Ulid::with_ms(ms).unwrap();
    assert_eq!(ulid.timestamp_ms(), ms);
}

#[test]
fn string_parse_roundtrip() {
    let original = Ulid::new().unwrap();
    let s = original.to_string();
    let parsed = Ulid::from_str(&s).expect("ParseULID should succeed");
    assert_eq!(original, parsed, "roundtrip failed for {s:?}");
}

#[test]
fn parse_invalid_length_returns_error() {
    let cases = ["", "TOOSHORT", "WAY-TOO-LONG-TO-BE-A-VALID-ULID-EVER"];
    for case in cases {
        let err = Ulid::from_str(case).unwrap_err();
        assert!(
            matches!(err, UlidError::InvalidLength { .. }),
            "expected InvalidLength for {case:?}, got {err:?}"
        );
    }
}

#[test]
fn lexicographic_order_matches_time_order() {
    let ms1 = 1_700_000_000_000u64;
    let ms2 = ms1 + 1;
    let u1 = Ulid::with_ms(ms1).unwrap();
    let u2 = Ulid::with_ms(ms2).unwrap();

    assert!(u1 < u2, "expected u1 < u2 (Ord), got u1 >= u2");
    assert!(
        u1.to_string() < u2.to_string(),
        "string order mismatch: {:?} >= {:?}",
        u1.to_string(),
        u2.to_string()
    );
}

#[test]
fn ulids_are_unique() {
    let mut seen = HashSet::with_capacity(1_000);
    for _ in 0..1_000 {
        let u = Ulid::new().unwrap();
        assert!(seen.insert(u), "duplicate ULID: {u}");
    }
}

#[test]
fn overflow_timestamp_is_rejected() {
    let too_large = 1u64 << 48; // 2^48 — one more than max 48-bit value
    let err = Ulid::with_ms(too_large).unwrap_err();
    assert!(
        matches!(err, UlidError::TimestampOverflow),
        "expected TimestampOverflow, got {err:?}"
    );
}

#[test]
fn decode_base32_ocr_aliases() {
    // O/o → 0
    assert_eq!(decode_base32_char('O').unwrap(), 0);
    assert_eq!(decode_base32_char('o').unwrap(), 0);

    // I/i, L/l → 1
    assert_eq!(decode_base32_char('I').unwrap(), 1);
    assert_eq!(decode_base32_char('i').unwrap(), 1);
    assert_eq!(decode_base32_char('L').unwrap(), 1);
    assert_eq!(decode_base32_char('l').unwrap(), 1);
}

#[test]
fn decode_base32_valid_digits() {
    assert_eq!(decode_base32_char('0').unwrap(), 0);
    assert_eq!(decode_base32_char('9').unwrap(), 9);
    assert_eq!(decode_base32_char('A').unwrap(), 10);
    assert_eq!(decode_base32_char('Z').unwrap(), 31);
    assert_eq!(decode_base32_char('a').unwrap(), 10); // lowercase
    assert_eq!(decode_base32_char('z').unwrap(), 31); // lowercase
}

#[test]
fn decode_base32_invalid_chars() {
    for c in ['U', 'u', '!', ' ', '\n'] {
        assert!(
            matches!(decode_base32_char(c), Err(UlidError::InvalidChar(_))),
            "expected InvalidChar for {c:?}"
        );
    }
}
