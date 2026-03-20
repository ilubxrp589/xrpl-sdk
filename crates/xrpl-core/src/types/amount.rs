use super::{AccountId, Currency};
use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// Maximum XRP in drops (100 billion XRP = 10^17 drops).
pub const MAX_XRP_DROPS: u64 = 100_000_000_000_000_000;

/// IOU mantissa minimum (normalized).
pub const IOU_MANTISSA_MIN: u64 = 1_000_000_000_000_000; // 10^15
/// IOU mantissa maximum (normalized, exclusive).
pub const IOU_MANTISSA_MAX: u64 = 10_000_000_000_000_000; // 10^16

/// IOU exponent range.
pub const IOU_EXPONENT_MIN: i8 = -96;
pub const IOU_EXPONENT_MAX: i8 = 80;

/// Custom floating-point representation for IOU amounts.
/// Mantissa is normalized to [10^15, 10^16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IouAmount {
    pub mantissa: u64,
    pub exponent: i8,
    pub is_negative: bool,
}

impl IouAmount {
    /// Create a zero IOU amount.
    pub fn zero() -> Self {
        Self {
            mantissa: 0,
            exponent: 0,
            is_negative: false,
        }
    }

    /// Returns true if this is zero.
    pub fn is_zero(&self) -> bool {
        self.mantissa == 0
    }

    /// Create and normalize an IOU amount from mantissa and exponent.
    pub fn new(mantissa: i64, exponent: i8) -> Result<Self, crate::CoreError> {
        if mantissa == 0 {
            return Ok(Self::zero());
        }

        let is_negative = mantissa < 0;
        let mut m = mantissa.unsigned_abs();
        let mut e = exponent as i16;

        // Normalize: shift mantissa until 10^15 <= m < 10^16
        while m < IOU_MANTISSA_MIN && e > IOU_EXPONENT_MIN as i16 {
            m *= 10;
            e -= 1;
        }
        while m >= IOU_MANTISSA_MAX && e < IOU_EXPONENT_MAX as i16 {
            m /= 10;
            e += 1;
        }

        if !(IOU_MANTISSA_MIN..IOU_MANTISSA_MAX).contains(&m) {
            return Err(crate::CoreError::InvalidAmount(
                "mantissa out of normalized range after adjustment".to_string(),
            ));
        }
        if e < IOU_EXPONENT_MIN as i16 || e > IOU_EXPONENT_MAX as i16 {
            return Err(crate::CoreError::InvalidAmount(format!(
                "exponent {e} out of range [{}, {}]",
                IOU_EXPONENT_MIN, IOU_EXPONENT_MAX
            )));
        }

        Ok(Self {
            mantissa: m,
            exponent: e as i8,
            is_negative,
        })
    }

    /// Parse from decimal string (e.g., "1.5", "-0.001", "1000000", "1.5e10").
    pub fn from_decimal(s: &str) -> Result<Self, crate::CoreError> {
        if s == "0" || s == "0.0" || s == "-0" {
            return Ok(Self::zero());
        }

        let (is_negative, s) = if let Some(stripped) = s.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, s)
        };

        // Handle scientific notation (e.g., "9999999999999999e80", "1.5e10")
        let (base, sci_exp) = if let Some((b, e)) = s.split_once('e').or_else(|| s.split_once('E'))
        {
            let exp: i16 = e
                .parse()
                .map_err(|_| crate::CoreError::InvalidAmount(format!("invalid exponent: {e}")))?;
            (b, exp)
        } else {
            (s, 0i16)
        };

        let (integer_part, decimal_part) = match base.split_once('.') {
            Some((i, d)) => (i, d),
            None => (base, ""),
        };

        // Combine into a single mantissa string
        let combined = format!("{integer_part}{decimal_part}");
        let combined = combined.trim_start_matches('0');
        if combined.is_empty() {
            return Ok(Self::zero());
        }

        let mantissa: u64 = combined.parse().map_err(|e: core::num::ParseIntError| {
            crate::CoreError::InvalidAmount(e.to_string())
        })?;
        let base_exp = -(decimal_part.len() as i16) + sci_exp;

        // Clamp to i8 range for IouAmount::new
        if !(-128..=127).contains(&base_exp) {
            return Err(crate::CoreError::InvalidAmount(format!(
                "exponent {base_exp} out of i8 range"
            )));
        }

        let signed_mantissa = if is_negative {
            -(mantissa as i64)
        } else {
            mantissa as i64
        };

        Self::new(signed_mantissa, base_exp as i8)
    }

    /// Convert to decimal string representation.
    pub fn to_decimal(&self) -> String {
        if self.is_zero() {
            return "0".to_string();
        }

        let sign = if self.is_negative { "-" } else { "" };
        let mut s = self.mantissa.to_string();
        let exp = self.exponent as i32;

        // The effective number is mantissa * 10^exponent
        // mantissa has 16 digits (since normalized to [10^15, 10^16))
        let digits = s.len() as i32;
        let decimal_point_pos = digits + exp;

        if decimal_point_pos <= 0 {
            // Need leading zeros: 0.000...mantissa
            let zeros = (-decimal_point_pos) as usize;
            s = format!("0.{}{}", "0".repeat(zeros), s);
        } else if decimal_point_pos >= digits {
            // No decimal point needed, or trailing zeros
            let extra_zeros = (decimal_point_pos - digits) as usize;
            s = format!("{}{}", s, "0".repeat(extra_zeros));
        } else {
            // Insert decimal point within the digits
            let pos = decimal_point_pos as usize;
            s.insert(pos, '.');
        }

        // Trim trailing zeros after decimal point
        if s.contains('.') {
            s = s.trim_end_matches('0').to_string();
            s = s.trim_end_matches('.').to_string();
        }

        format!("{sign}{s}")
    }
}

impl fmt::Display for IouAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_decimal())
    }
}

impl Serialize for IouAmount {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_decimal())
    }
}

impl<'de> Deserialize<'de> for IouAmount {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_decimal(&s).map_err(serde::de::Error::custom)
    }
}

/// XRPL amount — either XRP (drops) or IOU (value + currency + issuer).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Amount {
    /// XRP amount in drops. 1 XRP = 1,000,000 drops.
    Xrp(u64),
    /// IOU amount with value, currency, and issuer.
    Iou {
        value: IouAmount,
        currency: Currency,
        issuer: AccountId,
    },
}

impl Amount {
    /// Create an XRP amount from drops.
    pub fn xrp(drops: u64) -> Result<Self, crate::CoreError> {
        if drops > MAX_XRP_DROPS {
            return Err(crate::CoreError::InvalidAmount(format!(
                "XRP drops {drops} exceeds maximum {MAX_XRP_DROPS}"
            )));
        }
        Ok(Amount::Xrp(drops))
    }

    /// Returns true if this is an XRP amount.
    pub fn is_xrp(&self) -> bool {
        matches!(self, Amount::Xrp(_))
    }
}

impl Serialize for Amount {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Amount::Xrp(drops) => serializer.serialize_str(&drops.to_string()),
            Amount::Iou {
                value,
                currency,
                issuer,
            } => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("value", &value.to_decimal())?;
                map.serialize_entry("currency", currency)?;
                map.serialize_entry("issuer", issuer)?;
                map.end()
            }
        }
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde_json::Value;
        let v = Value::deserialize(deserializer)?;

        match v {
            Value::String(s) => {
                let drops: u64 = s.parse().map_err(serde::de::Error::custom)?;
                Ok(Amount::Xrp(drops))
            }
            Value::Object(map) => {
                let value_str = map
                    .get("value")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| serde::de::Error::missing_field("value"))?;
                let currency: Currency = serde_json::from_value(
                    map.get("currency")
                        .cloned()
                        .ok_or_else(|| serde::de::Error::missing_field("currency"))?,
                )
                .map_err(serde::de::Error::custom)?;
                let issuer: AccountId = serde_json::from_value(
                    map.get("issuer")
                        .cloned()
                        .ok_or_else(|| serde::de::Error::missing_field("issuer"))?,
                )
                .map_err(serde::de::Error::custom)?;
                let value = IouAmount::from_decimal(value_str).map_err(serde::de::Error::custom)?;
                Ok(Amount::Iou {
                    value,
                    currency,
                    issuer,
                })
            }
            _ => Err(serde::de::Error::custom("expected string or object")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iou_normalize_1_5() {
        let iou = IouAmount::from_decimal("1.5").unwrap();
        assert!(!iou.is_negative);
        // 1.5 = 15 * 10^-1 → normalized to 1500000000000000 * 10^-15
        assert!(iou.mantissa >= IOU_MANTISSA_MIN);
        assert!(iou.mantissa < IOU_MANTISSA_MAX);
        assert_eq!(iou.to_decimal(), "1.5");
    }

    #[test]
    fn iou_zero() {
        let zero = IouAmount::zero();
        assert!(zero.is_zero());
        assert_eq!(zero.to_decimal(), "0");
    }

    #[test]
    fn iou_negative() {
        let neg = IouAmount::from_decimal("-42").unwrap();
        assert!(neg.is_negative);
        assert_eq!(neg.to_decimal(), "-42");
    }

    #[test]
    fn iou_small_decimal() {
        let small = IouAmount::from_decimal("0.001").unwrap();
        assert_eq!(small.to_decimal(), "0.001");
    }

    #[test]
    fn iou_large_number() {
        let large = IouAmount::from_decimal("1000000").unwrap();
        assert_eq!(large.to_decimal(), "1000000");
    }

    #[test]
    fn amount_xrp_serde() {
        let amt = Amount::Xrp(1_000_000);
        let json = serde_json::to_string(&amt).unwrap();
        assert_eq!(json, "\"1000000\"");
        let back: Amount = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Amount::Xrp(1_000_000));
    }

    #[test]
    fn amount_iou_serde() {
        let amt = Amount::Iou {
            value: IouAmount::from_decimal("1.5").unwrap(),
            currency: Currency::from_ascii("USD").unwrap(),
            issuer: AccountId::from_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
        };
        let json = serde_json::to_string(&amt).unwrap();
        let back: Amount = serde_json::from_str(&json).unwrap();
        assert_eq!(amt, back);
    }

    #[test]
    fn amount_xrp_max_valid() {
        assert!(Amount::xrp(MAX_XRP_DROPS).is_ok());
    }

    #[test]
    fn amount_xrp_exceeds_max() {
        assert!(Amount::xrp(MAX_XRP_DROPS + 1).is_err());
    }
}
