//! Account reserves and spendable balance, in whole drops (exact integers).
//!
//! The reserves come from the network: `server_state` and the `ledger`
//! stream give `reserve_base` / `reserve_inc` in drops; `server_info` gives
//! `reserve_base_xrp` / `reserve_inc_xrp` in XRP, which [`reserve_xrp_to_drops`]
//! converts exactly from the JSON number's text.

use crate::CoreError;

/// The reserve an account must hold, in drops: the base reserve plus one
/// owner reserve per object it owns. Saturates instead of overflowing.
///
/// ```
/// use xrpl_core::utils::reserve::total_reserve_drops;
///
/// // Mainnet's reserves (FeeSettings): 1 XRP, and 0.2 XRP per object owned.
/// assert_eq!(total_reserve_drops(1_000_000, 200_000, 3), 1_600_000);
/// assert_eq!(total_reserve_drops(1_000_000, 200_000, 0), 1_000_000);
/// ```
pub fn total_reserve_drops(
    reserve_base_drops: u64,
    reserve_inc_drops: u64,
    owner_count: u32,
) -> u64 {
    reserve_base_drops.saturating_add(reserve_inc_drops.saturating_mul(u64::from(owner_count)))
}

/// What the account can spend, in drops: its balance above the reserve, or
/// 0 when the balance is below it.
///
/// ```
/// use xrpl_core::utils::reserve::available_balance_drops;
///
/// // 10 XRP, owning 3 objects, at mainnet's reserves (1 XRP + 0.2 XRP each).
/// assert_eq!(available_balance_drops(10_000_000, 1_000_000, 200_000, 3), 8_400_000);
/// assert_eq!(available_balance_drops(1_000_000, 1_000_000, 200_000, 1), 0);
/// ```
pub fn available_balance_drops(
    balance_drops: u64,
    reserve_base_drops: u64,
    reserve_inc_drops: u64,
    owner_count: u32,
) -> u64 {
    balance_drops.saturating_sub(total_reserve_drops(
        reserve_base_drops,
        reserve_inc_drops,
        owner_count,
    ))
}

/// `server_info`'s `reserve_base_xrp` or `reserve_inc_xrp` (XRP, such as
/// `0.2`, as the typed response holds it) in drops, exactly: the number is
/// read through its shortest round-trip decimal text, which is the text the
/// server wrote, never by multiplying a float (`1.001` is 1,001,000 drops,
/// not 1,000,999). A negative, non-finite or fractional-drop value is an
/// error.
///
/// ```
/// use xrpl_core::utils::reserve::reserve_xrp_to_drops;
///
/// assert_eq!(reserve_xrp_to_drops(0.2)?, 200_000);
/// assert_eq!(reserve_xrp_to_drops(1.001)?, 1_001_000); // not 1_000_999
/// assert!(reserve_xrp_to_drops(-1.0).is_err());
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::InvalidAmount`] for such a value, or one above 100 billion XRP.
#[cfg(feature = "std")]
pub fn reserve_xrp_to_drops(xrp: f64) -> Result<u64, CoreError> {
    if !xrp.is_finite() {
        return Err(CoreError::InvalidAmount(format!("a reserve of {xrp} XRP")));
    }
    super::xrp::xrp_to_drops(&format!("{xrp}"))
}

/// Parse an XRP balance string (in drops, as returned by rippled) to u64.
///
/// ```
/// use xrpl_core::utils::reserve::parse_drops;
///
/// assert_eq!(parse_drops("17229982")?, 17_229_982);
/// assert!(parse_drops("17.2").is_err()); // drops are whole
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::InvalidAmount`] if `balance` is not a whole number that fits a
/// `u64`.
pub fn parse_drops(balance: &str) -> Result<u64, CoreError> {
    balance
        .parse::<u64>()
        .map_err(|_| CoreError::InvalidAmount(format!("invalid drops string: {balance}")))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn reserves_are_exact_drops() {
        assert_eq!(total_reserve_drops(1_000_000, 200_000, 3), 1_600_000);
        assert_eq!(
            available_balance_drops(10_000_000, 1_000_000, 200_000, 3),
            8_400_000
        );
        assert_eq!(available_balance_drops(1_000_000, 1_000_000, 200_000, 1), 0);
        assert_eq!(total_reserve_drops(u64::MAX, u64::MAX, u32::MAX), u64::MAX);
    }

    #[cfg(feature = "std")]
    #[test]
    #[allow(
        clippy::cast_possible_truncation,
        reason = "shows the float truncation this avoids"
    )]
    fn server_info_reserves_convert_exactly() {
        assert_eq!(reserve_xrp_to_drops(0.2).unwrap(), 200_000);
        assert_eq!(reserve_xrp_to_drops(1.001).unwrap(), 1_001_000);
        assert_eq!(
            (1.001f64 * 1_000_000.0) as u64,
            1_000_999,
            "why floats are not multiplied"
        );
        assert_eq!(reserve_xrp_to_drops(1.0).unwrap(), 1_000_000);
        assert!(reserve_xrp_to_drops(-1.0).is_err());
        assert!(reserve_xrp_to_drops(0.0000001).is_err());
        assert!(reserve_xrp_to_drops(f64::NAN).is_err());
    }
}
