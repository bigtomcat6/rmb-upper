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

/// Error returned when parsing Chinese uppercase RMB text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// What kind of parse error occurred.
    pub kind: ParseErrorKind,
    /// Zero-based character offset in the original input, including any
    /// surrounding whitespace and the `人民币` prefix.
    ///
    /// For [`ParseErrorKind::MissingYuan`], this is the position where `元`
    /// should be inserted: before a trailing `整`/`正`, before the leading
    /// `零` of the fraction, before the first fraction digit (or `角`/`分`
    /// when there is no digit), or at the end of the input when the integer
    /// part ends there. [`ParseErrorKind::Empty`] reports the input length.
    pub position: usize,
}

/// Kinds of errors that can occur while parsing Chinese uppercase RMB text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// The input is empty after removing surrounding whitespace and the
    /// `人民币` prefix.
    Empty,
    /// The input contains a character that is not allowed.
    InvalidChar,
    /// The amount is written in a way that is ambiguous.
    Ambiguous,
    /// The token structure is invalid, for example a repeated unit.
    UnexpectedToken,
    /// An integer part is present but `元`, `圆` or `圓` is missing.
    MissingYuan,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self.kind {
            ParseErrorKind::Empty => "amount text is empty",
            ParseErrorKind::InvalidChar => "amount contains an invalid character",
            ParseErrorKind::Ambiguous => "amount is ambiguous",
            ParseErrorKind::UnexpectedToken => "amount structure is invalid",
            ParseErrorKind::MissingYuan => "amount is missing 元",
        };
        write!(f, "{reason} at character {}", self.position)
    }
}

impl Error for ParseError {}
