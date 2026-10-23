//! [`TransactionMetadata`]: what a transaction did, as `rippled` records it
//! with the transaction in the ledger (`meta`).

use crate::types::{Amount, Hash256};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

/// What a transaction in a ledger did: its result, its place in the ledger,
/// and every ledger entry it created, changed or deleted, as `rippled`
/// writes it (`meta`), with the fields `rippled` adds to it in its answers
/// (`delivered_amount`, `nftoken_id`, …).
///
/// ```
/// use xrpl_core::transaction::metadata::TransactionMetadata;
///
/// let meta: TransactionMetadata = serde_json::from_value(serde_json::json!({
///     "TransactionResult": "tesSUCCESS",
///     "TransactionIndex": 3,
///     "AffectedNodes": [{"ModifiedNode": {
///         "LedgerEntryType": "AccountRoot",
///         "LedgerIndex": "13F1A95D7AAB7108D5CE7EEAF504B2894B8C674E6D68499076441C4837282BF8",
///         "FinalFields": {"Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "Balance": "990"},
///         "PreviousFields": {"Balance": "1000"}
///     }}],
///     "delivered_amount": "10"
/// }))?;
/// assert!(meta.is_success());
/// assert_eq!(meta.affected_nodes[0].ledger_entry_type(), "AccountRoot");
/// assert_eq!(xrpl_core::utils::delivered_amount(&meta), Some(xrpl_core::types::Amount::Xrp(10)));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TransactionMetadata {
    /// The final result (`tesSUCCESS`, or a `tec` code: the transaction
    /// failed, but is in the ledger and its fee is charged)
    /// (`TransactionResult`).
    #[serde(rename = "TransactionResult")]
    pub transaction_result: String,
    /// Its position among the ledger's transactions, the order they were
    /// applied in (`TransactionIndex`).
    #[serde(rename = "TransactionIndex")]
    pub transaction_index: u32,
    /// Every ledger entry it created, changed or deleted (`AffectedNodes`).
    #[serde(rename = "AffectedNodes")]
    pub affected_nodes: Vec<AffectedNode>,
    /// What a Payment delivered, as `rippled` adds it to its answers
    /// (`delivered_amount`): read it with [`utils::delivered_amount`](crate::utils::delivered_amount),
    /// never from the Payment's `Amount`, which a partial payment does not
    /// reach.
    #[serde(
        rename = "delivered_amount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub delivered_amount: Option<DeliveredAmount>,
    /// What a partial payment delivered, as the ledger stores it
    /// (`DeliveredAmount`).
    #[serde(
        rename = "DeliveredAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub stored_delivered_amount: Option<Amount>,
    /// The Batch an inner transaction belongs to (`ParentBatchID`).
    #[serde(
        rename = "ParentBatchID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub parent_batch_id: Option<Hash256>,
    /// The NFT an `NFTokenMint` minted, or an `NFTokenAcceptOffer` moved, as
    /// `rippled` adds it (`nftoken_id`; see
    /// [`utils::get_nftoken_id`](crate::utils::get_nftoken_id)).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nftoken_id: Option<String>,
    /// The NFTs whose offers an `NFTokenCancelOffer` cancelled
    /// (`nftoken_ids`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nftoken_ids: Option<Vec<String>>,
    /// The offer an `NFTokenCreateOffer` created (`offer_id`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offer_id: Option<String>,
    /// The issuance an `MPTokenIssuanceCreate` created (`mpt_issuance_id`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mpt_issuance_id: Option<String>,
}

impl TransactionMetadata {
    /// Whether the transaction succeeded (`tesSUCCESS`). A `tec` result is
    /// in the ledger too, with its fee charged, but did not do what it was
    /// sent for.
    pub fn is_success(&self) -> bool {
        self.transaction_result == "tesSUCCESS"
    }
}

/// What a Payment delivered, as `rippled` reports it: an amount, or
/// `"unavailable"` for a transaction in a ledger from before 2014-01-20,
/// when the ledger did not yet record it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeliveredAmount {
    /// The amount delivered.
    Amount(Amount),
    /// Not recorded (ledgers before 2014-01-20).
    Unavailable,
}

impl Serialize for DeliveredAmount {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            DeliveredAmount::Amount(amount) => amount.serialize(serializer),
            DeliveredAmount::Unavailable => serializer.serialize_str("unavailable"),
        }
    }
}

impl<'de> Deserialize<'de> for DeliveredAmount {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        if value.as_str() == Some("unavailable") {
            return Ok(DeliveredAmount::Unavailable);
        }
        serde_json::from_value(value)
            .map(DeliveredAmount::Amount)
            .map_err(serde::de::Error::custom)
    }
}

/// A ledger entry a transaction created, changed or deleted. The fields are
/// the entry's own, as JSON (an entry's fields before and after are partial:
/// only those that changed, or that it has).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AffectedNode {
    /// An entry the transaction created (`CreatedNode`).
    CreatedNode(CreatedNode),
    /// An entry the transaction changed (`ModifiedNode`).
    ModifiedNode(ModifiedNode),
    /// An entry the transaction deleted (`DeletedNode`).
    DeletedNode(DeletedNode),
}

impl AffectedNode {
    /// The entry's type (`AccountRoot`, `RippleState`, `Offer`, …).
    pub fn ledger_entry_type(&self) -> &str {
        match self {
            AffectedNode::CreatedNode(n) => &n.ledger_entry_type,
            AffectedNode::ModifiedNode(n) => &n.ledger_entry_type,
            AffectedNode::DeletedNode(n) => &n.ledger_entry_type,
        }
    }

    /// The entry's ID (`LedgerIndex`).
    pub fn ledger_index(&self) -> &Hash256 {
        match self {
            AffectedNode::CreatedNode(n) => &n.ledger_index,
            AffectedNode::ModifiedNode(n) => &n.ledger_index,
            AffectedNode::DeletedNode(n) => &n.ledger_index,
        }
    }

    /// A created entry's fields (`NewFields`).
    pub fn new_fields(&self) -> Option<&Map<String, Value>> {
        match self {
            AffectedNode::CreatedNode(n) => n.new_fields.as_ref(),
            _ => None,
        }
    }

    /// A changed or deleted entry's fields after the transaction
    /// (`FinalFields`).
    pub fn final_fields(&self) -> Option<&Map<String, Value>> {
        match self {
            AffectedNode::ModifiedNode(n) => n.final_fields.as_ref(),
            AffectedNode::DeletedNode(n) => n.final_fields.as_ref(),
            AffectedNode::CreatedNode(_) => None,
        }
    }

    /// The values the transaction changed, as they were before
    /// (`PreviousFields`).
    pub fn previous_fields(&self) -> Option<&Map<String, Value>> {
        match self {
            AffectedNode::ModifiedNode(n) => n.previous_fields.as_ref(),
            AffectedNode::DeletedNode(n) => n.previous_fields.as_ref(),
            AffectedNode::CreatedNode(_) => None,
        }
    }
}

/// An entry a transaction created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CreatedNode {
    /// The entry's type (`LedgerEntryType`).
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,
    /// The entry's ID (`LedgerIndex`).
    #[serde(rename = "LedgerIndex")]
    pub ledger_index: Hash256,
    /// Its fields (`NewFields`; those with their default value are left
    /// out).
    #[serde(rename = "NewFields", default, skip_serializing_if = "Option::is_none")]
    pub new_fields: Option<Map<String, Value>>,
}

/// An entry a transaction changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModifiedNode {
    /// The entry's type (`LedgerEntryType`).
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,
    /// The entry's ID (`LedgerIndex`).
    #[serde(rename = "LedgerIndex")]
    pub ledger_index: Hash256,
    /// Its fields after the transaction (`FinalFields`).
    #[serde(
        rename = "FinalFields",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub final_fields: Option<Map<String, Value>>,
    /// The fields that changed, as they were before (`PreviousFields`).
    #[serde(
        rename = "PreviousFields",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_fields: Option<Map<String, Value>>,
    /// The transaction that changed the entry before this one
    /// (`PreviousTxnID`).
    #[serde(
        rename = "PreviousTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_id: Option<Hash256>,
    /// The ledger that transaction is in (`PreviousTxnLgrSeq`).
    #[serde(
        rename = "PreviousTxnLgrSeq",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_lgr_seq: Option<u32>,
}

/// An entry a transaction deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DeletedNode {
    /// The entry's type (`LedgerEntryType`).
    #[serde(rename = "LedgerEntryType")]
    pub ledger_entry_type: String,
    /// The entry's ID (`LedgerIndex`).
    #[serde(rename = "LedgerIndex")]
    pub ledger_index: Hash256,
    /// Its fields when it was deleted (`FinalFields`).
    #[serde(
        rename = "FinalFields",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub final_fields: Option<Map<String, Value>>,
    /// The fields the transaction changed before deleting it, as they were
    /// (`PreviousFields`).
    #[serde(
        rename = "PreviousFields",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_fields: Option<Map<String, Value>>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn delivered_amount_reads_unavailable_and_amounts() {
        for (json, expected) in [
            (
                serde_json::json!("unavailable"),
                DeliveredAmount::Unavailable,
            ),
            (
                serde_json::json!("15"),
                DeliveredAmount::Amount(Amount::Xrp(15)),
            ),
        ] {
            let read: DeliveredAmount = serde_json::from_value(json.clone()).unwrap();
            assert_eq!(read, expected);
            assert_eq!(serde_json::to_value(&read).unwrap(), json);
        }
        assert!(serde_json::from_value::<DeliveredAmount>(serde_json::json!("x")).is_err());
    }
}
