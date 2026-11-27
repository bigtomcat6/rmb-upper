#![deny(clippy::all)]

use napi_derive::napi;

fn to_napi_error(error: rmb_upper::FormatError) -> napi::Error {
    napi::Error::from_reason(error.to_string())
}

/// Backward-compatible JavaScript API.
///
/// This adapter accepts a JavaScript number and rounds it to the nearest cent.
/// Prefer `formatDecimal` for deterministic financial formatting.
#[napi]
pub fn format_number(amount: f64) -> napi::Result<String> {
    rmb_upper::format_f64(amount).map_err(to_napi_error)
}

/// Converts an amount represented in cents into uppercase RMB text.
///
/// This accepts a decimal string so callers do not need to rely on JavaScript
/// number precision for large cent values.
#[napi]
pub fn format_cents(cents: String) -> napi::Result<String> {
    rmb_upper::format_cents_str(&cents).map_err(to_napi_error)
}

/// Parses a decimal amount string and converts it into uppercase RMB text.
#[napi]
pub fn format_decimal(amount: String) -> napi::Result<String> {
    rmb_upper::format_decimal(&amount).map_err(to_napi_error)
}

/// Parses Chinese uppercase RMB text into a string of cents.
#[napi]
pub fn parse_cents(text: String) -> napi::Result<String> {
    rmb_upper::parse_cents(&text)
        .map(|cents| cents.to_string())
        .map_err(to_parse_error)
}

/// Parses Chinese uppercase RMB text into a fixed two-decimal string.
#[napi]
pub fn parse_decimal(text: String) -> napi::Result<String> {
    rmb_upper::parse_decimal(&text).map_err(to_parse_error)
}

fn to_parse_error(error: rmb_upper::ParseError) -> napi::Error {
    napi::Error::from_reason(error.to_string())
}
