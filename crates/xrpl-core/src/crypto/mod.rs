//! Keys and signatures. A [`Seed`] (16 bytes, `s…` in base58) derives a
//! [`Keypair`] the way `rippled`'s `wallet_propose` does, for Ed25519 or
//! secp256k1. [`Keypair::sign_signing_data`] signs as `rippled` verifies:
//! secp256k1 over SHA-512Half of the data, Ed25519 over the data itself.
//! [`channel`] signs and verifies payment channel claims.

#[cfg(feature = "std")]
pub mod channel;
/// Ed25519 keys and signatures.
pub mod ed25519;
/// secp256k1 keys and signatures (DER, low-S).
pub mod secp256k1;
/// Seeds, key pairs and transaction signing.
pub mod signing;

pub use crate::address::KeyType;
#[cfg(feature = "std")]
pub use channel::{sign_payment_channel_claim, verify_payment_channel_claim};
#[cfg(feature = "std")]
pub use signing::sign_transaction;
pub use signing::{sha512_half, Keypair, Seed};
