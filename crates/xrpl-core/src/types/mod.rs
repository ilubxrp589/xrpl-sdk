//! XRPL data types: [`Amount`] (XRP drops, tokens, MPTs), [`AccountId`],
//! [`Currency`], fixed-size hashes, blobs, and [`LedgerObject`], a typed
//! struct for each of `rippled` 3.4.1's 31 ledger entry types. All of them
//! (de)serialize as `rippled`'s JSON.

mod account_id;
mod amount;
mod asset;
mod blob;
mod currency;
mod hash;
#[cfg(feature = "std")]
pub mod ledger_object;

pub use account_id::AccountId;
pub use amount::{Amount, IouAmount, MAX_XRP_DROPS};
#[cfg(feature = "std")] // for the codec
pub(crate) use amount::{IOU_EXPONENT_MAX, IOU_EXPONENT_MIN, IOU_MANTISSA_MAX, IOU_MANTISSA_MIN};
pub use asset::Asset;
pub use blob::{Blob, UInt16, UInt32, UInt64, UInt8};
pub use currency::Currency;
pub use hash::{Hash128, Hash160, Hash192, Hash256};
#[cfg(feature = "std")]
pub use ledger_object::LedgerObject;
