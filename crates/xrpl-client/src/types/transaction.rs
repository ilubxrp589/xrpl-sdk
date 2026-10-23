//! Responses of the transaction methods: `submit`, `submit_multisigned`,
//! `tx`, `transaction_entry` and `simulate`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// [`submit`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/submit)
/// and [`submit_multisigned`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/submit_multisigned):
/// the preliminary result of applying the transaction to the open ledger.
/// Only a validated ledger makes a result final.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitResult {
    /// `tesSUCCESS`, `tec…`, `tem…`, `tef…`, `tel…` or `ter…`.
    pub engine_result: String,
    /// The engine result as a number.
    pub engine_result_code: i32,
    /// The engine result in words.
    pub engine_result_message: String,
    /// The transaction, as hex.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_blob: Option<String>,
    /// The transaction as submitted, with its `hash`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_json: Option<Value>,
    /// Whether it was applied, broadcast, kept or queued (rippled 1.5+).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted: Option<bool>,
    /// Whether it applied to the open ledger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied: Option<bool>,
    /// Whether it was relayed to peers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub broadcast: Option<bool>,
    /// Whether the server kept it to retry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kept: Option<bool>,
    /// Whether it was queued.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queued: Option<bool>,
    /// The fee (drops) the transaction would need to get into the open ledger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_ledger_cost: Option<String>,
    /// The next sequence the account can use, counting queued transactions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_sequence_available: Option<u32>,
    /// The account's next sequence on the ledger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_sequence_next: Option<u32>,
    /// The latest validated ledger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated_ledger_index: Option<u32>,
}

/// [`tx`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/tx):
/// a transaction by hash. Under API version 1 the transaction's own fields
/// sit at the top level next to `meta`, `hash`, `ledger_index` and the rest;
/// they are in `tx_fields`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TxResult {
    /// Metadata, once the transaction is in a ledger (a hex string with `binary: true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
    /// Whether the data is from a validated ledger (final).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
    /// The ledger the answer is from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    /// That ledger's hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    /// Everything else: the transaction's fields, `hash`, `ctid`, `date`,
    /// `inLedger`, …
    #[serde(flatten)]
    pub tx_fields: HashMap<String, Value>,
}

impl TxResult {
    /// The transaction's hash.
    pub fn hash(&self) -> Option<&str> {
        self.tx_fields.get("hash").and_then(Value::as_str)
    }

    /// The final engine result, from the metadata (`tesSUCCESS`, `tec…`).
    pub fn transaction_result(&self) -> Option<&str> {
        self.meta
            .as_ref()?
            .get("TransactionResult")
            .and_then(Value::as_str)
    }

    /// What the transaction delivered (`delivered_amount`): check this, not
    /// a Payment's `Amount`, which a partial payment does not reach. See
    /// `xrpl_core::utils::delivered_amount`.
    pub fn delivered_amount(&self) -> Option<xrpl_core::types::Amount> {
        xrpl_core::utils::delivered_amount(self.meta.as_ref()?)
    }
}

/// [`transaction_entry`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/transaction_entry):
/// a transaction from one given ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionEntryResult {
    /// The transaction, as JSON.
    pub tx_json: Value,
    /// The transaction's metadata.
    pub metadata: Value,
    /// That ledger's hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    /// The ledger the answer is from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    /// Whether the data is from a validated ledger (final).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// [`simulate`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/simulate):
/// what a transaction would do in the current open ledger, without
/// submitting it (rippled 2.4+). Nothing is signed or sent to the network.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulateResult {
    /// Whether it would apply (always `false`: nothing is applied).
    pub applied: bool,
    /// The engine result (`tesSUCCESS`, `tecPATH_DRY`, …).
    pub engine_result: String,
    /// The engine result as a number.
    pub engine_result_code: i32,
    /// The engine result in words.
    pub engine_result_message: String,
    /// The open ledger it was simulated against.
    pub ledger_index: u32,
    /// The metadata it would produce (JSON).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
    /// The metadata as hex (`binary: true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta_blob: Option<String>,
    /// The transaction as autofilled (JSON).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_json: Option<Value>,
    /// The transaction as hex (`binary: true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_blob: Option<String>,
}
