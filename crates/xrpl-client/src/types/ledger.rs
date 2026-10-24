//! Responses of the ledger methods: `ledger`, `ledger_closed`,
//! `ledger_current`, `ledger_data` and `ledger_entry`.

use super::LedgerObject;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// [`ledger`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger):
/// a ledger's header, and its transactions or state when asked for.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::LedgerResult;
///
/// let answer: LedgerResult = serde_json::from_value(json!({
///     "ledger": {"ledger_index": 107831520, "closed": true, "total_coins": "99985586671988789"},
///     "ledger_hash": "DECB05A6C02067FF57770846FD1E86F9199F294738148C3F5D1875AF14BBF9DA",
///     "ledger_index": 107831520, "validated": true
/// }))?;
/// let header = answer.ledger.unwrap();
/// assert_eq!(header.ledger_index, answer.ledger_index);
/// assert_eq!(header.total_coins.as_deref(), Some("99985586671988789"));
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::{LedgerInfo, LedgerTx};
/// use xrpl_core::utils::time::ripple_time_to_iso8601;
///
/// let header: LedgerInfo = serde_json::from_value(json!({
///     "ledger_index": 107831520, "close_time": 846004301, "close_time_iso": "2026-10-22T17:11:41Z",
///     "parent_hash": "BFD2A754CCDF25986B8E509B8986EC3D53398D7C07D7B17E88AA7D37E85C4FC6",
///     "transactions": ["01A21E4D21563C67D8FB19F5E3B07096D8338011E6B79A3B2E97EF2F2C49371F"]
/// }))?;
/// let closed = ripple_time_to_iso8601(header.close_time.unwrap() as u32);
/// assert_eq!(Some(closed), header.close_time_iso);
/// assert!(matches!(header.transactions.as_deref(), Some([LedgerTx::Hash(_)])));
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::LedgerTx;
///
/// let txs: Vec<LedgerTx> = serde_json::from_value(json!([
///     "01A21E4D21563C67D8FB19F5E3B07096D8338011E6B79A3B2E97EF2F2C49371F",
///     {"hash": "01A21E4D21563C67D8FB19F5E3B07096D8338011E6B79A3B2E97EF2F2C49371F",
///      "meta": {"AffectedNodes": [], "TransactionIndex": 11, "TransactionResult": "tesSUCCESS"}}
/// ]))?;
/// assert!(matches!(&txs[0], LedgerTx::Hash(id) if id.starts_with("01A21E4D")));
/// let LedgerTx::Expanded(tx) = &txs[1] else { panic!("not expanded") };
/// assert_eq!(tx.transaction_result(), Some("tesSUCCESS"));
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::LedgerClosedResult;
///
/// let closed: LedgerClosedResult = serde_json::from_value(json!({
///     "ledger_hash": "2F02AF7C7CA8C6ED83E5ECDD7E90CF3BD700659A870C2965000E5DE97BC6268D",
///     "ledger_index": 107856070
/// }))?;
/// assert_eq!(closed.ledger_index, 107856070);
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::LedgerCurrentResult;
///
/// let open: LedgerCurrentResult = serde_json::from_value(json!({
///     "ledger_current_index": 107856071
/// }))?;
/// assert_eq!(open.ledger_current_index, 107856071);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LedgerCurrentResult {
    /// The open ledger the answer is from (when not a validated one).
    pub ledger_current_index: u32,
}

/// [`ledger_data`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_data):
/// a page of a ledger's state, typed. With `binary: true` each entry is
/// `{ "data", "index" }`, kept as [`LedgerObject::Other`].
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::{LedgerDataResult, LedgerObject};
///
/// # let directory = json!({
/// #     "Flags": 0, "IndexNext": "426", "IndexPrevious": "424",
/// #     "Indexes": ["6BE53687261B6ED0F7C537328D1EA76F0F8A26264B973D85F6AA8B915C9EEF32"],
/// #     "LedgerEntryType": "DirectoryNode", "Owner": "rEMzCUeRZbx6ydthbi2RVNYdRbuqqsrYof",
/// #     "RootIndex": "BADEFEDEA9826CDFBD479D5747E441220ADC501BB32C8FE5D88C6C3CB629F542",
/// #     "index": "00000203EA7B91B283192593F8847E6641930185B3689530AAE24A64A815FF58"
/// # });
/// // `directory`: a `DirectoryNode` entry, as `rippled` writes it.
/// let page: LedgerDataResult = serde_json::from_value(json!({
///     "state": [directory], "ledger_index": 107831520, "validated": true,
///     "marker": "00000260CBB0B1DE602B32854B902F2B3817951486E685E768548EEE22C0818A"
/// }))?;
/// assert!(matches!(page.state[0], LedgerObject::DirectoryNode(_)));
/// assert!(page.marker.is_some()); // more of the state on the next page
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::{LedgerEntryResult, LedgerObject};
/// use xrpl_core::hashes::keylet;
///
/// # let node = json!({
/// #     "Account": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "Balance": "99770260",
/// #     "Domain": "68747470733A2F2F726970706C652E636F6D2F", "Flags": 2174353408u32,
/// #     "LedgerEntryType": "AccountRoot", "OwnerCount": 19,
/// #     "PreviousTxnID": "677DE150B130A24586B00006F16A0B146B1D5C80B196ADDD35047555F2E1B05E",
/// #     "PreviousTxnLgrSeq": 107831359, "Sequence": 89926490,
/// #     "index": "383BB05FBB110F9BE530B75C44DF100F86C5600F0F146593D3E7133FD1C99FF2"
/// # });
/// // `node`: the account's `AccountRoot` entry, as `rippled` writes it.
/// let entry: LedgerEntryResult = serde_json::from_value(json!({
///     "index": "383BB05FBB110F9BE530B75C44DF100F86C5600F0F146593D3E7133FD1C99FF2",
///     "node": node, "ledger_index": 107831520, "validated": true
/// }))?;
/// assert_eq!(entry.index, keylet::account("rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De")?.to_string());
/// let Some(LedgerObject::AccountRoot(root)) = &entry.node else { panic!("not an AccountRoot") };
/// assert_eq!(root.owner_count, 19);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
