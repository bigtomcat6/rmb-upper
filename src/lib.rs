#![deny(clippy::all)]

mod error;
mod format;
mod input;

pub use error::FormatError;

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
