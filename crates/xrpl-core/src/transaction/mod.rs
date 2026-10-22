//! Typed builders for common transactions ([`builders`]). Each checks its
//! required fields and produces the JSON the codec and `rippled` take. Any
//! transaction type can also be written directly as JSON.

#[cfg(feature = "std")]
pub mod builders;
mod common;
mod types;

#[cfg(feature = "std")]
pub use builders::*;
pub use common::CommonFields;
pub use types::TransactionType;
