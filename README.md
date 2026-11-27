# rmb-upper

Convert RMB/CNY amounts to and from Chinese uppercase financial text (人民币金额大写).

A small, dependency-free Rust library.

## Installation

```bash
cargo add rmb-upper
```

## Rust usage

The recommended API is `format_cents`, because it represents money as an integer number of cents and avoids floating-point precision issues.

```rust
use rmb_upper::format_cents;

fn main() {
  let value = format_cents(12_345).unwrap();
  assert_eq!(value, "壹佰贰拾叁元肆角伍分");
}
```

For decimal strings, use `format_decimal`:

```rust
use rmb_upper::format_decimal;

assert_eq!(format_decimal("123.45").unwrap(), "壹佰贰拾叁元肆角伍分");
assert_eq!(format_decimal("0.01").unwrap(), "零元壹分");
```

When the amount in cents arrives as a string (for example from JSON or a database), use `format_cents_str`:

```rust
use rmb_upper::format_cents_str;

assert_eq!(format_cents_str("12345").unwrap(), "壹佰贰拾叁元肆角伍分");
```

The `f64` adapter exists for compatibility only:

```rust
use rmb_upper::format_f64;

assert_eq!(format_f64(123.45).unwrap(), "壹佰贰拾叁元肆角伍分");
```

Prefer the cents or string APIs for financial code.

## Supported range

The maximum supported integer part is `MAX_YUAN`:

```text
9999万9999亿9999万9999
```

The maximum supported amount in cents is exposed as `MAX_CENTS` (`MAX_YUAN * 100 + 99`).

## Formatting rules

- The integer part is grouped into four-digit sections with units `万` and `亿`. Amounts of 10¹² and above use the `万亿` spelling (for example `壹万亿元整`); `兆` is never emitted.
- A trailing `整` is written when and only when the amount is a whole number of yuan. An amount ending in `角` or `分` has no `整`.
- Tens are always written as `壹拾`, never as a bare `拾`.
- Consecutive zeros in the integer part collapse into one `零`. A skipped group or a group smaller than 1000 after a higher group is prefixed with `零`: `100001000` becomes `壹亿零壹仟元整`. After `元`, `零` is written only when the integer part is non-zero and either there is a `分` but no `角`, or `角` is present and the integer part ends in zero: `1680.32` becomes `壹仟陆佰捌拾元零叁角贰分`. A zero-yuan amount does not gain an extra `零` after `元`: `0.05` is `零元伍分`. `零零` never appears.

## Errors

The Rust APIs return `Result<String, FormatError>`. Possible error variants include:

- `NegativeAmount`
- `NonFiniteAmount`
- `InvalidFormat`
- `TooManyDecimalPlaces`
- `TooLarge`

## Parsing uppercase amounts

`parse_cents` converts Chinese uppercase RMB text back into an integer number of cents:

```rust
use rmb_upper::parse_cents;

assert_eq!(parse_cents("壹万贰仟叁佰肆拾伍元陆角柒分").unwrap(), 1_234_567);
```

`parse_decimal` returns the same value as a fixed two-decimal string:

```rust
use rmb_upper::parse_decimal;

assert_eq!(parse_decimal("零元伍分").unwrap(), "0.05");
```

Parsing accepts an optional `人民币` prefix, surrounding whitespace, the traditional characters listed in the financial uppercase rules, and fraction-only amounts such as `伍角` or `伍分` without `元`. Invalid input returns `ParseError` with a `ParseErrorKind` and a zero-based character position in the original input. When an amount has an integer part, `元`/`圆`/`圓` is required; ambiguous spellings are rejected.

## Development

```bash
cargo build
cargo test
```

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
