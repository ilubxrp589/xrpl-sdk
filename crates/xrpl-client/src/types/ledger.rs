//! Responses of the ledger methods: `ledger`, `ledger_closed`,
//! `ledger_current`, `ledger_data` and `ledger_entry`.

use super::LedgerObject;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// [`ledger`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger):
/// a ledger's header, and its transactions or state when asked for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LedgerResult {
    /// The header (absent only for an unknown ledger, which is an error).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger: Option<LedgerInfo>,
    /// That ledger's hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    /// The ledger the answer is from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    /// The open ledger the answer is from (when not a validated one).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    /// Queued transactions (`queue: true`, current ledger only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_data: Option<Vec<Value>>,
    /// Whether the data is from a validated ledger (final).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// A ledger header, and the transactions or state asked for with it. With
/// `binary: true` the header is hex (`ledger_data`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LedgerInfo {
    /// The state tree's hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_hash: Option<String>,
    /// Close flags (1: the close time is not agreed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_flags: Option<u32>,
    /// Ledger time (seconds since 2000-01-01; see `xrpl_core::utils::time`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time: Option<u64>,
    /// When the ledger closed, in words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time_human: Option<String>,
    /// When the ledger closed, ISO 8601.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time_iso: Option<String>,
    /// How precise the close time is, in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time_resolution: Option<u32>,
    /// Whether the ledger is closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
    /// That ledger's hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    /// The ledger's index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    /// The header as hex (`binary: true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_data: Option<String>,
    /// When the previous ledger closed (ledger time).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_close_time: Option<u64>,
    /// The previous ledger's hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_hash: Option<String>,
    /// All the XRP there is, in drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_coins: Option<String>,
    /// The transaction tree's hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<String>,
    /// The ledger's transactions (`transactions: true`): their IDs, or with
    /// `expand: true` the transactions with their metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<LedgerTx>>,
    /// The whole state (`full: true`, admin only).
    #[serde(
        rename = "accountState",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub account_state: Option<Vec<Value>>,
}

/// A transaction in a [`LedgerInfo`]: its ID, or (with `expand: true`) the
/// transaction with its metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum LedgerTx {
    /// The transaction's ID.
    Hash(String),
    /// The transaction with its metadata (`expand: true`).
    Expanded(Box<super::TxResult>),
}

/// [`ledger_closed`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_closed):
/// the latest closed (not necessarily validated) ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LedgerClosedResult {
    /// That ledger's hash.
    pub ledger_hash: String,
    /// The ledger the answer is from.
    pub ledger_index: u32,
}

/// [`ledger_current`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_current):
/// the index of the open ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LedgerCurrentResult {
    /// The open ledger the answer is from (when not a validated one).
    pub ledger_current_index: u32,
}

/// [`ledger_data`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_data):
/// a page of a ledger's state, typed. With `binary: true` each entry is
/// `{ "data", "index" }`, kept as [`LedgerObject::Other`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LedgerDataResult {
    /// What the server reports.
    pub state: Vec<LedgerObject>,
    /// The header, on the first page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger: Option<LedgerInfo>,
    /// That ledger's hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    /// The ledger the answer is from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    /// Whether the data is from a validated ledger (final).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// [`ledger_entry`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_entry):
/// one ledger entry, typed (`node`) or as hex (`node_binary`, with `binary: true`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LedgerEntryResult {
    /// The entry's ID.
    pub index: String,
    /// The entry, typed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<LedgerObject>,
    /// The entry, as hex.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_binary: Option<String>,
    /// The ledger that deleted it, when asked about a deleted entry (Clio).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_ledger_index: Option<u32>,
    /// The open ledger the answer is from (when not a validated one).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
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
