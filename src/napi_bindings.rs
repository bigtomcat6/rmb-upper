use napi_derive::napi;

fn to_napi_error(error: crate::FormatError) -> napi::Error {
    napi::Error::from_reason(error.to_string())
}

/// Backward-compatible JavaScript API.
///
/// This adapter accepts a JavaScript number and rounds it to the nearest cent.
/// Prefer `formatDecimal` for deterministic financial formatting.
#[napi]
pub fn format_number(amount: f64) -> napi::Result<String> {
    crate::format_f64(amount).map_err(to_napi_error)
}

/// Converts an amount represented in cents into uppercase RMB text.
///
/// This accepts a decimal string so callers do not need to rely on JavaScript
/// number precision for large cent values.
#[napi]
pub fn format_cents(cents: String) -> napi::Result<String> {
    crate::format_cents_str(&cents).map_err(to_napi_error)
}

/// Parses a decimal amount string and converts it into uppercase RMB text.
#[napi]
pub fn format_decimal(amount: String) -> napi::Result<String> {
    crate::format_decimal(&amount).map_err(to_napi_error)
}
