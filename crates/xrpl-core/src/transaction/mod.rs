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
pub mod cost;
#[cfg(feature = "std")]
pub mod models;
mod types;

#[cfg(feature = "std")]
pub use builders::*;

/// A transaction in any form this crate builds, as the `rippled` JSON the
/// signers, `autofill` and the codec take: a typed model
/// ([`models::Payment`], …), a [`models::Tx`], a builder ([`Payment`], …,
/// checked with its `validate` first) or JSON (`serde_json::Value`, as is).
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
    /// A [`CoreError`](crate::CoreError) when a model does not serialize or
    /// a builder fails its checks.
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
        self.to_value()
    }
}

#[cfg(feature = "std")]
impl<T: models::TxModel> ToTxJson for T {
    fn to_tx_json(&self) -> Result<serde_json::Value, crate::CoreError> {
        self.to_json()
    }
}

/// Builders are checked (`validate`) before they become JSON.
#[cfg(feature = "std")]
macro_rules! builder_to_tx_json {
    ($($name:ident),* $(,)?) => {$(
        impl ToTxJson for builders::$name {
            fn to_tx_json(&self) -> Result<serde_json::Value, crate::CoreError> {
                builders::Transaction::validate(self)?;
                Ok(builders::Transaction::to_json(self))
            }
        }
    )*};
}

#[cfg(feature = "std")]
builder_to_tx_json!(
    AccountSet,
    AMMBid,
    AMMCreate,
    AMMDeposit,
    AMMVote,
    AMMWithdraw,
    CheckCancel,
    CheckCash,
    CheckCreate,
    Clawback,
    DelegateSet,
    EscrowCancel,
    EscrowCreate,
    EscrowFinish,
    NFTokenAcceptOffer,
    NFTokenBurn,
    NFTokenCancelOffer,
    NFTokenCreateOffer,
    NFTokenMint,
    OfferCancel,
    OfferCreate,
    Payment,
    PaymentChannelClaim,
    PaymentChannelCreate,
    PaymentChannelFund,
    SetRegularKey,
    SignerListSet,
    TrustSet,
);
#[allow(deprecated)]
pub use common::CommonFields;
#[allow(deprecated)]
pub use types::TransactionType;
