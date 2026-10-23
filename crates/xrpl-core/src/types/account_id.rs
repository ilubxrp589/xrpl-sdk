use core::fmt;
use core::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// 20-byte XRPL account identifier.
/// Stored as raw bytes; displayed as base58check classic address (starts with 'r').
/// Ordered by its bytes: the order `rippled` sorts multi-signers in (not the
/// order of the addresses as text).
///
/// ```
/// use xrpl_core::types::AccountId;
///
/// let account: AccountId = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".parse()?;
/// assert_eq!(account.to_string(), "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
/// assert_eq!(account.as_bytes()[..4], [0xB5, 0xF7, 0x62, 0x79]);
/// assert!(AccountId::ZERO < account); // ordered by bytes
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountId(pub [u8; 20]);

impl AsRef<[u8]> for AccountId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AccountId {
    /// All zero bytes.
    pub const ZERO: Self = Self([0u8; 20]);

    /// From the 20-byte account ID.
    ///
    /// ```
    /// use xrpl_core::types::AccountId;
    ///
    /// let zero = AccountId::from_bytes([0; 20]);
    /// assert_eq!(zero, AccountId::ZERO);
    /// assert_eq!(zero.to_address(), "rrrrrrrrrrrrrrrrrrrrrhoLvTp");
    /// ```
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    /// The account a public key is the master key of: RIPEMD-160 of
    /// SHA-256 of the key (33 bytes, as `SigningPubKey` holds it). For a key
    /// that lives in a KMS or HSM, this is its address.
    ///
    /// ```
    /// use xrpl_core::crypto::{Keypair, Seed};
    /// use xrpl_core::types::AccountId;
    ///
    /// let keys = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
    /// let account = AccountId::from_public_key(keys.public_key());
    /// assert_eq!(account.to_address(), "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1");
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn from_public_key(public_key: &[u8]) -> Self {
        Self(crate::crypto::signing::public_key_to_account_id(public_key))
    }

    /// The 20 bytes.
    ///
    /// ```
    /// use xrpl_core::types::AccountId;
    ///
    /// let account = AccountId::from_address("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
    /// let bytes: &[u8; 20] = account.as_bytes();
    /// assert_eq!(bytes[..4], [0xB5, 0xF7, 0x62, 0x79]);
    /// assert_eq!(AccountId::from_bytes(*bytes), account);
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    /// Encode as classic address (base58check with 0x00 prefix).
    ///
    /// ```
    /// use xrpl_core::types::AccountId;
    ///
    /// assert_eq!(AccountId::ZERO.to_address(), "rrrrrrrrrrrrrrrrrrrrrhoLvTp");
    /// let account = AccountId::from_address("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")?;
    /// assert_eq!(account.to_address(), "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn to_address(&self) -> String {
        crate::address::encode_account_id(&self.0)
    }

    /// Decode from classic address string.
    ///
    /// ```
    /// use xrpl_core::types::AccountId;
    ///
    /// let account = AccountId::from_address("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")?;
    /// assert_eq!(account.to_string(), "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    /// assert!(AccountId::from_address("not an address").is_err());
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidAddress`](crate::CoreError::InvalidAddress) (or
    /// `InvalidBase58`, `InvalidChecksum`) if `addr` is not a classic address.
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
