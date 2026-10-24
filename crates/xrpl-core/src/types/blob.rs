use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// Variable-length byte array. Serialized as uppercase hex in JSON.
/// Used for SigningPubKey, TxnSignature, MemoData, etc.
///
/// ```
/// use xrpl_core::types::Blob;
///
/// let blob = Blob::from_hex("cafebabe")?;
/// assert_eq!(blob.as_bytes(), &[0xCA, 0xFE, 0xBA, 0xBE]);
/// assert_eq!(serde_json::to_value(&blob).unwrap(), "CAFEBABE");
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Blob(pub Vec<u8>);

impl Blob {
    /// From bytes.
    ///
    /// ```
    /// use xrpl_core::types::Blob;
    ///
    /// let blob = Blob::new(b"hi".to_vec());
    /// assert_eq!(blob.to_string(), "6869");
    /// ```
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// From hex.
    ///
    /// ```
    /// use xrpl_core::types::Blob;
    ///
    /// assert_eq!(Blob::from_hex("DEADBEEF")?.len(), 4);
    /// assert!(Blob::from_hex("GG").is_err());
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidHex`](crate::CoreError::InvalidHex) if `s` is not
    /// hex.
    pub fn from_hex(s: &str) -> Result<Self, crate::CoreError> {
        let bytes = hex::decode(s).map_err(|e| crate::CoreError::InvalidHex(e.to_string()))?;
        Ok(Self(bytes))
    }

    /// The bytes.
    ///
    /// ```
    /// use xrpl_core::types::Blob;
    ///
    /// assert_eq!(Blob::from_hex("0102")?.as_bytes(), &[1, 2]);
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Whether there are no bytes.
    ///
    /// ```
    /// use xrpl_core::types::Blob;
    ///
    /// assert!(Blob::new(vec![]).is_empty());
    /// assert!(!Blob::new(vec![0]).is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// How many bytes.
    ///
    /// ```
    /// use xrpl_core::types::Blob;
    ///
    /// assert_eq!(Blob::from_hex("DEADBEEF")?.len(), 4);
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
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
///
/// ```
/// use xrpl_core::types::UInt8;
///
/// assert_eq!(serde_json::to_string(&UInt8(7)).unwrap(), "7");
/// assert_eq!(serde_json::from_str::<UInt8>("255").unwrap(), UInt8(255));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UInt8(pub u8);

/// 16-bit unsigned integer. Wire: 2 bytes big-endian. JSON: number.
///
/// ```
/// use xrpl_core::types::UInt16;
///
/// assert_eq!(serde_json::to_string(&UInt16(1000)).unwrap(), "1000");
/// assert_eq!(serde_json::from_str::<UInt16>("65535").unwrap(), UInt16(u16::MAX));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UInt16(pub u16);

/// 32-bit unsigned integer. Wire: 4 bytes big-endian. JSON: number.
///
/// ```
/// use xrpl_core::types::UInt32;
///
/// assert_eq!(serde_json::to_string(&UInt32(12345)).unwrap(), "12345");
/// assert_eq!(serde_json::from_str::<UInt32>("12345").unwrap(), UInt32(12345));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UInt32(pub u32);

/// 64-bit unsigned integer. Wire: 8 bytes big-endian.
/// JSON: decimal string (JS can't handle 64-bit ints).
///
/// ```
/// use xrpl_core::types::UInt64;
///
/// let max = UInt64(u64::MAX);
/// assert_eq!(serde_json::to_string(&max).unwrap(), "\"18446744073709551615\"");
/// assert_eq!(serde_json::from_str::<UInt64>("\"18446744073709551615\"").unwrap(), max);
/// ```
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
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
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
