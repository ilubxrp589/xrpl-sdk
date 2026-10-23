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

use crate::types::{count_as_string, BookChange, BookOffer, PathFindResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use xrpl_core::transaction::{models::Tx, TransactionMetadata};

/// One message from a subscription, or a connection event.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::stream::WsEvent;
///
/// let ledger = WsEvent::from_message(json!({"type": "ledgerClosed", "ledger_index": 7}));
/// assert!(matches!(ledger, Some(WsEvent::Ledger(l)) if l.ledger_index == 7));
/// // A type this crate does not know is kept as JSON.
/// let new = WsEvent::from_message(json!({"type": "somethingNew"}));
/// assert!(matches!(new, Some(WsEvent::Other(_))));
/// // One it knows that does not parse is reported, not dropped.
/// let bad = WsEvent::from_message(json!({"type": "ledgerClosed"}));
/// assert!(matches!(bad, Some(WsEvent::Unparsed { kind, .. }) if kind == "ledgerClosed"));
/// ```
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
    /// An update of the connection's `path_find` search: the paths as the
    /// latest ledger has them.
    PathFind(Box<PathFindResult>),
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::stream::WsEvent;
    ///
    /// assert!(WsEvent::Reconnected.is_connection_event());
    /// assert!(WsEvent::Missed(3).is_connection_event());
    /// let ledger = WsEvent::from_message(json!({"type": "ledgerClosed", "ledger_index": 7}));
    /// assert!(ledger.is_some_and(|e| !e.is_connection_event()));
    /// ```
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::stream::WsEvent;
    ///
    /// let phase = WsEvent::from_message(json!({"type": "consensusPhase", "consensus": "accepted"}));
    /// assert!(matches!(phase, Some(WsEvent::Consensus(c)) if c.consensus == "accepted"));
    /// // A reply to a request is not a stream message.
    /// assert!(WsEvent::from_message(json!({"id": 1, "status": "success", "result": {}})).is_none());
    /// ```
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
                } = &t.tx_json
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
            "path_find" => typed(&message, |p: PathFindResult| WsEvent::PathFind(Box::new(p))),
            _ => WsEvent::Other(message),
        })
    }
}

/// `ledgerClosed`: a ledger was closed and validated.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::stream::WsEvent;
///
/// let message = json!({"type": "ledgerClosed", "ledger_index": 107856091, "ledger_time": 846100681,
///     "ledger_hash": "5514A59D56A2BD2E8EC70C4229586AD1866C189B6945C371DF06AAB8FF6C83D5",
///     "txn_count": 377, "fee_base": 10, "reserve_base": 1000000, "reserve_inc": 200000,
///     "validated_ledgers": "107436647-107856091", "network_id": 0});
/// let Some(WsEvent::Ledger(ledger)) = WsEvent::from_message(message) else { panic!() };
/// assert_eq!((ledger.ledger_index, ledger.txn_count), (107856091, Some(377)));
/// assert_eq!(ledger.reserve_base, Some(1_000_000)); // 1 XRP to hold an account
/// ```
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
///
/// ```
/// use xrpl_client::stream::WsEvent;
///
/// # let message = serde_json::json!({"type": "transaction", "validated": true, "ledger_index": 5,
/// #     "engine_result": "tesSUCCESS", "engine_result_code": 0, "engine_result_message": "The transaction was applied.",
/// #     "tx_json": {"TransactionType": "Payment", "Account": "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD",
/// #         "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "DeliverMax": "1000000", "Fee": "12",
/// #         "Sequence": 1, "SigningPubKey": "", "Flags": 0},
/// #     "meta": {"TransactionResult": "tesSUCCESS", "TransactionIndex": 0, "AffectedNodes": [],
/// #         "delivered_amount": "1000000"}});
/// // A validated Payment of 1 XRP from rLUEXYuL… to rPT1Sjq2…, as a stream sends it.
/// let Some(WsEvent::Transaction(tx)) = WsEvent::from_message(message) else { panic!() };
/// assert!(tx.validated);
/// assert_eq!(tx.engine_result, "tesSUCCESS");
/// assert_eq!(tx.ledger_index, Some(5));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TransactionEvent {
    /// The transaction, typed by its `TransactionType` (see
    /// `xrpl_core::transaction::models`) (`tx_json`).
    pub tx_json: Tx,
    /// The transaction's ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
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
    /// What it did, once validated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<TransactionMetadata>,
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
    ///
    /// ```
    /// use xrpl_client::stream::WsEvent;
    /// use xrpl_core::types::Amount;
    ///
    /// # let message = serde_json::json!({"type": "transaction", "validated": true, "ledger_index": 5,
    /// #     "engine_result": "tesSUCCESS", "engine_result_code": 0, "engine_result_message": "The transaction was applied.",
    /// #     "tx_json": {"TransactionType": "Payment", "Account": "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD",
    /// #         "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "DeliverMax": "1000000", "Fee": "12",
    /// #         "Sequence": 1, "SigningPubKey": "", "Flags": 0},
    /// #     "meta": {"TransactionResult": "tesSUCCESS", "TransactionIndex": 0, "AffectedNodes": [],
    /// #         "delivered_amount": "1000000"}});
    /// // A validated Payment that delivered 1 XRP.
    /// let Some(WsEvent::Transaction(tx)) = WsEvent::from_message(message) else { panic!() };
    /// assert_eq!(tx.delivered_amount(), Some(Amount::xrp(1_000_000)?));
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn delivered_amount(&self) -> Option<xrpl_core::types::Amount> {
        xrpl_core::utils::delivered_amount(self.meta.as_ref()?)
    }

    /// Whether `account` (a classic address) appears in the transaction or
    /// its metadata (the accounts `rippled` sends an `accounts` subscriber
    /// the transaction for: sender, destination, issuers, every account
    /// whose entries it changed).
    ///
    /// ```
    /// use xrpl_client::stream::WsEvent;
    ///
    /// # let message = serde_json::json!({"type": "transaction", "validated": true, "ledger_index": 5,
    /// #     "engine_result": "tesSUCCESS", "engine_result_code": 0, "engine_result_message": "The transaction was applied.",
    /// #     "tx_json": {"TransactionType": "Payment", "Account": "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD",
    /// #         "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "DeliverMax": "1000000", "Fee": "12",
    /// #         "Sequence": 1, "SigningPubKey": "", "Flags": 0},
    /// #     "meta": {"TransactionResult": "tesSUCCESS", "TransactionIndex": 0, "AffectedNodes": [],
    /// #         "delivered_amount": "1000000"}});
    /// // A Payment from rLUEXYuL… to rPT1Sjq2….
    /// let Some(WsEvent::Transaction(tx)) = WsEvent::from_message(message) else { panic!() };
    /// assert!(tx.affects("rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD"));
    /// assert!(tx.affects("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"));
    /// assert!(!tx.affects("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"));
    /// ```
    pub fn affects(&self, account: &str) -> bool {
        fn names(v: &Value, account: &str) -> bool {
            match v {
                Value::String(s) => s == account,
                Value::Array(a) => a.iter().any(|v| names(v, account)),
                Value::Object(o) => o.values().any(|v| names(v, account)),
                _ => false,
            }
        }
        self.tx_json.to_value().is_ok_and(|tx| names(&tx, account))
            || self
                .meta
                .as_ref()
                .and_then(|m| serde_json::to_value(m).ok())
                .is_some_and(|m| names(&m, account))
    }
}

/// `validationReceived`: a validator signed a ledger.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::stream::WsEvent;
///
/// let message = json!({"type": "validationReceived", "full": true, "flags": 0x8000_0001u32,
///     "ledger_hash": "58429A00D59B798B62C1A9C426358699F8216A3795387CBA197B6D779561D4DC",
///     "ledger_index": 107856090, "signing_time": 846100681, "network_id": 0,
///     "validation_public_key": "n9KXyhPmB1QDrWLo9Ut4WhWho9iNn5Si9igmnTfXHDfKLPvsSBvS",
///     "signature": "304402207E8491BB957E052268183497D25EB54D6FF1AD377DBE161D4657A296C48274F802204044E77DF265CAEAF2A27BBFC19A2C5FA3B3E2F023BCC17F9FAD0F9BB6D988A1"});
/// let Some(WsEvent::Validation(v)) = WsEvent::from_message(message) else { panic!() };
/// assert_eq!(v.ledger_index, Some(107856090));
/// assert!(v.full);
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::stream::WsEvent;
///
/// let message = json!({"type": "serverStatus", "server_status": "full", "base_fee": 10,
///     "load_base": 256, "load_factor": 128333, "load_factor_fee_escalation": 128333,
///     "load_factor_fee_queue": 256, "load_factor_fee_reference": 256, "load_factor_server": 256});
/// let Some(WsEvent::Server(status)) = WsEvent::from_message(message) else { panic!() };
/// // What a reference transaction costs to get into the open ledger now:
/// // the base fee × load_factor / load_base.
/// let drops = status.base_fee.unwrap_or(10) * status.load_factor / status.load_base;
/// assert_eq!(drops, 5013);
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::stream::WsEvent;
///
/// let message = json!({"type": "consensusPhase", "consensus": "establish"});
/// let Some(WsEvent::Consensus(phase)) = WsEvent::from_message(message) else { panic!() };
/// assert_eq!(phase.consensus, "establish");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConsensusEvent {
    /// The consensus phase (`open`, `establish`, `accepted`).
    pub consensus: String,
}

/// `bookChanges`: each order book's open, high, low, close and volume in a
/// validated ledger.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::stream::WsEvent;
///
/// let message = json!({"type": "bookChanges", "validated": true, "ledger_index": 107856092, "ledger_time": 846100690,
///     "ledger_hash": "4A98DEE7F8D18C9865761A37D28B14E18C7856BB5E31623C7BBAA879793F0B7C",
///     "changes": [{"currency_a": "XRP_drops",
///         "currency_b": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De/524C555344000000000000000000000000000000",
///         "volume_a": "4902965822", "volume_b": "6381.896062861999", "open": "768261.6957471645",
///         "high": "768403.5", "low": "768261.5776082667", "close": "768403.5"}]});
/// let Some(WsEvent::BookChanges(trading)) = WsEvent::from_message(message) else { panic!() };
/// assert_eq!(trading.changes.len(), 1);
/// assert_eq!(trading.changes[0].volume_a, "4902965822"); // drops of XRP traded
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::stream::WsEvent;
///
/// let message = json!({"type": "manifestReceived", "seq": 2, "domain": "ripple.ittc.ku.edu",
///     "master_key": "nHUVPzAmAmQ2QSc4oE1iLfsGi17qN2ado8PhxvgEkou76FLxAz7C",
///     "signing_key": "n9JeA5Q54JhUQYHieb2j7ZTCg9RTBagDkam3UP2kWQxTXfxFkA8R",
///     "signature": "304402200296DCF23518D44E47B28D3722C139399AB87C2504F19CCCF127B3733089569E022015116F6EA2B131A73557E49FB689F27E15823EB4F4E599925C0C6CCAF1AE4A56",
///     "master_signature": "10B6C15F942827A98FAF41AB5EC02A358B7DF6D1288A719C68BEF11333FF9CEAC48E4FE6D659086C993DCC87B549C7330D4FECBA48AA66423B500DCB3011C601"});
/// let Some(WsEvent::Manifest(manifest)) = WsEvent::from_message(message) else { panic!() };
/// assert_eq!(manifest.seq, 2);
/// assert_eq!(manifest.domain.as_deref(), Some("ripple.ittc.ku.edu"));
/// ```
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
    /// The domain the validator claims, as text (`example.com`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

/// `peerStatusChange`: a peer's ledger or state changed.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::stream::WsEvent;
///
/// let message = json!({"type": "peerStatusChange", "action": "ACCEPTED_LEDGER", "date": 846100681,
///     "ledger_hash": "58429A00D59B798B62C1A9C426358699F8216A3795387CBA197B6D779561D4DC",
///     "ledger_index": 107856090, "ledger_index_min": 107855833, "ledger_index_max": 107856089});
/// let Some(WsEvent::PeerStatus(peer)) = WsEvent::from_message(message) else { panic!() };
/// assert_eq!(peer.action, "ACCEPTED_LEDGER");
/// assert_eq!(peer.ledger_index, Some(107856090));
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::stream::OrderBookSnapshot;
///
/// // The reply to a `books` subscription with `snapshot` and `both`.
/// let reply = json!({"bids": [], "asks": [{"Account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "Flags": 0,
///     "Sequence": 7, "TakerPays": "5000000",
///     "TakerGets": {"currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "value": "10"}}]});
/// let snapshot: OrderBookSnapshot = serde_json::from_value(reply)?;
/// assert_eq!(snapshot.asks.map(|a| a.len()), Some(1));
/// assert_eq!(snapshot.offers, None); // only without `both`
/// # Ok::<(), serde_json::Error>(())
/// ```
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
