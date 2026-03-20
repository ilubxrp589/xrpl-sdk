use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// 128-bit hash (16 bytes). Used for EmailHash field.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash128(pub [u8; 16]);

/// 160-bit hash (20 bytes). Used for account IDs internally.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash160(pub [u8; 20]);

/// 256-bit hash (32 bytes). Used for transaction hashes, ledger hashes, etc.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash256(pub [u8; 32]);

// --- Hash128 ---

impl Hash128 {
    pub const ZERO: Self = Self([0u8; 16]);

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
    pub const ZERO: Self = Self([0u8; 20]);

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

// --- Hash256 ---

impl Hash256 {
    pub const ZERO: Self = Self([0u8; 32]);

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
mod tests {
    use super::*;

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
