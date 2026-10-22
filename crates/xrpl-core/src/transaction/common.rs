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
    /// The transaction type.
    pub transaction_type: String,
    /// The account that sends the transaction or owns the entry (`Account`).
    pub account: AccountId,
    /// The transaction cost, in drops (`Fee`).
    pub fee: Amount,
    /// The sequence number (`Sequence`).
    pub sequence: u32,
    /// Flag bits (`Flags`).
    #[serde(default)]
    pub flags: u32,
    /// The last ledger the transaction may appear in (`LastLedgerSequence`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_ledger_sequence: Option<u32>,
    /// A tag the sender uses to tell its own payments apart (`SourceTag`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,
    /// Valid only if the account's last transaction had this hash (`AccountTxnID`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_txn_id: Option<Hash256>,
    /// The signing public key, hex; empty when multi-signed (`SigningPubKey`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_pub_key: Option<Blob>,
    /// The signature, hex (`TxnSignature`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txn_signature: Option<Blob>,
}

impl CommonFields {
    /// The common fields from their values.
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
