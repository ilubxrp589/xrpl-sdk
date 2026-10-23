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
    /// A message of a type this crate knows that did not parse (a field a
    /// newer `rippled` added, for instance): its type, why, and the message.
    /// Nothing is dropped silently; it is also logged at WARN.
    Unparsed {
        /// The message's `type`.
        kind: String,
        /// Why it did not parse.
        error: String,
        /// The message, as JSON.
        message: Value,
    },
    /// The connection dropped; the client is reconnecting. Events until
    /// [`WsEvent::Reconnected`] are not seen.
    Disconnected,
    /// The connection was re-established and every subscription asked for
    /// again (a refused one is a [`WsEvent::ResubscribeFailed`] first). A
    /// restored stream's first events can arrive just before this event, as
    /// the server may send them as soon as it has taken the subscription.
    Reconnected,
    /// After a reconnect, the server refused to restore this subscription.
    ResubscribeFailed {
        /// The subscription's parameters.
        subscription: Value,
        /// The server's answer.
        error: String,
    },
    /// This receiver fell behind and this many events were dropped for it.
    Missed(u64),
}

impl WsEvent {
    /// Whether this is about the connection, not a stream: every
    /// subscription receives these.
    pub fn is_connection_event(&self) -> bool {
        matches!(
            self,
            WsEvent::Disconnected
                | WsEvent::Reconnected
                | WsEvent::ResubscribeFailed { .. }
                | WsEvent::Missed(_)
        )
    }

    /// The event a stream message (an object with a `type`) carries; `None`
    /// for anything else (such as a reply to a request).
    pub fn from_message(message: Value) -> Option<WsEvent> {
        let kind = message.get("type").and_then(Value::as_str)?.to_owned();
        fn typed<T: serde::de::DeserializeOwned>(
            m: &Value,
            wrap: impl Fn(T) -> WsEvent,
        ) -> WsEvent {
            match serde_json::from_value(m.clone()) {
                Ok(t) => wrap(t),
                Err(e) => {
                    let kind = m.get("type").and_then(Value::as_str).unwrap_or("?");
                    tracing::warn!(kind, error = %e, "a stream message did not parse");
                    WsEvent::Unparsed {
                        kind: kind.to_string(),
                        error: e.to_string(),
                        message: m.clone(),
                    }
                }
            }
        }
        Some(match kind.as_str() {
            "ledgerClosed" => typed(&message, WsEvent::Ledger),
            "transaction" => typed(&message, |t: TransactionEvent| {
                if let Tx::Unparsed {
                    transaction_type,
                    error,
                    ..
                } = &t.transaction
                {
                    tracing::warn!(%transaction_type, %error, "a streamed transaction did not parse as its type");
                }
                WsEvent::Transaction(Box::new(t))
            }),
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
#[non_exhaustive]
pub struct LedgerEvent {
    /// The ledger the answer is from.
    pub ledger_index: u32,
    /// That ledger's hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    /// Ledger time (seconds since 2000-01-01).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_time: Option<u64>,
    /// Transactions in the ledger.
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
    /// The network's ID (0 mainnet, 1 Testnet, 2 Devnet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_id: Option<u32>,
}

/// `transaction`: a transaction, typed, with its result. A proposed one
/// (`validated: false`) is only a preliminary result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TransactionEvent {
    /// The transaction, typed by its `TransactionType` (see
    /// `xrpl_core::transaction::models`).
    pub transaction: Tx,
    /// The engine result (`tesSUCCESS`, `tecPATH_DRY`, …).
    pub engine_result: String,
    /// The engine result as a number.
    pub engine_result_code: i32,
    /// The engine result in words.
    pub engine_result_message: String,
    /// `"closed"` (validated) or `"proposed"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Whether the data is from a validated ledger (final).
    pub validated: bool,
    /// The metadata, once validated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
    /// The ledger the answer is from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    /// That ledger's hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    /// The open ledger (proposed transactions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    /// When the ledger closed, ISO 8601.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time_iso: Option<String>,
    /// The compact transaction ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ctid: Option<String>,
    /// `account_history_tx_stream` subscriptions only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_history_tx_index: Option<i64>,
    /// Whether this is the account's first transaction (history streams).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_history_tx_first: Option<bool>,
    /// Whether this marks the end of the history sent (history streams).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_history_boundary: Option<bool>,
}

impl TransactionEvent {
    /// What the transaction delivered (`delivered_amount`), once validated:
    /// check this, not a Payment's `Amount`, which a partial payment does
    /// not reach.
    pub fn delivered_amount(&self) -> Option<xrpl_core::types::Amount> {
        xrpl_core::utils::delivered_amount(self.meta.as_ref()?)
    }

    /// Whether `account` (a classic address) appears in the transaction or
    /// its metadata (the accounts `rippled` sends an `accounts` subscriber
    /// the transaction for: sender, destination, issuers, every account
    /// whose entries it changed).
    pub fn affects(&self, account: &str) -> bool {
        fn names(v: &Value, account: &str) -> bool {
            match v {
                Value::String(s) => s == account,
                Value::Array(a) => a.iter().any(|v| names(v, account)),
                Value::Object(o) => o.values().any(|v| names(v, account)),
                _ => false,
            }
        }
        self.transaction
            .to_value()
            .is_ok_and(|tx| names(&tx, account))
            || self.meta.as_ref().is_some_and(|m| names(m, account))
    }
}

/// `validationReceived`: a validator signed a ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ValidationEvent {
    /// That ledger's hash.
    pub ledger_hash: String,
    /// The ledger the answer is from.
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
    /// A signature, hex (`Signature`).
    pub signature: String,
    /// Ledger time the validation was signed.
    pub signing_time: u64,
    /// The validation's flags.
    pub flags: u32,
    /// A full validation (not partial).
    pub full: bool,
    /// The hash of the latest ledger the validator considers validated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated_hash: Option<String>,
    /// A number the validator picks at startup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cookie: Option<String>,
    /// The network's ID (0 mainnet, 1 Testnet, 2 Devnet).
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
    /// The account reserve, in drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve_base: Option<u64>,
    /// The reserve per owned entry, in drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve_inc: Option<u64>,
    /// The validator's load fee vote.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_fee: Option<u64>,
    /// The validator's software version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_version: Option<String>,
}

/// `serverStatus`: the server's load or fee changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ServerStatusEvent {
    /// `full`, `syncing`, …
    pub server_status: String,
    /// The reference transaction cost, in drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_fee: Option<u64>,
    /// The fee level with no load (256).
    pub load_base: u64,
    /// The load multiplier on the transaction cost.
    pub load_factor: u64,
    /// The open-ledger fee escalation, as a fee level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_escalation: Option<u64>,
    /// The fee level to get into the queue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_queue: Option<u64>,
    /// The reference fee level (256).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_reference: Option<u64>,
    /// The server's load factor, as a fee level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_server: Option<u64>,
    /// The network's ID (0 mainnet, 1 Testnet, 2 Devnet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_id: Option<u32>,
}

/// `consensusPhase`: `"open"`, `"establish"` or `"accepted"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConsensusEvent {
    /// The consensus phase (`open`, `establish`, `accepted`).
    pub consensus: String,
}

/// `bookChanges`: each order book's open, high, low, close and volume in a
/// validated ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BookChangesEvent {
    /// Each book's trading.
    pub changes: Vec<BookChange>,
    /// The ledger the answer is from.
    pub ledger_index: u32,
    /// That ledger's hash.
    pub ledger_hash: String,
    /// When the ledger closed (ledger time).
    pub ledger_time: u64,
    /// Whether the data is from a validated ledger (final).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// `manifestReceived`: a validator's new manifest (key rotation).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ManifestEvent {
    /// The validator's master key.
    pub master_key: String,
    /// The manifest's signature by the master key.
    pub master_signature: String,
    /// The sequence of the transaction that created it.
    pub seq: u32,
    /// A signature, hex (`Signature`).
    pub signature: String,
    /// The new signing key.
    pub signing_key: String,
    /// A domain the account claims, hex (ASCII, lower case) (`Domain`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

/// `peerStatusChange`: a peer's ledger or state changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PeerStatusEvent {
    /// `CLOSING_LEDGER`, `ACCEPTED_LEDGER`, `SWITCHED_LEDGER` or `LOST_SYNC`.
    pub action: String,
    /// Ledger time of the change.
    pub date: u64,
    /// That ledger's hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    /// The ledger the answer is from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    /// The earliest ledger searched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index_min: Option<u32>,
    /// The latest ledger searched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index_max: Option<u32>,
}

/// The offers of a book when subscribing with `snapshot` (`asks` and `bids`
/// with `both`, otherwise `offers`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OrderBookSnapshot {
    /// The offers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offers: Option<Vec<BookOffer>>,
    /// Offers selling the book's first asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asks: Option<Vec<BookOffer>>,
    /// Offers buying it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bids: Option<Vec<BookOffer>>,
}
