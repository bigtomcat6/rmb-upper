use std::error::Error;
use std::fmt;

/// Error returned by the formatting functions when an amount cannot be
/// formatted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    /// The amount is negative.
    NegativeAmount,
    /// The amount is NaN or infinite.
    NonFiniteAmount,
    /// The input string is not a valid unsigned decimal amount, for example
    /// an empty string or a lone `.`.
    InvalidFormat,
    /// The fractional part is longer than two bytes (more than two decimal
    /// places for ASCII input). This is checked before validating the
    /// fraction's characters, so `1.xyz` reports this variant rather than
    /// `InvalidFormat`.
    TooManyDecimalPlaces,
    /// The amount exceeds [`MAX_CENTS`](crate::MAX_CENTS).
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
    /// A character in the input is not allowed, such as a lowercase numeral,
    /// an Arabic digit, or whitespace inside the amount. This lexical check
    /// runs before structural parsing, so `壹元整X` is `InvalidChar`.
    InvalidChar,
    /// The amount is ambiguous, for example a section-final digit that could
    /// be a unit digit or shorthand for the next lower unit
    /// (`壹仟肆佰玖元` can mean 1490 or 1409 yuan).
    Ambiguous,
    /// The token structure is invalid, for example a repeated or out-of-order
    /// unit, a misplaced `零`, mixed `兆` and `万亿` spellings, repeated
    /// `元`/`角`/`分`, a recognized token after `整` (`壹元整伍角`), or a
    /// bare `整` with no amount.
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
