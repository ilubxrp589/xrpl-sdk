//! Typed requests for every public `rippled` method, and the Clio-only ones.
//!
//! Each request names its method and the type its `result` decodes into
//! ([`Request`]); [`crate::Client::request`] sends any of them over HTTP or
//! WebSocket alike, and [`crate::Client::request_all`] follows `marker`
//! through every page of a [`Paginated`] one. Each has a `new` with its
//! required parameters; the optional ones are public fields that start unset.
//!
//! | Group | Requests |
//! |---|---|
//! | Account | [`AccountChannelsRequest`], [`AccountCurrenciesRequest`], [`AccountInfoRequest`], [`AccountLinesRequest`], [`AccountNftsRequest`], [`AccountObjectsRequest`], [`AccountOffersRequest`], [`AccountTxRequest`], [`GatewayBalancesRequest`], [`NoRippleCheckRequest`] |
//! | Ledger | [`LedgerRequest`], [`LedgerClosedRequest`], [`LedgerCurrentRequest`], [`LedgerDataRequest`], [`LedgerEntryRequest`] |
//! | Transaction | [`SimulateRequest`], [`SubmitRequest`], [`SubmitMultisignedRequest`], [`TransactionEntryRequest`], [`TxRequest`] |
//! | Path and order book | [`AmmInfoRequest`], [`BookChangesRequest`], [`BookOffersRequest`], [`DepositAuthorizedRequest`], [`GetAggregatePriceRequest`], [`NftBuyOffersRequest`], [`NftSellOffersRequest`], [`RipplePathFindRequest`] |
//! | Payment channel | [`ChannelVerifyRequest`] |
//! | Server info | [`FeeRequest`], [`FeatureRequest`], [`ManifestRequest`], [`ServerDefinitionsRequest`], [`ServerInfoRequest`], [`ServerStateRequest`], [`VersionRequest`] |
//! | Utility | [`PingRequest`], [`RandomRequest`] |
//! | Vault | [`VaultInfoRequest`] |
//! | Clio only | [`NftInfoRequest`], [`NftHistoryRequest`], [`NftsByIssuerRequest`], [`MptHoldersRequest`] |
//!
//! Not here on purpose: `channel_authorize` and `wallet_propose` send a
//! secret to the server (sign claims and make keys locally with
//! `xrpl_core::crypto`); `path_find` and `subscribe` are WebSocket streams
//! ([`crate::XrplWsClient`]).
//!
//! ```
//! use xrpl_client::requests::{AccountLinesRequest, Request};
//!
//! let req = AccountLinesRequest { limit: Some(100), ..AccountLinesRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh") };
//! assert_eq!(AccountLinesRequest::METHOD, "account_lines");
//! assert_eq!(
//!     serde_json::to_value(&req).unwrap(),
//!     serde_json::json!({"account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "ledger_index": "validated", "limit": 100}),
//! );
//! ```

use crate::types::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

/// A typed `rippled` request: the method it calls and the type of its `result`.
pub trait Request: Serialize {
    /// The `rippled` method (`account_info`, `fee`, …).
    const METHOD: &'static str;
    /// What the response's `result` deserializes into.
    type Response: DeserializeOwned;
}

/// A request whose response pages with `marker` (see [`crate::Client::request_all`]).
pub trait Paginated: Request + Clone {
    /// One entry of a page.
    type Item;
    /// This request, asking for the page after `marker`.
    fn with_marker(self, marker: Value) -> Self;
    /// A page's entries and the marker of the next page, if any.
    fn page(response: Self::Response) -> (Vec<Self::Item>, Option<Value>);
    /// The ledger a page is from, when the request names a ledger: a
    /// marker is only good for the ledger it came from, so the pages after
    /// the first ask for that same ledger.
    fn ledger_of(_response: &Self::Response) -> Option<u32> {
        None
    }
    /// This request, at ledger `index`.
    fn at_ledger(self, _index: u32) -> Self {
        self
    }
}

macro_rules! request {
    ($req:ident => $method:literal, $resp:ty) => {
        impl Request for $req {
            const METHOD: &'static str = $method;
            type Response = $resp;
        }
    };
}

macro_rules! paginated {
    ($req:ident, $item:ty, $field:ident) => {
        impl Paginated for $req {
            type Item = $item;
            fn with_marker(self, marker: Value) -> Self {
                Self {
                    marker: Some(marker),
                    ..self
                }
            }
            fn page(response: Self::Response) -> (Vec<$item>, Option<Value>) {
                (response.$field, response.marker)
            }
        }
    };
    ($req:ident, $item:ty, $field:ident, pinned $(, $current:ident)?) => {
        impl Paginated for $req {
            type Item = $item;
            fn with_marker(self, marker: Value) -> Self {
                Self {
                    marker: Some(marker),
                    ..self
                }
            }
            fn page(response: Self::Response) -> (Vec<$item>, Option<Value>) {
                (response.$field, response.marker)
            }
            fn ledger_of(response: &Self::Response) -> Option<u32> {
                response.ledger_index $(.or(response.$current))?
            }
            fn at_ledger(self, index: u32) -> Self {
                Self {
                    ledger_index: LedgerIndex::Index(index),
                    ..self
                }
            }
        }
    };
}

/// An oracle to read in [`GetAggregatePriceRequest`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct OracleId {
    /// The account that sends the transaction or owns the entry (`Account`).
    pub account: String,
    /// The oracle's document ID.
    pub oracle_document_id: u32,
}

// ---------------------------------------------------------------- account

/// [`account_channels`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_channels):
/// payment channels the account is the source of.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AccountChannelsRequest {
    /// The account that sends the transaction or owns the entry (`Account`).
    pub account: String,
    /// Only channels to this destination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_account: Option<String>,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
    /// The most entries per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
impl AccountChannelsRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }
}
request!(AccountChannelsRequest => "account_channels", AccountChannelsResult);
paginated!(
    AccountChannelsRequest,
    PaymentChannel,
    channels,
    pinned,
    ledger_current_index
);

/// [`account_currencies`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_currencies):
/// currencies the account can send and receive.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AccountCurrenciesRequest {
    /// The account that sends the transaction or owns the entry (`Account`).
    pub account: String,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
}
impl AccountCurrenciesRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }
}
request!(AccountCurrenciesRequest => "account_currencies", AccountCurrenciesResult);

/// [`account_info`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_info):
/// an account's settings and XRP balance.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AccountInfoRequest {
    /// The account that sends the transaction or owns the entry (`Account`).
    pub account: String,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
    /// Include the account's signer lists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signer_lists: Option<bool>,
    /// Include queued transactions (needs `LedgerIndex::Current`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue: Option<bool>,
}
impl AccountInfoRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }
}
request!(AccountInfoRequest => "account_info", AccountInfo);

/// [`account_lines`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_lines):
/// an account's trust lines.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AccountLinesRequest {
    /// The account that sends the transaction or owns the entry (`Account`).
    pub account: String,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
    /// Only the line with this peer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer: Option<String>,
    /// 10–400 per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
impl AccountLinesRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }
}
request!(AccountLinesRequest => "account_lines", AccountLinesResult);
paginated!(
    AccountLinesRequest,
    TrustLine,
    lines,
    pinned,
    ledger_current_index
);

/// [`account_nfts`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_nfts):
/// the NFTs an account holds.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AccountNftsRequest {
    /// The account that sends the transaction or owns the entry (`Account`).
    pub account: String,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
    /// 20–400 per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
impl AccountNftsRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }
}
request!(AccountNftsRequest => "account_nfts", AccountNftsResult);
paginated!(
    AccountNftsRequest,
    NFToken,
    account_nfts,
    pinned,
    ledger_current_index
);

/// [`account_objects`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_objects):
/// the ledger entries an account owns.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AccountObjectsRequest {
    /// The account that sends the transaction or owns the entry (`Account`).
    pub account: String,
    /// Only entries of this type (`offer`, `state`, `check`, `escrow`,
    /// `payment_channel`, `nft_offer`, `ticket`, `did`, `oracle`, `mptoken`, …).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    /// Only entries that block deleting the account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletion_blockers_only: Option<bool>,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
    /// 10–400 per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
impl AccountObjectsRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }
}
request!(AccountObjectsRequest => "account_objects", AccountObjectsResult);
paginated!(
    AccountObjectsRequest,
    LedgerObject,
    account_objects,
    pinned,
    ledger_current_index
);

/// [`account_offers`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_offers):
/// an account's open DEX offers.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AccountOffersRequest {
    /// The account that sends the transaction or owns the entry (`Account`).
    pub account: String,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
    /// 10–400 per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
impl AccountOffersRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }
}
request!(AccountOffersRequest => "account_offers", AccountOffersResult);
paginated!(
    AccountOffersRequest,
    AccountOffer,
    offers,
    pinned,
    ledger_current_index
);

/// [`account_tx`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_tx):
/// transactions that affected an account. `-1` bounds mean the earliest /
/// latest validated ledger.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AccountTxRequest {
    /// The account that sends the transaction or owns the entry (`Account`).
    pub account: String,
    /// The earliest ledger searched.
    pub ledger_index_min: i32,
    /// The latest ledger searched.
    pub ledger_index_max: i32,
    /// Hex (`tx_blob`, `meta`) instead of JSON.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
    /// Oldest first.
    pub forward: bool,
    /// The most entries per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
impl Default for AccountTxRequest {
    fn default() -> Self {
        Self {
            account: String::new(),
            ledger_index_min: -1,
            ledger_index_max: -1,
            binary: None,
            forward: false,
            limit: None,
            marker: None,
        }
    }
}
impl AccountTxRequest {
    /// Every validated ledger, newest first.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }
}
request!(AccountTxRequest => "account_tx", AccountTxResult);
paginated!(AccountTxRequest, AccountTxEntry, transactions);

/// [`gateway_balances`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/gateway_balances):
/// what an issuer has issued, and to whom.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct GatewayBalancesRequest {
    /// The account that sends the transaction or owns the entry (`Account`).
    pub account: String,
    /// Operational addresses to leave out of `obligations` (listed in `balances`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hotwallet: Option<Vec<String>>,
    /// Accept only addresses, not public keys.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
}
impl GatewayBalancesRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }
}
request!(GatewayBalancesRequest => "gateway_balances", GatewayBalancesResult);

/// [`noripple_check`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/noripple_check):
/// trust-line settings that do not suit the account's role.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct NoRippleCheckRequest {
    /// The account that sends the transaction or owns the entry (`Account`).
    pub account: String,
    /// `"gateway"` (an issuer) or `"user"`.
    pub role: String,
    /// Include the transactions that fix the problems.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transactions: Option<bool>,
    /// The most entries per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
}
impl NoRippleCheckRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(account: impl Into<String>, role: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            role: role.into(),
            ..Default::default()
        }
    }
}
request!(NoRippleCheckRequest => "noripple_check", NoRippleCheckResult);

// ----------------------------------------------------------------- ledger

/// [`ledger`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger):
/// a ledger's header, and optionally its transactions.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct LedgerRequest {
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
    /// The ledger by hash instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    /// Include the transactions (hashes, or objects with `expand`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transactions: Option<bool>,
    /// Whole transactions instead of hashes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand: Option<bool>,
    /// With `expand`: hex instead of JSON.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
    /// Add `owner_funds` to OfferCreate transactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_funds: Option<bool>,
    /// Include queued transactions (current ledger only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue: Option<bool>,
}
impl LedgerRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(ledger_index: LedgerIndex) -> Self {
        Self {
            ledger_index,
            ..Default::default()
        }
    }
}
request!(LedgerRequest => "ledger", LedgerResult);

/// [`ledger_closed`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_closed):
/// the latest closed ledger.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct LedgerClosedRequest {}
request!(LedgerClosedRequest => "ledger_closed", LedgerClosedResult);

/// [`ledger_current`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_current):
/// the open ledger's index.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct LedgerCurrentRequest {}
request!(LedgerCurrentRequest => "ledger_current", LedgerCurrentResult);

/// [`ledger_data`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_data):
/// a ledger's state, a page at a time.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct LedgerDataRequest {
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
    /// That ledger's hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    /// Only entries of this type (`account`, `amm`, `offer`, `state`, …).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub entry_type: Option<String>,
    /// Hex (`data`, `index`) instead of JSON.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
    /// Up to 256 per page (2048 with `binary`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
impl LedgerDataRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(ledger_index: LedgerIndex) -> Self {
        Self {
            ledger_index,
            ..Default::default()
        }
    }
}
request!(LedgerDataRequest => "ledger_data", LedgerDataResult);
paginated!(LedgerDataRequest, LedgerObject, state, pinned);

/// [`ledger_entry`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_entry):
/// one ledger entry by ID. `xrpl_core::hashes::keylet` computes the ID of any
/// entry (account, trust line, offer, escrow, AMM, MPT, credential, …).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct LedgerEntryRequest {
    /// The ID.
    pub index: String,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
    /// Hex (`node_binary`) instead of JSON.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
    /// Clio: also find an entry that has been deleted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_deleted: Option<bool>,
}
impl LedgerEntryRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(index: impl Into<String>) -> Self {
        Self {
            index: index.into(),
            ..Default::default()
        }
    }
}
request!(LedgerEntryRequest => "ledger_entry", LedgerEntryResult);

// ------------------------------------------------------------ transaction

/// [`simulate`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/simulate):
/// what a transaction would do, without signing or submitting it. Give it
/// unsigned (no `SigningPubKey`/`TxnSignature`, or empty ones); missing
/// `Fee`, `Sequence` and the like are filled in.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct SimulateRequest {
    /// The transaction, as JSON.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_json: Option<Value>,
    /// The transaction, as hex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_blob: Option<String>,
    /// Hex (`tx_blob`, `meta_blob`) in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
}
impl SimulateRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(tx_json: Value) -> Self {
        Self {
            tx_json: Some(tx_json),
            ..Default::default()
        }
    }
}
request!(SimulateRequest => "simulate", SimulateResult);

/// [`submit`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/submit)
/// a signed blob. (Submitting with a secret, `rippled`'s sign-and-submit mode,
/// is left out: keys stay local.)
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct SubmitRequest {
    /// The transaction, as hex.
    pub tx_blob: String,
    /// Do not retry or relay it if it fails locally.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_hard: Option<bool>,
}
impl SubmitRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(tx_blob: impl Into<String>) -> Self {
        Self {
            tx_blob: tx_blob.into(),
            ..Default::default()
        }
    }
}
request!(SubmitRequest => "submit", SubmitResult);

/// [`submit_multisigned`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/submit_multisigned):
/// a multi-signed transaction as JSON (with `Signers` and an empty
/// `SigningPubKey`; see `xrpl_sdk::wallet::collect_signers`).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct SubmitMultisignedRequest {
    /// The transaction, as JSON.
    pub tx_json: Value,
    /// Do not retry or relay it if it fails locally.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_hard: Option<bool>,
}
impl SubmitMultisignedRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(tx_json: Value) -> Self {
        Self {
            tx_json,
            ..Default::default()
        }
    }
}
request!(SubmitMultisignedRequest => "submit_multisigned", SubmitResult);

/// [`transaction_entry`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/transaction_entry):
/// a transaction from one given ledger.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct TransactionEntryRequest {
    /// The transaction's hash.
    pub tx_hash: String,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
    /// That ledger's hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
}
impl TransactionEntryRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(tx_hash: impl Into<String>, ledger_index: LedgerIndex) -> Self {
        Self {
            tx_hash: tx_hash.into(),
            ledger_index,
            ledger_hash: None,
        }
    }
}
request!(TransactionEntryRequest => "transaction_entry", TransactionEntryResult);

/// [`tx`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/tx):
/// a transaction by hash (or by CTID in `transaction`'s place).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct TxRequest {
    /// The transaction's hash.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub transaction: String,
    /// Its compact ID (`C…`, 16 hex digits) instead of the hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ctid: Option<String>,
    /// Hex (`tx`, `meta`) instead of JSON.
    pub binary: bool,
    /// With `max_ledger`: the range to search, so a miss says whether the
    /// server has all of it (`searched_all`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_ledger: Option<u32>,
    /// The latest ledger to search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ledger: Option<u32>,
}
impl TxRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(hash: impl Into<String>) -> Self {
        Self {
            transaction: hash.into(),
            ..Default::default()
        }
    }
}
request!(TxRequest => "tx", TxResult);

// -------------------------------------------------- path and order book

/// [`amm_info`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/amm_info):
/// an AMM pool, by its two assets or by its account.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AmmInfoRequest {
    /// `{"currency": "XRP"}` or `{"currency", "issuer"}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<Value>,
    /// The pool's second asset (`Asset2`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset2: Option<Value>,
    /// The AMM's account, instead of the assets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amm_account: Option<String>,
    /// Include this LP's vote and auction-slot details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// The ledger the answer is from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerIndex>,
}
impl AmmInfoRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(asset: Value, asset2: Value) -> Self {
        Self {
            asset: Some(asset),
            asset2: Some(asset2),
            ..Default::default()
        }
    }
}
request!(AmmInfoRequest => "amm_info", AmmInfoResult);

/// [`book_changes`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/book_changes):
/// each order book's trading in one ledger (open, high, low, close, volume).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct BookChangesRequest {
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
}
impl BookChangesRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(ledger_index: LedgerIndex) -> Self {
        Self { ledger_index }
    }
}
request!(BookChangesRequest => "book_changes", BookChangesResult);

/// [`book_offers`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/book_offers):
/// an order book. `taker_gets` / `taker_pays` are what an offer's taker would
/// get and pay: `{"currency": "XRP"}` or `{"currency", "issuer"}`.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct BookOffersRequest {
    /// What the offer wants in return (`TakerPays`).
    pub taker_pays: Value,
    /// What the offer gives (`TakerGets`).
    pub taker_gets: Value,
    /// The most entries per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Compute funding from this account's point of view.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taker: Option<String>,
    /// A permissioned domain's book (PermissionedDEX).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// The ledger the answer is from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerIndex>,
}
impl BookOffersRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(taker_gets: Value, taker_pays: Value) -> Self {
        Self {
            taker_gets,
            taker_pays,
            ..Default::default()
        }
    }
}
request!(BookOffersRequest => "book_offers", BookOffersResult);

/// [`deposit_authorized`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/deposit_authorized):
/// whether `source_account` may pay `destination_account` directly.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct DepositAuthorizedRequest {
    /// The sending account.
    pub source_account: String,
    /// The receiving account.
    pub destination_account: String,
    /// Credential IDs the payment would present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<Vec<String>>,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
}
impl DepositAuthorizedRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(source_account: impl Into<String>, destination_account: impl Into<String>) -> Self {
        Self {
            source_account: source_account.into(),
            destination_account: destination_account.into(),
            ..Default::default()
        }
    }
}
request!(DepositAuthorizedRequest => "deposit_authorized", DepositAuthorizedResult);

/// [`get_aggregate_price`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/get_aggregate_price):
/// mean, median and spread of several oracles' prices for one pair.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct GetAggregatePriceRequest {
    /// e.g. `"XRP"`.
    pub base_asset: String,
    /// e.g. `"USD"`.
    pub quote_asset: String,
    /// Up to 200 oracles.
    pub oracles: Vec<OracleId>,
    /// Percentage of outliers to drop for `trimmed_set` (1–25).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trim: Option<u32>,
    /// Ignore prices older than this many seconds before the newest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_threshold: Option<u32>,
    /// The ledger the answer is from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<LedgerIndex>,
}
impl GetAggregatePriceRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(
        base_asset: impl Into<String>,
        quote_asset: impl Into<String>,
        oracles: Vec<OracleId>,
    ) -> Self {
        Self {
            base_asset: base_asset.into(),
            quote_asset: quote_asset.into(),
            oracles,
            ..Default::default()
        }
    }
}
request!(GetAggregatePriceRequest => "get_aggregate_price", AggregatePriceResult);

/// [`nft_buy_offers`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/nft_buy_offers):
/// offers to buy one NFT.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct NftBuyOffersRequest {
    /// The NFT's ID.
    pub nft_id: String,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
    /// The most entries per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
impl NftBuyOffersRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(nft_id: impl Into<String>) -> Self {
        Self {
            nft_id: nft_id.into(),
            ..Default::default()
        }
    }
}
request!(NftBuyOffersRequest => "nft_buy_offers", NftOffersResult);
paginated!(
    NftBuyOffersRequest,
    NftOffer,
    offers,
    pinned,
    ledger_current_index
);

/// [`nft_sell_offers`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/nft_sell_offers):
/// offers to sell one NFT.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct NftSellOffersRequest {
    /// The NFT's ID.
    pub nft_id: String,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
    /// The most entries per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
impl NftSellOffersRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(nft_id: impl Into<String>) -> Self {
        Self {
            nft_id: nft_id.into(),
            ..Default::default()
        }
    }
}
request!(NftSellOffersRequest => "nft_sell_offers", NftOffersResult);
paginated!(
    NftSellOffersRequest,
    NftOffer,
    offers,
    pinned,
    ledger_current_index
);

/// [`ripple_path_find`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/ripple_path_find):
/// paths that deliver `destination_amount`, and what each costs.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct RipplePathFindRequest {
    /// The sending account.
    pub source_account: String,
    /// The receiving account.
    pub destination_account: String,
    /// Drops as a string, or `{"currency", "issuer", "value"}`; `"-1"` / value
    /// `"-1"` for as much as possible.
    pub destination_amount: Value,
    /// The most the source will spend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_max: Option<Value>,
    /// Currencies the source can spend: `[{"currency", "issuer"?}]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_currencies: Option<Vec<Value>>,
    /// Only paths through this permissioned domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
}
impl RipplePathFindRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(
        source_account: impl Into<String>,
        destination_account: impl Into<String>,
        destination_amount: Value,
    ) -> Self {
        Self {
            source_account: source_account.into(),
            destination_account: destination_account.into(),
            destination_amount,
            ..Default::default()
        }
    }
}
request!(RipplePathFindRequest => "ripple_path_find", RipplePathFindResult);

// -------------------------------------------------------- payment channel

/// [`channel_verify`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/payment-channel-methods/channel_verify):
/// check a claim's signature on the server. (`xrpl_core::crypto::verify_payment_channel_claim`
/// does the same offline.)
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ChannelVerifyRequest {
    /// The channel's ID.
    pub channel_id: String,
    /// The claimed total, in drops.
    pub amount: String,
    /// Hex or base58.
    pub public_key: String,
    /// A signature, hex (`Signature`).
    pub signature: String,
}
impl ChannelVerifyRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(
        channel_id: impl Into<String>,
        amount_drops: u64,
        public_key: impl Into<String>,
        signature: impl Into<String>,
    ) -> Self {
        Self {
            channel_id: channel_id.into(),
            amount: amount_drops.to_string(),
            public_key: public_key.into(),
            signature: signature.into(),
        }
    }
}
request!(ChannelVerifyRequest => "channel_verify", ChannelVerifyResult);

// ------------------------------------------------------------ server info

/// [`fee`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/fee):
/// the transaction cost now.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct FeeRequest {}
request!(FeeRequest => "fee", FeeResult);

/// [`feature`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/feature):
/// every amendment the server knows, and whether it is enabled.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct FeatureRequest {}
request!(FeatureRequest => "feature", FeatureResult);

/// [`manifest`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/manifest):
/// a validator's manifest, by its public key (`n…`).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ManifestRequest {
    /// A public key, hex (`PublicKey`).
    pub public_key: String,
}
impl ManifestRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(public_key: impl Into<String>) -> Self {
        Self {
            public_key: public_key.into(),
        }
    }
}
request!(ManifestRequest => "manifest", ManifestResult);

/// [`server_definitions`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/server_definitions):
/// the server's binary-format definitions.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ServerDefinitionsRequest {
    /// The `hash` of definitions already held: the server then sends only the
    /// hash if nothing changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}
request!(ServerDefinitionsRequest => "server_definitions", ServerDefinitionsResult);

/// [`server_info`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/server_info):
/// the server's state in human units.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ServerInfoRequest {}
request!(ServerInfoRequest => "server_info", ServerInfoResult);

/// [`server_state`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/server_state):
/// the server's state in machine units.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ServerStateRequest {}
request!(ServerStateRequest => "server_state", ServerStateResult);

/// [`version`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/version):
/// the API versions the server supports.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct VersionRequest {}
request!(VersionRequest => "version", VersionResult);

// ---------------------------------------------------------------- utility

/// [`ping`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/utility-methods/ping).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct PingRequest {}
request!(PingRequest => "ping", PingResult);

/// [`random`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/utility-methods/random).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct RandomRequest {}
request!(RandomRequest => "random", RandomResult);

// ------------------------------------------------------------------ vault

/// [`vault_info`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/vault-methods/vault_info):
/// a vault, by ID or by owner and sequence.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct VaultInfoRequest {
    /// The vault's ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_id: Option<String>,
    /// The account that owns the entry (`Owner`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// The sequence of the transaction that created it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq: Option<u32>,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
}
impl VaultInfoRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(vault_id: impl Into<String>) -> Self {
        Self {
            vault_id: Some(vault_id.into()),
            ..Default::default()
        }
    }
}
request!(VaultInfoRequest => "vault_info", VaultInfoResult);

// ------------------------------------------------------------ Clio only

/// [`nft_info`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/nft_info)
/// (Clio): one NFT, including a burned one.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct NftInfoRequest {
    /// The NFT's ID.
    pub nft_id: String,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
}
impl NftInfoRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(nft_id: impl Into<String>) -> Self {
        Self {
            nft_id: nft_id.into(),
            ..Default::default()
        }
    }
}
request!(NftInfoRequest => "nft_info", NftInfoResult);

/// [`nft_history`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/nft_history)
/// (Clio): every transaction that touched an NFT.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NftHistoryRequest {
    /// The NFT's ID.
    pub nft_id: String,
    /// The earliest ledger searched.
    pub ledger_index_min: i32,
    /// The latest ledger searched.
    pub ledger_index_max: i32,
    /// Hex instead of JSON.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
    /// Oldest first.
    pub forward: bool,
    /// The most entries per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
impl Default for NftHistoryRequest {
    fn default() -> Self {
        Self {
            nft_id: String::new(),
            ledger_index_min: -1,
            ledger_index_max: -1,
            binary: None,
            forward: false,
            limit: None,
            marker: None,
        }
    }
}
impl NftHistoryRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(nft_id: impl Into<String>) -> Self {
        Self {
            nft_id: nft_id.into(),
            ..Default::default()
        }
    }
}
request!(NftHistoryRequest => "nft_history", NftHistoryResult);
paginated!(NftHistoryRequest, AccountTxEntry, transactions);

/// [`nfts_by_issuer`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/nfts_by_issuer)
/// (Clio): every NFT an account issued.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct NftsByIssuerRequest {
    /// The issuer (`Issuer`).
    pub issuer: String,
    /// Only this taxon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nft_taxon: Option<u32>,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
    /// The most entries per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
impl NftsByIssuerRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(issuer: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            ..Default::default()
        }
    }
}
request!(NftsByIssuerRequest => "nfts_by_issuer", NftsByIssuerResult);
paginated!(NftsByIssuerRequest, NftInfo, nfts, pinned);

/// [`mpt_holders`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/mpt_holders)
/// (Clio): the holders of an MPT issuance.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct MptHoldersRequest {
    /// The MPT issuance's ID.
    pub mpt_issuance_id: String,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
    /// The most entries per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
impl MptHoldersRequest {
    /// The request with its required parameters set and every optional one unset.
    pub fn new(mpt_issuance_id: impl Into<String>) -> Self {
        Self {
            mpt_issuance_id: mpt_issuance_id.into(),
            ..Default::default()
        }
    }
}
request!(MptHoldersRequest => "mpt_holders", MptHoldersResult);
paginated!(MptHoldersRequest, MptHolder, mptokens, pinned);
