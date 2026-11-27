use crate::{ParseError, ParseErrorKind, MAX_CENTS, MAX_YUAN};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenKind {
    Zero,
    Digit(u8),
    Small(u16),
    Big(BigUnit),
    Yuan,
    Jiao,
    Fen,
    Zheng,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BigUnit {
    Wan,
    Yi,
    Zhao,
}

#[derive(Clone, Copy, Debug)]
struct Token {
    kind: TokenKind,
    position: usize,
}

#[derive(Clone, Copy, Debug)]
struct Pending {
    digit: u8,
    position: usize,
    allowed_naked: bool,
    fraction_position: usize,
}

fn error(kind: ParseErrorKind, position: usize) -> ParseError {
    ParseError { kind, position }
}

pub(crate) fn parse_cents(text: &str) -> Result<i128, ParseError> {
    let (tokens, end) = tokenize(text)?;
    parse_tokens(&tokens, end)
}

fn tokenize(text: &str) -> Result<(Vec<Token>, usize), ParseError> {
    let chars: Vec<char> = text.chars().collect();
    let end = chars.len();

    let mut start = 0;
    while start < end && chars[start].is_whitespace() {
        start += 1;
    }
    let mut finish = end;
    while finish > start && chars[finish - 1].is_whitespace() {
        finish -= 1;
    }

    if start == finish {
        return Err(error(ParseErrorKind::Empty, end));
    }

    if finish - start >= 3
        && chars[start] == '人'
        && chars[start + 1] == '民'
        && chars[start + 2] == '币'
    {
        start += 3;
    }

    if start == finish {
        return Err(error(ParseErrorKind::Empty, end));
    }

    let mut tokens = Vec::with_capacity(finish - start);
    for (offset, ch) in chars[start..finish].iter().enumerate() {
        let kind = match ch {
            '零' => TokenKind::Zero,
            '壹' => TokenKind::Digit(1),
            '贰' | '貳' => TokenKind::Digit(2),
            '叁' | '參' | '叄' => TokenKind::Digit(3),
            '肆' => TokenKind::Digit(4),
            '伍' => TokenKind::Digit(5),
            '陆' | '陸' => TokenKind::Digit(6),
            '柒' => TokenKind::Digit(7),
            '捌' => TokenKind::Digit(8),
            '玖' => TokenKind::Digit(9),
            '拾' => TokenKind::Small(10),
            '佰' => TokenKind::Small(100),
            '仟' => TokenKind::Small(1_000),
            '万' | '萬' => TokenKind::Big(BigUnit::Wan),
            '亿' | '億' => TokenKind::Big(BigUnit::Yi),
            '兆' => TokenKind::Big(BigUnit::Zhao),
            '元' | '圆' | '圓' => TokenKind::Yuan,
            '角' => TokenKind::Jiao,
            '分' => TokenKind::Fen,
            '整' | '正' => TokenKind::Zheng,
            _ => return Err(error(ParseErrorKind::InvalidChar, start + offset)),
        };
        tokens.push(Token {
            kind,
            position: start + offset,
        });
    }

    Ok((tokens, end))
}

fn parse_tokens(tokens: &[Token], end: usize) -> Result<i128, ParseError> {
    if tokens.is_empty() {
        return Err(error(ParseErrorKind::Empty, end));
    }

    if tokens[0].kind == TokenKind::Zero {
        if matches!(tokens.get(1).map(|token| token.kind), Some(TokenKind::Yuan)) {
            return parse_fraction(tokens, 2, 0, end);
        }
        if tokens.len() == 1 {
            return Err(error(ParseErrorKind::MissingYuan, end));
        }
        return Err(error(ParseErrorKind::UnexpectedToken, tokens[0].position));
    }

    if matches!(tokens[0].kind, TokenKind::Digit(_))
        && matches!(
            tokens.get(1).map(|token| token.kind),
            Some(TokenKind::Jiao | TokenKind::Fen)
        )
    {
        return parse_fraction(tokens, 0, 0, end);
    }

    parse_integer(tokens, end)
}

fn parse_integer(tokens: &[Token], end: usize) -> Result<i128, ParseError> {
    let mut groups: Vec<i128> = Vec::new();
    let mut units: Vec<(BigUnit, usize)> = Vec::new();
    let mut section = 0_i128;
    let mut pending: Option<Pending> = None;
    let mut last_small: Option<u16> = None;
    let mut zero_last = false;
    let mut zero_position = 0_usize;
    let mut started = false;
    let mut previous_kind: Option<TokenKind> = None;
    let mut deferred_ambiguity: Option<usize> = None;
    let mut index = 0;

    while index < tokens.len() {
        let token = tokens[index];
        match token.kind {
            TokenKind::Zero => {
                if zero_last || pending.is_some() || !started {
                    return Err(error(ParseErrorKind::UnexpectedToken, token.position));
                }
                zero_last = true;
                zero_position = token.position;
                previous_kind = Some(TokenKind::Zero);
                index += 1;
            }
            TokenKind::Digit(digit) => {
                if pending.is_some() {
                    return Err(error(ParseErrorKind::UnexpectedToken, token.position));
                }
                let allowed_naked =
                    !started || zero_last || matches!(previous_kind, Some(TokenKind::Small(10)));
                let fraction_position = if zero_last {
                    zero_position
                } else {
                    token.position
                };
                pending = Some(Pending {
                    digit,
                    position: token.position,
                    allowed_naked,
                    fraction_position,
                });
                zero_last = false;
                started = true;
                previous_kind = Some(TokenKind::Digit(digit));
                index += 1;
            }
            TokenKind::Small(unit) if !started && unit == 10 => {
                section = 10;
                last_small = Some(10);
                started = true;
                previous_kind = Some(TokenKind::Small(10));
                index += 1;
            }
            TokenKind::Small(unit) => {
                if zero_last {
                    return Err(error(ParseErrorKind::UnexpectedToken, token.position));
                }
                let Some(value) = pending.take() else {
                    return Err(error(ParseErrorKind::UnexpectedToken, token.position));
                };
                if last_small.is_some_and(|last| unit >= last) {
                    return Err(error(ParseErrorKind::UnexpectedToken, token.position));
                }
                let increment = i128::from(value.digit)
                    .checked_mul(i128::from(unit))
                    .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, token.position))?;
                section = section
                    .checked_add(increment)
                    .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, token.position))?;
                last_small = Some(unit);
                started = true;
                previous_kind = Some(TokenKind::Small(unit));
                index += 1;
            }
            TokenKind::Big(unit) => {
                let composite =
                    unit == BigUnit::Yi && units.len() == 1 && units[0].0 == BigUnit::Wan;
                push_unit(&mut units, unit, token.position)?;
                if let Some(position) = resolve_pending(&mut pending, &mut section)? {
                    deferred_ambiguity.get_or_insert(position);
                }
                if zero_last {
                    return Err(error(ParseErrorKind::UnexpectedToken, token.position));
                }
                if section == 0 && !composite {
                    return Err(error(ParseErrorKind::UnexpectedToken, token.position));
                }
                groups.push(section);
                section = 0;
                last_small = None;
                zero_last = false;
                started = true;
                previous_kind = Some(TokenKind::Big(unit));
                index += 1;
            }
            TokenKind::Yuan => {
                if let Some(position) = resolve_pending(&mut pending, &mut section)? {
                    deferred_ambiguity.get_or_insert(position);
                }
                if let Some(position) = deferred_ambiguity {
                    return Err(error(ParseErrorKind::Ambiguous, position));
                }
                if !started || zero_last {
                    return Err(error(ParseErrorKind::UnexpectedToken, token.position));
                }
                groups.push(section);
                let yuan = combine(&groups, &units, end)?;
                return parse_fraction(tokens, index + 1, yuan, end);
            }
            TokenKind::Jiao | TokenKind::Fen => {
                if !started {
                    return Err(error(ParseErrorKind::UnexpectedToken, token.position));
                }
                if zero_last {
                    return Err(error(ParseErrorKind::UnexpectedToken, token.position));
                }
                let position = pending.map_or(token.position, |value| value.fraction_position);
                return Err(error(ParseErrorKind::MissingYuan, position));
            }
            TokenKind::Zheng => {
                if !started {
                    return Err(error(ParseErrorKind::UnexpectedToken, token.position));
                }
                if zero_last {
                    return Err(error(ParseErrorKind::UnexpectedToken, token.position));
                }
                return Err(error(ParseErrorKind::MissingYuan, token.position));
            }
        }
    }

    if let Some(position) = resolve_pending(&mut pending, &mut section)? {
        deferred_ambiguity.get_or_insert(position);
    }
    if let Some(position) = deferred_ambiguity {
        return Err(error(ParseErrorKind::Ambiguous, position));
    }
    if !started {
        return Err(error(ParseErrorKind::UnexpectedToken, end));
    }
    if zero_last {
        return Err(error(ParseErrorKind::UnexpectedToken, end));
    }

    Err(error(ParseErrorKind::MissingYuan, end))
}

fn resolve_pending(
    pending: &mut Option<Pending>,
    section: &mut i128,
) -> Result<Option<usize>, ParseError> {
    let Some(value) = pending.take() else {
        return Ok(None);
    };
    *section = section
        .checked_add(i128::from(value.digit))
        .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, value.position))?;
    if value.allowed_naked {
        Ok(None)
    } else {
        Ok(Some(value.position))
    }
}

fn push_unit(
    units: &mut Vec<(BigUnit, usize)>,
    unit: BigUnit,
    position: usize,
) -> Result<(), ParseError> {
    let has_zhao = units.iter().any(|(value, _)| *value == BigUnit::Zhao);
    let has_yi = units.iter().any(|(value, _)| *value == BigUnit::Yi);
    let wan_count = units
        .iter()
        .filter(|(value, _)| *value == BigUnit::Wan)
        .count();
    let wan_after_yi = units
        .iter()
        .position(|(value, _)| *value == BigUnit::Yi)
        .map_or(0, |index| {
            units[index + 1..]
                .iter()
                .filter(|(value, _)| *value == BigUnit::Wan)
                .count()
        });
    let wan_before_yi = wan_count - wan_after_yi;

    let valid = match unit {
        BigUnit::Zhao => units.is_empty(),
        BigUnit::Yi if has_zhao => !has_yi && wan_count == 0,
        BigUnit::Yi => !has_yi,
        BigUnit::Wan if has_zhao => wan_count == 0,
        BigUnit::Wan if has_yi => wan_after_yi == 0,
        BigUnit::Wan => wan_before_yi == 0,
    };

    if !valid {
        return Err(error(ParseErrorKind::UnexpectedToken, position));
    }
    units.push((unit, position));
    Ok(())
}

fn combine(groups: &[i128], units: &[(BigUnit, usize)], end: usize) -> Result<i128, ParseError> {
    let mut total = 0_i128;

    if units.first().map(|(unit, _)| *unit) == Some(BigUnit::Zhao) {
        total = groups[0]
            .checked_mul(1_000_000_000_000)
            .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, end))?;
        for (index, (unit, _)) in units.iter().enumerate().skip(1) {
            let factor = match unit {
                BigUnit::Yi => 100_000_000,
                BigUnit::Wan => 10_000,
                BigUnit::Zhao => unreachable!(),
            };
            let term = groups[index]
                .checked_mul(factor)
                .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, end))?;
            total = total
                .checked_add(term)
                .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, end))?;
        }
        total = total
            .checked_add(groups[units.len()])
            .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, end))?;
    } else if units.first().map(|(unit, _)| *unit) == Some(BigUnit::Wan)
        && units.get(1).map(|(unit, _)| *unit) == Some(BigUnit::Yi)
    {
        let high = groups[0]
            .checked_mul(10_000)
            .and_then(|value| value.checked_add(groups[1]))
            .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, end))?;
        total = high
            .checked_mul(100_000_000)
            .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, end))?;
        if units.len() == 3 {
            let term = groups[2]
                .checked_mul(10_000)
                .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, end))?;
            total = total
                .checked_add(term)
                .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, end))?;
        }
        total = total
            .checked_add(groups[units.len()])
            .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, end))?;
    } else {
        for (index, (unit, _)) in units.iter().enumerate() {
            let factor = match unit {
                BigUnit::Yi => 100_000_000,
                BigUnit::Wan => 10_000,
                BigUnit::Zhao => unreachable!(),
            };
            let term = groups[index]
                .checked_mul(factor)
                .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, end))?;
            total = total
                .checked_add(term)
                .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, end))?;
        }
        total = total
            .checked_add(groups[units.len()])
            .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, end))?;
    }

    if total > MAX_YUAN {
        return Err(error(ParseErrorKind::UnexpectedToken, end));
    }
    Ok(total)
}

fn parse_fraction(
    tokens: &[Token],
    mut index: usize,
    yuan: i128,
    end: usize,
) -> Result<i128, ParseError> {
    let mut jiao = 0_u8;
    let mut fen = 0_u8;
    let mut has_jiao = false;
    let mut has_fen = false;
    let mut zero_after_yuan = false;

    while index < tokens.len() {
        let token = tokens[index];
        match token.kind {
            TokenKind::Zero => {
                if zero_after_yuan || has_jiao || has_fen {
                    return Err(error(ParseErrorKind::UnexpectedToken, token.position));
                }
                zero_after_yuan = true;
                index += 1;
                if index == tokens.len() {
                    return Err(error(ParseErrorKind::UnexpectedToken, end));
                }
                if !matches!(tokens[index].kind, TokenKind::Digit(_)) {
                    return Err(error(
                        ParseErrorKind::UnexpectedToken,
                        tokens[index].position,
                    ));
                }
            }
            TokenKind::Digit(digit) => {
                let Some(unit) = tokens.get(index + 1) else {
                    return Err(error(ParseErrorKind::UnexpectedToken, end));
                };
                match unit.kind {
                    TokenKind::Jiao if !has_jiao && !has_fen => {
                        has_jiao = true;
                        jiao = digit;
                        zero_after_yuan = false;
                        index += 2;
                    }
                    TokenKind::Fen if !has_fen => {
                        has_fen = true;
                        fen = digit;
                        zero_after_yuan = false;
                        index += 2;
                    }
                    _ => return Err(error(ParseErrorKind::UnexpectedToken, unit.position)),
                }
            }
            TokenKind::Zheng => {
                if index + 1 != tokens.len() {
                    return Err(error(
                        ParseErrorKind::UnexpectedToken,
                        tokens[index + 1].position,
                    ));
                }
                index += 1;
            }
            TokenKind::Jiao
            | TokenKind::Fen
            | TokenKind::Yuan
            | TokenKind::Small(_)
            | TokenKind::Big(_) => {
                return Err(error(ParseErrorKind::UnexpectedToken, token.position));
            }
        }
    }

    let fraction = i128::from(jiao)
        .checked_mul(10)
        .and_then(|value| value.checked_add(i128::from(fen)))
        .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, end))?;
    let cents = yuan
        .checked_mul(100)
        .and_then(|value| value.checked_add(fraction))
        .ok_or_else(|| error(ParseErrorKind::UnexpectedToken, end))?;
    if cents > MAX_CENTS {
        return Err(error(ParseErrorKind::UnexpectedToken, end));
    }
    Ok(cents)
}
