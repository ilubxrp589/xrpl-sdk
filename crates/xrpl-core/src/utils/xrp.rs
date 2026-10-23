//! XRP ↔ drops, exactly (1 XRP = 1,000,000 drops).

use super::decimal::Decimal;
use crate::CoreError;

/// Drops in one XRP.
pub const DROPS_PER_XRP: u64 = 1_000_000;

/// All the XRP there will ever be (100 billion), in drops.
pub const MAX_DROPS: u64 = 100_000_000_000 * DROPS_PER_XRP;

/// Convert an XRP amount written in decimal (`"1.5"`, `"0.000001"`) to drops.
///
/// Refuses what is not an amount of XRP: a sign, an exponent, more than six
/// decimal places, or more than 100 billion XRP.
///
/// ```
/// use xrpl_core::utils::xrp::xrp_to_drops;
/// assert_eq!(xrp_to_drops("1.5").unwrap(), 1_500_000);
/// assert!(xrp_to_drops("0.0000001").is_err());
/// ```
///
/// # Errors
///
/// [`CoreError::InvalidAmount`] for any of those, or text that is not a number.
pub fn xrp_to_drops(xrp: &str) -> Result<u64, CoreError> {
    if !xrp.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return Err(CoreError::InvalidAmount(format!(
            "{xrp:?} is not an amount of XRP (digits and one decimal point only)"
        )));
    }
    let value = Decimal::parse(xrp)?;
    let drops = value.scaled(6);
    if drops.has_fraction() {
        return Err(CoreError::InvalidAmount(format!(
            "{xrp:?} has more than 6 decimal places"
        )));
    }
    match drops.to_u64() {
        Some(d) if d <= MAX_DROPS => Ok(d),
        _ => Err(CoreError::InvalidAmount(format!(
            "{xrp:?} is more than 100 billion XRP"
        ))),
    }
}

/// Convert drops to XRP, in plain decimal with no trailing zeros
/// (`1500000` → `"1.5"`).
///
/// ```
/// use xrpl_core::utils::xrp::drops_to_xrp;
/// assert_eq!(drops_to_xrp(1), "0.000001");
/// assert_eq!(drops_to_xrp(100_012_000), "100.012");
/// ```
pub fn drops_to_xrp(drops: u64) -> String {
    Decimal::from_u64(drops).scaled(-6).to_string()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // The cases of xrpl.js's xrpToDrops / dropsToXrp tests that fit unsigned drops.
    #[test]
    fn xrp_to_drops_matches_xrpl_js() {
        for (xrp, drops) in [
            ("2", 2_000_000),
            ("1", 1_000_000),
            ("0", 0),
            ("2.", 2_000_000),
            ("2.0", 2_000_000),
            (".2", 200_000),
            ("0.000001", 1),
            ("100000000000", MAX_DROPS),
            ("3.14159", 3_141_590),
        ] {
            assert_eq!(xrp_to_drops(xrp).unwrap(), drops, "{xrp}");
        }
        for bad in [
            "FOO",
            "1e-7",
            "0.0000001",
            "1.1.1",
            "-1",
            "100000000000.000001",
            "",
            ".",
        ] {
            assert!(xrp_to_drops(bad).is_err(), "{bad:?} converted");
        }
    }

    #[test]
    fn drops_to_xrp_matches_xrpl_js() {
        for (drops, xrp) in [
            (2_000_000, "2"),
            (1, "0.000001"),
            (0, "0"),
            (3_141_590, "3.14159"),
            (MAX_DROPS, "100000000000"),
            (u64::MAX, "18446744073709.551615"),
        ] {
            assert_eq!(drops_to_xrp(drops), xrp, "{drops}");
        }
    }
}
