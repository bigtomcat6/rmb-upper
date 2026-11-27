#![warn(missing_docs)]
#![deny(clippy::all)]

//! Convert RMB/CNY amounts to and from Chinese uppercase financial text
//! (人民币金额大写).
//!
//! For formatting, prefer [`format_cents`] when the amount is already an
//! integer number of cents, or [`format_decimal`] when it is a decimal
//! string. [`format_cents_str`] is the string-based cents entry point for
//! callers that cannot use a Rust integer. [`format_f64`] exists only for
//! compatibility with binary floating-point input; prefer the first three for
//! financial code. For parsing, use [`parse_cents`] or [`parse_decimal`].
//!
//! ```
//! use rmb_upper::{format_cents, parse_cents};
//!
//! assert_eq!(format_cents(12_345).unwrap(), "壹佰贰拾叁元肆角伍分");
//! assert_eq!(parse_cents("壹佰贰拾叁元肆角伍分").unwrap(), 12_345);
//! ```
//!
//! # Formatting rules
//!
//! - The integer part is grouped into four-digit sections. From low to high
//!   their units are none, `万`, `亿`, and `万`. A non-zero 10¹² section uses
//!   `万`, and when the `亿` section is zero an `亿` is appended, so values of
//!   10¹² and above use the `万亿` spelling (for example `壹万亿元整`).
//!   `兆` is never emitted.
//! - A trailing `整` is written when and only when the amount is a whole
//!   number of yuan. An amount ending in `角` or `分` has no `整`.
//! - Tens are always written as `壹拾`, never as a bare `拾`.
//! - Consecutive zeros in the integer part collapse into a single `零`. A
//!   skipped group or a group smaller than 1000 after a higher group is
//!   prefixed with `零` (`100001000` becomes `壹亿零壹仟元整`), and `零零`
//!   never appears. After `元`, a `零` is written only when the integer part
//!   is non-zero and either there is a `分` but no `角`, or `角` is present
//!   and the integer part ends in zero (`1680.32` becomes
//!   `壹仟陆佰捌拾元零叁角贰分`). A zero-yuan amount does not gain an extra
//!   `零` after `元` (`0.05` is `零元伍分`).
//! - The supported range is [`MAX_YUAN`] yuan or [`MAX_CENTS`] cents,
//!   inclusive.
//!
//! # Parsing rules
//!
//! Parsing is lenient whenever the value is unambiguous. It accepts an
//! optional `人民币` prefix and surrounding whitespace, the traditional
//! characters `貳`, `參`/`叄`, `陸`, `萬`, `億`, and `圓`, either
//! `元`/`圆`/`圓`, an optional `整`/`正` after `元` or `角` (and tolerates it
//! after `分`), omitted `零` where the rules allow it, a leading `拾` meaning
//! `壹拾`, fraction-only amounts such as `伍角` or `伍分` without `元`, and
//! both the `兆` and `万亿` spellings for values of 10¹² and above (but not a
//! mixture of the two).
//!
//! It rejects lowercase numerals and other non-standard characters (`一`,
//! `二`, `两`, `十`, `〇`, `念`, `毛`, `另`, Arabic digits), whitespace inside
//! the amount, an ambiguous trailing digit in a section (`壹仟肆佰玖元`,
//! `壹佰伍元`, `壹万伍元`, `壹亿伍万元`), repeated or out-of-order units,
//! misplaced zeros, out-of-order or repeated `元`/`角`/`分`, a recognized
//! token after `整` (`壹元整伍角`), a bare `整` with no amount, and an
//! integer part without `元`. Illegal characters are rejected before
//! structural parsing, so `壹元整X` is an invalid character rather than a
//! structural error. Errors carry a [`ParseErrorKind`] and a zero-based
//! character position.

mod error;
mod format;
mod input;
mod parse;

pub use error::{FormatError, ParseError, ParseErrorKind};

/// Largest supported whole-yuan amount:
/// `9_999_999_999_999_999` (`9999万9999亿9999万9999`).
pub const MAX_YUAN: i128 = 9_999_999_999_999_999;

/// Largest supported amount in cents: [`MAX_YUAN`] `* 100 + 99`.
pub const MAX_CENTS: i128 = MAX_YUAN * 100 + 99;

/// Formats an integer number of cents as Chinese uppercase RMB text.
///
/// # Errors
///
/// - [`FormatError::NegativeAmount`] if `cents` is negative.
/// - [`FormatError::TooLarge`] if `cents` exceeds [`MAX_CENTS`].
///
/// # Examples
///
/// ```
/// use rmb_upper::format_cents;
///
/// assert_eq!(format_cents(12_345).unwrap(), "壹佰贰拾叁元肆角伍分");
/// assert_eq!(format_cents(0).unwrap(), "零元整");
/// ```
pub fn format_cents(cents: i128) -> Result<String, FormatError> {
    if cents < 0 {
        return Err(FormatError::NegativeAmount);
    }

    if cents > MAX_CENTS {
        return Err(FormatError::TooLarge);
    }

    let integer = (cents / 100) as u128;
    let fraction = (cents % 100) as u8;
    Ok(format::format_amount(integer, fraction))
}

/// Formats a string containing an integer number of cents as Chinese
/// uppercase RMB text.
///
/// Leading and trailing whitespace and an optional leading `+` are ignored.
///
/// # Errors
///
/// - [`FormatError::NegativeAmount`] if the value starts with `-`.
/// - [`FormatError::InvalidFormat`] if the string is empty or contains
///   anything other than ASCII digits after the optional `+`.
/// - [`FormatError::TooLarge`] if the value exceeds [`MAX_CENTS`] or does not
///   fit in [`i128`].
///
/// # Examples
///
/// ```
/// use rmb_upper::format_cents_str;
///
/// assert_eq!(format_cents_str("12345").unwrap(), "壹佰贰拾叁元肆角伍分");
/// assert_eq!(format_cents_str(" 7 ").unwrap(), "零元柒分");
/// ```
pub fn format_cents_str(cents: &str) -> Result<String, FormatError> {
    input::parse_cents(cents).and_then(format_cents)
}

/// Formats a decimal amount string, such as `"123.45"`, as Chinese uppercase
/// RMB text.
///
/// Leading and trailing whitespace and an optional leading `+` are ignored.
/// `"5"`, `"5."`, and `".5"` are accepted; the fractional part is read as
/// `角` and `分`.
///
/// # Errors
///
/// - [`FormatError::NegativeAmount`] if the value starts with `-`.
/// - [`FormatError::InvalidFormat`] if the string is empty, is a lone `.` with
///   no digits on either side, has more than one decimal point, or has a
///   non-digit in the integer part or in a fractional part of at most two
///   bytes.
/// - [`FormatError::TooManyDecimalPlaces`] if the fractional part is longer
///   than two bytes (more than two decimal places for ASCII input). This is
///   checked before validating the fraction's characters, so `1.xyz` reports
///   this variant rather than `InvalidFormat`.
/// - [`FormatError::TooLarge`] if the integer part exceeds [`MAX_YUAN`] or
///   the amount exceeds [`MAX_CENTS`].
///
/// # Examples
///
/// ```
/// use rmb_upper::format_decimal;
///
/// assert_eq!(format_decimal("123.45").unwrap(), "壹佰贰拾叁元肆角伍分");
/// assert_eq!(format_decimal("0.01").unwrap(), "零元壹分");
/// ```
pub fn format_decimal(amount: &str) -> Result<String, FormatError> {
    input::parse_decimal_amount(amount).and_then(format_cents)
}

/// Formats an `f64` amount as Chinese uppercase RMB text, rounding to the
/// nearest cent.
///
/// This adapter exists for compatibility with binary floating-point input.
/// Prefer [`format_cents`] or [`format_decimal`] for financial code, because
/// `f64` cannot represent most decimal amounts exactly. Rounding is only as
/// accurate as the stored value: `1.005` is stored slightly below 1.005 and
/// therefore formats as `壹元整`, and large amounts lose precision, so
/// `99999999999999.99` formats with a `fen` digit of 8 rather than 9.
///
/// # Errors
///
/// - [`FormatError::NonFiniteAmount`] if `amount` is NaN or infinite.
/// - [`FormatError::NegativeAmount`] if `amount` is negative. `-0.0` is
///   accepted and formats as `零元整`.
/// - [`FormatError::TooLarge`] if the rounded amount exceeds [`MAX_CENTS`].
///
/// # Examples
///
/// ```
/// use rmb_upper::format_f64;
///
/// assert_eq!(format_f64(123.456).unwrap(), "壹佰贰拾叁元肆角陆分");
/// assert_eq!(format_f64(1.005).unwrap(), "壹元整");
/// assert_eq!(
///     format_f64(99999999999999.99).unwrap(),
///     "玖拾玖万玖仟玖佰玖拾玖亿玖仟玖佰玖拾玖万玖仟玖佰玖拾玖元玖角捌分"
/// );
/// ```
pub fn format_f64(amount: f64) -> Result<String, FormatError> {
    if !amount.is_finite() {
        return Err(FormatError::NonFiniteAmount);
    }

    if amount < 0.0 {
        return Err(FormatError::NegativeAmount);
    }

    let cents = (amount * 100.0).round();
    if cents > MAX_CENTS as f64 {
        return Err(FormatError::TooLarge);
    }

    format_cents(cents as i128)
}

/// Parses Chinese uppercase RMB text into an integer number of cents.
///
/// The accepted spellings and rejected forms are described in the
/// [crate-level documentation](crate#parsing-rules).
///
/// # Errors
///
/// Returns [`ParseError`] with one of these kinds:
///
/// - [`ParseErrorKind::Empty`] if the input is empty after trimming
///   whitespace and removing an optional `人民币` prefix.
/// - [`ParseErrorKind::InvalidChar`] if the input contains a character that
///   is not allowed, including lowercase numerals, ASCII digits, or
///   whitespace inside the amount. This lexical check runs before structural
///   parsing, so `壹元整X` is `InvalidChar`.
/// - [`ParseErrorKind::Ambiguous`] if a digit at the end of a section could
///   also be shorthand for the next lower unit, as in `壹仟肆佰玖元`,
///   `壹佰伍元`, `壹万伍元`, or `壹亿伍万元`.
/// - [`ParseErrorKind::UnexpectedToken`] if units are repeated or out of
///   order, `零` is misplaced, the `兆` and `万亿` spellings are mixed,
///   `元`/`角`/`分` are out of order or repeated, a recognized token follows
///   `整` (`壹元整伍角`), or the input is a bare `整` with no amount.
/// - [`ParseErrorKind::MissingYuan`] if an integer part is present but
///   `元`/`圆`/`圓` is missing.
///
/// The error also carries a zero-based character position in the original
/// input; see [`ParseError::position`].
///
/// # Examples
///
/// ```
/// use rmb_upper::parse_cents;
///
/// assert_eq!(parse_cents("壹万贰仟叁佰肆拾伍元陆角柒分").unwrap(), 1_234_567);
/// assert_eq!(parse_cents("人民币壹佰元整").unwrap(), 10_000);
/// ```
pub fn parse_cents(text: &str) -> Result<i128, ParseError> {
    parse::parse_cents(text)
}

/// Parses Chinese uppercase RMB text into a fixed two-decimal string.
///
/// For example, `"零元伍分"` becomes `"0.05"` and `"壹佰元整"` becomes
/// `"100.00"`. The accepted spellings and rejected forms are described in the
/// [crate-level documentation](crate#parsing-rules).
///
/// # Errors
///
/// Returns the same errors as [`parse_cents`]:
///
/// - [`ParseErrorKind::Empty`] if the input is empty after trimming
///   whitespace and removing an optional `人民币` prefix.
/// - [`ParseErrorKind::InvalidChar`] if the input contains a character that
///   is not allowed, including lowercase numerals, ASCII digits, or
///   whitespace inside the amount. This lexical check runs before structural
///   parsing, so `壹元整X` is `InvalidChar`.
/// - [`ParseErrorKind::Ambiguous`] if a digit at the end of a section could
///   also be shorthand for the next lower unit, as in `壹仟肆佰玖元`,
///   `壹佰伍元`, `壹万伍元`, or `壹亿伍万元`.
/// - [`ParseErrorKind::UnexpectedToken`] if units are repeated or out of
///   order, `零` is misplaced, the `兆` and `万亿` spellings are mixed,
///   `元`/`角`/`分` are out of order or repeated, a recognized token follows
///   `整` (`壹元整伍角`), or the input is a bare `整` with no amount.
/// - [`ParseErrorKind::MissingYuan`] if an integer part is present but
///   `元`/`圆`/`圓` is missing.
///
/// The error also carries a zero-based character position in the original
/// input; see [`ParseError::position`].
///
/// # Examples
///
/// ```
/// use rmb_upper::parse_decimal;
///
/// assert_eq!(parse_decimal("零元伍分").unwrap(), "0.05");
/// assert_eq!(parse_decimal("壹佰元整").unwrap(), "100.00");
/// ```
pub fn parse_decimal(text: &str) -> Result<String, ParseError> {
    let cents = parse_cents(text)?;
    Ok(format!("{}.{:02}", cents / 100, cents % 100))
}
