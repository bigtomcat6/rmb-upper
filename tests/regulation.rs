#![cfg(not(feature = "napi"))]

use rmb_upper::*;

mod common;

#[test]
fn formats_regulation_examples_with_zero_after_yuan() {
    let cases = [
        ("1680.32", "壹仟陆佰捌拾元零叁角贰分"),
        ("107000.53", "壹拾万柒仟元零伍角叁分"),
        ("10.5", "壹拾元零伍角"),
        ("1000000.5", "壹佰万元零伍角"),
        ("100100000.10", "壹亿零壹拾万元零壹角"),
        ("1409.50", "壹仟肆佰零玖元伍角"),
        ("6007.14", "陆仟零柒元壹角肆分"),
        ("16409.02", "壹万陆仟肆佰零玖元零贰分"),
        ("325.04", "叁佰贰拾伍元零肆分"),
        ("100000.05", "壹拾万元零伍分"),
        ("0.5", "零元伍角"),
        ("0.05", "零元伍分"),
        ("107000", "壹拾万柒仟元整"),
    ];

    for (input, expected) in cases {
        assert_eq!(format_decimal(input).unwrap(), expected, "input {input}");
    }
}

#[test]
fn formats_trillions_with_wan_yi_units() {
    let cases = [
        (
            "999999999999",
            "玖仟玖佰玖拾玖亿玖仟玖佰玖拾玖万玖仟玖佰玖拾玖元整",
        ),
        ("1000000000000", "壹万亿元整"),
        ("1000000000005", "壹万亿零伍元整"),
        ("1000010000000", "壹万亿零壹仟万元整"),
        ("1000100000000", "壹万零壹亿元整"),
        ("1000100010001", "壹万零壹亿零壹万零壹元整"),
        ("1200000000000", "壹万贰仟亿元整"),
        (
            "1234567890123",
            "壹万贰仟叁佰肆拾伍亿陆仟柒佰捌拾玖万零壹佰贰拾叁元整",
        ),
        ("10000000000000", "壹拾万亿元整"),
        ("100000000000000", "壹佰万亿元整"),
        (
            "9999999999999999",
            "玖仟玖佰玖拾玖万玖仟玖佰玖拾玖亿玖仟玖佰玖拾玖万玖仟玖佰玖拾玖元整",
        ),
    ];

    for (input, expected) in cases {
        let actual = format_decimal(input).unwrap();
        assert_eq!(actual, expected, "input {input}");
        assert!(!actual.contains('兆'), "input {input} still uses 兆");
    }

    let max = format_cents(MAX_CENTS).unwrap();
    assert_eq!(
        max,
        "玖仟玖佰玖拾玖万玖仟玖佰玖拾玖亿玖仟玖佰玖拾玖万玖仟玖佰玖拾玖元玖角玖分"
    );
    assert!(!max.contains('兆'));
}

#[test]
fn entries_agree_and_structural_invariants_hold() {
    for cents in common::sample_cents() {
        let from_cents = format_cents(cents).unwrap();
        let from_str = format_cents_str(&cents.to_string()).unwrap();
        let decimal = common::decimal_string(cents);
        let from_decimal = format_decimal(&decimal).unwrap();

        assert_eq!(from_cents, from_str, "cents {cents}");
        assert_eq!(from_cents, from_decimal, "decimal {decimal}");
        assert!(
            !from_cents.contains("零零"),
            "double 零 for {cents}: {from_cents}"
        );
        assert!(!from_cents.contains('兆'), "兆 for {cents}: {from_cents}");

        let chars: Vec<char> = from_cents.chars().collect();
        for (index, ch) in chars.iter().enumerate() {
            if *ch != '零' {
                continue;
            }
            if let Some(&next) = chars.get(index + 1) {
                assert!(
                    !matches!(next, '拾' | '佰' | '仟' | '万' | '亿' | '角' | '分' | '整'),
                    "零 before unit for {cents}: {from_cents}"
                );
            }
        }

        let has_zero_yuan = from_cents.contains("零元");
        assert_eq!(
            has_zero_yuan,
            cents < 100,
            "零元 only below one yuan for {cents}: {from_cents}"
        );
        let rest = if cents < 100 {
            from_cents
                .strip_prefix("零元")
                .unwrap_or_else(|| panic!("零元 must be at the start for {cents}: {from_cents}"))
        } else {
            &from_cents
        };
        assert!(
            !rest.contains("零元"),
            "零元 must only appear as the prefix for {cents}: {from_cents}"
        );

        assert_eq!(
            from_cents.ends_with('整'),
            cents % 100 == 0,
            "整 iff whole yuan for {cents}: {from_cents}"
        );
        assert_eq!(
            from_cents.contains('分'),
            cents % 10 != 0,
            "分 iff nonzero fen for {cents}: {from_cents}"
        );
        assert_eq!(
            from_cents.contains('角'),
            (cents / 10) % 10 != 0,
            "角 iff nonzero jiao for {cents}: {from_cents}"
        );
    }
}
