use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// 128-bit hash (16 bytes). Used for EmailHash field. Written as 32 hex
/// digits, upper case.
///
/// ```
/// use xrpl_core::types::Hash128;
///
/// let hash: Hash128 = "aabbccddee0011223344556677889900".parse()?;
/// assert_eq!(hash.to_string(), "AABBCCDDEE0011223344556677889900");
/// assert_ne!(hash, Hash128::ZERO);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash128(pub [u8; 16]);

/// 160-bit hash (20 bytes): the `TakerPaysCurrency`, `TakerPaysIssuer`,
/// `TakerGetsCurrency` and `TakerGetsIssuer` fields of an order-book
/// `DirectoryNode`. Written as 40 hex digits, upper case.
///
/// ```
/// use xrpl_core::types::{Currency, Hash160};
///
/// // A mainnet order book's TakerPaysCurrency: the code "BGHOUSE".
/// let currency = Hash160::from_hex("4247484F55534500000000000000000000000000")?;
/// assert_eq!(currency.0, Currency::from_code("BGHOUSE")?.to_bytes());
/// assert_ne!(currency, Hash160::ZERO);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash160(pub [u8; 20]);

/// 192-bit hash (24 bytes). Used for MPTokenIssuanceID. Written as 48 hex
/// digits, upper case.
///
/// ```
/// use xrpl_core::types::Hash192;
///
/// let id: Hash192 = "000004C463C52827307480341125DA0577DEFC38405B0E3E".parse()?;
/// assert_eq!(id.as_bytes()[..4], 1220u32.to_be_bytes()); // the issuing sequence
/// assert_ne!(id, Hash192::ZERO);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash192(pub [u8; 24]);

/// 256-bit hash (32 bytes). Used for transaction hashes, ledger hashes, etc.
/// Written as 64 hex digits, upper case.
///
/// ```
/// use xrpl_core::types::Hash256;
///
/// // Mainnet ledger 107852951's hash.
/// let hash: Hash256 = "76768fffd98d55385887c1018eb1074a9f8745512e84ab09250ee76b54bedc93".parse()?;
/// assert_eq!(hash.to_string(), "76768FFFD98D55385887C1018EB1074A9F8745512E84AB09250EE76B54BEDC93");
/// assert!(Hash256::ZERO < hash);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash256(pub [u8; 32]);

// --- Hash128 ---

impl Hash128 {
    /// All zero bytes.
    pub const ZERO: Self = Self([0u8; 16]);

    /// From hex (either case).
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidHex`](crate::CoreError::InvalidHex) if `s` is not
    /// hex or not 16 bytes.
    ///
    /// ```
    /// use xrpl_core::types::Hash128;
    ///
    /// let hash = Hash128::from_hex("aabbccddee0011223344556677889900")?;
    /// assert_eq!(hash.to_string(), "AABBCCDDEE0011223344556677889900");
    /// assert!(Hash128::from_hex("AABB").is_err()); // not 16 bytes
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn from_hex(s: &str) -> Result<Self, crate::CoreError> {
        let bytes = hex::decode(s).map_err(|e| crate::CoreError::InvalidHex(e.to_string()))?;
        if bytes.len() != 16 {
            return Err(crate::CoreError::InvalidHex(format!(
                "expected 16 bytes, got {}",
                bytes.len()
            )));
        }
        let mut arr = [0u8; 16];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// The bytes.
    ///
    /// ```
    /// use xrpl_core::types::Hash128;
    ///
    /// let hash = Hash128::from_hex("AABBCCDDEE0011223344556677889900")?;
    /// assert_eq!(hash.as_bytes()[0], 0xAA);
    /// assert_eq!(Hash128(*hash.as_bytes()), hash);
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for Hash128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash128({})", hex::encode_upper(self.0))
    }
}

impl fmt::Display for Hash128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode_upper(self.0))
    }
}

impl Serialize for Hash128 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex::encode_upper(self.0))
    }
}

impl<'de> Deserialize<'de> for Hash128 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

// --- Hash160 ---

impl Hash160 {
    /// All zero bytes.
    pub const ZERO: Self = Self([0u8; 20]);

    /// From hex (either case).
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidHex`](crate::CoreError::InvalidHex) if `s` is not
    /// hex or not 20 bytes.
    ///
    /// ```
    /// use xrpl_core::types::Hash160;
    ///
    /// let hash = Hash160::from_hex("b5f762798a53d543a014caf8b297cff8f2f937e8")?;
    /// assert_eq!(hash.to_string(), "B5F762798A53D543A014CAF8B297CFF8F2F937E8");
    /// assert!(Hash160::from_hex("AABB").is_err()); // not 20 bytes
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn from_hex(s: &str) -> Result<Self, crate::CoreError> {
        let bytes = hex::decode(s).map_err(|e| crate::CoreError::InvalidHex(e.to_string()))?;
        if bytes.len() != 20 {
            return Err(crate::CoreError::InvalidHex(format!(
                "expected 20 bytes, got {}",
                bytes.len()
            )));
        }
        let mut arr = [0u8; 20];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// The bytes.
    ///
    /// ```
    /// use xrpl_core::types::Hash160;
    ///
    /// let hash = Hash160::from_hex("B5F762798A53D543A014CAF8B297CFF8F2F937E8")?;
    /// assert_eq!(hash.as_bytes()[0], 0xB5);
    /// assert_eq!(Hash160(*hash.as_bytes()), hash);
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl fmt::Debug for Hash160 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash160({})", hex::encode_upper(self.0))
    }
}

impl fmt::Display for Hash160 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode_upper(self.0))
    }
}

impl Serialize for Hash160 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex::encode_upper(self.0))
    }
}

impl<'de> Deserialize<'de> for Hash160 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

// --- Hash192 ---

impl Hash192 {
    /// All zero bytes.
    pub const ZERO: Self = Self([0u8; 24]);

    /// From hex (either case).
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidHex`](crate::CoreError::InvalidHex) if `s` is not
    /// hex or not 24 bytes.
    ///
    /// ```
    /// use xrpl_core::types::Hash192;
    ///
    /// let hash = Hash192::from_hex("000004c463c52827307480341125da0577defc38405b0e3e")?;
    /// assert_eq!(hash.to_string(), "000004C463C52827307480341125DA0577DEFC38405B0E3E");
    /// assert!(Hash192::from_hex("AABB").is_err()); // not 24 bytes
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn from_hex(s: &str) -> Result<Self, crate::CoreError> {
        let bytes = hex::decode(s).map_err(|e| crate::CoreError::InvalidHex(e.to_string()))?;
        if bytes.len() != 24 {
            return Err(crate::CoreError::InvalidHex(format!(
                "expected 24 bytes, got {}",
                bytes.len()
            )));
        }
        let mut arr = [0u8; 24];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// The bytes.
    ///
    /// ```
    /// use xrpl_core::types::Hash192;
    ///
    /// let hash = Hash192::from_hex("000004C463C52827307480341125DA0577DEFC38405B0E3E")?;
    /// assert_eq!(hash.as_bytes()[0], 0x00);
    /// assert_eq!(Hash192(*hash.as_bytes()), hash);
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn as_bytes(&self) -> &[u8; 24] {
        &self.0
    }
}

impl fmt::Debug for Hash192 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash192({})", hex::encode_upper(self.0))
    }
}

impl fmt::Display for Hash192 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode_upper(self.0))
    }
}

impl Serialize for Hash192 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex::encode_upper(self.0))
    }
}

impl<'de> Deserialize<'de> for Hash192 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

// --- Hash256 ---

impl Hash256 {
    /// All zero bytes.
    pub const ZERO: Self = Self([0u8; 32]);

    /// From hex (either case).
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidHex`](crate::CoreError::InvalidHex) if `s` is not
    /// hex or not 32 bytes.
    ///
    /// ```
    /// use xrpl_core::types::Hash256;
    ///
    /// let hash = Hash256::from_hex("4b4e9c06f24296074f7bc48f92a97916c6dc5ea9aaae7d26c7b8684e7d1ac31d")?;
    /// assert_eq!(hash.to_string(), "4B4E9C06F24296074F7BC48F92A97916C6DC5EA9AAAE7D26C7B8684E7D1AC31D");
    /// assert!(Hash256::from_hex("AABB").is_err()); // not 32 bytes
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn from_hex(s: &str) -> Result<Self, crate::CoreError> {
        let bytes = hex::decode(s).map_err(|e| crate::CoreError::InvalidHex(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(crate::CoreError::InvalidHex(format!(
                "expected 32 bytes, got {}",
                bytes.len()
            )));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// The bytes.
    ///
    /// ```
    /// use xrpl_core::types::Hash256;
    ///
    /// let hash = Hash256::from_hex("4B4E9C06F24296074F7BC48F92A97916C6DC5EA9AAAE7D26C7B8684E7D1AC31D")?;
    /// assert_eq!(hash.as_bytes()[0], 0x4B);
    /// assert_eq!(Hash256(*hash.as_bytes()), hash);
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash256({})", hex::encode_upper(self.0))
    }
}

impl fmt::Display for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode_upper(self.0))
    }
}

impl Serialize for Hash256 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex::encode_upper(self.0))
    }
}

impl<'de> Deserialize<'de> for Hash256 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn parse_order_and_bytes() {
        let low: Hash256 = "00".repeat(32).parse().unwrap();
        let high: Hash256 = "ff".repeat(32).parse().unwrap();
        assert!(low < high);
        assert_eq!(high.as_ref(), &[0xFF; 32]);
        assert!("00".parse::<Hash128>().is_err());
        let id: Hash192 = "000004C463C52827307480341125DA0577DEFC38405B0E3E"
            .parse()
            .unwrap();
        assert_eq!(
            id.to_string(),
            "000004C463C52827307480341125DA0577DEFC38405B0E3E"
        );
    }

    #[test]
    fn hash256_hex_roundtrip() {
        let hex_str = "4B4E9C06F24296074F7BC48F92A97916C6DC5EA9AAAE7D26C7B8684E7D1AC31D";
        let hash = Hash256::from_hex(hex_str).unwrap();
        assert_eq!(hash.to_string(), hex_str);
    }

    #[test]
    fn hash160_hex_roundtrip() {
        let hex_str = "B5F762798A53D543A014CAF8B297CFF8F2F937E8";
        let hash = Hash160::from_hex(hex_str).unwrap();
        assert_eq!(hash.to_string(), hex_str);
    }

    #[test]
    fn hash128_hex_roundtrip() {
        let hex_str = "AABBCCDDEE0011223344556677889900";
        let hash = Hash128::from_hex(hex_str).unwrap();
        assert_eq!(hash.to_string(), hex_str);
    }

    #[test]
    fn hash256_zero() {
        assert_eq!(
            Hash256::ZERO.to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn hash256_invalid_length() {
        assert!(Hash256::from_hex("AABB").is_err());
    }

    #[test]
    fn hash256_invalid_hex() {
        assert!(Hash256::from_hex("GGGG").is_err());
    }

    #[test]
    fn hash256_serde_roundtrip() {
        let hash =
            Hash256::from_hex("4B4E9C06F24296074F7BC48F92A97916C6DC5EA9AAAE7D26C7B8684E7D1AC31D")
                .unwrap();
        let json = serde_json::to_string(&hash).unwrap();
        let back: Hash256 = serde_json::from_str(&json).unwrap();
        assert_eq!(hash, back);
    }
}

/// `FromStr` (hex, either case) and `AsRef<[u8]>` for each hash type.
macro_rules! hash_traits {
    ($($name:ident),*) => {$(
        impl core::str::FromStr for $name {
            type Err = crate::CoreError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::from_hex(s)
            }
        }

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }
    )*};
}

hash_traits!(Hash128, Hash160, Hash192, Hash256);
