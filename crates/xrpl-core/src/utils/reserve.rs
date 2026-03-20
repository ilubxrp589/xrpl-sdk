use crate::CoreError;

/// Base reserve in drops.
/// Pass the value from ServerInfo.validated_ledger.reserve_base_xrp.
pub fn base_reserve_drops(reserve_base_xrp: f64) -> u64 {
    (reserve_base_xrp * 1_000_000.0) as u64
}

/// Owner reserve per ledger object in drops.
pub fn owner_reserve_drops(reserve_inc_xrp: f64, owner_count: u32) -> u64 {
    (reserve_inc_xrp * 1_000_000.0) as u64 * owner_count as u64
}

/// Total minimum reserve in drops an account must hold.
pub fn total_reserve_drops(reserve_base_xrp: f64, reserve_inc_xrp: f64, owner_count: u32) -> u64 {
    base_reserve_drops(reserve_base_xrp) + owner_reserve_drops(reserve_inc_xrp, owner_count)
}

/// Maximum spendable balance in drops after accounting for reserves.
/// Returns 0 if balance is below reserve.
pub fn available_balance_drops(
    balance_drops: u64,
    reserve_base_xrp: f64,
    reserve_inc_xrp: f64,
    owner_count: u32,
) -> u64 {
    let reserve = total_reserve_drops(reserve_base_xrp, reserve_inc_xrp, owner_count);
    balance_drops.saturating_sub(reserve)
}

/// Parse an XRP balance string (in drops, as returned by rippled) to u64.
pub fn parse_drops(balance: &str) -> Result<u64, CoreError> {
    balance
        .parse::<u64>()
        .map_err(|_| CoreError::InvalidAmount(format!("invalid drops string: {balance}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_reserve_drops() {
        assert_eq!(base_reserve_drops(2.0), 2_000_000);
    }

    #[test]
    fn test_owner_reserve_drops() {
        assert_eq!(owner_reserve_drops(0.2, 5), 1_000_000);
    }

    #[test]
    fn test_total_reserve_drops() {
        assert_eq!(total_reserve_drops(2.0, 0.2, 3), 2_600_000);
    }

    #[test]
    fn test_available_balance_spendable() {
        assert_eq!(available_balance_drops(10_000_000, 2.0, 0.2, 3), 7_400_000);
    }

    #[test]
    fn test_available_balance_below_reserve() {
        assert_eq!(available_balance_drops(2_000_000, 2.0, 0.2, 0), 0);
    }
}
