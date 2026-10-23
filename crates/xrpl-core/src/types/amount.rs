use super::{AccountId, Currency, Hash192};
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
    /// The significant digits.
    pub mantissa: u64,
    /// The power of ten.
    pub exponent: i8,
    /// Whether it is negative.
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

    /// Create and normalize an IOU amount from mantissa and exponent, as
    /// `rippled` does (see [`IouAmount::from_decimal`] for the rules).
    pub fn new(mantissa: i64, exponent: i8) -> Result<Self, crate::CoreError> {
        Self::normalize(mantissa < 0, mantissa.unsigned_abs(), i64::from(exponent))
    }

    /// `rippled`'s normal form of ±`mantissa`×10^`exponent` (its `Number`
    /// normalization, then `IOUAmount`'s range): a mantissa in
    /// [10^15, 10^16), digits past the sixteenth rounded to nearest (a tie
    /// goes to the even digit, unless a later digit is non-zero); an exponent
    /// below -96 is zero, one above 80 an error.
    fn normalize(negative: bool, mantissa: u64, exponent: i64) -> Result<Self, crate::CoreError> {
        if mantissa == 0 {
            return Ok(Self::zero());
        }
        let (mut m, mut e) = (mantissa, exponent);
        while m < IOU_MANTISSA_MIN {
            m *= 10;
            e -= 1;
        }
        // The most significant dropped digit, and whether any after it was not 0.
        let (mut dropped, mut sticky) = (0, false);
        while m >= IOU_MANTISSA_MAX {
            sticky |= dropped != 0;
            dropped = m % 10;
            m /= 10;
            e += 1;
        }
        if dropped > 5 || (dropped == 5 && (sticky || m % 2 == 1)) {
            m += 1;
            if m == IOU_MANTISSA_MAX {
                m /= 10;
                e += 1;
            }
        }
        if e < i64::from(IOU_EXPONENT_MIN) {
            return Ok(Self::zero());
        }
        let exponent = i8::try_from(e)
            .ok()
            .filter(|e| *e <= IOU_EXPONENT_MAX)
            .ok_or_else(|| {
                crate::CoreError::InvalidAmount(format!(
                    "{}{mantissa}e{exponent} is larger than a token amount can be (9999999999999999e80)",
                    if negative { "-" } else { "" }
                ))
            })?;
        Ok(Self {
            mantissa: m,
            exponent,
            is_negative: negative,
        })
    }

    /// Parse a token value written in `rippled`'s JSON number syntax
    /// (`amountFromString`): `[-+]?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][-+]?[0-9]+)?`,
    /// so `"1.5"`, `"-0.001"`, `"1e-3"` and `"+5"`, but not `""`, `".5"`,
    /// `"5."` or `"007"`. Past sixteen significant digits the value rounds
    /// to nearest, as `rippled` rounds it (a tie goes to the even digit
    /// unless a later digit is non-zero); below 10^-81 it is zero; above
    /// 9999999999999999e80 it is an error.
    ///
    /// Any number of digits is read exactly, rounded by the same rule.
    /// (`rippled`'s JSON reader takes at most a `u64` of digits; the SDK
    /// writes binary, so a longer value reaches the ledger as written.)
    pub fn from_decimal(s: &str) -> Result<Self, crate::CoreError> {
        let parts = crate::number_text::split(s)?;
        // Sixteen significant digits; the rest only move the exponent and
        // decide the rounding.
        let (mut m, mut e) = (0u64, parts.exponent);
        let (mut dropped, mut sticky) = (None, false);
        for d in parts.digits() {
            if m < IOU_MANTISSA_MIN {
                m = m * 10 + d;
            } else {
                match dropped {
                    None => dropped = Some(d),
                    Some(_) => sticky |= d != 0,
                }
                e += 1;
            }
        }
        if let Some(d) = dropped {
            if d > 5 || (d == 5 && (sticky || m % 2 == 1)) {
                m += 1;
            }
        }
        Self::normalize(parts.negative, m, e)
    }

    /// Convert to decimal string representation.
    pub fn to_decimal(&self) -> String {
        if self.is_zero() {
            return "0".to_string();
        }

        let sign = if self.is_negative { "-" } else { "" };
        let mut s = self.mantissa.to_string();
        let exp = self.exponent as i32;

        // rippled STAmount::getText: extreme exponents print as
        // "<mantissa>e<exponent>" (STAmount.cpp:696).
        if exp != 0 && !(-25..=-5).contains(&exp) {
            return format!("{sign}{s}e{exp}");
        }

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

/// XRPL amount — XRP (drops), an IOU (value + currency + issuer), or an MPT
/// (integer value + MPTokenIssuanceID).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Amount {
    /// XRP amount in drops. 1 XRP = 1,000,000 drops.
    Xrp(u64),
    /// IOU amount with value, currency, and issuer.
    Iou {
        /// The value.
        value: IouAmount,
        /// The currency code.
        currency: Currency,
        /// The issuer.
        issuer: AccountId,
    },
    /// MPT amount: rippled's JSON is `{"mpt_issuance_id": <hex>, "value": "<integer>"}`.
    Mpt {
        /// The amount, in the issuance's smallest unit.
        value: i64,
        /// The MPTokenIssuanceID.
        issuance_id: Hash192,
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

    /// A token amount: `value` in decimal, a currency code and its issuer.
    /// Codes longer than three characters (`"RLUSD"`) are written as hex,
    /// as the ledger stores them; 40 hex digits are taken as they are.
    ///
    /// ```
    /// use xrpl_core::types::Amount;
    /// let usd = Amount::token("10.5", "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
    /// assert_eq!(serde_json::to_value(&usd).unwrap()["value"], "10.5");
    /// let rlusd = Amount::token("1", "RLUSD", "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De").unwrap();
    /// assert_eq!(serde_json::to_value(&rlusd).unwrap()["currency"], "524C555344000000000000000000000000000000");
    /// ```
    pub fn token(value: &str, currency: &str, issuer: &str) -> Result<Self, crate::CoreError> {
        let currency = if currency.len() == 40 && currency.bytes().all(|b| b.is_ascii_hexdigit()) {
            Currency::from_hex(currency)?
        } else if currency.len() == 3 {
            Currency::from_ascii(currency)?
        } else if !currency.is_empty() && currency.len() <= 20 && currency.is_ascii() {
            let mut bytes = [0u8; 20];
            for (slot, b) in bytes.iter_mut().zip(currency.bytes()) {
                *slot = b;
            }
            Currency::from_bytes(bytes)
        } else {
            return Err(crate::CoreError::InvalidCurrency(currency.to_string()));
        };
        Ok(Amount::Iou {
            value: IouAmount::from_decimal(value)?,
            currency,
            issuer: AccountId::from_address(issuer)?,
        })
    }

    /// An MPT amount: `value` in the issuance's smallest unit (an
    /// `AssetScale` of 2 makes 12345 mean 123.45), and its
    /// MPTokenIssuanceID (48 hex digits).
    pub fn mpt(value: i64, mpt_issuance_id: &str) -> Result<Self, crate::CoreError> {
        Ok(Amount::Mpt {
            value,
            issuance_id: Hash192::from_hex(mpt_issuance_id)?,
        })
    }
}

/// For people, not for the ledger: `25 XRP`, `50 USD.rIssuer…`,
/// `1.5 RLUSD.rIssuer…` (a hex code that spells text is shown as the text),
/// `12345 MPT:00000001…`. Use the `Serialize` form for `rippled`.
///
/// ```
/// use xrpl_core::types::Amount;
/// assert_eq!(Amount::Xrp(1_500_000).to_string(), "1.5 XRP");
/// let rlusd = Amount::token("10", "RLUSD", "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De").unwrap();
/// assert_eq!(rlusd.to_string(), "10 RLUSD.rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De");
/// ```
impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Amount::Xrp(drops) => {
                let (whole, frac) = (drops / 1_000_000, drops % 1_000_000);
                if frac == 0 {
                    write!(f, "{whole} XRP")
                } else {
                    let frac = format!("{frac:06}");
                    write!(f, "{whole}.{} XRP", frac.trim_end_matches('0'))
                }
            }
            Amount::Iou {
                value,
                currency,
                issuer,
            } => {
                let bytes = currency.to_bytes();
                let text_len = bytes.iter().take_while(|b| b.is_ascii_graphic()).count();
                let spelled = text_len >= 3
                    && bytes
                        .get(text_len..)
                        .is_some_and(|rest| rest.iter().all(|b| *b == 0))
                    && !matches!(currency, Currency::Standard(_));
                match (spelled, bytes.get(..text_len)) {
                    (true, Some(text)) => {
                        let text = core::str::from_utf8(text).unwrap_or("?");
                        write!(f, "{} {text}.{issuer}", value.to_decimal())
                    }
                    _ => write!(f, "{} {currency}.{issuer}", value.to_decimal()),
                }
            }
            Amount::Mpt { value, issuance_id } => write!(f, "{value} MPT:{issuance_id}"),
        }
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
            Amount::Mpt { value, issuance_id } => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("mpt_issuance_id", issuance_id)?;
                map.serialize_entry("value", &value.to_string())?;
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
            Value::Object(map) if map.contains_key("mpt_issuance_id") => {
                let value = map
                    .get("value")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| serde::de::Error::missing_field("value"))?
                    .parse::<i64>()
                    .map_err(serde::de::Error::custom)?;
                let issuance_id: Hash192 =
                    serde_json::from_value(map.get("mpt_issuance_id").cloned().unwrap_or_default())
                        .map_err(serde::de::Error::custom)?;
                Ok(Amount::Mpt { value, issuance_id })
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
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn amounts_round_trip_rippled_json() {
        // The three shapes rippled's JSON uses (STAmount::getJson).
        for j in [
            serde_json::json!("1000000"),
            serde_json::json!({"value": "1.5e-20", "currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"}),
            serde_json::json!({"mpt_issuance_id": "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47", "value": "100"}),
        ] {
            let a: Amount = serde_json::from_value(j.clone()).unwrap();
            let back = serde_json::to_value(&a).unwrap();
            let again: Amount = serde_json::from_value(back).unwrap();
            assert_eq!(a, again, "{j}");
        }
        let mpt: Amount = serde_json::from_value(serde_json::json!(
            {"mpt_issuance_id": "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47", "value": "100"}
        ))
        .unwrap();
        assert!(matches!(mpt, Amount::Mpt { value: 100, .. }));
    }

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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod getext_tests {
    use super::*;

    // rippled STAmount::getText parity on extreme exponents + lossless re-parse.
    #[test]
    fn scientific_spelling_round_trips() {
        for (m, e, expect) in [
            (1_000_000_000_000_000i64, 12i8, "1000000000000000e12"),
            (1_000_000_000_000_000, -96, "1000000000000000e-96"),
            (9_999_999_999_999_999, 80, "9999999999999999e80"),
            (1_234_567_890_123_456, -4, "1234567890123456e-4"),
            (-1_000_000_000_000_000, 12, "-1000000000000000e12"),
            (1_234_560_000_000_000, -13, "123.456"),
            (1_000_000_000_000_000, 0, "1000000000000000e0"),
        ] {
            let a = IouAmount::new(m, e).unwrap();
            let s = a.to_decimal();
            let b = IouAmount::from_decimal(&s).unwrap();
            assert_eq!(
                (b.mantissa, b.exponent, b.is_negative),
                (a.mantissa, a.exponent, a.is_negative),
                "{s}"
            );
            if e != 0 {
                assert_eq!(s, expect);
            }
        }
        // plain giant expansion parses losslessly too
        let c = IouAmount::from_decimal("1000000000000000000000000000").unwrap();
        assert_eq!((c.mantissa, c.exponent), (1_000_000_000_000_000, 12));
    }

    /// Token values as rippled 3.4.1 reads them from JSON: `simulate` of a
    /// Payment from genesis (applies nothing), mantissa and exponent read
    /// back from the bytes rippled wrote (scripts/probe_iou.py, 2026-10-23).
    /// `None` = zero.
    #[test]
    fn reads_token_values_as_rippled_does() {
        type Read = Option<(bool, u64, i8)>;
        let cases: &[(&str, Read)] = &[
            ("1.5", Some((false, 1_500_000_000_000_000, -15))),
            ("+5", Some((false, 5_000_000_000_000_000, -15))),
            (
                "1.99999999999999999",
                Some((false, 2_000_000_000_000_000, -15)),
            ),
            ("12345678901234565", Some((false, 1_234_567_890_123_456, 1))),
            ("12345678901234575", Some((false, 1_234_567_890_123_458, 1))),
            (
                "1234567890123456501",
                Some((false, 1_234_567_890_123_457, 3)),
            ),
            (
                "1234567890123456500",
                Some((false, 1_234_567_890_123_456, 3)),
            ),
            ("-12345678901234565", Some((true, 1_234_567_890_123_456, 1))),
            ("-12345678901234575", Some((true, 1_234_567_890_123_458, 1))),
            (
                "-1234567890123456501",
                Some((true, 1_234_567_890_123_457, 3)),
            ),
            ("99999999999999995", Some((false, 1_000_000_000_000_000, 2))),
            ("99999999999999994", Some((false, 9_999_999_999_999_999, 1))),
            (
                "1234567890123456789",
                Some((false, 1_234_567_890_123_457, 3)),
            ),
            (
                "9223372036854775807",
                Some((false, 9_223_372_036_854_776, 3)),
            ),
            (
                "1000000000000000000",
                Some((false, 1_000_000_000_000_000, 3)),
            ),
            (
                "9999999999999999e80",
                Some((false, 9_999_999_999_999_999, 80)),
            ),
            ("1e95", Some((false, 1_000_000_000_000_000, 80))),
            ("1e81", Some((false, 1_000_000_000_000_000, 66))),
            (
                "1000000000000000e-96",
                Some((false, 1_000_000_000_000_000, -96)),
            ),
            ("1e-81", Some((false, 1_000_000_000_000_000, -96))),
            ("1e-96", None),
            ("1e-110", None),
            ("1.5e-32768", None),
            ("0", None),
            ("-0", None),
            ("0.0", None),
        ];
        for (text, want) in cases {
            let got = IouAmount::from_decimal(text).unwrap_or_else(|e| panic!("{text}: {e}"));
            match want {
                None => assert!(got.is_zero(), "{text}: {got:?}"),
                Some((negative, mantissa, exponent)) => assert_eq!(
                    (got.is_negative, got.mantissa, got.exponent),
                    (*negative, *mantissa, *exponent),
                    "{text}"
                ),
            }
        }
        // rippled refuses these ("Field 'tx_json.Amount' has invalid data").
        for text in [
            "",
            "-",
            ".",
            "e5",
            "1e",
            "1.2.3",
            ".5",
            "5.",
            "007",
            "1e96",
            "99999999999999995e80",
            "9999999999999999e81",
            "1,5",
            " 1",
            "0x10",
            "1e2147483648",
        ] {
            assert!(IouAmount::from_decimal(text).is_err(), "{text:?} was read");
        }
        // More digits than rippled's JSON reader takes: read as
        // written, rounded by the same rule.
        let deliberate: &[(&str, (bool, u64, i8))] = &[
            ("9999999999999999999", (false, 1_000_000_000_000_000, 4)),
            ("-9999999999999999999", (true, 1_000_000_000_000_000, 4)),
            ("18446744073709551615", (false, 1_844_674_407_370_955, 4)),
            ("18446744073709551616", (false, 1_844_674_407_370_955, 4)),
            ("10000000000000000000", (false, 1_000_000_000_000_000, 4)),
            ("9223372036854775808", (false, 9_223_372_036_854_776, 3)),
            ("-9223372036854775808", (true, 9_223_372_036_854_776, 3)),
            ("0.9999999999999999999", (false, 1_000_000_000_000_000, -15)),
            (
                "1000000000000000000000000000",
                (false, 1_000_000_000_000_000, 12),
            ),
            (
                "1234567890123456500000000000000001",
                (false, 1_234_567_890_123_457, 18),
            ),
        ];
        for (text, want) in deliberate {
            let got = IouAmount::from_decimal(text).unwrap_or_else(|e| panic!("{text}: {e}"));
            assert_eq!(
                (got.is_negative, got.mantissa, got.exponent),
                *want,
                "{text}"
            );
        }
    }
}
