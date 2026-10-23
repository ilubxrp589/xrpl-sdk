//! Transactions:
//!
//! - [`models`]: a typed struct for each of `rippled` 3.4.1's 82 transaction
//!   types, generated from its own definitions, with every field and flag,
//!   `new` for the required fields and `with_…` for the rest, and
//!   [`models::TxModel::validate`] for `rippled`'s checks that need no ledger.
//! - [`batch`]: `Batch` (XLS-56) — inner transactions, `BatchSigners`, the fee.
//! - [`cost`]: what `rippled` charges for a transaction.
//!
//! Any transaction can also be written directly as `rippled`'s JSON.

#[cfg(feature = "std")]
pub mod batch;
#[cfg(feature = "std")]
pub mod cost;
#[cfg(feature = "std")]
pub mod models;

/// A transaction in any form this crate builds, as the `rippled` JSON the
/// signers, `autofill` and the codec take: a typed model
/// ([`models::Payment`], …) or a [`models::Tx`], each checked with
/// [`models::TxModel::validate`] first, or JSON (`serde_json::Value`, as
/// is).
///
/// ```
/// use xrpl_core::transaction::models;
/// use xrpl_core::transaction::ToTxJson;
/// use xrpl_core::types::Amount;
///
/// let payment = models::Payment::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     Amount::xrp(1_000_000)?,
/// );
/// let json = payment.to_tx_json()?;
/// assert_eq!(json["TransactionType"], "Payment");
/// assert_eq!(json.to_tx_json()?, json);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
#[cfg(feature = "std")]
pub trait ToTxJson {
    /// The transaction as `rippled` JSON, `TransactionType` included.
    ///
    /// # Errors
    ///
    /// A [`CoreError`](crate::CoreError) when a model fails its checks
    /// ([`models::TxModel::validate`]) or does not serialize.
    fn to_tx_json(&self) -> Result<serde_json::Value, crate::CoreError>;
}

#[cfg(feature = "std")]
impl ToTxJson for serde_json::Value {
    fn to_tx_json(&self) -> Result<serde_json::Value, crate::CoreError> {
        Ok(self.clone())
    }
}

#[cfg(feature = "std")]
impl ToTxJson for models::Tx {
    fn to_tx_json(&self) -> Result<serde_json::Value, crate::CoreError> {
        self.validate()?;
        self.to_value()
    }
}

#[cfg(feature = "std")]
impl<T: models::TxModel> ToTxJson for T {
    fn to_tx_json(&self) -> Result<serde_json::Value, crate::CoreError> {
        self.validate()?;
        self.to_json()
    }
}
