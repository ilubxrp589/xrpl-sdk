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
