// Integration tests link the library as a separate rlib; with the `napi`
// feature on that rlib carries Node C symbols a test executable cannot link.
// Keep the five core smoke tests running under `cargo test`; the napi-enabled
// build is checked separately (`cargo build --features napi`, `pnpm build`).
#![cfg(not(feature = "napi"))]

use rmb_upper::*;

fn s(codes: &[u32]) -> String {
    codes
        .iter()
        .map(|code| char::from_u32(*code).expect("valid CJK codepoint"))
        .collect()
}

#[test]
fn formats_zero_and_plain_integers() {
    assert_eq!(format_cents(0).unwrap(), s(&[0x96f6, 0x5143, 0x6574]),);
    assert_eq!(format_cents(100).unwrap(), s(&[0x58f9, 0x5143, 0x6574]),);
    assert_eq!(
        format_cents(1_000).unwrap(),
        s(&[0x58f9, 0x62fe, 0x5143, 0x6574]),
    );
}

#[test]
fn formats_decimal_parts() {
    assert_eq!(
        format_cents(1).unwrap(),
        s(&[0x96f6, 0x5143, 0x58f9, 0x5206]),
    );
    assert_eq!(
        format_cents(10).unwrap(),
        s(&[0x96f6, 0x5143, 0x58f9, 0x89d2]),
    );
    assert_eq!(
        format_cents(101).unwrap(),
        s(&[0x58f9, 0x5143, 0x96f6, 0x58f9, 0x5206]),
    );
}

#[test]
fn formats_group_boundaries_and_internal_zeroes() {
    assert_eq!(
        format_cents(1_000_000).unwrap(),
        s(&[0x58f9, 0x4e07, 0x5143, 0x6574]),
    );
    assert_eq!(
        format_cents(1_000_100).unwrap(),
        s(&[0x58f9, 0x4e07, 0x96f6, 0x58f9, 0x5143, 0x6574]),
    );
}

#[test]
fn parses_strings() {
    assert_eq!(
        format_decimal("001.20").unwrap(),
        s(&[0x58f9, 0x5143, 0x8d30, 0x89d2]),
    );
    assert_eq!(
        format_cents_str("12345").unwrap(),
        format_decimal("123.45").unwrap(),
    );
}

#[test]
fn rejects_invalid_amounts() {
    assert_eq!(format_cents(-1), Err(FormatError::NegativeAmount),);
    assert_eq!(format_cents(MAX_CENTS + 1), Err(FormatError::TooLarge),);
    assert_eq!(format_cents_str("-1"), Err(FormatError::NegativeAmount),);
    assert_eq!(format_cents_str("1.00"), Err(FormatError::InvalidFormat),);
    assert_eq!(
        format_decimal("1.234"),
        Err(FormatError::TooManyDecimalPlaces),
    );
    assert_eq!(format_f64(f64::NAN), Err(FormatError::NonFiniteAmount),);
}
