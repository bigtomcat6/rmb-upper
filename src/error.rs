use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    NegativeAmount,
    NonFiniteAmount,
    InvalidFormat,
    TooManyDecimalPlaces,
    TooLarge,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeAmount => write!(f, "amount cannot be negative"),
            Self::NonFiniteAmount => write!(f, "amount must be a finite number"),
            Self::InvalidFormat => write!(f, "amount format is invalid"),
            Self::TooManyDecimalPlaces => {
                write!(f, "amount cannot have more than two decimal places")
            }
            Self::TooLarge => write!(f, "amount is too large"),
        }
    }
}

impl Error for FormatError {}
