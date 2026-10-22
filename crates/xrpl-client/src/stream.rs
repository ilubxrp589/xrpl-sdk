//! Typed messages of `rippled`'s subscription streams, as [`WsEvent`]s.
//!
//! | Stream (`subscribe`) | Message `type` | Event |
//! |---|---|---|
//! | `ledger` | `ledgerClosed` | [`WsEvent::Ledger`] |
//! | `transactions`, `transactions_proposed`, `accounts`, `accounts_proposed`, `books` | `transaction` | [`WsEvent::Transaction`] |
//! | `validations` | `validationReceived` | [`WsEvent::Validation`] |
//! | `server` | `serverStatus` | [`WsEvent::Server`] |
//! | `consensus` | `consensusPhase` | [`WsEvent::Consensus`] |
//! | `book_changes` | `bookChanges` | [`WsEvent::BookChanges`] |
//! | `manifests` | `manifestReceived` | [`WsEvent::Manifest`] |
//! | `peer_status` (admin) | `peerStatusChange` | [`WsEvent::PeerStatus`] |
//! | `path_find` | `path_find` | [`WsEvent::PathFind`] |
//!
//! Each event type is checked against messages recorded from `rippled` 3.4.1
//! on mainnet (`tests/stream_events.rs`): it must parse, and keep every field.

use crate::types::{count_as_string, BookChange, BookOffer};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use xrpl_core::transaction::models::Tx;

/// One message from a subscription, or a connection event.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum WsEvent {
    /// A ledger closed and was validated (`ledger` stream).
    Ledger(LedgerEvent),
    /// A transaction, proposed or validated (`transactions`, `accounts`, `books`, …).
    Transaction(Box<TransactionEvent>),
    /// A validator's validation (`validations` stream).
    Validation(ValidationEvent),
    /// The server's load and fee changed (`server` stream).
    Server(ServerStatusEvent),
    /// The consensus round moved to a new phase (`consensus` stream).
    Consensus(ConsensusEvent),
    /// Order-book trading in a validated ledger (`book_changes` stream).
    BookChanges(BookChangesEvent),
    /// A validator published a new manifest (`manifests` stream).
    Manifest(ManifestEvent),
    /// A peer's ledger state changed (`peer_status` stream, admin).
    PeerStatus(PeerStatusEvent),
    /// An update of an open `path_find` request.
    PathFind(Value),
    /// A message of a type this crate does not know, as JSON.
    Other(Value),
    /// The connection dropped and was re-established; subscriptions were
    /// restored.
    Reconnected,
}

impl WsEvent {
    /// The event a stream message (an object with a `type`) carries; `None`
    /// for anything else (such as a reply to a request).
    pub fn from_message(message: Value) -> Option<WsEvent> {
        let kind = message.get("type").and_then(Value::as_str)?.to_owned();
        fn typed<T: serde::de::DeserializeOwned>(
            m: &Value,
            wrap: impl Fn(T) -> WsEvent,
        ) -> WsEvent {
            serde_json::from_value(m.clone())
                .map(wrap)
                .unwrap_or_else(|_| WsEvent::Other(m.clone()))
        }
        Some(match kind.as_str() {
            "ledgerClosed" => typed(&message, WsEvent::Ledger),
            "transaction" => typed(&message, |t| WsEvent::Transaction(Box::new(t))),
            "validationReceived" => typed(&message, WsEvent::Validation),
            "serverStatus" => typed(&message, WsEvent::Server),
            "consensusPhase" => typed(&message, WsEvent::Consensus),
            "bookChanges" => typed(&message, WsEvent::BookChanges),
            "manifestReceived" => typed(&message, WsEvent::Manifest),
            "peerStatusChange" => typed(&message, WsEvent::PeerStatus),
            "path_find" => WsEvent::PathFind(message),
            _ => WsEvent::Other(message),
        })
    }
}

/// `ledgerClosed`: a ledger was closed and validated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerEvent {
    pub ledger_index: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    /// Ledger time (seconds since 2000-01-01).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_time: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub txn_count: Option<u32>,
    /// The reference transaction cost, in drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_base: Option<u64>,
    /// The account reserve, in drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve_base: Option<u64>,
    /// The reserve per owned object, in drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve_inc: Option<u64>,
    /// The ledgers the server has, e.g. `"32570-107831520"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated_ledgers: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_id: Option<u32>,
}

/// `transaction`: a transaction, typed, with its result. A proposed one
/// (`validated: false`) is only a preliminary result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionEvent {
    /// The transaction, typed by its `TransactionType` (see
    /// `xrpl_core::transaction::models`).
    pub transaction: Tx,
    pub engine_result: String,
    pub engine_result_code: i32,
    pub engine_result_message: String,
    /// `"closed"` (validated) or `"proposed"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub validated: bool,
    /// The metadata, once validated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    /// The open ledger (proposed transactions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time_iso: Option<String>,
    /// The compact transaction ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ctid: Option<String>,
    /// `account_history_tx_stream` subscriptions only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_history_tx_index: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_history_tx_first: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_history_boundary: Option<bool>,
}

/// `validationReceived`: a validator signed a ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationEvent {
    pub ledger_hash: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "count_as_string"
    )]
    pub ledger_index: Option<u32>,
    /// The validator's ephemeral signing key (`n…`).
    pub validation_public_key: String,
    /// Its master key, when it uses a manifest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub master_key: Option<String>,
    pub signature: String,
    /// Ledger time the validation was signed.
    pub signing_time: u64,
    pub flags: u32,
    /// A full validation (not partial).
    pub full: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cookie: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_id: Option<u32>,
    /// The serialized validation, hex (admin connections).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Amendments the validator votes for (flag ledgers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amendments: Option<Vec<String>>,
    /// Fee votes (flag ledgers), in drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_fee: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve_base: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve_inc: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_fee: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_version: Option<String>,
}

/// `serverStatus`: the server's load or fee changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerStatusEvent {
    /// `full`, `syncing`, …
    pub server_status: String,
    /// The reference transaction cost, in drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_fee: Option<u64>,
    /// The fee level with no load (256).
    pub load_base: u64,
    pub load_factor: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_escalation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_queue: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_reference: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_server: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_id: Option<u32>,
}

/// `consensusPhase`: `"open"`, `"establish"` or `"accepted"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusEvent {
    pub consensus: String,
}

/// `bookChanges`: each order book's open, high, low, close and volume in a
/// validated ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookChangesEvent {
    pub changes: Vec<BookChange>,
    pub ledger_index: u32,
    pub ledger_hash: String,
    pub ledger_time: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// `manifestReceived`: a validator's new manifest (key rotation).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestEvent {
    pub master_key: String,
    pub master_signature: String,
    pub seq: u32,
    pub signature: String,
    pub signing_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

/// `peerStatusChange`: a peer's ledger or state changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerStatusEvent {
    /// `CLOSING_LEDGER`, `ACCEPTED_LEDGER`, `SWITCHED_LEDGER` or `LOST_SYNC`.
    pub action: String,
    /// Ledger time of the change.
    pub date: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index_min: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index_max: Option<u32>,
}

/// The offers of a book when subscribing with `snapshot` (`asks` and `bids`
/// with `both`, otherwise `offers`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offers: Option<Vec<BookOffer>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asks: Option<Vec<BookOffer>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bids: Option<Vec<BookOffer>>,
}
