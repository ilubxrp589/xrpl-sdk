//! Order-book arithmetic. [`liquidity_at_price`] is exact (decimal); the
//! price helpers ([`offer_quality_xrp`], [`midpoint_price`],
//! [`spread_percent`], [`amount_to_f64`]) are `f64` approximations for
//! display and estimates. Never sign an amount computed with them.
//!
//! Units: `book_offers`' `quality` is TakerPays per TakerGets in the units
//! the ledger stores, so an XRP side counts in **drops**, not XRP.

use super::decimal::Decimal;
use crate::CoreError;

/// A quality (TakerPays per TakerGets) from two amounts given as whole
/// numbers in the same units (drops, say), as an approximate `f64`; `None`
/// when `taker_gets_drops` is 0. It is the plain ratio: no offer has XRP on
/// both sides.
///
/// ```
/// use xrpl_core::utils::dex::offer_quality_xrp;
///
/// assert_eq!(offer_quality_xrp(1_500_000, 1_000_000), Some(1.5));
/// assert_eq!(offer_quality_xrp(5, 0), None);
/// ```
pub fn offer_quality_xrp(taker_pays_drops: u64, taker_gets_drops: u64) -> Option<f64> {
    (taker_gets_drops != 0).then(|| taker_pays_drops as f64 / taker_gets_drops as f64)
}

/// An amount as an approximate `f64`, for display: XRP (a string of drops)
/// in XRP, a token in its value. Not for amounts you sign.
///
/// ```
/// use serde_json::json;
/// use xrpl_core::utils::dex::amount_to_f64;
///
/// assert_eq!(amount_to_f64(&json!("1500000"))?, 1.5); // drops, shown in XRP
/// let usd = json!({"currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "value": "0.25"});
/// assert_eq!(amount_to_f64(&usd)?, 0.25);
/// assert!(amount_to_f64(&json!(5)).is_err());
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::InvalidAmount`] for a string that is not a whole number of
/// drops, an object without a `value` that parses as a number, or any other
/// JSON value.
pub fn amount_to_f64(amount: &serde_json::Value) -> Result<f64, CoreError> {
    match amount {
        serde_json::Value::String(s) => {
            let drops: u64 = s
                .parse()
                .map_err(|_| CoreError::InvalidAmount(format!("invalid drops: {s}")))?;
            Ok(drops as f64 / 1_000_000.0)
        }
        serde_json::Value::Object(obj) => {
            let value = obj
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or_else(|| CoreError::InvalidAmount("IOU missing value".into()))?;
            value
                .parse::<f64>()
                .map_err(|_| CoreError::InvalidAmount(format!("invalid IOU value: {value}")))
        }
        _ => Err(CoreError::InvalidAmount(
            "amount must be string or object".into(),
        )),
    }
}

/// The midpoint of the best bid and ask (approximate).
///
/// ```
/// use xrpl_core::utils::dex::midpoint_price;
///
/// assert_eq!(midpoint_price(0.5, 0.75), 0.625);
/// ```
pub fn midpoint_price(best_bid: f64, best_ask: f64) -> f64 {
    (best_bid + best_ask) / 2.0
}

/// The spread as a percentage of the best bid (approximate); `None` when
/// the bid is not positive.
///
/// ```
/// use xrpl_core::utils::dex::spread_percent;
///
/// assert_eq!(spread_percent(0.5, 0.75), Some(50.0));
/// assert_eq!(spread_percent(0.0, 0.75), None);
/// ```
pub fn spread_percent(best_bid: f64, best_ask: f64) -> Option<f64> {
    (best_bid > 0.0).then(|| ((best_ask - best_bid) / best_bid) * 100.0)
}

/// An amount in display units, exactly: XRP (a string of drops) in XRP, a
/// token in its value.
fn amount_decimal(amount: &serde_json::Value) -> Result<Decimal, CoreError> {
    match amount {
        serde_json::Value::String(drops) => Ok(Decimal::parse(drops)?.scaled(-6)),
        serde_json::Value::Object(obj) => obj
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CoreError::InvalidAmount("token amount missing value".into()))
            .and_then(Decimal::parse),
        _ => Err(CoreError::InvalidAmount(
            "amount must be string or object".into(),
        )),
    }
}

/// How much of the book's TakerGets side is offered at a quality of
/// `max_quality` or better, exactly, as a decimal string in display units
/// (XRP, or the token's value). `offers` are `book_offers` entries;
/// `max_quality` is in the same units as their `quality` (TakerPays per
/// TakerGets, drops for an XRP side). An offer counts what its owner can
/// fund (`taker_gets_funded`, when the server sent it), not what it names.
///
/// ```
/// use serde_json::json;
/// use xrpl_core::utils::dex::liquidity_at_price;
///
/// let usd = |value: &str| json!({"currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "value": value});
/// // `book_offers` entries giving USD for XRP; quality is drops per USD.
/// let offers = [
///     json!({"quality": "500000", "taker_gets": usd("0.1")}),
///     json!({"quality": "500000", "taker_gets": usd("10"), "taker_gets_funded": usd("0.2")}),
///     json!({"quality": "600000", "taker_gets": usd("99")}),
/// ];
/// assert_eq!(liquidity_at_price(&offers, "550000")?, "0.3");
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::InvalidAmount`] if `max_quality` or an offer's `quality` is not
/// a decimal number, or a counted offer's `taker_gets` (or `taker_gets_funded`)
/// is not an amount with a decimal value.
pub fn liquidity_at_price(
    offers: &[serde_json::Value],
    max_quality: &str,
) -> Result<String, CoreError> {
    let limit = Decimal::parse(max_quality)?;
    let mut total = Decimal::zero();
    for offer in offers {
        let Some(quality) = offer.get("quality").and_then(|q| q.as_str()) else {
            continue;
        };
        if Decimal::parse(quality)? > limit {
            continue;
        }
        if let Some(gets) = offer
            .get("taker_gets_funded")
            .or_else(|| offer.get("taker_gets"))
        {
            total = total.add(&amount_decimal(gets)?);
        }
    }
    Ok(total.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_amount_to_f64_xrp() {
        let amt = json!("1000000");
        assert!((amount_to_f64(&amt).unwrap() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_amount_to_f64_iou() {
        let amt = json!({"value": "1.5", "currency": "USD", "issuer": "rSomeIssuer"});
        assert!((amount_to_f64(&amt).unwrap() - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_midpoint_price() {
        assert!((midpoint_price(1.0, 1.1) - 1.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_spread_percent() {
        assert!((spread_percent(1.0, 1.1).unwrap() - 10.0).abs() < 1e-10);
        assert_eq!(spread_percent(0.0, 1.0), None);
        assert_eq!(offer_quality_xrp(5, 0), None);
    }

    /// Exact, in display units, counting what each owner can fund, with the
    /// limit in the book's own quality units (drops per token here).
    #[test]
    fn liquidity_is_exact_and_funded() {
        let offers = [
            json!({"quality": "500000", "taker_gets": {"currency": "USD", "issuer": "r", "value": "0.1"}}),
            json!({"quality": "500000", "taker_gets": {"currency": "USD", "issuer": "r", "value": "10"},
                   "taker_gets_funded": {"currency": "USD", "issuer": "r", "value": "0.2"}}),
            json!({"quality": "600000", "taker_gets": {"currency": "USD", "issuer": "r", "value": "99"}}),
        ];
        assert_eq!(liquidity_at_price(&offers, "550000").unwrap(), "0.3");
        let xrp = [json!({"quality": "0.5", "taker_gets": "1500000"})];
        assert_eq!(liquidity_at_price(&xrp, "1").unwrap(), "1.5");
    }
}
