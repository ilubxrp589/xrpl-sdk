//! XRPL data types: [`Amount`] (XRP drops, tokens, MPTs), [`AccountId`],
//! [`Currency`], fixed-size hashes, blobs, and [`LedgerObject`], a typed
//! struct for each of `rippled` 3.4.1's 31 ledger entry types. All of them
//! (de)serialize as `rippled`'s JSON.

mod account_id;
mod amount;
mod blob;
mod currency;
mod hash;
#[cfg(feature = "std")]
pub mod ledger_object;

pub use account_id::AccountId;
pub use amount::{Amount, IouAmount, MAX_XRP_DROPS};
pub use blob::{Blob, UInt16, UInt32, UInt64, UInt8};
pub use currency::Currency;
pub use hash::{Hash128, Hash160, Hash192, Hash256};
#[cfg(feature = "std")]
pub use ledger_object::LedgerObject;
