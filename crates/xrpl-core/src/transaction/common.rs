use super::TransactionType;
use crate::types::{AccountId, Amount, Blob, Hash256};
#[cfg(feature = "std")]
use serde::Deserialize;
use serde::Serialize;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// Fields present on all XRPL transactions.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "std", derive(Deserialize))]
#[serde(rename_all = "PascalCase")]
pub struct CommonFields {
    pub transaction_type: String,
    pub account: AccountId,
    pub fee: Amount,
    pub sequence: u32,
    #[serde(default)]
    pub flags: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_ledger_sequence: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_txn_id: Option<Hash256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_pub_key: Option<Blob>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txn_signature: Option<Blob>,
}

impl CommonFields {
    pub fn new(
        tx_type: TransactionType,
        account: AccountId,
        fee_drops: u64,
        sequence: u32,
    ) -> Self {
        Self {
            transaction_type: tx_type.name().to_string(),
            account,
            fee: Amount::Xrp(fee_drops),
            sequence,
            flags: 0,
            last_ledger_sequence: None,
            source_tag: None,
            account_txn_id: None,
            signing_pub_key: None,
            txn_signature: None,
        }
    }
}
