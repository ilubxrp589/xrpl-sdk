//! Transactions, three ways:
//!
//! - [`models`]: a typed struct for each of `rippled` 3.4.1's 82 transaction
//!   types, generated from its own definitions, with every field and flag.
//! - [`batch`]: `Batch` (XLS-56) — inner transactions, `BatchSigners`, the fee.
//! - [`builders`]: fluent builders for common transactions that check their
//!   required fields.
//!
//! Any transaction can also be written directly as `rippled`'s JSON.

#[cfg(feature = "std")]
pub mod batch;
/// Fluent builders for common transactions, with required-field checks.
#[cfg(feature = "std")]
pub mod builders;
mod common;
#[cfg(feature = "std")]
pub mod models;
mod types;

#[cfg(feature = "std")]
pub use builders::*;
#[allow(deprecated)]
pub use common::CommonFields;
#[allow(deprecated)]
pub use types::TransactionType;
