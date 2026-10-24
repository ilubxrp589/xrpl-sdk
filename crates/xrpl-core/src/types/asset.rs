//! [`Asset`]: XRP, a token or an MPT, without an amount — one side of an
//! order book, an AMM pool, a vault or a bridge.

use super::{AccountId, Amount, Currency, Hash192};
use crate::CoreError;
use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

/// An asset: XRP, a token (currency and issuer) or an MPT (its issuance).
/// `rippled` writes it `{"currency": "XRP"}`, `{"currency": "USD", "issuer":
/// "r…"}` or `{"mpt_issuance_id": "…"}`.
///
/// Assets are ordered as `rippled` orders them (an AMM pool's two assets are
/// hashed smaller first): MPTs before XRP and tokens, tokens by currency and
/// then issuer, MPTs by issuance ID.
///
/// ```
/// use xrpl_core::types::{Amount, Asset};
///
/// let usd = Asset::token("USD", "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")?;
/// assert_eq!(serde_json::to_value(&usd).unwrap()["currency"], "USD");
/// assert_eq!(Amount::token("10", "USD", "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")?.asset(), usd);
/// assert_eq!(Asset::xrp().to_string(), "XRP");
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Asset {
    /// XRP.
    Xrp,
    /// A token.
    Token {
        /// Its currency code.
        currency: Currency,
        /// Its issuer.
        issuer: AccountId,
    },
    /// A multi-purpose token.
    Mpt {
        /// Its `MPTokenIssuanceID`.
        issuance_id: Hash192,
    },
}

impl Asset {
    /// XRP.
    ///
    /// ```
    /// use xrpl_core::types::Asset;
    ///
    /// assert!(Asset::xrp().is_xrp());
    /// assert_eq!(serde_json::to_value(Asset::xrp()).unwrap(), serde_json::json!({"currency": "XRP"}));
    /// ```
    pub fn xrp() -> Self {
        Asset::Xrp
    }

    /// A token: `currency` as people write it (`"USD"`, `"RLUSD"`, 40 hex
    /// digits) and its issuer's address.
    ///
    /// ```
    /// use xrpl_core::types::Asset;
    ///
    /// let usd = Asset::token("USD", "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")?;
    /// assert_eq!(usd.to_string(), "USD.rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    /// let rlusd = Asset::token("RLUSD", "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")?;
    /// assert_eq!(serde_json::to_value(&rlusd).unwrap()["currency"], "524C555344000000000000000000000000000000");
    /// assert!(Asset::token("XRP", "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe").is_err());
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidCurrency`] for a code that does not parse, or
    /// XRP's (use [`Asset::xrp`]); [`CoreError::InvalidAddress`] (or
    /// `InvalidBase58`, `InvalidChecksum`) for an issuer that is not a
    /// classic address.
    pub fn token(currency: &str, issuer: &str) -> Result<Self, CoreError> {
        let currency = Currency::from_code(currency)?;
        if currency.is_xrp() {
            return Err(CoreError::InvalidCurrency(
                "a token cannot use XRP's currency code; use Asset::xrp".into(),
            ));
        }
        Ok(Asset::Token {
            currency,
            issuer: AccountId::from_address(issuer)?,
        })
    }

    /// An MPT, by its `MPTokenIssuanceID` (48 hex digits).
    ///
    /// ```
    /// use xrpl_core::types::Asset;
    ///
    /// let mpt = Asset::mpt("000004C463C52827307480341125DA0577DEFC38405B0E3E")?;
    /// assert_eq!(mpt.to_string(), "MPT:000004C463C52827307480341125DA0577DEFC38405B0E3E");
    /// assert!(mpt < Asset::xrp()); // MPTs sort first
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidHex`] for anything but 48 hex digits.
    pub fn mpt(issuance_id: &str) -> Result<Self, CoreError> {
        Ok(Asset::Mpt {
            issuance_id: Hash192::from_hex(issuance_id)?,
        })
    }

    /// Whether this is XRP.
    ///
    /// ```
    /// use xrpl_core::types::Asset;
    ///
    /// assert!(Asset::xrp().is_xrp());
    /// assert!(!Asset::token("USD", "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")?.is_xrp());
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn is_xrp(&self) -> bool {
        matches!(self, Asset::Xrp)
    }

    /// The order `rippled` gives assets: MPTs first (by ID), then XRP and
    /// tokens by currency and then issuer (XRP's currency is all zeros).
    fn sort_key(&self) -> (u8, [u8; 24], [u8; 20], [u8; 20]) {
        match self {
            Asset::Mpt { issuance_id } => (0, issuance_id.0, [0; 20], [0; 20]),
            Asset::Xrp => (1, [0; 24], [0; 20], [0; 20]),
            Asset::Token { currency, issuer } => (1, [0; 24], currency.to_bytes(), issuer.0),
        }
    }
}

impl PartialOrd for Asset {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Asset {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}

impl fmt::Display for Asset {
    /// `XRP`, `USD.r…` (the currency, a dot, the issuer) or `MPT:<id>`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Asset::Xrp => f.write_str("XRP"),
            Asset::Token { currency, issuer } => write!(f, "{currency}.{issuer}"),
            Asset::Mpt { issuance_id } => write!(f, "MPT:{issuance_id}"),
        }
    }
}

impl core::str::FromStr for Asset {
    type Err = CoreError;

    /// What [`Display`](fmt::Display) writes: `XRP` (any case), a token as
    /// `CURRENCY.rIssuer` (`USD.r…`, `RLUSD.r…`, 40 hex digits), or
    /// `MPT:<48 hex digits>`.
    fn from_str(s: &str) -> Result<Self, CoreError> {
        if s.eq_ignore_ascii_case("XRP") {
            return Ok(Asset::Xrp);
        }
        if let Some(id) = s.strip_prefix("MPT:") {
            return Asset::mpt(id);
        }
        match s.split_once('.') {
            Some((currency, issuer)) => Asset::token(currency, issuer),
            None => Err(CoreError::InvalidCurrency(format!(
                "an asset is XRP, CURRENCY.rIssuer or MPT:<id>, not {s:?}"
            ))),
        }
    }
}

impl Amount {
    /// The amount's asset (XRP, its token or its MPT), without the value.
    ///
    /// ```
    /// use xrpl_core::types::{Amount, Asset};
    ///
    /// let usd = Amount::token("10", "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
    /// assert_eq!(usd.asset(), Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?);
    /// assert_eq!(Amount::xrp(5)?.asset(), Asset::xrp());
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn asset(&self) -> Asset {
        match self {
            Amount::Xrp(_) => Asset::Xrp,
            Amount::Iou {
                currency, issuer, ..
            } => Asset::Token {
                currency: *currency,
                issuer: *issuer,
            },
            Amount::Mpt { issuance_id, .. } => Asset::Mpt {
                issuance_id: *issuance_id,
            },
        }
    }
}

impl Serialize for Asset {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        match self {
            Asset::Xrp => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("currency", "XRP")?;
                map.end()
            }
            Asset::Token { currency, issuer } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("currency", currency)?;
                map.serialize_entry("issuer", issuer)?;
                map.end()
            }
            Asset::Mpt { issuance_id } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("mpt_issuance_id", issuance_id)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Asset {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            currency: Option<String>,
            issuer: Option<String>,
            mpt_issuance_id: Option<String>,
        }
        let raw = Raw::deserialize(deserializer)?;
        let bad = |m: &str| serde::de::Error::custom(format!("an asset: {m}"));
        match (raw.currency, raw.issuer, raw.mpt_issuance_id) {
            (None, None, Some(id)) => Asset::mpt(&id).map_err(serde::de::Error::custom),
            (Some(c), None, None) => {
                let currency = Currency::parse(&c).map_err(serde::de::Error::custom)?;
                if currency.is_xrp() {
                    Ok(Asset::Xrp)
                } else {
                    Err(bad("a token needs its issuer"))
                }
            }
            (Some(c), Some(i), None) => {
                let currency = Currency::parse(&c).map_err(serde::de::Error::custom)?;
                if currency.is_xrp() {
                    return Err(bad("XRP has no issuer"));
                }
                Ok(Asset::Token {
                    currency,
                    issuer: AccountId::from_address(&i).map_err(serde::de::Error::custom)?,
                })
            }
            _ => Err(bad(
                "it is {\"currency\": \"XRP\"}, {\"currency\", \"issuer\"} or {\"mpt_issuance_id\"}",
            )),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    const ISSUER: &str = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
    const MPT: &str = "000004C463C52827307480341125DA0577DEFC38405B0E3E";

    #[test]
    fn json_round_trips() {
        for (asset, json) in [
            (Asset::xrp(), serde_json::json!({"currency": "XRP"})),
            (
                Asset::token("USD", ISSUER).unwrap(),
                serde_json::json!({"currency": "USD", "issuer": ISSUER}),
            ),
            (
                Asset::mpt(MPT).unwrap(),
                serde_json::json!({"mpt_issuance_id": MPT}),
            ),
        ] {
            assert_eq!(serde_json::to_value(&asset).unwrap(), json);
            assert_eq!(serde_json::from_value::<Asset>(json).unwrap(), asset);
        }
    }

    #[test]
    fn malformed_assets_are_refused() {
        for bad in [
            serde_json::json!({"currency": "USD"}),
            serde_json::json!({"currency": "XRP", "issuer": ISSUER}),
            serde_json::json!({"currency": "USD", "issuer": ISSUER, "mpt_issuance_id": MPT}),
            serde_json::json!({"issuer": ISSUER}),
            serde_json::json!({"currency": "USD", "issuer": ISSUER, "value": "1"}),
        ] {
            assert!(
                serde_json::from_value::<Asset>(bad.clone()).is_err(),
                "{bad}"
            );
        }
        assert!(Asset::token("XRP", ISSUER).is_err());
        assert!(Asset::mpt("00").is_err());
    }

    #[test]
    fn text_round_trips() {
        for asset in [
            Asset::xrp(),
            Asset::token("USD", ISSUER).unwrap(),
            Asset::token("RLUSD", ISSUER).unwrap(),
            Asset::mpt(MPT).unwrap(),
        ] {
            assert_eq!(asset.to_string().parse::<Asset>().unwrap(), asset);
        }
        assert_eq!("xrp".parse::<Asset>().unwrap(), Asset::xrp());
        assert!("USD".parse::<Asset>().is_err());
    }

    #[test]
    fn rippled_order() {
        let mpt = Asset::mpt(MPT).unwrap();
        let usd = Asset::token("USD", ISSUER).unwrap();
        assert!(mpt < Asset::xrp() && Asset::xrp() < usd);
        assert!(Asset::token("EUR", ISSUER).unwrap() < usd);
    }
}
