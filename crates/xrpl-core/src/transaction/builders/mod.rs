mod account_set;
mod amm;
mod check;
mod clawback;
mod delegate_set;
mod escrow;
mod nft;
mod offer;
mod payment;
mod payment_channel;
mod set_regular_key;
mod signer_list;
mod trust_set;

use crate::CoreError;
use serde_json::Value;

/// Trait implemented by all typed transactions.
pub trait Transaction {
    /// Returns the XRPL transaction type string (e.g. "Payment").
    fn transaction_type(&self) -> &'static str;

    /// Serialize the transaction to a JSON Value suitable for signing/submission.
    fn to_json(&self) -> Value;

    /// Validate that all required fields are present and valid.
    fn validate(&self) -> Result<(), CoreError>;
}

/// Common fields shared by all transactions.
/// Fields that can be autofilled (fee, sequence, last_ledger_sequence) are Optional.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TxCommon {
    pub account: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_ledger_sequence: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memos: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_txn_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_sequence: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_pub_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txn_signature: Option<String>,
}

impl TxCommon {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            fee: None,
            sequence: None,
            last_ledger_sequence: None,
            flags: None,
            memos: None,
            source_tag: None,
            account_txn_id: None,
            ticket_sequence: None,
            signing_pub_key: None,
            txn_signature: None,
        }
    }
}

/// Helper: merge TxCommon fields into a JSON object.
fn merge_common(base: &mut serde_json::Map<String, Value>, common: &TxCommon) {
    let common_val = serde_json::to_value(common).unwrap_or_default();
    if let Value::Object(map) = common_val {
        for (k, v) in map {
            base.entry(k).or_insert(v);
        }
    }
}

pub use account_set::{AccountSet, AccountSetBuilder};
pub use amm::{
    AMMBid, AMMBidBuilder, AMMCreate, AMMCreateBuilder, AMMDeposit, AMMDepositBuilder, AMMVote,
    AMMVoteBuilder, AMMWithdraw, AMMWithdrawBuilder,
};
pub use check::{
    CheckCancel, CheckCancelBuilder, CheckCash, CheckCashBuilder, CheckCreate, CheckCreateBuilder,
};
pub use clawback::{Clawback, ClawbackBuilder};
pub use delegate_set::{DelegateSet, DelegateSetBuilder};
pub use escrow::{
    EscrowCancel, EscrowCancelBuilder, EscrowCreate, EscrowCreateBuilder, EscrowFinish,
    EscrowFinishBuilder,
};
pub use nft::{
    NFTokenAcceptOffer, NFTokenAcceptOfferBuilder, NFTokenBurn, NFTokenBurnBuilder,
    NFTokenCancelOffer, NFTokenCancelOfferBuilder, NFTokenCreateOffer, NFTokenCreateOfferBuilder,
    NFTokenMint, NFTokenMintBuilder,
};
pub use offer::{OfferCancel, OfferCancelBuilder, OfferCreate, OfferCreateBuilder};
pub use payment::{Payment, PaymentBuilder};
pub use payment_channel::{
    PaymentChannelClaim, PaymentChannelClaimBuilder, PaymentChannelCreate,
    PaymentChannelCreateBuilder, PaymentChannelFund, PaymentChannelFundBuilder,
};
pub use set_regular_key::{SetRegularKey, SetRegularKeyBuilder};
pub use signer_list::{SignerListSet, SignerListSetBuilder};
pub use trust_set::{TrustSet, TrustSetBuilder};
