use crate::{FormatError, MAX_CENTS, MAX_YUAN};

pub(crate) fn parse_cents(input: &str) -> Result<i128, FormatError> {
    parse_unsigned(clean_amount(input)?, MAX_CENTS)
}

pub(crate) fn parse_decimal_amount(input: &str) -> Result<i128, FormatError> {
    let input = clean_amount(input)?;
    let (integer, fraction) = split_decimal(input)?;
    let integer = if integer.is_empty() {
        0
    } else {
        parse_unsigned(integer, MAX_YUAN)?
    };
    let fraction = parse_fraction(fraction)?;
    integer
        .checked_mul(100)
        .and_then(|value| value.checked_add(fraction))
        .ok_or(FormatError::TooLarge)
}

fn clean_amount(input: &str) -> Result<&str, FormatError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(FormatError::InvalidFormat);
    }
    if input.starts_with('-') {
        return Err(FormatError::NegativeAmount);
    }
    let input = input.strip_prefix('+').unwrap_or(input);
    if input.is_empty() {
        return Err(FormatError::InvalidFormat);
    }
    Ok(input)
}

fn split_decimal(input: &str) -> Result<(&str, Option<&str>), FormatError> {
    let mut parts = input.split('.');
    let integer = parts.next().ok_or(FormatError::InvalidFormat)?;
    let fraction = parts.next();
    let has_more_parts = parts.next().is_some();
    let is_empty_decimal = integer.is_empty() && fraction.unwrap_or("").is_empty();

    if has_more_parts || is_empty_decimal {
        return Err(FormatError::InvalidFormat);
    }
    Ok((integer, fraction))
}

fn parse_unsigned(input: &str, max: i128) -> Result<i128, FormatError> {
    if input.is_empty() || !input.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(FormatError::InvalidFormat);
    }
    let mut value = 0_i128;
    for ch in input.chars() {
        let digit = ch.to_digit(10).ok_or(FormatError::InvalidFormat)? as i128;
        value = value
            .checked_mul(10)
            .and_then(|current| current.checked_add(digit))
            .ok_or(FormatError::TooLarge)?;
        if value > max {
            return Err(FormatError::TooLarge);
        }
    }
    Ok(value)
}

fn parse_fraction(input: Option<&str>) -> Result<i128, FormatError> {
    let Some(input) = input else {
        return Ok(0);
    };
    if input.len() > 2 {
        return Err(FormatError::TooManyDecimalPlaces);
    }
    if !input.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(FormatError::InvalidFormat);
    }
    let value = match input.len() {
        0 => 0,
        1 => {
            let digit = input.chars().next().unwrap().to_digit(10).unwrap();
            digit as i128 * 10
        }
        2 => input
            .parse::<i128>()
            .map_err(|_| FormatError::InvalidFormat)?,
        _ => unreachable!(),
    };
    Ok(value)
}
