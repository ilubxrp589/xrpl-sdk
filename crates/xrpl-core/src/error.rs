#[cfg(not(feature = "std"))]
use alloc::string::String;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("invalid base58: {0}")]
    InvalidBase58(String),

    #[error("invalid checksum: expected {expected:?}, got {got:?}")]
    InvalidChecksum { expected: [u8; 4], got: [u8; 4] },

    #[error("invalid account address: {0}")]
    InvalidAddress(String),

    #[error("invalid seed: {0}")]
    InvalidSeed(String),

    #[error("codec error: {0}")]
    CodecError(String),

    #[error("invalid amount: {0}")]
    InvalidAmount(String),

    #[error("unknown field: {0}")]
    UnknownField(String),

    #[error("key derivation failed")]
    KeyDerivationFailed,

    #[error("signing failed: {0}")]
    SigningFailed(String),

    #[error("invalid public key length: expected {expected}, got {got}")]
    InvalidKeyLength { expected: usize, got: usize },

    #[error("invalid currency: {0}")]
    InvalidCurrency(String),

    #[error("invalid hex: {0}")]
    InvalidHex(String),

    #[error("validation error: {0}")]
    ValidationError(String),
}
