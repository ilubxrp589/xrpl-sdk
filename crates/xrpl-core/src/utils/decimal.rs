//! Exact decimal arithmetic for amounts written as strings (XRP, drops, token
//! values, rates). No floating point, so nothing is rounded away.

use crate::CoreError;
use std::cmp::Ordering;
use std::fmt;

/// Exponents beyond this are refused when parsing: token amounts span
/// 1e-96..1e80, and a huge exponent would make alignment allocate without bound.
const MAX_EXPONENT: i32 = 400;

/// `digits` × 10^`exp`, with the sign kept apart. `digits` (most significant
/// first) has no leading or trailing zeros; zero is an empty `digits`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Decimal {
    negative: bool,
    digits: Vec<u8>,
    exp: i32,
}

impl Decimal {
    pub(crate) fn zero() -> Self {
        Decimal {
            negative: false,
            digits: Vec::new(),
            exp: 0,
        }
    }

    pub(crate) fn from_u64(n: u64) -> Self {
        Self::normalized(false, n.to_string().bytes().map(|b| b - b'0').collect(), 0)
    }

    /// Parse `[+-]digits[.digits][e[+-]digits]`.
    pub(crate) fn parse(s: &str) -> Result<Self, CoreError> {
        let bad = || CoreError::InvalidAmount(format!("not a decimal number: {s:?}"));
        let (negative, rest) = match s.as_bytes().first() {
            Some(b'-') => (true, s.get(1..).unwrap_or_default()),
            Some(b'+') => (false, s.get(1..).unwrap_or_default()),
            _ => (false, s),
        };
        let (mantissa, exp_part) = match rest.find(['e', 'E']) {
            Some(i) => (rest.get(..i).unwrap_or_default(), rest.get(i + 1..)),
            None => (rest, None),
        };
        let (int_part, frac_part) = match mantissa.split_once('.') {
            Some((i, f)) => (i, f),
            None => (mantissa, ""),
        };
        if int_part.is_empty() && frac_part.is_empty() {
            return Err(bad());
        }
        if !int_part
            .bytes()
            .chain(frac_part.bytes())
            .all(|b| b.is_ascii_digit())
        {
            return Err(bad());
        }
        let mut exp: i32 = match exp_part {
            None => 0,
            Some(e) => {
                let digits = e.strip_prefix(['+', '-']).unwrap_or(e);
                if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(bad());
                }
                let value: i32 = e.parse().map_err(|_| bad())?;
                if value.abs() > MAX_EXPONENT {
                    return Err(CoreError::InvalidAmount(format!(
                        "exponent out of range: {s:?}"
                    )));
                }
                value
            }
        };
        let frac_len = i32::try_from(frac_part.len()).map_err(|_| bad())?;
        exp -= frac_len;
        let digits = int_part
            .bytes()
            .chain(frac_part.bytes())
            .map(|b| b - b'0')
            .collect();
        Ok(Self::normalized(negative, digits, exp))
    }

    fn normalized(negative: bool, mut digits: Vec<u8>, mut exp: i32) -> Self {
        let leading = digits.iter().take_while(|&&d| d == 0).count();
        digits.drain(..leading);
        while digits.last() == Some(&0) {
            digits.pop();
            exp += 1;
        }
        if digits.is_empty() {
            return Self::zero();
        }
        Decimal {
            negative,
            digits,
            exp,
        }
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }

    pub(crate) fn is_negative(&self) -> bool {
        self.negative && !self.is_zero()
    }

    pub(crate) fn negated(&self) -> Self {
        let mut d = self.clone();
        d.negative = !d.negative && !d.is_zero();
        d
    }

    /// `self` × 10^`pow10`.
    pub(crate) fn scaled(&self, pow10: i32) -> Self {
        if self.is_zero() {
            return Self::zero();
        }
        Decimal {
            exp: self.exp + pow10,
            ..self.clone()
        }
    }

    /// Whether this has digits after the decimal point.
    pub(crate) fn has_fraction(&self) -> bool {
        self.exp < 0
    }

    pub(crate) fn add(&self, other: &Self) -> Self {
        if self.is_zero() {
            return other.clone();
        }
        if other.is_zero() {
            return self.clone();
        }
        let exp = self.exp.min(other.exp);
        let a = self.aligned(exp);
        let b = other.aligned(exp);
        if self.negative == other.negative {
            return Self::normalized(self.negative, add_magnitudes(&a, &b), exp);
        }
        match cmp_magnitudes(&a, &b) {
            Ordering::Equal => Self::zero(),
            Ordering::Greater => Self::normalized(self.negative, sub_magnitudes(&a, &b), exp),
            Ordering::Less => Self::normalized(other.negative, sub_magnitudes(&b, &a), exp),
        }
    }

    pub(crate) fn sub(&self, other: &Self) -> Self {
        self.add(&other.negated())
    }

    /// The digits at exponent `exp` (≤ `self.exp`).
    fn aligned(&self, exp: i32) -> Vec<u8> {
        let mut digits = self.digits.clone();
        digits.extend(std::iter::repeat_n(0, (self.exp - exp) as usize));
        digits
    }

    /// The value as a `u64`, if it is a non-negative whole number that fits.
    pub(crate) fn to_u64(&self) -> Option<u64> {
        if self.is_negative() || self.has_fraction() {
            return None;
        }
        let mut n: u64 = 0;
        for &d in &self.digits {
            n = n.checked_mul(10)?.checked_add(u64::from(d))?;
        }
        for _ in 0..self.exp {
            n = n.checked_mul(10)?;
        }
        Some(n)
    }
}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Decimal {
    fn cmp(&self, other: &Self) -> Ordering {
        let diff = self.sub(other);
        if diff.is_zero() {
            Ordering::Equal
        } else if diff.negative {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

/// Plain decimal notation, never an exponent: `-0.012`, `100`, `0.0000001`.
impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            return f.write_str("0");
        }
        if self.negative {
            f.write_str("-")?;
        }
        let digits: String = self.digits.iter().map(|d| char::from(b'0' + d)).collect();
        if self.exp >= 0 {
            f.write_str(&digits)?;
            return f.write_str(&"0".repeat(usize::try_from(self.exp).map_err(|_| fmt::Error)?));
        }
        let point = digits.len() as i64 + i64::from(self.exp);
        if point > 0 {
            let (int, frac) = digits.split_at(usize::try_from(point).map_err(|_| fmt::Error)?);
            write!(f, "{int}.{frac}")
        } else {
            let zeros = usize::try_from(-point).map_err(|_| fmt::Error)?;
            write!(f, "0.{}{digits}", "0".repeat(zeros))
        }
    }
}

fn cmp_magnitudes(a: &[u8], b: &[u8]) -> Ordering {
    let a = strip_leading_zeros(a);
    let b = strip_leading_zeros(b);
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

fn strip_leading_zeros(d: &[u8]) -> &[u8] {
    let n = d.iter().take_while(|&&x| x == 0).count();
    d.get(n..).unwrap_or_default()
}

fn add_magnitudes(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
    let mut carry = 0u8;
    let mut ia = a.iter().rev();
    let mut ib = b.iter().rev();
    loop {
        let (x, y) = (ia.next(), ib.next());
        if x.is_none() && y.is_none() {
            break;
        }
        let s = x.copied().unwrap_or(0) + y.copied().unwrap_or(0) + carry;
        out.push(s % 10);
        carry = s / 10;
    }
    if carry > 0 {
        out.push(carry);
    }
    out.reverse();
    out
}

/// `a` − `b`, for `a` ≥ `b`.
fn sub_magnitudes(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(a.len());
    let mut borrow = 0i8;
    let mut ib = b.iter().rev();
    for &x in a.iter().rev() {
        let mut d = x as i8 - ib.next().copied().unwrap_or(0) as i8 - borrow;
        borrow = if d < 0 {
            d += 10;
            1
        } else {
            0
        };
        out.push(d as u8);
    }
    out.reverse();
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal {
        Decimal::parse(s).unwrap()
    }

    #[test]
    fn parses_and_prints_plain_decimals() {
        for (input, printed) in [
            ("0", "0"),
            ("-0", "0"),
            ("100", "100"),
            ("100.000", "100"),
            ("-100.012", "-100.012"),
            (".5", "0.5"),
            ("5.", "5"),
            ("1e-7", "0.0000001"),
            ("1.5E3", "1500"),
            (
                "9999999999999999e80",
                &format!("9999999999999999{}", "0".repeat(80)),
            ),
            ("+0.010", "0.01"),
        ] {
            assert_eq!(d(input).to_string(), printed, "{input}");
        }
        for bad in [
            "", ".", "-", "1e", "e5", "1.2.3", "1,5", "abc", "1e401", "0x10", " 1",
        ] {
            assert!(Decimal::parse(bad).is_err(), "{bad:?} parsed");
        }
    }

    #[test]
    fn adds_and_subtracts_exactly() {
        assert_eq!(d("0.1").add(&d("0.2")).to_string(), "0.3");
        assert_eq!(d("1").sub(&d("100.012")).to_string(), "-99.012");
        assert_eq!(d("-5").sub(&d("-5")).to_string(), "0");
        assert_eq!(
            d("1e80").sub(&d("1e-81")).to_string(),
            format!("{}.{}", "9".repeat(80), "9".repeat(81))
        );
        assert_eq!(d("999").add(&d("1")).to_string(), "1000");
        assert_eq!(d("-0.01").add(&d("0.025")).to_string(), "0.015");
    }

    #[test]
    fn scales_and_converts() {
        assert_eq!(d("1.5").scaled(6).to_u64(), Some(1_500_000));
        assert_eq!(d("0.0000001").scaled(6).to_u64(), None);
        assert_eq!(d("-1").to_u64(), None);
        assert_eq!(d("18446744073709551615").to_u64(), Some(u64::MAX));
        assert_eq!(d("18446744073709551616").to_u64(), None);
        assert_eq!(Decimal::from_u64(1_000_000).scaled(-6).to_string(), "1");
        assert!(d("-1") < d("0.5"));
    }
}
