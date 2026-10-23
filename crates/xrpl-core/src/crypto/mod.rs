//! Keys and signatures. A [`Seed`] (16 bytes, `s…` in base58) derives a
//! [`Keypair`] the way `rippled`'s `wallet_propose` does, for Ed25519 or
//! secp256k1. [`Keypair::sign_signing_data`] signs as `rippled` verifies:
//! secp256k1 over SHA-512Half of the data, Ed25519 over the data itself.
//! [`channel`] signs and verifies payment channel claims.
//!
//! A key can also come from the backups other wallets show:
//!
//! | Backup | Example | Read with |
//! |---|---|---|
//! | XRPL seed | `sEdT…` | [`Seed::from_base58`] |
//! | Secret numbers (Xaman) | `399150 474506 …` (8 × 6 digits) | [`Seed::from_secret_numbers`] ([`secret_numbers`]) |
//! | RFC 1751 key (`rippled`'s `master_key`) | `YOKE DREW DIME …` (12 words) | [`Seed::from_rfc1751`] ([`rfc1751`]) |
//! | BIP-39 recovery phrase (Ledger, multi-coin wallets) | `abandon ability …` (12–24 words) | [`Keypair::from_mnemonic`] ([`mnemonic`], [`hd`]) |
//!
//! The first three hold an XRPL seed and need the key type said; a BIP-39
//! phrase derives a secp256k1 key along a BIP-32 path.

#[cfg(feature = "std")]
pub mod channel;
/// Ed25519 keys and signatures.
pub mod ed25519;
pub mod hd;
pub mod mnemonic;
pub mod rfc1751;
/// secp256k1 keys and signatures (DER, low-S).
pub mod secp256k1;
pub mod secret_numbers;
/// Seeds, key pairs and transaction signing.
pub mod signing;
mod wordlists;

pub use crate::address::KeyType;
#[cfg(feature = "std")]
pub use channel::{sign_payment_channel_claim, verify_payment_channel_claim};
#[cfg(feature = "std")]
pub use signing::sign_transaction;
pub use signing::{sha512_half, verify_signing_data, Keypair, Seed};

use sha2::{Digest, Sha512};
use zeroize::{Zeroize, Zeroizing};

/// SHA-512Half of `parts` (concatenated), for secret inputs: the half is
/// returned in a buffer wiped on drop, and the full digest is wiped here.
pub(crate) fn secret_sha512_half(parts: &[&[u8]]) -> Zeroizing<[u8; 32]> {
    let mut hasher = Sha512::new();
    for part in parts {
        hasher.update(part);
    }
    let mut full = hasher.finalize();
    let digest: &mut [u8] = full.as_mut();
    let mut half = Zeroizing::new([0u8; 32]);
    for (out, byte) in half.iter_mut().zip(digest.iter()) {
        *out = *byte;
    }
    digest.zeroize();
    half
}
