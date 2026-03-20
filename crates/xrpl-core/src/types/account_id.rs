use core::fmt;
use core::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// 20-byte XRPL account identifier.
/// Stored as raw bytes; displayed as base58check classic address (starts with 'r').
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccountId(pub [u8; 20]);

impl AccountId {
    pub const ZERO: Self = Self([0u8; 20]);

    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// Encode as classic address (base58check with 0x00 prefix).
    pub fn to_address(&self) -> String {
        crate::address::encode_account_id(&self.0)
    }

    /// Decode from classic address string.
    pub fn from_address(addr: &str) -> Result<Self, crate::CoreError> {
        let bytes = crate::address::decode_account_id(addr)?;
        Ok(Self(bytes))
    }
}

impl fmt::Debug for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AccountId({})", self.to_address())
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_address())
    }
}

impl FromStr for AccountId {
    type Err = crate::CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_address(s)
    }
}

impl Serialize for AccountId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_address())
    }
}

impl<'de> Deserialize<'de> for AccountId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_address(&s).map_err(serde::de::Error::custom)
    }
}
