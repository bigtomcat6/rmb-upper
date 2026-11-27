#![deny(clippy::all)]

mod error;
mod format;
mod input;
mod parse;

pub use error::{FormatError, ParseError, ParseErrorKind};

pub const MAX_YUAN: i128 = 9_999_999_999_999_999;
pub const MAX_CENTS: i128 = MAX_YUAN * 100 + 99;

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

pub fn format_cents_str(cents: &str) -> Result<String, FormatError> {
    input::parse_cents(cents).and_then(format_cents)
}

pub fn format_decimal(amount: &str) -> Result<String, FormatError> {
    input::parse_decimal_amount(amount).and_then(format_cents)
}

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
/// The parser accepts the variants allowed by the financial uppercase amount
/// rules, including traditional characters, an optional `人民币` prefix and
/// surrounding whitespace. Error positions are zero-based character offsets
/// in the original input.
pub fn parse_cents(text: &str) -> Result<i128, ParseError> {
    parse::parse_cents(text)
}

/// Parses Chinese uppercase RMB text into a fixed two-decimal string.
///
/// For example, `"零元伍分"` becomes `"0.05"` and `"壹佰元整"` becomes
/// `"100.00"`.
pub fn parse_decimal(text: &str) -> Result<String, ParseError> {
    let cents = parse_cents(text)?;
    Ok(format!("{}.{:02}", cents / 100, cents % 100))
}
