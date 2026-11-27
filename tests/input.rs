use rmb_upper::*;

#[test]
fn format_decimal_accepts_supported_forms() {
    let cases = [
        (" 12.30 ", "壹拾贰元叁角"),
        ("+1", "壹元整"),
        (".5", "零元伍角"),
        ("5.", "伍元整"),
        ("000000000000000000000001", "壹元整"),
    ];

    for (input, expected) in cases {
        assert_eq!(format_decimal(input).unwrap(), expected, "input {input:?}");
    }
}

#[test]
fn format_decimal_rejects_invalid_forms() {
    for input in ["", ".", "+", "+-1", "1.2.3", "1,000", "１２", "1e3", "1.2a"] {
        assert_eq!(
            format_decimal(input),
            Err(FormatError::InvalidFormat),
            "input {input:?}"
        );
    }

    for input in ["-0", "- 1"] {
        assert_eq!(
            format_decimal(input),
            Err(FormatError::NegativeAmount),
            "input {input:?}"
        );
    }

    assert_eq!(
        format_decimal("1.234"),
        Err(FormatError::TooManyDecimalPlaces)
    );
    assert_eq!(
        format_decimal("10000000000000000"),
        Err(FormatError::TooLarge)
    );
}

#[test]
fn format_cents_str_accepts_and_rejects() {
    assert_eq!(
        format_cents_str("999999999999999999").unwrap(),
        "玖仟玖佰玖拾玖万玖仟玖佰玖拾玖亿玖仟玖佰玖拾玖万玖仟玖佰玖拾玖元玖角玖分"
    );
    assert_eq!(format_cents_str(" 7 ").unwrap(), "零元柒分");
    assert_eq!(format_cents_str("+0").unwrap(), "零元整");

    assert_eq!(
        format_cents_str("1000000000000000000"),
        Err(FormatError::TooLarge)
    );
    assert_eq!(
        format_cents_str(&"9".repeat(50)),
        Err(FormatError::TooLarge)
    );
    assert_eq!(format_cents_str("1.00"), Err(FormatError::InvalidFormat));
}

#[test]
fn format_f64_rounds_and_rejects_unusable_values() {
    assert_eq!(format_f64(123.456).unwrap(), "壹佰贰拾叁元肆角陆分");
    assert_eq!(format_f64(-0.0).unwrap(), "零元整");
    assert_eq!(format_f64(-0.01), Err(FormatError::NegativeAmount));
    assert_eq!(format_f64(f64::NAN), Err(FormatError::NonFiniteAmount));
    assert_eq!(format_f64(f64::INFINITY), Err(FormatError::NonFiniteAmount));
    assert_eq!(format_f64(1e17), Err(FormatError::TooLarge));

    // 1.005 is stored slightly below 1.005, so rounding to fen gives 1.00.
    assert_eq!(format_f64(1.005).unwrap(), "壹元整");
}
