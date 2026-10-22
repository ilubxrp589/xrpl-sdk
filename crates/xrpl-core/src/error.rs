//! The error type of every fallible function in this crate.

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Everything a core function can fail with.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// Not valid base58.
    #[error("invalid base58: {0}")]
    InvalidBase58(String),

    /// A base58check checksum does not match.
    #[error("invalid checksum: expected {expected:?}, got {got:?}")]
    InvalidChecksum {
        /// The checksum the payload hashes to.
        expected: [u8; 4],
        /// The checksum the string carries.
        got: [u8; 4],
    },

    /// Not a valid address.
    #[error("invalid account address: {0}")]
    InvalidAddress(String),

    /// Not a valid seed.
    #[error("invalid seed: {0}")]
    InvalidSeed(String),

    /// The codec could not encode or decode the value.
    #[error("codec error: {0}")]
    CodecError(String),

    /// Not a valid amount.
    #[error("invalid amount: {0}")]
    InvalidAmount(String),

    /// A field the definitions, or a type, do not have.
    #[error("unknown field: {0}")]
    UnknownField(String),

    /// No key could be derived from the seed.
    #[error("key derivation failed")]
    KeyDerivationFailed,

    /// Signing or verifying failed.
    #[error("signing failed: {0}")]
    SigningFailed(String),

    /// A key of the wrong length.
    #[error("invalid public key length: expected {expected}, got {got}")]
    InvalidKeyLength {
        /// The length the key should have.
        expected: usize,
        /// The length it has.
        got: usize,
    },

    /// Not a valid currency code.
    #[error("invalid currency: {0}")]
    InvalidCurrency(String),

    /// Not valid hex.
    #[error("invalid hex: {0}")]
    InvalidHex(String),

    /// A value that breaks a rule (for example an unsigned transaction to hash).
    #[error("validation error: {0}")]
    ValidationError(String),
}
