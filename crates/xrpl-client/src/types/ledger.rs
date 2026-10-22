//! Responses of the ledger methods: `ledger`, `ledger_closed`,
//! `ledger_current`, `ledger_data` and `ledger_entry`.

use super::{count_as_string, LedgerObject};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// [`ledger`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger):
/// a ledger's header, and its transactions or state when asked for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerResult {
    /// The header (absent only for an unknown ledger, which is an error).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger: Option<LedgerInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    /// Queued transactions (`queue: true`, current ledger only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_data: Option<Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// A ledger header. `rippled`'s API version 1 writes `ledger_index` and
/// `total_coins` as strings; both versions are read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_flags: Option<u32>,
    /// Ledger time (seconds since 2000-01-01; see `xrpl_core::utils::time`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time_human: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time_iso: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time_resolution: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "count_as_string"
    )]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_close_time: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_hash: Option<String>,
    /// All the XRP there is, in drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_coins: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<String>,
    /// The ledger's transactions (`transactions: true`): hashes, or with
    /// `expand: true` the transactions with their metadata (`metaData`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<Value>>,
    /// The whole state (`full: true`, admin only).
    #[serde(
        rename = "accountState",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub account_state: Option<Vec<Value>>,
}

/// [`ledger_closed`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_closed):
/// the latest closed (not necessarily validated) ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerClosedResult {
    pub ledger_hash: String,
    pub ledger_index: u32,
}

/// [`ledger_current`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_current):
/// the index of the open ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerCurrentResult {
    pub ledger_current_index: u32,
}

/// [`ledger_data`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_data):
/// a page of a ledger's state, typed. With `binary: true` each entry is
/// `{ "data", "index" }`, kept as [`LedgerObject::Other`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerDataResult {
    pub state: Vec<LedgerObject>,
    /// The header, on the first page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger: Option<LedgerInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// [`ledger_entry`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_entry):
/// one ledger entry, typed (`node`) or as hex (`node_binary`, with `binary: true`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerEntryResult {
    /// The entry's ID.
    pub index: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<LedgerObject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_binary: Option<String>,
    /// The ledger that deleted it, when asked about a deleted entry (Clio).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}
