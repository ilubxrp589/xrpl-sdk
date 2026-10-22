use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// XRPL currency identifier.
/// - Standard: 3-character ASCII (e.g., "USD", "EUR", "BTC")
/// - NonStandard: 20-byte hex (used for NFT/AMM pools)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    /// 3-char ISO currency code stored as ASCII bytes.
    Standard([u8; 3]),
    /// 20-byte non-standard currency code.
    NonStandard([u8; 20]),
}

/// rippled `detail::isoCharSet` (UintTypes.cpp:39-43): the characters a
/// three-letter currency code may use — letters, digits and the symbols
/// `<>(){}[]|?!@#$%^&*`. Mainnet carries codes such as `W&B`; rejecting the
/// symbols made the validator unable to re-encode such a trust line
/// (#106699694), and would render the line's code as hex where rippled
/// renders the ISO string.
pub fn is_iso_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b"<>(){}[]|?!@#$%^&*".contains(&b)
}

impl Currency {
    /// The special XRP currency (20 zero bytes). Only used in PathStep, never in Amount.
    pub fn xrp() -> Self {
        Currency::NonStandard([0u8; 20])
    }

    /// Create a standard 3-char currency from a string.
    pub fn from_ascii(s: &str) -> Result<Self, crate::CoreError> {
        if s.len() != 3 {
            return Err(crate::CoreError::InvalidCurrency(format!(
                "standard currency must be 3 chars, got '{s}'"
            )));
        }
        if !s.bytes().all(is_iso_char) {
            return Err(crate::CoreError::InvalidCurrency(format!(
                "standard currency must use rippled's ISO character set, got '{s}'"
            )));
        }
        let mut arr = [0u8; 3];
        arr.copy_from_slice(s.as_bytes());
        Ok(Currency::Standard(arr))
    }

    /// Create from 20-byte hex string.
    pub fn from_hex(s: &str) -> Result<Self, crate::CoreError> {
        let bytes = hex::decode(s).map_err(|e| crate::CoreError::InvalidHex(e.to_string()))?;
        if bytes.len() != 20 {
            return Err(crate::CoreError::InvalidCurrency(format!(
                "non-standard currency must be 20 bytes, got {}",
                bytes.len()
            )));
        }
        let mut arr = [0u8; 20];
        arr.copy_from_slice(&bytes);
        Ok(Currency::NonStandard(arr))
    }

    /// Encode to 20-byte wire format.
    pub fn to_bytes(&self) -> [u8; 20] {
        match self {
            Currency::Standard(code) => {
                let mut buf = [0u8; 20];
                buf[12] = code[0];
                buf[13] = code[1];
                buf[14] = code[2];
                buf
            }
            Currency::NonStandard(bytes) => *bytes,
        }
    }

    /// Decode from 20-byte wire format.
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        // All zeros = XRP currency marker
        if bytes == [0u8; 20] {
            return Currency::NonStandard(bytes);
        }

        // Check standard format: bytes 0-11 = 0, bytes 15-19 = 0, bytes 12-14 = ASCII
        // rippled `to_string(Currency)` (UintTypes.cpp:60-80): the three bytes
        // must all be ISO characters, and the ISO spelling of the system
        // currency ("XRP" in the code position) is never rendered as ISO —
        // it falls through to hex.
        let is_standard = bytes[..12].iter().all(|&b| b == 0)
            && bytes[15..].iter().all(|&b| b == 0)
            && bytes[12..15].iter().all(|&b| is_iso_char(b))
            && &bytes[12..15] != b"XRP";

        if is_standard {
            Currency::Standard([bytes[12], bytes[13], bytes[14]])
        } else {
            Currency::NonStandard(bytes)
        }
    }

    /// Returns true if this is the XRP currency marker.
    pub fn is_xrp(&self) -> bool {
        matches!(self, Currency::NonStandard(bytes) if *bytes == [0u8; 20])
    }
}

impl fmt::Debug for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::Standard(code) => {
                let s = core::str::from_utf8(code).unwrap_or("???");
                write!(f, "Currency::Standard({s})")
            }
            Currency::NonStandard(bytes) => {
                write!(f, "Currency::NonStandard({})", hex::encode_upper(bytes))
            }
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::Standard(code) => {
                let s = core::str::from_utf8(code).unwrap_or("???");
                write!(f, "{s}")
            }
            Currency::NonStandard(bytes) if *bytes == [0u8; 20] => {
                write!(f, "XRP")
            }
            Currency::NonStandard(bytes) => {
                write!(f, "{}", hex::encode_upper(bytes))
            }
        }
    }
}

impl Serialize for Currency {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Currency::Standard(code) => {
                let s = core::str::from_utf8(code).map_err(serde::ser::Error::custom)?;
                serializer.serialize_str(s)
            }
            Currency::NonStandard(bytes) if *bytes == [0u8; 20] => serializer.serialize_str("XRP"),
            Currency::NonStandard(bytes) => serializer.serialize_str(&hex::encode_upper(bytes)),
        }
    }
}

impl<'de> Deserialize<'de> for Currency {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        if s == "XRP" {
            return Ok(Currency::xrp());
        }
        if s.len() == 3 {
            Currency::from_ascii(&s).map_err(serde::de::Error::custom)
        } else if s.len() == 40 {
            Currency::from_hex(&s).map_err(serde::de::Error::custom)
        } else {
            Err(serde::de::Error::custom(format!("invalid currency: '{s}'")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_currency_roundtrip() {
        let usd = Currency::from_ascii("USD").unwrap();
        let bytes = usd.to_bytes();
        let back = Currency::from_bytes(bytes);
        assert_eq!(usd, back);
    }

    #[test]
    fn standard_currency_wire_format() {
        let usd = Currency::from_ascii("USD").unwrap();
        let bytes = usd.to_bytes();
        // First 12 bytes = 0, bytes 12-14 = ASCII "USD", last 5 bytes = 0
        assert!(bytes[..12].iter().all(|&b| b == 0));
        assert_eq!(&bytes[12..15], b"USD");
        assert!(bytes[15..].iter().all(|&b| b == 0));
    }

    #[test]
    fn xrp_currency() {
        let xrp = Currency::xrp();
        assert!(xrp.is_xrp());
        assert_eq!(xrp.to_string(), "XRP");
    }

    #[test]
    fn non_standard_currency() {
        let hex_str = "0158415500000000C1F76FF6ECB0BAC600000000";
        let c = Currency::from_hex(hex_str).unwrap();
        assert_eq!(c.to_string(), hex_str);
    }

    #[test]
    fn currency_serde_roundtrip() {
        let usd = Currency::from_ascii("USD").unwrap();
        let json = serde_json::to_string(&usd).unwrap();
        assert_eq!(json, "\"USD\"");
        let back: Currency = serde_json::from_str(&json).unwrap();
        assert_eq!(usd, back);
    }

    #[test]
    fn iso_symbol_currency_roundtrip() {
        // rippled's isoCharSet admits symbols; "W&B" is live on mainnet.
        let c = Currency::from_ascii("W&B").unwrap();
        let bytes = c.to_bytes();
        assert_eq!(&bytes[12..15], b"W&B");
        assert_eq!(Currency::from_bytes(bytes), c);
        assert_eq!(c.to_string(), "W&B");
        assert_eq!(serde_json::to_string(&c).unwrap(), "\"W&B\"");
        for good in ["<>(", "){}", "[]|", "?!@", "#$%", "^&*", "a1Z"] {
            assert!(Currency::from_ascii(good).is_ok(), "{good}");
        }
        for bad in [
            "A B", "A.B", "A,B", "A-B", "A_B", "A/B", "A\\B", "A~B", "A`B", "A'B", "A\"B", "A;B",
            "A:B", "A+B", "A=B",
        ] {
            assert!(Currency::from_ascii(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn iso_xrp_spelling_renders_as_hex() {
        // rippled never renders the system currency's ISO spelling as ISO.
        let mut bytes = [0u8; 20];
        bytes[12..15].copy_from_slice(b"XRP");
        let c = Currency::from_bytes(bytes);
        assert_eq!(c, Currency::NonStandard(bytes));
        assert_eq!(c.to_string(), "0000000000000000000000005852500000000000");
        // A non-ISO byte in the code position is hex too.
        bytes[12..15].copy_from_slice(b"A B");
        assert_eq!(Currency::from_bytes(bytes), Currency::NonStandard(bytes));
    }

    #[test]
    fn invalid_currency_length() {
        assert!(Currency::from_ascii("US").is_err());
        assert!(Currency::from_ascii("USDT").is_err());
    }
}
