//! Keys and signatures. A [`Seed`] (16 bytes, `s…` in base58) derives a
//! [`Keypair`] the way `rippled`'s `wallet_propose` does, for Ed25519 or
//! secp256k1. [`Keypair::sign_signing_data`] signs as `rippled` verifies:
//! secp256k1 over SHA-512Half of the data, Ed25519 over the data itself.
//! [`channel`] signs and verifies payment channel claims.

#[cfg(feature = "std")]
pub mod channel;
pub mod ed25519;
pub mod secp256k1;
pub mod signing;

pub use crate::address::KeyType;
#[cfg(feature = "std")]
#[cfg(feature = "std")]
pub use channel::{sign_payment_channel_claim, verify_payment_channel_claim};
pub use signing::sign_transaction;
pub use signing::{sha512_half, Keypair, Seed};
