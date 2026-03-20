use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// Variable-length byte array. Serialized as uppercase hex in JSON.
/// Used for SigningPubKey, TxnSignature, MemoData, etc.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Blob(pub Vec<u8>);

impl Blob {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn from_hex(s: &str) -> Result<Self, crate::CoreError> {
        let bytes = hex::decode(s).map_err(|e| crate::CoreError::InvalidHex(e.to_string()))?;
        Ok(Self(bytes))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl fmt::Debug for Blob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Blob({})", hex::encode_upper(&self.0))
    }
}

impl fmt::Display for Blob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode_upper(&self.0))
    }
}

impl Serialize for Blob {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex::encode_upper(&self.0))
    }
}

impl<'de> Deserialize<'de> for Blob {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

impl From<Vec<u8>> for Blob {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<&[u8]> for Blob {
    fn from(v: &[u8]) -> Self {
        Self(v.to_vec())
    }
}

// --- UInt wrappers ---
// These provide type safety for wire encoding and JSON serde.

/// 8-bit unsigned integer. Wire: 1 byte. JSON: number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UInt8(pub u8);

/// 16-bit unsigned integer. Wire: 2 bytes big-endian. JSON: number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UInt16(pub u16);

/// 32-bit unsigned integer. Wire: 4 bytes big-endian. JSON: number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UInt32(pub u32);

/// 64-bit unsigned integer. Wire: 8 bytes big-endian.
/// JSON: decimal string (JS can't handle 64-bit ints).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UInt64(pub u64);

impl Serialize for UInt64 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for UInt64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let v = s.parse::<u64>().map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blob_hex_roundtrip() {
        let blob = Blob::from_hex("DEADBEEF").unwrap();
        assert_eq!(blob.to_string(), "DEADBEEF");
        assert_eq!(blob.len(), 4);
    }

    #[test]
    fn blob_empty() {
        let blob = Blob::new(vec![]);
        assert!(blob.is_empty());
        assert_eq!(blob.to_string(), "");
    }

    #[test]
    fn blob_serde_roundtrip() {
        let blob = Blob::from_hex("CAFEBABE").unwrap();
        let json = serde_json::to_string(&blob).unwrap();
        assert_eq!(json, "\"CAFEBABE\"");
        let back: Blob = serde_json::from_str(&json).unwrap();
        assert_eq!(blob, back);
    }

    #[test]
    fn uint64_serde_as_string() {
        let v = UInt64(18446744073709551615);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "\"18446744073709551615\"");
        let back: UInt64 = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn uint32_serde_as_number() {
        let v = UInt32(12345);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "12345");
    }
}
