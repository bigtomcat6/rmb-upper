use rmb_upper::*;

mod common;

fn expect_cents(text: &str, cents: i128) {
    assert_eq!(parse_cents(text), Ok(cents), "input {text}");
    assert_eq!(
        parse_decimal(text),
        Ok(common::decimal_string(cents)),
        "input {text}"
    );
}

fn expect_error(text: &str, kind: ParseErrorKind, position: usize) {
    let error = parse_cents(text).expect_err(&format!("input {text} should fail"));
    assert_eq!(error.kind, kind, "input {text}");
    assert_eq!(error.position, position, "input {text}");
    assert_eq!(
        parse_decimal(text).map_err(|error| error.kind),
        Err(kind),
        "input {text}"
    );
}

#[test]
fn round_trips_every_sample() {
    for cents in common::sample_cents() {
        let text = format_cents(cents).unwrap();
        assert_eq!(parse_cents(&text), Ok(cents), "cents {cents}: {text}");
        assert_eq!(
            parse_decimal(&text),
            Ok(common::decimal_string(cents)),
            "cents {cents}: {text}"
        );
    }
}

#[test]
fn parses_regulation_examples() {
    let cases = [
        ("壹拾万柒仟元零伍角叁分", 10_700_053),
        ("壹拾万零柒仟元伍角叁分", 10_700_053),
        ("壹拾万柒仟元伍角叁分", 10_700_053),
        ("壹仟陆佰捌拾元零叁角贰分", 168_032),
        ("壹仟陆佰捌拾元叁角贰分", 168_032),
        ("人民币壹仟肆佰零玖元伍角", 140_950),
        ("陆仟零柒元壹角肆分", 600_714),
        ("壹万陆仟肆佰零玖元零贰分", 1_640_902),
        ("叁佰贰拾伍元零肆分", 32_504),
    ];

    for (text, cents) in cases {
        expect_cents(text, cents);
    }
}

#[test]
fn parses_variant_spellings() {
    let cases = [
        ("人民币壹佰元整", 10_000),
        ("壹佰圆正", 10_000),
        ("壹佰元", 10_000),
        ("  壹元整 ", 100),
        ("伍角整", 50),
        ("零元伍角", 50),
        ("伍分", 5),
        ("拾元整", 1_000),
        ("拾伍元", 1_500),
        ("壹萬貳仟參佰圓整", 1_230_000),
        ("陸億元整", 60_000_000_000),
        ("壹兆元整", 100_000_000_000_000),
        ("壹万亿元整", 100_000_000_000_000),
        ("壹兆贰仟亿元整", 120_000_000_000_000),
        ("壹万贰仟亿元整", 120_000_000_000_000),
        (
            "玖仟玖佰玖拾玖万玖仟玖佰玖拾玖亿玖仟玖佰玖拾玖万玖仟玖佰玖拾玖元玖角玖分",
            MAX_CENTS,
        ),
        ("壹元壹角壹分整", 111),
        ("零元", 0),
        ("零元整", 0),
    ];

    for (text, cents) in cases {
        expect_cents(text, cents);
    }
}

#[test]
fn parses_decimal_output() {
    assert_eq!(parse_decimal("零元伍分"), Ok("0.05".to_string()));
    assert_eq!(parse_decimal("壹佰元整"), Ok("100.00".to_string()));
    assert_eq!(parse_decimal("人民币壹元壹角壹分"), Ok("1.11".to_string()));
}

#[test]
fn rejects_empty_input() {
    let cases = [("", 0), ("人民币", 3), ("   ", 3), (" 人民币 ", 5)];
    for (text, position) in cases {
        expect_error(text, ParseErrorKind::Empty, position);
    }
}

#[test]
fn rejects_invalid_characters() {
    for text in [
        "一", "二", "两", "三", "四", "五", "六", "七", "八", "九", "十", "〇", "念", "毛", "另",
        "5",
    ] {
        expect_error(text, ParseErrorKind::InvalidChar, 0);
    }

    expect_error("壹 佰元", ParseErrorKind::InvalidChar, 1);
    expect_error("壹佰5元", ParseErrorKind::InvalidChar, 2);
    expect_error("壹佰伍元一", ParseErrorKind::InvalidChar, 4);
}

#[test]
fn rejects_ambiguous_spellings() {
    let cases = [
        ("壹仟肆佰玖元", 4),
        ("壹佰伍元", 2),
        ("壹万伍元", 2),
        ("壹亿伍万元", 2),
    ];
    for (text, position) in cases {
        expect_error(text, ParseErrorKind::Ambiguous, position);
    }
}

#[test]
fn rejects_unexpected_tokens() {
    let cases = [
        ("壹拾壹佰元", 3),
        ("壹佰佰元", 2),
        ("壹亿壹兆元", 3),
        ("壹亿亿元", 2),
        ("壹万万元", 2),
        ("壹兆壹万亿元", 4),
        ("壹佰零零伍元", 3),
        ("壹零拾元", 1),
        ("壹拾零元", 3),
        ("伍角壹元", 3),
        ("壹角壹角", 3),
        ("壹元整伍角", 3),
        ("零伍角", 0),
        ("整", 0),
    ];
    for (text, position) in cases {
        expect_error(text, ParseErrorKind::UnexpectedToken, position);
    }
}

#[test]
fn rejects_missing_yuan() {
    let cases = [
        ("壹佰", 2),
        ("壹佰伍角", 2),
        ("壹佰零伍角", 2),
        ("壹佰伍分", 2),
        ("壹拾伍整", 3),
        ("人民币壹佰", 5),
        ("人民币壹佰伍角", 5),
        ("零", 1),
        ("　人民币零　", 6),
    ];
    for (text, position) in cases {
        expect_error(text, ParseErrorKind::MissingYuan, position);
    }
}

#[test]
fn does_not_panic_on_random_strings() {
    let alphabet: Vec<char> = "零壹贰貳叁參叄肆伍陆陸柒捌玖拾佰仟万萬亿億兆元圆圓角分整正一两毛 5"
        .chars()
        .collect();
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;

    for _ in 0..10_000 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let length = (state % 21) as usize;
        let mut text = String::new();
        for _ in 0..length {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let index = (state % alphabet.len() as u64) as usize;
            text.push(alphabet[index]);
        }

        match parse_cents(&text) {
            Ok(cents) => {
                assert!(cents <= MAX_CENTS, "out of range for {text}");
                assert_eq!(
                    parse_decimal(&text),
                    Ok(common::decimal_string(cents)),
                    "input {text}"
                );
                let formatted = format_cents(cents).unwrap();
                assert_eq!(parse_cents(&formatted), Ok(cents), "input {text}");
            }
            Err(_) => {
                assert!(parse_decimal(&text).is_err(), "input {text}");
            }
        }
    }
}
