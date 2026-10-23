//! `TransferRate` (AccountSet) and `QualityIn` / `QualityOut` (TrustSet) ↔
//! decimals and percentages. Both count billionths: a TransferRate of
//! 1_010_000_000 charges 1%, a quality of 990_000_000 values a token at 0.99.

use super::decimal::Decimal;
use crate::CoreError;

const BILLION: u64 = 1_000_000_000;

/// `"1%"` → 1_010_000_000. `"0%"` is 0 (no fee). The fee must be 0–100%.
///
/// # Errors
///
/// [`CoreError::ValidationError`] if `percent` does not end with `%`, is not a
/// number, is outside 0–100%, or is finer than a billionth of the amount.
pub fn percent_to_transfer_rate(percent: &str) -> Result<u32, CoreError> {
    decimal_to_transfer_rate(&percent_to_decimal(percent)?.to_string())
}

/// `"0.01"` → 1_010_000_000. `"0"` is 0 (no fee). The fee must be 0–1.
///
/// # Errors
///
/// [`CoreError::ValidationError`] if `decimal` is not a number, is outside 0–1,
/// or is finer than a billionth.
pub fn decimal_to_transfer_rate(decimal: &str) -> Result<u32, CoreError> {
    let fee = parse(decimal)?;
    if fee.is_negative() || fee > Decimal::from_u64(1) {
        return Err(invalid("Decimal value must be between 0 and 1.00."));
    }
    if fee.is_zero() {
        return Ok(0);
    }
    billionths(&fee.add(&Decimal::from_u64(1)))
}

/// 1_010_000_000 → `"0.01"`; 0 (no fee) → `"0"`. A rate rippled would refuse
/// (between 1 and 999,999,999, or over 2,000,000,000) is an error.
///
/// # Errors
///
/// [`CoreError::ValidationError`] for such a rate.
pub fn transfer_rate_to_decimal(rate: u32) -> Result<String, CoreError> {
    if rate == 0 {
        return Ok("0".into());
    }
    if u64::from(rate) < BILLION || u64::from(rate) > 2 * BILLION {
        return Err(invalid(
            "TransferRate must be 0 or 1,000,000,000–2,000,000,000.",
        ));
    }
    Ok(Decimal::from_u64(u64::from(rate) - BILLION)
        .scaled(-9)
        .to_string())
}

/// `"101%"` → 1_010_000_000; `"100%"` → 0 (rippled's default, 1:1).
///
/// # Errors
///
/// [`CoreError::ValidationError`] if `percent` does not end with `%`, is not a
/// number, is negative, is finer than a billionth, or is too large for a `u32`
/// quality (above 429.4967295%).
pub fn percent_to_quality(percent: &str) -> Result<u32, CoreError> {
    decimal_to_quality(&percent_to_decimal(percent)?.to_string())
}

/// `"1.01"` → 1_010_000_000; `"1"` → 0 (rippled's default, 1:1).
///
/// # Errors
///
/// [`CoreError::ValidationError`] if `decimal` is not a number, is negative, is
/// finer than a billionth, or is too large for a `u32` quality (above
/// 4.294967295).
pub fn decimal_to_quality(decimal: &str) -> Result<u32, CoreError> {
    let q = parse(decimal)?;
    if q.is_negative() {
        return Err(invalid("Quality must not be negative."));
    }
    let b = billionths(&q)?;
    Ok(if u64::from(b) == BILLION { 0 } else { b })
}

/// 1_010_000_000 → `"1.01"`; 0 → `"1"`.
pub fn quality_to_decimal(quality: u32) -> String {
    if quality == 0 {
        return "1".into();
    }
    Decimal::from_u64(u64::from(quality)).scaled(-9).to_string()
}

fn percent_to_decimal(percent: &str) -> Result<Decimal, CoreError> {
    let number = percent
        .strip_suffix('%')
        .ok_or_else(|| invalid(&format!("Value {percent} must end with %")))?;
    Ok(parse(number)?.scaled(-2))
}

fn parse(s: &str) -> Result<Decimal, CoreError> {
    Decimal::parse(s).map_err(|_| invalid("Value is not a number"))
}

/// `d` × 1e9 as a whole number that fits a `u32`.
fn billionths(d: &Decimal) -> Result<u32, CoreError> {
    let b = d.scaled(9);
    if b.has_fraction() {
        return Err(invalid("Decimal exceeds maximum precision."));
    }
    b.to_u64()
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| invalid("Value is too large."))
}

fn invalid(msg: &str) -> CoreError {
    CoreError::ValidationError(msg.into())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // xrpl.js transferRate.test.ts and quality.test.ts.
    #[test]
    fn transfer_rate_matches_xrpl_js() {
        assert_eq!(percent_to_transfer_rate("1%").unwrap(), 1_010_000_000);
        assert_eq!(decimal_to_transfer_rate(".01").unwrap(), 1_010_000_000);
        assert_eq!(transfer_rate_to_decimal(1_010_000_000).unwrap(), "0.01");
        assert_eq!(percent_to_transfer_rate("0%").unwrap(), 0);
        assert_eq!(decimal_to_transfer_rate("0").unwrap(), 0);
        assert_eq!(transfer_rate_to_decimal(0).unwrap(), "0");
        assert_eq!(percent_to_transfer_rate("100%").unwrap(), 2_000_000_000);
        for bad in ["-1%", "101%", ".0000000000000011221%", "1", "3dsadflk%"] {
            assert!(percent_to_transfer_rate(bad).is_err(), "{bad}");
        }
        for bad in ["-.01", "1.01", ".000000000000000011221", "x"] {
            assert!(decimal_to_transfer_rate(bad).is_err(), "{bad}");
        }
        assert!(transfer_rate_to_decimal(999_999_999).is_err());
        assert!(transfer_rate_to_decimal(2_000_000_001).is_err());
    }

    #[test]
    fn quality_matches_xrpl_js() {
        assert_eq!(percent_to_quality("101%").unwrap(), 1_010_000_000);
        assert_eq!(decimal_to_quality("1.01").unwrap(), 1_010_000_000);
        assert_eq!(quality_to_decimal(1_010_000_000), "1.01");
        assert_eq!(percent_to_quality("99%").unwrap(), 990_000_000);
        assert_eq!(decimal_to_quality(".99").unwrap(), 990_000_000);
        assert_eq!(quality_to_decimal(990_000_000), "0.99");
        assert_eq!(percent_to_quality("100%").unwrap(), 0);
        assert_eq!(decimal_to_quality("1.00").unwrap(), 0);
        assert_eq!(quality_to_decimal(0), "1");
        for bad in [".0000000000000011221%", "3dsadflk%"] {
            assert!(percent_to_quality(bad).is_err(), "{bad}");
        }
        for bad in [".000000000000000011221", "3dsadflk%", "-1", "5"] {
            assert!(decimal_to_quality(bad).is_err(), "{bad}");
        }
    }
}
