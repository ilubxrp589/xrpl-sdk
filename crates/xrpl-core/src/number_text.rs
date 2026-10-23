//! Decimal text as `rippled` reads it (`partsFromString`), shared by the
//! codec (XRP, MPT and `Number` values) and [`crate::types::IouAmount`].

use crate::CoreError;

/// A decimal string split as `rippled`'s `partsFromString` reads it:
/// `^[-+]?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][-+]?[0-9]+)?$`, with the written
/// exponent fitting an `i32`. The value is ±`int``frac` × 10^`exponent`.
pub(crate) struct Parts<'a> {
    pub negative: bool,
    pub int: &'a str,
    pub frac: &'a str,
    /// The written exponent, less the number of fraction digits.
    pub exponent: i64,
}

impl Parts<'_> {
    /// The digits, integer then fraction part.
    pub fn digits(&self) -> impl Iterator<Item = u64> + '_ {
        self.int
            .bytes()
            .chain(self.frac.bytes())
            .map(|b| u64::from(b - b'0'))
    }
}

pub(crate) fn split(s: &str) -> Result<Parts<'_>, CoreError> {
    let not_read = |why: &str| CoreError::InvalidAmount(format!("{s:?} {why}"));
    let bad = || not_read("is not a number rippled reads");
    let (negative, rest) = match (s.strip_prefix('-'), s.strip_prefix('+')) {
        (Some(rest), _) => (true, rest),
        (_, Some(rest)) => (false, rest),
        _ => (false, s),
    };
    let (body, exp) = match rest.split_once(['e', 'E']) {
        Some((body, exp)) => (body, Some(exp)),
        None => (rest, None),
    };
    let (int, frac) = match body.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (body, None),
    };
    let digits = |d: &str| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit());
    if !digits(int) || (int.len() > 1 && int.starts_with('0')) || frac.is_some_and(|f| !digits(f)) {
        return Err(bad());
    }
    let frac = frac.unwrap_or("");
    let mut exponent = -i64::try_from(frac.len()).map_err(|_| bad())?;
    if let Some(exp) = exp {
        let (sign, n) = match (exp.strip_prefix('-'), exp.strip_prefix('+')) {
            (Some(n), _) => (-1, n),
            (_, Some(n)) => (1, n),
            _ => (1, exp),
        };
        if !digits(n) {
            return Err(bad());
        }
        let n: i32 = n
            .parse()
            .map_err(|_| not_read("has an exponent rippled does not read"))?;
        exponent += sign * i64::from(n);
    }
    Ok(Parts {
        negative,
        int,
        frac,
        exponent,
    })
}

/// `rippled`'s `partsFromString`: sign, mantissa and exponent of a decimal
/// string ([`split`]'s form), with the digits fitting a `u64`.
#[cfg(feature = "std")]
pub(crate) fn parse(s: &str) -> Result<(bool, u64, i64), CoreError> {
    let parts = split(s)?;
    let mut mantissa: u64 = 0;
    for d in parts.digits() {
        mantissa = mantissa
            .checked_mul(10)
            .and_then(|m| m.checked_add(d))
            .ok_or_else(|| {
                CoreError::InvalidAmount(format!("{s:?} has more digits than rippled reads"))
            })?;
    }
    Ok((parts.negative, mantissa, parts.exponent))
}
