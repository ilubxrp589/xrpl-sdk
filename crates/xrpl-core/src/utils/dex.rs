use crate::CoreError;

/// Compute the quality (exchange rate) of an offer.
/// quality = taker_pays_drops / taker_gets_drops
pub fn offer_quality_xrp(taker_pays_drops: u64, taker_gets_drops: u64) -> f64 {
    taker_pays_drops as f64 / taker_gets_drops as f64
}

/// Parse an XRPL amount Value to a human-readable f64.
/// Handles both XRP (string of drops) and IOU (object with "value" field).
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

/// Compute midpoint price from best bid and best ask.
pub fn midpoint_price(best_bid: f64, best_ask: f64) -> f64 {
    (best_bid + best_ask) / 2.0
}

/// Compute spread as a percentage.
pub fn spread_percent(best_bid: f64, best_ask: f64) -> f64 {
    if best_bid == 0.0 {
        return 0.0;
    }
    ((best_ask - best_bid) / best_bid) * 100.0
}

/// Compute total liquidity available up to a given price limit from a book_offers response.
/// `offers`: slice of BookOffer values from the book_offers response.
/// `price_limit`: maximum price (quality) to include.
/// Returns total base currency available within the price limit.
pub fn liquidity_at_price(
    offers: &[serde_json::Value],
    price_limit: f64,
) -> Result<f64, CoreError> {
    let mut total = 0.0f64;
    for offer in offers {
        if let Some(quality) = offer.get("quality").and_then(|q| q.as_str()) {
            let q: f64 = quality
                .parse()
                .map_err(|_| CoreError::CodecError(format!("invalid quality: {quality}")))?;
            if q <= price_limit {
                if let Some(taker_gets) = offer.get("taker_gets") {
                    total += amount_to_f64(taker_gets)?;
                }
            }
        }
    }
    Ok(total)
}

#[cfg(test)]
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
        assert!((spread_percent(1.0, 1.1) - 10.0).abs() < 1e-10);
    }
}
