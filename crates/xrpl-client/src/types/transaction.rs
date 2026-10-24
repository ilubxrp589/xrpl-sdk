//! Responses of the transaction methods: `submit`, `submit_multisigned`,
//! `tx`, `transaction_entry` and `simulate`, and [`TxResult`], a transaction
//! with its metadata as every method that returns one writes it.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use xrpl_core::transaction::{models::Tx, TransactionMetadata};
use xrpl_core::CoreError;

/// [`submit`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/submit)
/// and [`submit_multisigned`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/submit_multisigned):
/// the preliminary result of applying the transaction to the open ledger.
/// Only a validated ledger makes a result final.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::SubmitResult;
///
/// let submitted: SubmitResult = serde_json::from_value(json!({
///     "engine_result": "tesSUCCESS", "engine_result_code": 0,
///     "engine_result_message": "The transaction was applied. Only final in a validated ledger.",
///     "accepted": true, "applied": true, "broadcast": true, "queued": false,
///     "open_ledger_cost": "10", "validated_ledger_index": 21768525
/// }))?;
/// // Applied to the open ledger: not final until a validated ledger holds it.
/// assert_eq!(submitted.engine_result, "tesSUCCESS");
/// assert_eq!(submitted.applied, Some(true));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
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
    /// The transaction as submitted (`submit` writes its `hash` in it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_json: Option<Value>,
    /// The transaction's ID (`submit_multisigned` writes it here, beside
    /// `tx_json`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
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

/// A transaction in a ledger, with its metadata, as `rippled`'s API
/// version 2 writes it: the answer of
/// [`tx`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/tx)
/// and of `transaction_entry`, and each transaction of `account_tx`,
/// `nft_history` and an expanded `ledger`. With `binary: true` the
/// transaction and metadata are hex (`tx_blob`, `meta_blob`).
///
/// The transaction stays JSON (`tx_json`), so a field or type from a newer
/// `rippled` never stops an answer from being read; [`TxResult::transaction`]
/// gives it typed.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::TxResult;
///
/// let tx: TxResult = serde_json::from_value(json!({
///     "tx_json": {
///         "TransactionType": "OfferCreate", "Account": "rMsXVzCug7nUsMYFGPXBTWY732fRZqxgfY"
///     },
///     "hash": "01A21E4D21563C67D8FB19F5E3B07096D8338011E6B79A3B2E97EF2F2C49371F",
///     "ctid": "C66D60E0000B0000",
///     "ledger_index": 107831520, "close_time_iso": "2026-10-22T17:11:41Z", "validated": true,
///     "meta": {"AffectedNodes": [], "TransactionIndex": 11, "TransactionResult": "tesSUCCESS"}
/// }))?;
/// assert_eq!(tx.transaction_result(), Some("tesSUCCESS"));
/// assert_eq!(tx.validated, Some(true)); // final
/// assert_eq!(tx.tx_json.unwrap()["TransactionType"], "OfferCreate");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TxResult {
    /// The transaction, as JSON (`tx_json`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_json: Option<Value>,
    /// The transaction, as hex (`binary: true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_blob: Option<String>,
    /// What it did, once it is in a ledger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<TransactionMetadata>,
    /// The metadata, as hex (`binary: true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta_blob: Option<String>,
    /// The transaction's ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// The ledger it is in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    /// That ledger's hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    /// When that ledger closed, ISO 8601.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time_iso: Option<String>,
    /// The compact transaction ID (XLS-37).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ctid: Option<String>,
    /// Whether the ledger it is in is validated (final).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

impl TxResult {
    /// The transaction's ID.
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::types::TxResult;
    ///
    /// let tx: TxResult = serde_json::from_value(json!({
    ///     "hash": "01A21E4D21563C67D8FB19F5E3B07096D8338011E6B79A3B2E97EF2F2C49371F",
    ///     "ctid": "C66D60E0000B0000",
    ///     "ledger_index": 107831520, "close_time_iso": "2026-10-22T17:11:41Z", "validated": true,
    ///     "meta": {"AffectedNodes": [], "TransactionIndex": 11, "TransactionResult": "tesSUCCESS"}
    /// }))?;
    /// assert_eq!(
    ///     tx.hash(),
    ///     Some("01A21E4D21563C67D8FB19F5E3B07096D8338011E6B79A3B2E97EF2F2C49371F"),
    /// );
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn hash(&self) -> Option<&str> {
        self.hash.as_deref()
    }

    /// The final engine result, from the metadata (`tesSUCCESS`, `tec…`).
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::types::TxResult;
    ///
    /// let tx: TxResult = serde_json::from_value(json!({
    ///     "hash": "01A21E4D21563C67D8FB19F5E3B07096D8338011E6B79A3B2E97EF2F2C49371F",
    ///     "ctid": "C66D60E0000B0000",
    ///     "ledger_index": 107831520, "close_time_iso": "2026-10-22T17:11:41Z", "validated": true,
    ///     "meta": {"AffectedNodes": [], "TransactionIndex": 11, "TransactionResult": "tesSUCCESS"}
    /// }))?;
    /// assert_eq!(tx.transaction_result(), Some("tesSUCCESS"));
    /// // No metadata (not in a ledger yet, or a binary answer): no result.
    /// let pending: TxResult = serde_json::from_value(json!({
    ///     "hash": "01A21E4D21563C67D8FB19F5E3B07096D8338011E6B79A3B2E97EF2F2C49371F"
    /// }))?;
    /// assert_eq!(pending.transaction_result(), None);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn transaction_result(&self) -> Option<&str> {
        Some(self.meta.as_ref()?.transaction_result.as_str())
    }

    /// What the transaction delivered (`delivered_amount`): check this, not
    /// a Payment's `Amount`, which a partial payment does not reach. See
    /// `xrpl_core::utils::delivered_amount`.
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::types::TxResult;
    /// use xrpl_core::types::Amount;
    ///
    /// // A partial payment: up to 6.87161 XRP, of which 3.435805 arrived.
    /// let tx: TxResult = serde_json::from_value(json!({
    ///     "tx_json": {"TransactionType": "Payment", "DeliverMax": "6871610", "Flags": 131072},
    ///     "meta": {"AffectedNodes": [], "TransactionIndex": 0, "TransactionResult": "tesSUCCESS",
    ///         "DeliveredAmount": "3435805", "delivered_amount": "3435805"}
    /// }))?;
    /// assert_eq!(tx.delivered_amount(), Some(Amount::Xrp(3_435_805)));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn delivered_amount(&self) -> Option<xrpl_core::types::Amount> {
        xrpl_core::utils::delivered_amount(self.meta.as_ref()?)
    }

    /// The transaction, typed ([`Tx`]), from `tx_json`. A Payment's
    /// `DeliverMax` (API version 2's name for its `Amount`) is read as its
    /// amount.
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::types::TxResult;
    /// use xrpl_core::transaction::models::Tx;
    /// use xrpl_core::types::Amount;
    ///
    /// let answer: TxResult = serde_json::from_value(json!({
    ///     "hash": "D081D150B4132DE0C907ECF90DA9087EFC3F2D95EAB3D42ED59910B67BAD7D51",
    ///     "tx_json": {
    ///         "TransactionType": "Payment", "Account": "rUuF7Y6ENRCKCK9uiV5W8DNV4cPhDGkbmc",
    ///         "Destination": "rPzCAtF7ujdg4ME5Hcozum9d2MzE9rSrng", "DestinationTag": 702456,
    ///         "DeliverMax": "480000000", "Fee": "10", "Sequence": 105033442
    ///     },
    ///     "ledger_index": 107831520, "validated": true
    /// }))?;
    /// let Tx::Payment(payment) = answer.transaction()? else { panic!("not a Payment") };
    /// assert_eq!(payment.amount, Amount::Xrp(480_000_000)); // from `DeliverMax`
    /// assert_eq!(payment.destination_tag, Some(702456));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`CoreError::ValidationError`] when the answer has no `tx_json` (a
    /// binary one), and the errors of [`Tx::from_value`] for a transaction
    /// type or field this version does not know.
    pub fn transaction(&self) -> Result<Tx, CoreError> {
        let mut tx = self
            .tx_json
            .clone()
            .ok_or_else(|| CoreError::ValidationError("no tx_json (a binary answer)".into()))?;
        if let Some(fields) = tx.as_object_mut() {
            // Written into tx_json by `account_tx`, not fields of the transaction.
            for key in ["ctid", "ledger_index"] {
                fields.remove(key);
            }
            if let (Some(hash), false) = (&self.hash, fields.contains_key("hash")) {
                fields.insert("hash".into(), Value::String(hash.clone()));
            }
        }
        Tx::from_value(tx)
    }
}

impl SubmitResult {
    /// The transaction's ID, wherever the method wrote it (`hash` in
    /// `tx_json` for `submit`, beside it for `submit_multisigned`). The ID
    /// can also be computed before submitting
    /// (`xrpl_core::hashes::hash_signed_tx_blob`).
    ///
    /// ```
    /// # use serde_json::{json, Value};
    /// # use xrpl_client::types::SubmitResult;
    /// # let answer = |fields: Value| -> serde_json::Result<SubmitResult> {
    /// #     let mut v = json!({"engine_result": "tesSUCCESS", "engine_result_code": 0,
    /// #         "engine_result_message": "The transaction was applied. Only final in a validated ledger."});
    /// #     v.as_object_mut().unwrap().extend(fields.as_object().unwrap().clone());
    /// #     serde_json::from_value(v)
    /// # };
    /// // `answer(…)`: a `tesSUCCESS` answer with these fields.
    /// let id = "14AF7EE2124F1C5177130EDE299CF841CEEA7164C7A9FE263922A2BB03DFF3E4";
    /// // `submit` writes the hash in `tx_json`, `submit_multisigned` beside it.
    /// assert_eq!(answer(json!({"tx_json": {"hash": id}}))?.hash(), Some(id));
    /// assert_eq!(answer(json!({"hash": id, "tx_json": {}}))?.hash(), Some(id));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn hash(&self) -> Option<&str> {
        self.hash
            .as_deref()
            .or_else(|| self.tx_json.as_ref()?.get("hash").and_then(Value::as_str))
    }
}

/// [`simulate`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/simulate):
/// what a transaction would do in the current open ledger, without
/// submitting it (rippled 2.4+). Nothing is signed or sent to the network.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::SimulateResult;
///
/// let simulated: SimulateResult = serde_json::from_value(json!({
///     "applied": false, "engine_result": "tesSUCCESS", "engine_result_code": 0,
///     "engine_result_message": "The simulated transaction would have been applied.",
///     "ledger_index": 107856072,
///     "meta": {
///         "AffectedNodes": [], "TransactionIndex": 223, "TransactionResult": "tesSUCCESS",
///         "delivered_amount": "1"
///     }
/// }))?;
/// assert_eq!(simulated.engine_result, "tesSUCCESS");
/// assert!(!simulated.applied); // nothing was submitted
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
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
    /// The metadata it would produce.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<TransactionMetadata>,
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
