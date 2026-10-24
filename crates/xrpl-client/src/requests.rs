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
//! | Path and order book | [`AmmInfoRequest`], [`BookChangesRequest`], [`BookOffersRequest`], [`DepositAuthorizedRequest`], [`GetAggregatePriceRequest`], [`NftBuyOffersRequest`], [`NftSellOffersRequest`], [`PathFindRequest`] (WebSocket), [`RipplePathFindRequest`] |
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
//! // `new` takes the required parameters; `with_…` sets an optional one.
//! let req = AccountLinesRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").with_limit(100u32);
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
use xrpl_core::types::Asset;

/// A typed `rippled` request: the method it calls and the type of its `result`.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{AccountInfoRequest, Request};
///
/// assert_eq!(AccountInfoRequest::METHOD, "account_info");
/// // The answer's `result` reads as the request's `Response`: an `AccountInfo`.
/// let info: <AccountInfoRequest as Request>::Response = serde_json::from_value(json!({
///     "account_data": {"Account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "Balance": "1000000",
///         "Flags": 0, "OwnerCount": 0, "Sequence": 1}
/// }))?;
/// assert_eq!(info.account_data.balance_drops(), Some(1_000_000));
/// # Ok::<(), serde_json::Error>(())
/// ```
pub trait Request: Serialize + crate::client::MaybeSend + crate::client::MaybeSync {
    /// The `rippled` method (`account_info`, `fee`, …).
    const METHOD: &'static str;
    /// What the response's `result` deserializes into.
    type Response: DeserializeOwned + crate::client::MaybeSend;
}

/// A request whose response pages with `marker` (see [`crate::Client::request_all`]).
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{AccountLinesRequest, Paginated};
/// use xrpl_client::types::{AccountLinesResult, LedgerIndex};
///
/// # let line = json!({
/// #     "account": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "balance": "689748.581365565",
/// #     "currency": "524C555344000000000000000000000000000000", "limit": "10000000000",
/// #     "limit_peer": "0"
/// # });
/// // What `Client::pages` does with each page (`line`: one trust line): keep its
/// // entries, then ask for the page after its marker, from the same ledger.
/// let req = AccountLinesRequest::new("rURtT5MMFzSUR54MAq9MKfzBoezTBZbKns");
/// let response: AccountLinesResult = serde_json::from_value(json!({
///     "lines": [line], "ledger_index": 107831520,
///     "marker": "2DE7C6A023569FD79077E9907D39289E2A8FBC2D3D8F24C42F05B1E13F455E7D,1"
/// }))?;
/// let ledger = AccountLinesRequest::ledger_of(&response);
/// let (lines, marker) = AccountLinesRequest::page(response);
/// assert_eq!(lines.len(), 1);
/// let next = Paginated::with_marker(req.at_ledger(ledger.unwrap()), marker.unwrap());
/// assert_eq!(next.ledger_index, LedgerIndex::Index(107831520));
/// # Ok::<(), serde_json::Error>(())
/// ```
pub trait Paginated: Request + Clone {
    /// One entry of a page.
    type Item: crate::client::MaybeSend;
    /// This request, asking for the page after `marker`.
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::{AccountTxRequest, Paginated};
    ///
    /// let req = AccountTxRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    /// let next = Paginated::with_marker(req, json!({"ledger": 107856058, "seq": 39}));
    /// assert_eq!(next.marker, Some(json!({"ledger": 107856058, "seq": 39})));
    /// ```
    fn with_marker(self, marker: Value) -> Self;
    /// A page's entries and the marker of the next page, if any.
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::{AccountTxRequest, Paginated};
    /// use xrpl_client::types::AccountTxResult;
    ///
    /// let response: AccountTxResult = serde_json::from_value(json!({
    ///     "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "transactions": [],
    ///     "marker": {"ledger": 107856058, "seq": 39}
    /// }))?;
    /// let (transactions, marker) = AccountTxRequest::page(response);
    /// assert!(transactions.is_empty());
    /// assert_eq!(marker, Some(json!({"ledger": 107856058, "seq": 39})));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    fn page(response: Self::Response) -> (Vec<Self::Item>, Option<Value>);
    /// The ledger a page is from, when the request names a ledger: a
    /// marker is only good for the ledger it came from, so the pages after
    /// the first ask for that same ledger.
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::{AccountLinesRequest, AccountTxRequest, Paginated};
    /// use xrpl_client::types::{AccountLinesResult, AccountTxResult};
    ///
    /// let validated: AccountLinesResult = serde_json::from_value(json!({
    ///     "lines": [], "ledger_index": 107831520
    /// }))?;
    /// assert_eq!(AccountLinesRequest::ledger_of(&validated), Some(107831520));
    /// let open: AccountLinesResult = serde_json::from_value(json!({
    ///     "lines": [], "ledger_current_index": 107856071
    /// }))?;
    /// assert_eq!(AccountLinesRequest::ledger_of(&open), Some(107856071));
    /// // `account_tx` names no ledger: its marker alone says where the next page starts.
    /// let txs: AccountTxResult = serde_json::from_value(json!({
    ///     "transactions": [], "ledger_index_max": 107856070
    /// }))?;
    /// assert_eq!(AccountTxRequest::ledger_of(&txs), None);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    fn ledger_of(_response: &Self::Response) -> Option<u32> {
        None
    }
    /// This request, at ledger `index`.
    ///
    /// ```
    /// use xrpl_client::requests::{AccountLinesRequest, AccountTxRequest, Paginated};
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let lines =
    ///     AccountLinesRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe").at_ledger(107831520);
    /// assert_eq!(lines.ledger_index, LedgerIndex::Index(107831520));
    /// // A request that names no ledger is unchanged.
    /// let txs = AccountTxRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    /// assert_eq!(txs.clone().at_ledger(107831520), txs);
    /// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{GetAggregatePriceRequest, OracleId};
///
/// let oracle =
///     OracleId { account: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".into(), oracle_document_id: 1 };
/// let req = GetAggregatePriceRequest::new("XRP", "USD", vec![oracle]);
/// assert_eq!(
///     serde_json::to_value(&req)?["oracles"],
///     json!([{"account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "oracle_document_id": 1}])
/// );
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct OracleId {
    /// The account to look up (a classic address).
    pub account: String,
    /// The oracle's document ID.
    pub oracle_document_id: u32,
}

// ---------------------------------------------------------------- account

/// [`account_channels`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_channels):
/// payment channels the account is the source of.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{AccountChannelsRequest, Request};
///
/// let req = AccountChannelsRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
///     .with_destination_account("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
/// assert_eq!(AccountChannelsRequest::METHOD, "account_channels");
/// assert_eq!(
///     serde_json::to_value(&req)?,
///     json!({
///         "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///         "destination_account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "ledger_index": "validated"
///     })
/// );
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct AccountChannelsRequest {
    /// The account to look up (a classic address).
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::AccountChannelsRequest;
    ///
    /// let req = AccountChannelsRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    /// assert_eq!(req.limit, None);
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "ledger_index": "validated"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }

    /// Only channels to this destination (`destination_account`).
    #[must_use]
    pub fn with_destination_account(mut self, value: impl Into<String>) -> Self {
        self.destination_account = Some(value.into());
        self
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }

    /// The most entries per page (`limit`).
    #[must_use]
    pub fn with_limit(mut self, value: impl Into<u32>) -> Self {
        self.limit = Some(value.into());
        self
    }

    /// Where the next page starts: pass it back in the request; absent on the last page (`marker`).
    #[must_use]
    pub fn with_marker(mut self, value: impl Into<Value>) -> Self {
        self.marker = Some(value.into());
        self
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{AccountCurrenciesRequest, Request};
/// use xrpl_client::types::LedgerIndex;
///
/// let req = AccountCurrenciesRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
///     .with_ledger_index(LedgerIndex::Current);
/// assert_eq!(AccountCurrenciesRequest::METHOD, "account_currencies");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "ledger_index": "current"
/// }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct AccountCurrenciesRequest {
    /// The account to look up (a classic address).
    pub account: String,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
}
impl AccountCurrenciesRequest {
    /// The request with its required parameters set and every optional one unset.
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::AccountCurrenciesRequest;
    ///
    /// let req = AccountCurrenciesRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "ledger_index": "validated"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }
}
request!(AccountCurrenciesRequest => "account_currencies", AccountCurrenciesResult);

/// [`account_info`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_info):
/// an account's settings and XRP balance.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{AccountInfoRequest, Request};
/// use xrpl_client::types::LedgerIndex;
///
/// // The open ledger, with the account's queued transactions.
/// let req = AccountInfoRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
///     .with_ledger_index(LedgerIndex::Current)
///     .with_queue(true);
/// assert_eq!(AccountInfoRequest::METHOD, "account_info");
/// assert_eq!(
///     serde_json::to_value(&req)?,
///     json!({
///         "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "ledger_index": "current",
///         "queue": true
///     })
/// );
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct AccountInfoRequest {
    /// The account to look up (a classic address).
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::AccountInfoRequest;
    ///
    /// let req = AccountInfoRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    /// assert_eq!((req.signer_lists, req.queue), (None, None));
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "ledger_index": "validated"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }

    /// Include the account's signer lists (`signer_lists`).
    #[must_use]
    pub fn with_signer_lists(mut self, value: impl Into<bool>) -> Self {
        self.signer_lists = Some(value.into());
        self
    }

    /// Include queued transactions (needs `LedgerIndex::Current`) (`queue`).
    #[must_use]
    pub fn with_queue(mut self, value: impl Into<bool>) -> Self {
        self.queue = Some(value.into());
        self
    }
}
request!(AccountInfoRequest => "account_info", AccountInfo);

/// [`account_lines`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_lines):
/// an account's trust lines.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{AccountLinesRequest, Request};
///
/// // Only the line between the two accounts.
/// let req = AccountLinesRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
///     .with_peer("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
/// assert_eq!(AccountLinesRequest::METHOD, "account_lines");
/// assert_eq!(
///     serde_json::to_value(&req)?,
///     json!({
///         "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "ledger_index": "validated",
///         "peer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"
///     })
/// );
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct AccountLinesRequest {
    /// The account to look up (a classic address).
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::AccountLinesRequest;
    ///
    /// let req = AccountLinesRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    /// assert!(req.peer.is_none() && req.limit.is_none());
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "ledger_index": "validated"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }

    /// Only the line with this peer (`peer`).
    #[must_use]
    pub fn with_peer(mut self, value: impl Into<String>) -> Self {
        self.peer = Some(value.into());
        self
    }

    /// 10–400 per page (`limit`).
    #[must_use]
    pub fn with_limit(mut self, value: impl Into<u32>) -> Self {
        self.limit = Some(value.into());
        self
    }

    /// Where the next page starts: pass it back in the request; absent on the last page (`marker`).
    #[must_use]
    pub fn with_marker(mut self, value: impl Into<Value>) -> Self {
        self.marker = Some(value.into());
        self
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{AccountNftsRequest, Request};
///
/// let req = AccountNftsRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe").with_limit(100u32);
/// assert_eq!(AccountNftsRequest::METHOD, "account_nfts");
/// assert_eq!(
///     serde_json::to_value(&req)?,
///     json!({
///         "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "ledger_index": "validated",
///         "limit": 100
///     })
/// );
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct AccountNftsRequest {
    /// The account to look up (a classic address).
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::AccountNftsRequest;
    ///
    /// let req = AccountNftsRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "ledger_index": "validated"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }

    /// 20–400 per page (`limit`).
    #[must_use]
    pub fn with_limit(mut self, value: impl Into<u32>) -> Self {
        self.limit = Some(value.into());
        self
    }

    /// Where the next page starts: pass it back in the request; absent on the last page (`marker`).
    #[must_use]
    pub fn with_marker(mut self, value: impl Into<Value>) -> Self {
        self.marker = Some(value.into());
        self
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{AccountObjectsRequest, Request};
///
/// // Only the account's offers (the type goes out as `type`).
/// let req = AccountObjectsRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
///     .with_object_type("offer");
/// assert_eq!(AccountObjectsRequest::METHOD, "account_objects");
/// assert_eq!(
///     serde_json::to_value(&req)?,
///     json!({
///         "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "type": "offer",
///         "ledger_index": "validated"
///     })
/// );
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct AccountObjectsRequest {
    /// The account to look up (a classic address).
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::AccountObjectsRequest;
    ///
    /// let req = AccountObjectsRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    /// assert!(req.object_type.is_none() && req.deletion_blockers_only.is_none());
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "ledger_index": "validated"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }

    /// Only entries of this type (`offer`, `state`, `check`, `escrow`, `payment_channel`, `nft_offer`, `ticket`, `did`, `oracle`, `mptoken`, …) (`object_type`).
    #[must_use]
    pub fn with_object_type(mut self, value: impl Into<String>) -> Self {
        self.object_type = Some(value.into());
        self
    }

    /// Only entries that block deleting the account (`deletion_blockers_only`).
    #[must_use]
    pub fn with_deletion_blockers_only(mut self, value: impl Into<bool>) -> Self {
        self.deletion_blockers_only = Some(value.into());
        self
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }

    /// 10–400 per page (`limit`).
    #[must_use]
    pub fn with_limit(mut self, value: impl Into<u32>) -> Self {
        self.limit = Some(value.into());
        self
    }

    /// Where the next page starts: pass it back in the request; absent on the last page (`marker`).
    #[must_use]
    pub fn with_marker(mut self, value: impl Into<Value>) -> Self {
        self.marker = Some(value.into());
        self
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{AccountOffersRequest, Request};
/// use xrpl_client::types::LedgerIndex;
///
/// let req = AccountOffersRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
///     .with_ledger_index(LedgerIndex::Index(107831520));
/// assert_eq!(AccountOffersRequest::METHOD, "account_offers");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "ledger_index": 107831520
/// }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct AccountOffersRequest {
    /// The account to look up (a classic address).
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::AccountOffersRequest;
    ///
    /// let req = AccountOffersRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "ledger_index": "validated"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }

    /// 10–400 per page (`limit`).
    #[must_use]
    pub fn with_limit(mut self, value: impl Into<u32>) -> Self {
        self.limit = Some(value.into());
        self
    }

    /// Where the next page starts: pass it back in the request; absent on the last page (`marker`).
    #[must_use]
    pub fn with_marker(mut self, value: impl Into<Value>) -> Self {
        self.marker = Some(value.into());
        self
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{AccountTxRequest, Request};
///
/// // The oldest ten transactions.
/// let req = AccountTxRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
///     .with_forward(true)
///     .with_limit(10u32);
/// assert_eq!(AccountTxRequest::METHOD, "account_tx");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "ledger_index_min": -1,
///     "ledger_index_max": -1,
///     "forward": true, "limit": 10
/// }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub struct AccountTxRequest {
    /// The account to look up (a classic address).
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::AccountTxRequest;
    ///
    /// let req = AccountTxRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "ledger_index_min": -1,
    ///     "ledger_index_max": -1, "forward": false
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }

    /// The earliest ledger searched (`ledger_index_min`).
    #[must_use]
    pub fn with_ledger_index_min(mut self, value: impl Into<i32>) -> Self {
        self.ledger_index_min = value.into();
        self
    }

    /// The latest ledger searched (`ledger_index_max`).
    #[must_use]
    pub fn with_ledger_index_max(mut self, value: impl Into<i32>) -> Self {
        self.ledger_index_max = value.into();
        self
    }

    /// Hex (`tx_blob`, `meta`) instead of JSON (`binary`).
    #[must_use]
    pub fn with_binary(mut self, value: impl Into<bool>) -> Self {
        self.binary = Some(value.into());
        self
    }

    /// Oldest first (`forward`).
    #[must_use]
    pub fn with_forward(mut self, value: impl Into<bool>) -> Self {
        self.forward = value.into();
        self
    }

    /// The most entries per page (`limit`).
    #[must_use]
    pub fn with_limit(mut self, value: impl Into<u32>) -> Self {
        self.limit = Some(value.into());
        self
    }

    /// Where the next page starts: pass it back in the request; absent on the last page (`marker`).
    #[must_use]
    pub fn with_marker(mut self, value: impl Into<Value>) -> Self {
        self.marker = Some(value.into());
        self
    }
}
request!(AccountTxRequest => "account_tx", AccountTxResult);
paginated!(AccountTxRequest, TxResult, transactions);

/// [`gateway_balances`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/gateway_balances):
/// what an issuer has issued, and to whom.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{GatewayBalancesRequest, Request};
///
/// // The issuer's own operational account goes in `balances`, not `obligations`.
/// let req = GatewayBalancesRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
///     .with_hotwallet(vec!["rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string()]);
/// assert_eq!(GatewayBalancesRequest::METHOD, "gateway_balances");
/// assert_eq!(
///     serde_json::to_value(&req)?,
///     json!({
///         "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///         "hotwallet": ["rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"], "ledger_index": "validated"
///     })
/// );
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct GatewayBalancesRequest {
    /// The account to look up (a classic address).
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::GatewayBalancesRequest;
    ///
    /// let req = GatewayBalancesRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    /// assert!(req.hotwallet.is_none() && req.strict.is_none());
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "ledger_index": "validated"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            ..Default::default()
        }
    }

    /// Operational addresses to leave out of `obligations` (listed in `balances`) (`hotwallet`).
    #[must_use]
    pub fn with_hotwallet(mut self, value: impl Into<Vec<String>>) -> Self {
        self.hotwallet = Some(value.into());
        self
    }

    /// Accept only addresses, not public keys (`strict`).
    #[must_use]
    pub fn with_strict(mut self, value: impl Into<bool>) -> Self {
        self.strict = Some(value.into());
        self
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }
}
request!(GatewayBalancesRequest => "gateway_balances", GatewayBalancesResult);

/// [`noripple_check`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/noripple_check):
/// trust-line settings that do not suit the account's role.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{NoRippleCheckRequest, Request};
///
/// // Check an issuer's lines, with the transactions that would fix them.
/// let req = NoRippleCheckRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "gateway")
///     .with_transactions(true);
/// assert_eq!(NoRippleCheckRequest::METHOD, "noripple_check");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "role": "gateway", "transactions": true,
///     "ledger_index": "validated"
/// }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct NoRippleCheckRequest {
    /// The account to look up (a classic address).
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::NoRippleCheckRequest;
    ///
    /// let req = NoRippleCheckRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "user");
    /// assert_eq!(
    ///     serde_json::to_value(&req)?,
    ///     json!({
    ///         "account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "role": "user",
    ///         "ledger_index": "validated"
    ///     })
    /// );
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(account: impl Into<String>, role: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            role: role.into(),
            ..Default::default()
        }
    }

    /// Include the transactions that fix the problems (`transactions`).
    #[must_use]
    pub fn with_transactions(mut self, value: impl Into<bool>) -> Self {
        self.transactions = Some(value.into());
        self
    }

    /// The most entries per page (`limit`).
    #[must_use]
    pub fn with_limit(mut self, value: impl Into<u32>) -> Self {
        self.limit = Some(value.into());
        self
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }
}
request!(NoRippleCheckRequest => "noripple_check", NoRippleCheckResult);

// ----------------------------------------------------------------- ledger

/// [`ledger`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger):
/// a ledger's header, and optionally its transactions.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{LedgerRequest, Request};
/// use xrpl_client::types::LedgerIndex;
///
/// // The latest validated ledger, with its transactions in full.
/// let req = LedgerRequest::new(LedgerIndex::Validated).with_transactions(true).with_expand(true);
/// assert_eq!(LedgerRequest::METHOD, "ledger");
/// assert_eq!(
///     serde_json::to_value(&req)?,
///     json!({"ledger_index": "validated", "transactions": true, "expand": true})
/// );
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::LedgerRequest;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let req = LedgerRequest::new(LedgerIndex::Index(107831520));
    /// assert_eq!(serde_json::to_value(&req)?, json!({"ledger_index": 107831520}));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(ledger_index: LedgerIndex) -> Self {
        Self {
            ledger_index,
            ..Default::default()
        }
    }

    /// The ledger by hash instead (`ledger_hash`).
    #[must_use]
    pub fn with_ledger_hash(mut self, value: impl Into<String>) -> Self {
        self.ledger_hash = Some(value.into());
        self
    }

    /// Include the transactions (hashes, or objects with `expand`) (`transactions`).
    #[must_use]
    pub fn with_transactions(mut self, value: impl Into<bool>) -> Self {
        self.transactions = Some(value.into());
        self
    }

    /// Whole transactions instead of hashes (`expand`).
    #[must_use]
    pub fn with_expand(mut self, value: impl Into<bool>) -> Self {
        self.expand = Some(value.into());
        self
    }

    /// With `expand`: hex instead of JSON (`binary`).
    #[must_use]
    pub fn with_binary(mut self, value: impl Into<bool>) -> Self {
        self.binary = Some(value.into());
        self
    }

    /// Add `owner_funds` to OfferCreate transactions (`owner_funds`).
    #[must_use]
    pub fn with_owner_funds(mut self, value: impl Into<bool>) -> Self {
        self.owner_funds = Some(value.into());
        self
    }

    /// Include queued transactions (current ledger only) (`queue`).
    #[must_use]
    pub fn with_queue(mut self, value: impl Into<bool>) -> Self {
        self.queue = Some(value.into());
        self
    }
}
request!(LedgerRequest => "ledger", LedgerResult);

/// [`ledger_closed`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_closed):
/// the latest closed ledger.
///
/// ```
/// use xrpl_client::requests::{LedgerClosedRequest, Request};
///
/// assert_eq!(LedgerClosedRequest::METHOD, "ledger_closed");
/// // It has no parameters.
/// assert_eq!(serde_json::to_value(LedgerClosedRequest::new())?, serde_json::json!({}));
/// # Ok::<(), serde_json::Error>(())
/// ```
///
/// ```no_run
/// # async fn run(client: impl xrpl_client::Client) -> Result<(), xrpl_client::ClientError> {
/// use xrpl_client::requests::LedgerClosedRequest;
///
/// let closed = client.request(&LedgerClosedRequest::new()).await?;
/// println!("ledger {} is closed: {}", closed.ledger_index, closed.ledger_hash);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct LedgerClosedRequest {}
impl LedgerClosedRequest {
    /// The request with every optional parameter unset.
    ///
    /// ```
    /// use xrpl_client::requests::LedgerClosedRequest;
    ///
    /// assert_eq!(LedgerClosedRequest::new(), LedgerClosedRequest::default());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}
request!(LedgerClosedRequest => "ledger_closed", LedgerClosedResult);

/// [`ledger_current`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_current):
/// the open ledger's index.
///
/// ```
/// use xrpl_client::requests::{LedgerCurrentRequest, Request};
///
/// assert_eq!(LedgerCurrentRequest::METHOD, "ledger_current");
/// // It has no parameters.
/// assert_eq!(serde_json::to_value(LedgerCurrentRequest::new())?, serde_json::json!({}));
/// # Ok::<(), serde_json::Error>(())
/// ```
///
/// ```no_run
/// # async fn run(client: impl xrpl_client::Client) -> Result<(), xrpl_client::ClientError> {
/// use xrpl_client::requests::LedgerCurrentRequest;
///
/// let open = client.request(&LedgerCurrentRequest::new()).await?;
/// println!("the open ledger is {}", open.ledger_current_index);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct LedgerCurrentRequest {}
impl LedgerCurrentRequest {
    /// The request with every optional parameter unset.
    ///
    /// ```
    /// use xrpl_client::requests::LedgerCurrentRequest;
    ///
    /// assert_eq!(LedgerCurrentRequest::new(), LedgerCurrentRequest::default());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}
request!(LedgerCurrentRequest => "ledger_current", LedgerCurrentResult);

/// [`ledger_data`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_data):
/// a ledger's state, a page at a time.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{LedgerDataRequest, Request};
/// use xrpl_client::types::LedgerIndex;
///
/// // Every AMM in the ledger (the type goes out as `type`), a page at a time.
/// let req = LedgerDataRequest::new(LedgerIndex::Validated)
///     .with_entry_type("amm")
///     .with_limit(256u32);
/// assert_eq!(LedgerDataRequest::METHOD, "ledger_data");
/// assert_eq!(
///     serde_json::to_value(&req)?,
///     json!({"ledger_index": "validated", "type": "amm", "limit": 256})
/// );
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::LedgerDataRequest;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let req = LedgerDataRequest::new(LedgerIndex::Index(107831520));
    /// assert_eq!(serde_json::to_value(&req)?, json!({"ledger_index": 107831520}));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(ledger_index: LedgerIndex) -> Self {
        Self {
            ledger_index,
            ..Default::default()
        }
    }

    /// That ledger's hash (`ledger_hash`).
    #[must_use]
    pub fn with_ledger_hash(mut self, value: impl Into<String>) -> Self {
        self.ledger_hash = Some(value.into());
        self
    }

    /// Only entries of this type (`account`, `amm`, `offer`, `state`, …) (`entry_type`).
    #[must_use]
    pub fn with_entry_type(mut self, value: impl Into<String>) -> Self {
        self.entry_type = Some(value.into());
        self
    }

    /// Hex (`data`, `index`) instead of JSON (`binary`).
    #[must_use]
    pub fn with_binary(mut self, value: impl Into<bool>) -> Self {
        self.binary = Some(value.into());
        self
    }

    /// Up to 256 per page (2048 with `binary`) (`limit`).
    #[must_use]
    pub fn with_limit(mut self, value: impl Into<u32>) -> Self {
        self.limit = Some(value.into());
        self
    }

    /// Where the next page starts: pass it back in the request; absent on the last page (`marker`).
    #[must_use]
    pub fn with_marker(mut self, value: impl Into<Value>) -> Self {
        self.marker = Some(value.into());
        self
    }
}
request!(LedgerDataRequest => "ledger_data", LedgerDataResult);
paginated!(LedgerDataRequest, LedgerObject, state, pinned);

/// [`ledger_entry`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/ledger-methods/ledger_entry):
/// one ledger entry by ID. `xrpl_core::hashes::keylet` computes the ID of any
/// entry (account, trust line, offer, escrow, AMM, MPT, credential, …).
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{LedgerEntryRequest, Request};
/// use xrpl_core::hashes::keylet;
///
/// // An account's AccountRoot, by the ID `keylet` computes; the answer as hex.
/// let id = keylet::account("rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De")?;
/// let req = LedgerEntryRequest::new(id.to_string()).with_binary(true);
/// assert_eq!(LedgerEntryRequest::METHOD, "ledger_entry");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "index": "383BB05FBB110F9BE530B75C44DF100F86C5600F0F146593D3E7133FD1C99FF2",
///     "ledger_index": "validated", "binary": true
/// }));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::LedgerEntryRequest;
    ///
    /// // The AccountRoot of rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De.
    /// let id = "383BB05FBB110F9BE530B75C44DF100F86C5600F0F146593D3E7133FD1C99FF2";
    /// let req = LedgerEntryRequest::new(id);
    /// assert_eq!(serde_json::to_value(&req)?, json!({"index": id, "ledger_index": "validated"}));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(index: impl Into<String>) -> Self {
        Self {
            index: index.into(),
            ..Default::default()
        }
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }

    /// Hex (`node_binary`) instead of JSON (`binary`).
    #[must_use]
    pub fn with_binary(mut self, value: impl Into<bool>) -> Self {
        self.binary = Some(value.into());
        self
    }

    /// Clio: also find an entry that has been deleted (`include_deleted`).
    #[must_use]
    pub fn with_include_deleted(mut self, value: impl Into<bool>) -> Self {
        self.include_deleted = Some(value.into());
        self
    }
}
request!(LedgerEntryRequest => "ledger_entry", LedgerEntryResult);

// ------------------------------------------------------------ transaction

/// [`simulate`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/simulate):
/// what a transaction would do, without signing or submitting it. Give it
/// unsigned (no `SigningPubKey`/`TxnSignature`, or empty ones); missing
/// `Fee`, `Sequence` and the like are filled in.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{Request, SimulateRequest};
///
/// // Unsigned: the server fills in Fee, Sequence and the like.
/// let tx = json!({
///     "TransactionType": "Payment", "Account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "Destination": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "Amount": "1000000"
/// });
/// let req = SimulateRequest::new(tx.clone()).with_binary(true);
/// assert_eq!(SimulateRequest::METHOD, "simulate");
/// assert_eq!(serde_json::to_value(&req)?, json!({"tx_json": tx, "binary": true}));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::SimulateRequest;
    ///
    /// let tx = json!({
    ///     "TransactionType": "Payment", "Account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
    ///     "Destination": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "Amount": "1000000"
    /// });
    /// let req = SimulateRequest::new(tx.clone());
    /// assert_eq!(req.tx_blob, None);
    /// assert_eq!(serde_json::to_value(&req)?, json!({"tx_json": tx}));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(tx_json: Value) -> Self {
        Self {
            tx_json: Some(tx_json),
            ..Default::default()
        }
    }

    /// The transaction, as hex (`tx_blob`).
    #[must_use]
    pub fn with_tx_blob(mut self, value: impl Into<String>) -> Self {
        self.tx_blob = Some(value.into());
        self
    }

    /// Hex (`tx_blob`, `meta_blob`) in the response (`binary`).
    #[must_use]
    pub fn with_binary(mut self, value: impl Into<bool>) -> Self {
        self.binary = Some(value.into());
        self
    }
}
request!(SimulateRequest => "simulate", SimulateResult);

/// [`submit`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/submit)
/// a signed blob. (Submitting with a secret, `rippled`'s sign-and-submit mode,
/// is left out: keys stay local.)
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{Request, SubmitRequest};
///
/// # let tx_blob =
/// #     "12000024014C294B201B014C29626140000000000F424068400000000000000A7321ED5392518D18161CC0501B8FD2B8AEFA959E81DB43DD5A4B4810AAEC38AD42A29B744013CDE91762B90BF50E1F5F1206BAEC910F13840E8CFCF6ADC12F6FF859B5EE90D989D1DF4E87633D4B0073D9B69F70453044F655D8887A0BE9039FF0E155F70D81142B3DD7A7E525987A2FAFF91EECF53984A1FC79068314E5B74827BB08AEB85D0CF4E6CEB64F8349EA600D";
/// // `tx_blob`: a signed transaction, hex.
/// let req = SubmitRequest::new(tx_blob).with_fail_hard(true);
/// assert_eq!(SubmitRequest::METHOD, "submit");
/// assert_eq!(serde_json::to_value(&req)?, json!({"tx_blob": tx_blob, "fail_hard": true}));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct SubmitRequest {
    /// The transaction, as hex.
    pub tx_blob: String,
    /// Do not retry or relay it if it fails locally.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_hard: Option<bool>,
}
impl SubmitRequest {
    /// The request with its required parameters set and every optional one unset.
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::SubmitRequest;
    ///
    /// # let tx_blob =
    /// #     "12000024014C294B201B014C29626140000000000F424068400000000000000A7321ED5392518D18161CC0501B8FD2B8AEFA959E81DB43DD5A4B4810AAEC38AD42A29B744013CDE91762B90BF50E1F5F1206BAEC910F13840E8CFCF6ADC12F6FF859B5EE90D989D1DF4E87633D4B0073D9B69F70453044F655D8887A0BE9039FF0E155F70D81142B3DD7A7E525987A2FAFF91EECF53984A1FC79068314E5B74827BB08AEB85D0CF4E6CEB64F8349EA600D";
    /// let req = SubmitRequest::new(tx_blob);
    /// assert_eq!(serde_json::to_value(&req)?, json!({"tx_blob": tx_blob}));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(tx_blob: impl Into<String>) -> Self {
        Self {
            tx_blob: tx_blob.into(),
            ..Default::default()
        }
    }

    /// Do not retry or relay it if it fails locally (`fail_hard`).
    #[must_use]
    pub fn with_fail_hard(mut self, value: impl Into<bool>) -> Self {
        self.fail_hard = Some(value.into());
        self
    }
}
request!(SubmitRequest => "submit", SubmitResult);

/// [`submit_multisigned`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/submit_multisigned):
/// a multi-signed transaction as JSON (with `Signers` and an empty
/// `SigningPubKey`; see `xrpl_sdk::wallet::collect_signers`).
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{Request, SubmitMultisignedRequest};
///
/// # let tx = json!({
/// #     "TransactionType": "AccountSet", "Account": "rhAe93ohNJkzvgD2rutsfnayqccnnJgS8W",
/// #     "Domain": "6578616D706C652E636F6D", "Fee": "20", "LastLedgerSequence": 21768585,
/// #     "Sequence": 21768545, "SigningPubKey": "",
/// #     "Signers": [{
/// #         "Signer": {
/// #             "Account": "rMAdJ2PxkGkGtmt7XbTZiMczcZ4BA1E2wp",
/// #             "SigningPubKey": "ED2BDBC88105FA003F8EB59484C00EFE3E95A40A3412E2FA4CED71C97CE92FFBE8",
/// #             "TxnSignature": "4B7B32EEC741139FD212D443287D1835670F9594CD2E801FAC2FCBE2D096B913C78F20C9B3E99F49C49F10D839CD4F29C244D16EFCB3EB82105C4B93DC74CA04"
/// #         }
/// #     }]
/// # });
/// // `tx`: the transaction with its `Signers`, and an empty `SigningPubKey`.
/// let req = SubmitMultisignedRequest::new(tx.clone()).with_fail_hard(true);
/// assert_eq!(SubmitMultisignedRequest::METHOD, "submit_multisigned");
/// assert_eq!(serde_json::to_value(&req)?, json!({"tx_json": tx, "fail_hard": true}));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct SubmitMultisignedRequest {
    /// The transaction, as JSON.
    pub tx_json: Value,
    /// Do not retry or relay it if it fails locally.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_hard: Option<bool>,
}
impl SubmitMultisignedRequest {
    /// The request with its required parameters set and every optional one unset.
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::SubmitMultisignedRequest;
    ///
    /// # let tx = json!({
    /// #     "TransactionType": "AccountSet", "Account": "rhAe93ohNJkzvgD2rutsfnayqccnnJgS8W",
    /// #     "Domain": "6578616D706C652E636F6D", "Fee": "20", "LastLedgerSequence": 21768585,
    /// #     "Sequence": 21768545, "SigningPubKey": "",
    /// #     "Signers": [{
    /// #         "Signer": {
    /// #             "Account": "rMAdJ2PxkGkGtmt7XbTZiMczcZ4BA1E2wp",
    /// #             "SigningPubKey": "ED2BDBC88105FA003F8EB59484C00EFE3E95A40A3412E2FA4CED71C97CE92FFBE8",
    /// #             "TxnSignature": "4B7B32EEC741139FD212D443287D1835670F9594CD2E801FAC2FCBE2D096B913C78F20C9B3E99F49C49F10D839CD4F29C244D16EFCB3EB82105C4B93DC74CA04"
    /// #         }
    /// #     }]
    /// # });
    /// let req = SubmitMultisignedRequest::new(tx);
    /// assert_eq!(req.tx_json["SigningPubKey"], "");
    /// assert_eq!(req.fail_hard, None);
    /// ```
    pub fn new(tx_json: Value) -> Self {
        Self {
            tx_json,
            ..Default::default()
        }
    }

    /// Do not retry or relay it if it fails locally (`fail_hard`).
    #[must_use]
    pub fn with_fail_hard(mut self, value: impl Into<bool>) -> Self {
        self.fail_hard = Some(value.into());
        self
    }
}
request!(SubmitMultisignedRequest => "submit_multisigned", SubmitResult);

/// [`transaction_entry`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/transaction_entry):
/// a transaction from one given ledger.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{Request, TransactionEntryRequest};
/// use xrpl_client::types::LedgerIndex;
///
/// let hash = "01A21E4D21563C67D8FB19F5E3B07096D8338011E6B79A3B2E97EF2F2C49371F";
/// let ledger_hash = "DECB05A6C02067FF57770846FD1E86F9199F294738148C3F5D1875AF14BBF9DA";
/// let req = TransactionEntryRequest::new(hash, LedgerIndex::Index(107831520))
///     .with_ledger_hash(ledger_hash);
/// assert_eq!(TransactionEntryRequest::METHOD, "transaction_entry");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "tx_hash": hash, "ledger_index": 107831520, "ledger_hash": ledger_hash
/// }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::TransactionEntryRequest;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let hash = "01A21E4D21563C67D8FB19F5E3B07096D8338011E6B79A3B2E97EF2F2C49371F";
    /// let req = TransactionEntryRequest::new(hash, LedgerIndex::Index(107831520));
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "tx_hash": hash, "ledger_index": 107831520
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(tx_hash: impl Into<String>, ledger_index: LedgerIndex) -> Self {
        Self {
            tx_hash: tx_hash.into(),
            ledger_index,
            ledger_hash: None,
        }
    }

    /// That ledger's hash (`ledger_hash`).
    #[must_use]
    pub fn with_ledger_hash(mut self, value: impl Into<String>) -> Self {
        self.ledger_hash = Some(value.into());
        self
    }
}
request!(TransactionEntryRequest => "transaction_entry", TxResult);

/// [`tx`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/transaction-methods/tx):
/// a transaction by hash (or by CTID in `transaction`'s place).
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{Request, TxRequest};
///
/// // Search a range, so a miss says whether the server has all of it.
/// let hash = "01A21E4D21563C67D8FB19F5E3B07096D8338011E6B79A3B2E97EF2F2C49371F";
/// let req = TxRequest::new(hash).with_min_ledger(107436647u32).with_max_ledger(107856071u32);
/// assert_eq!(TxRequest::METHOD, "tx");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "transaction": hash, "binary": false,
///     "min_ledger": 107436647, "max_ledger": 107856071
/// }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::TxRequest;
    ///
    /// let hash = "01A21E4D21563C67D8FB19F5E3B07096D8338011E6B79A3B2E97EF2F2C49371F";
    /// let req = TxRequest::new(hash);
    /// assert_eq!(serde_json::to_value(&req)?, json!({"transaction": hash, "binary": false}));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(hash: impl Into<String>) -> Self {
        Self {
            transaction: hash.into(),
            ..Default::default()
        }
    }

    /// The transaction with this compact transaction ID (XLS-37 `ctid`, 16
    /// hex digits) instead of its hash.
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::TxRequest;
    ///
    /// let req = TxRequest::by_ctid("C66D60E0000B0000");
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "ctid": "C66D60E0000B0000", "binary": false
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn by_ctid(ctid: impl Into<String>) -> Self {
        Self {
            ctid: Some(ctid.into()),
            ..Default::default()
        }
    }

    /// Its compact ID (`C…`, 16 hex digits) instead of the hash (`ctid`).
    #[must_use]
    pub fn with_ctid(mut self, value: impl Into<String>) -> Self {
        self.ctid = Some(value.into());
        self
    }

    /// Hex (`tx`, `meta`) instead of JSON (`binary`).
    #[must_use]
    pub fn with_binary(mut self, value: impl Into<bool>) -> Self {
        self.binary = value.into();
        self
    }

    /// With `max_ledger`: the range to search, so a miss says whether the server has all of it (`searched_all`) (`min_ledger`).
    #[must_use]
    pub fn with_min_ledger(mut self, value: impl Into<u32>) -> Self {
        self.min_ledger = Some(value.into());
        self
    }

    /// The latest ledger to search (`max_ledger`).
    #[must_use]
    pub fn with_max_ledger(mut self, value: impl Into<u32>) -> Self {
        self.max_ledger = Some(value.into());
        self
    }
}
request!(TxRequest => "tx", TxResult);

// -------------------------------------------------- path and order book

/// [`amm_info`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/amm_info):
/// an AMM pool, by its two assets or by its account.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{AmmInfoRequest, Request};
/// use xrpl_client::types::LedgerIndex;
/// use xrpl_core::types::Asset;
///
/// let rlusd = Asset::token("RLUSD", "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De")?;
/// let req = AmmInfoRequest::new(Asset::xrp(), rlusd).with_ledger_index(LedgerIndex::Validated);
/// assert_eq!(AmmInfoRequest::METHOD, "amm_info");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "asset": {"currency": "XRP"},
///     "asset2": {
///         "currency": "524C555344000000000000000000000000000000",
///         "issuer": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De"
///     },
///     "ledger_index": "validated"
/// }));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct AmmInfoRequest {
    /// `{"currency": "XRP"}` or `{"currency", "issuer"}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<Asset>,
    /// The pool's second asset (`Asset2`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset2: Option<Asset>,
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
    ///
    /// ```
    /// use xrpl_client::requests::AmmInfoRequest;
    /// use xrpl_core::types::Asset;
    ///
    /// let rlusd = Asset::token("RLUSD", "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De")?;
    /// let req = AmmInfoRequest::new(Asset::xrp(), rlusd.clone());
    /// assert_eq!((req.asset, req.asset2), (Some(Asset::xrp()), Some(rlusd)));
    /// assert_eq!(req.ledger_index, None);
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn new(asset: Asset, asset2: Asset) -> Self {
        Self {
            asset: Some(asset),
            asset2: Some(asset2),
            ..Default::default()
        }
    }

    /// The pool with this AMM account, instead of its assets.
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::AmmInfoRequest;
    ///
    /// let req = AmmInfoRequest::by_account("rhWTXC2m2gGGA9WozUaoMm6kLAVPb1tcS3");
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "amm_account": "rhWTXC2m2gGGA9WozUaoMm6kLAVPb1tcS3"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn by_account(amm_account: impl Into<String>) -> Self {
        Self {
            amm_account: Some(amm_account.into()),
            ..Default::default()
        }
    }

    /// The AMM's account, instead of the assets (`amm_account`).
    #[must_use]
    pub fn with_amm_account(mut self, value: impl Into<String>) -> Self {
        self.amm_account = Some(value.into());
        self
    }

    /// Include this LP's vote and auction-slot details (`account`).
    #[must_use]
    pub fn with_account(mut self, value: impl Into<String>) -> Self {
        self.account = Some(value.into());
        self
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = Some(value.into());
        self
    }
}
request!(AmmInfoRequest => "amm_info", AmmInfoResult);

/// [`book_changes`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/book_changes):
/// each order book's trading in one ledger (open, high, low, close, volume).
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{BookChangesRequest, Request};
/// use xrpl_client::types::LedgerIndex;
///
/// let req = BookChangesRequest::new(LedgerIndex::Index(107831526));
/// assert_eq!(BookChangesRequest::METHOD, "book_changes");
/// assert_eq!(serde_json::to_value(&req)?, json!({"ledger_index": 107831526}));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct BookChangesRequest {
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
}
impl BookChangesRequest {
    /// The request with its required parameters set and every optional one unset.
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::BookChangesRequest;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let req = BookChangesRequest::new(LedgerIndex::Validated);
    /// assert_eq!(serde_json::to_value(&req)?, json!({"ledger_index": "validated"}));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(ledger_index: LedgerIndex) -> Self {
        Self { ledger_index }
    }
}
request!(BookChangesRequest => "book_changes", BookChangesResult);

/// [`book_offers`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/book_offers):
/// an order book. `taker_gets` / `taker_pays` are what an offer's taker would
/// get and pay: `{"currency": "XRP"}` or `{"currency", "issuer"}`.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{BookOffersRequest, Request};
/// use xrpl_core::types::Asset;
///
/// // Offers that give XRP for RLUSD, best first.
/// let rlusd = Asset::token("RLUSD", "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De")?;
/// let req = BookOffersRequest::new(Asset::xrp(), rlusd).with_limit(3u32);
/// assert_eq!(BookOffersRequest::METHOD, "book_offers");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "taker_gets": {"currency": "XRP"},
///     "taker_pays": {
///         "currency": "524C555344000000000000000000000000000000",
///         "issuer": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De"
///     },
///     "limit": 3
/// }));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub struct BookOffersRequest {
    /// What the offer wants in return (`TakerPays`).
    pub taker_pays: Asset,
    /// What the offer gives (`TakerGets`).
    pub taker_gets: Asset,
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
    ///
    /// ```
    /// use xrpl_client::requests::BookOffersRequest;
    /// use xrpl_core::types::Asset;
    ///
    /// // What the offers give, then what they want.
    /// let usd = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
    /// let req = BookOffersRequest::new(Asset::xrp(), usd.clone());
    /// assert_eq!((req.taker_gets, req.taker_pays), (Asset::xrp(), usd));
    /// assert_eq!(req.ledger_index, None);
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn new(taker_gets: Asset, taker_pays: Asset) -> Self {
        Self {
            taker_gets,
            taker_pays,
            limit: None,
            taker: None,
            domain: None,
            ledger_index: None,
        }
    }

    /// The most entries per page (`limit`).
    #[must_use]
    pub fn with_limit(mut self, value: impl Into<u32>) -> Self {
        self.limit = Some(value.into());
        self
    }

    /// Compute funding from this account's point of view (`taker`).
    #[must_use]
    pub fn with_taker(mut self, value: impl Into<String>) -> Self {
        self.taker = Some(value.into());
        self
    }

    /// A permissioned domain's book (PermissionedDEX) (`domain`).
    #[must_use]
    pub fn with_domain(mut self, value: impl Into<String>) -> Self {
        self.domain = Some(value.into());
        self
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = Some(value.into());
        self
    }
}
request!(BookOffersRequest => "book_offers", BookOffersResult);

/// [`deposit_authorized`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/deposit_authorized):
/// whether `source_account` may pay `destination_account` directly.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{DepositAuthorizedRequest, Request};
///
/// // With a credential the payment would present.
/// let credential = "0DD4CB95CAE443CF942BC50407FE410E14E2D8D3AF9007B8C47A2A69AFA62573";
/// let req = DepositAuthorizedRequest::new(
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
/// )
/// .with_credentials(vec![credential.to_string()]);
/// assert_eq!(DepositAuthorizedRequest::METHOD, "deposit_authorized");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "source_account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "destination_account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "credentials": [credential], "ledger_index": "validated"
/// }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::DepositAuthorizedRequest;
    ///
    /// let req = DepositAuthorizedRequest::new(
    ///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
    ///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
    /// );
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "source_account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
    ///     "destination_account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "ledger_index": "validated"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(source_account: impl Into<String>, destination_account: impl Into<String>) -> Self {
        Self {
            source_account: source_account.into(),
            destination_account: destination_account.into(),
            ..Default::default()
        }
    }

    /// Credential IDs the payment would present (`credentials`).
    #[must_use]
    pub fn with_credentials(mut self, value: impl Into<Vec<String>>) -> Self {
        self.credentials = Some(value.into());
        self
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }
}
request!(DepositAuthorizedRequest => "deposit_authorized", DepositAuthorizedResult);

/// [`get_aggregate_price`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/get_aggregate_price):
/// mean, median and spread of several oracles' prices for one pair.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{GetAggregatePriceRequest, OracleId, Request};
///
/// let oracle =
///     OracleId { account: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".into(), oracle_document_id: 1 };
/// // Also the statistics with 20% of outliers trimmed (`trimmed_set`).
/// let req = GetAggregatePriceRequest::new("XRP", "USD", vec![oracle]).with_trim(20u32);
/// assert_eq!(GetAggregatePriceRequest::METHOD, "get_aggregate_price");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "base_asset": "XRP", "quote_asset": "USD", "trim": 20,
///     "oracles": [{"account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "oracle_document_id": 1}]
/// }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::{GetAggregatePriceRequest, OracleId};
    ///
    /// let oracle =
    ///     OracleId { account: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".into(), oracle_document_id: 1 };
    /// let req = GetAggregatePriceRequest::new("XRP", "USD", vec![oracle]);
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "base_asset": "XRP", "quote_asset": "USD",
    ///     "oracles": [{"account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "oracle_document_id": 1}]
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
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

    /// Percentage of outliers to drop for `trimmed_set` (1–25) (`trim`).
    #[must_use]
    pub fn with_trim(mut self, value: impl Into<u32>) -> Self {
        self.trim = Some(value.into());
        self
    }

    /// Ignore prices older than this many seconds before the newest (`time_threshold`).
    #[must_use]
    pub fn with_time_threshold(mut self, value: impl Into<u32>) -> Self {
        self.time_threshold = Some(value.into());
        self
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = Some(value.into());
        self
    }
}
request!(GetAggregatePriceRequest => "get_aggregate_price", AggregatePriceResult);

/// [`nft_buy_offers`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/nft_buy_offers):
/// offers to buy one NFT.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{NftBuyOffersRequest, Request};
///
/// let nft_id = "0000000047D6BDCD766717B67A937E83CFEF22B1760537E3EE0FC28306441448";
/// let req = NftBuyOffersRequest::new(nft_id).with_limit(50u32);
/// assert_eq!(NftBuyOffersRequest::METHOD, "nft_buy_offers");
/// assert_eq!(
///     serde_json::to_value(&req)?,
///     json!({"nft_id": nft_id, "ledger_index": "validated", "limit": 50})
/// );
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::NftBuyOffersRequest;
    ///
    /// let nft_id = "0000000047D6BDCD766717B67A937E83CFEF22B1760537E3EE0FC28306441448";
    /// let req = NftBuyOffersRequest::new(nft_id);
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "nft_id": nft_id, "ledger_index": "validated"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(nft_id: impl Into<String>) -> Self {
        Self {
            nft_id: nft_id.into(),
            ..Default::default()
        }
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }

    /// The most entries per page (`limit`).
    #[must_use]
    pub fn with_limit(mut self, value: impl Into<u32>) -> Self {
        self.limit = Some(value.into());
        self
    }

    /// Where the next page starts: pass it back in the request; absent on the last page (`marker`).
    #[must_use]
    pub fn with_marker(mut self, value: impl Into<Value>) -> Self {
        self.marker = Some(value.into());
        self
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{NftSellOffersRequest, Request};
///
/// let nft_id = "0000000047D6BDCD766717B67A937E83CFEF22B1760537E3EE0FC28306441448";
/// let req = NftSellOffersRequest::new(nft_id).with_limit(50u32);
/// assert_eq!(NftSellOffersRequest::METHOD, "nft_sell_offers");
/// assert_eq!(
///     serde_json::to_value(&req)?,
///     json!({"nft_id": nft_id, "ledger_index": "validated", "limit": 50})
/// );
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::NftSellOffersRequest;
    ///
    /// let nft_id = "0000000047D6BDCD766717B67A937E83CFEF22B1760537E3EE0FC28306441448";
    /// let req = NftSellOffersRequest::new(nft_id);
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "nft_id": nft_id, "ledger_index": "validated"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(nft_id: impl Into<String>) -> Self {
        Self {
            nft_id: nft_id.into(),
            ..Default::default()
        }
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }

    /// The most entries per page (`limit`).
    #[must_use]
    pub fn with_limit(mut self, value: impl Into<u32>) -> Self {
        self.limit = Some(value.into());
        self
    }

    /// Where the next page starts: pass it back in the request; absent on the last page (`marker`).
    #[must_use]
    pub fn with_marker(mut self, value: impl Into<Value>) -> Self {
        self.marker = Some(value.into());
        self
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{Request, RipplePathFindRequest};
///
/// // Ways to deliver 1 XRP, paying only in XRP.
/// let req = RipplePathFindRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", json!("1000000")).with_source_currencies(vec![json!({
///     "currency": "XRP"
/// })]);
/// assert_eq!(RipplePathFindRequest::METHOD, "ripple_path_find");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "source_account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "destination_account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "destination_amount": "1000000",
///     "source_currencies": [{"currency": "XRP"}], "ledger_index": "validated"
/// }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::RipplePathFindRequest;
    ///
    /// // Deliver 10 USD, from whatever the source holds.
    /// let usd = json!({
    ///     "currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "value": "10"
    /// });
    /// let req = RipplePathFindRequest::new(
    ///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
    ///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
    ///     usd.clone(),
    /// );
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "source_account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
    ///     "destination_account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
    ///     "destination_amount": usd, "ledger_index": "validated"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
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

    /// The most the source will spend (`send_max`).
    #[must_use]
    pub fn with_send_max(mut self, value: impl Into<Value>) -> Self {
        self.send_max = Some(value.into());
        self
    }

    /// Currencies the source can spend: `[{"currency", "issuer"?}]` (`source_currencies`).
    #[must_use]
    pub fn with_source_currencies(mut self, value: impl Into<Vec<Value>>) -> Self {
        self.source_currencies = Some(value.into());
        self
    }

    /// Only paths through this permissioned domain (`domain`).
    #[must_use]
    pub fn with_domain(mut self, value: impl Into<String>) -> Self {
        self.domain = Some(value.into());
        self
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }
}
request!(RipplePathFindRequest => "ripple_path_find", RipplePathFindResult);

/// [`path_find`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/path_find)
/// (WebSocket only): ask for paths that deliver `destination_amount`, kept
/// up to date with each ledger. [`PathFindRequest::new`] creates the search;
/// its first answer comes back, and the updates arrive as
/// [`WsEvent::PathFind`](crate::stream::WsEvent::PathFind) (see
/// `XrplWsClient::path_find`). One search per connection: a new one
/// replaces the last. [`PathFindRequest::close`] ends it,
/// [`PathFindRequest::status`] asks for the latest answer.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{PathFindRequest, Request};
///
/// // Start a search, spending at most 2 XRP; later, end it.
/// let create = PathFindRequest::new(
///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     json!("1000000"),
/// )
/// .with_send_max(json!("2000000"));
/// assert_eq!(PathFindRequest::METHOD, "path_find");
/// assert_eq!(serde_json::to_value(&create)?, json!({
///     "subcommand": "create", "source_account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///     "destination_account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "destination_amount": "1000000", "send_max": "2000000"
/// }));
/// assert_eq!(serde_json::to_value(PathFindRequest::close())?, json!({"subcommand": "close"}));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub struct PathFindRequest {
    /// `create`, `close` or `status`.
    pub subcommand: String,
    /// The sending account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_account: Option<String>,
    /// The receiving account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_account: Option<String>,
    /// Drops as a string, or `{"currency", "issuer", "value"}`; `"-1"` / value
    /// `"-1"` for as much as possible.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_amount: Option<Value>,
    /// The most the source will spend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_max: Option<Value>,
    /// Paths to start the search from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<Vec<PathStep>>>,
    /// Only paths through this permissioned domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}
impl PathFindRequest {
    /// Start a search for paths from `source_account` that deliver
    /// `destination_amount` to `destination_account` (`create`).
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::PathFindRequest;
    ///
    /// let req = PathFindRequest::new(
    ///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
    ///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
    ///     json!("1000000"),
    /// );
    /// assert_eq!(req.subcommand, "create");
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "subcommand": "create", "source_account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
    ///     "destination_account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
    ///     "destination_amount": "1000000"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(
        source_account: impl Into<String>,
        destination_account: impl Into<String>,
        destination_amount: Value,
    ) -> Self {
        Self {
            subcommand: "create".into(),
            source_account: Some(source_account.into()),
            destination_account: Some(destination_account.into()),
            destination_amount: Some(destination_amount),
            send_max: None,
            paths: None,
            domain: None,
        }
    }

    /// End the connection's search (`close`).
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::PathFindRequest;
    ///
    /// assert_eq!(serde_json::to_value(PathFindRequest::close())?, json!({"subcommand": "close"}));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn close() -> Self {
        Self::subcommand("close")
    }

    /// The connection's search's latest answer (`status`).
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::PathFindRequest;
    ///
    /// assert_eq!(serde_json::to_value(PathFindRequest::status())?, json!({
    ///     "subcommand": "status"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn status() -> Self {
        Self::subcommand("status")
    }

    fn subcommand(subcommand: &str) -> Self {
        Self {
            subcommand: subcommand.into(),
            source_account: None,
            destination_account: None,
            destination_amount: None,
            send_max: None,
            paths: None,
            domain: None,
        }
    }

    /// The most the source will spend (`send_max`).
    #[must_use]
    pub fn with_send_max(mut self, value: impl Into<Value>) -> Self {
        self.send_max = Some(value.into());
        self
    }

    /// Paths to start the search from (`paths`).
    #[must_use]
    pub fn with_paths(mut self, value: impl Into<Vec<Vec<PathStep>>>) -> Self {
        self.paths = Some(value.into());
        self
    }

    /// Only paths through this permissioned domain (`domain`).
    #[must_use]
    pub fn with_domain(mut self, value: impl Into<String>) -> Self {
        self.domain = Some(value.into());
        self
    }
}
request!(PathFindRequest => "path_find", PathFindResult);

// -------------------------------------------------------- payment channel

/// [`channel_verify`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/payment-channel-methods/channel_verify):
/// check a claim's signature on the server. (`xrpl_core::crypto::verify_payment_channel_claim`
/// does the same offline.)
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{ChannelVerifyRequest, Request};
///
/// # let channel_id = "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3";
/// # let public_key = "ED01FA53FA5A7E77798F882ECE20B1ABC00BB358A9E55A202D0D0676BD0CE37A63";
/// # let signature =
/// #     "F86EFC14E70AAC691277620DA42C71906948C232187A68C75C0825D9AC2B5F2C3EA483AC6BD0FD6FDFC54F2B92E9CFFF82FEF541D6639B67DC5F66BE875CED01";
/// // A claim of 1 XRP from the channel, signed by its key.
/// let req = ChannelVerifyRequest::new(channel_id, 1_000_000, public_key, signature);
/// assert_eq!(ChannelVerifyRequest::METHOD, "channel_verify");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "channel_id": channel_id, "amount": "1000000",
///     "public_key": public_key, "signature": signature
/// }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use xrpl_client::requests::ChannelVerifyRequest;
    ///
    /// # let channel_id = "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3";
    /// # let public_key = "ED01FA53FA5A7E77798F882ECE20B1ABC00BB358A9E55A202D0D0676BD0CE37A63";
    /// # let signature =
    /// #     "F86EFC14E70AAC691277620DA42C71906948C232187A68C75C0825D9AC2B5F2C3EA483AC6BD0FD6FDFC54F2B92E9CFFF82FEF541D6639B67DC5F66BE875CED01";
    /// let req = ChannelVerifyRequest::new(channel_id, 1_000_000, public_key, signature);
    /// // The amount, in drops, goes out as a string.
    /// assert_eq!(req.amount, "1000000");
    /// ```
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
///
/// ```
/// use xrpl_client::requests::{FeeRequest, Request};
///
/// assert_eq!(FeeRequest::METHOD, "fee");
/// // It has no parameters.
/// assert_eq!(serde_json::to_value(FeeRequest::new())?, serde_json::json!({}));
/// # Ok::<(), serde_json::Error>(())
/// ```
///
/// ```no_run
/// # async fn run(client: impl xrpl_client::Client) -> Result<(), xrpl_client::ClientError> {
/// use xrpl_client::requests::FeeRequest;
///
/// let fee = client.request(&FeeRequest::new()).await?;
/// println!("open ledger cost: {} drops", fee.drops.open_ledger_fee);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct FeeRequest {}
impl FeeRequest {
    /// The request with every optional parameter unset.
    ///
    /// ```
    /// use xrpl_client::requests::FeeRequest;
    ///
    /// assert_eq!(FeeRequest::new(), FeeRequest::default());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}
request!(FeeRequest => "fee", FeeResult);

/// [`feature`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/feature):
/// every amendment the server knows, and whether it is enabled.
///
/// ```
/// use xrpl_client::requests::{FeatureRequest, Request};
///
/// assert_eq!(FeatureRequest::METHOD, "feature");
/// // It has no parameters.
/// assert_eq!(serde_json::to_value(FeatureRequest::new())?, serde_json::json!({}));
/// # Ok::<(), serde_json::Error>(())
/// ```
///
/// ```no_run
/// # async fn run(client: impl xrpl_client::Client) -> Result<(), xrpl_client::ClientError> {
/// use xrpl_client::requests::FeatureRequest;
///
/// let known = client.request(&FeatureRequest::new()).await?;
/// let enabled = known.features.values().filter(|a| a.enabled).count();
/// println!("{enabled} of {} amendments enabled", known.features.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct FeatureRequest {}
impl FeatureRequest {
    /// The request with every optional parameter unset.
    ///
    /// ```
    /// use xrpl_client::requests::FeatureRequest;
    ///
    /// assert_eq!(FeatureRequest::new(), FeatureRequest::default());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}
request!(FeatureRequest => "feature", FeatureResult);

/// [`manifest`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/manifest):
/// a validator's manifest, by its public key (`n…`).
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{ManifestRequest, Request};
///
/// let req = ManifestRequest::new("nHUVPzAmAmQ2QSc4oE1iLfsGi17qN2ado8PhxvgEkou76FLxAz7C");
/// assert_eq!(ManifestRequest::METHOD, "manifest");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "public_key": "nHUVPzAmAmQ2QSc4oE1iLfsGi17qN2ado8PhxvgEkou76FLxAz7C"
/// }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct ManifestRequest {
    /// A public key, hex (`PublicKey`).
    pub public_key: String,
}
impl ManifestRequest {
    /// The request with its required parameters set and every optional one unset.
    ///
    /// ```
    /// use xrpl_client::requests::ManifestRequest;
    ///
    /// let req = ManifestRequest::new("nHUVPzAmAmQ2QSc4oE1iLfsGi17qN2ado8PhxvgEkou76FLxAz7C");
    /// assert_eq!(req.public_key, "nHUVPzAmAmQ2QSc4oE1iLfsGi17qN2ado8PhxvgEkou76FLxAz7C");
    /// ```
    pub fn new(public_key: impl Into<String>) -> Self {
        Self {
            public_key: public_key.into(),
        }
    }
}
request!(ManifestRequest => "manifest", ManifestResult);

/// [`server_definitions`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/server_definitions):
/// the server's binary-format definitions.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{Request, ServerDefinitionsRequest};
///
/// // Only the hash comes back if the server's definitions still have this one.
/// let req = ServerDefinitionsRequest::new()
///     .with_hash("1EA05B0FC11101F7C500BD0DAC794A8BC746A7FBA6250B75489603EB820E0FF5");
/// assert_eq!(ServerDefinitionsRequest::METHOD, "server_definitions");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "hash": "1EA05B0FC11101F7C500BD0DAC794A8BC746A7FBA6250B75489603EB820E0FF5"
/// }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct ServerDefinitionsRequest {
    /// The `hash` of definitions already held: the server then sends only the
    /// hash if nothing changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}
impl ServerDefinitionsRequest {
    /// The request with every optional parameter unset.
    ///
    /// ```
    /// use xrpl_client::requests::ServerDefinitionsRequest;
    ///
    /// let req = ServerDefinitionsRequest::new();
    /// assert_eq!(req.hash, None);
    /// assert_eq!(serde_json::to_value(&req)?, serde_json::json!({}));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// The `hash` of definitions already held: the server then sends only the hash if nothing changed (`hash`).
    #[must_use]
    pub fn with_hash(mut self, value: impl Into<String>) -> Self {
        self.hash = Some(value.into());
        self
    }
}
request!(ServerDefinitionsRequest => "server_definitions", ServerDefinitionsResult);

/// [`server_info`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/server_info):
/// the server's state in human units.
///
/// ```
/// use xrpl_client::requests::{ServerInfoRequest, Request};
///
/// assert_eq!(ServerInfoRequest::METHOD, "server_info");
/// // It has no parameters.
/// assert_eq!(serde_json::to_value(ServerInfoRequest::new())?, serde_json::json!({}));
/// # Ok::<(), serde_json::Error>(())
/// ```
///
/// ```no_run
/// # async fn run(client: impl xrpl_client::Client) -> Result<(), xrpl_client::ClientError> {
/// use xrpl_client::requests::ServerInfoRequest;
///
/// let answer = client.request(&ServerInfoRequest::new()).await?;
/// println!("{:?} {:?}", answer.info.build_version, answer.info.server_state);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct ServerInfoRequest {}
impl ServerInfoRequest {
    /// The request with every optional parameter unset.
    ///
    /// ```
    /// use xrpl_client::requests::ServerInfoRequest;
    ///
    /// assert_eq!(ServerInfoRequest::new(), ServerInfoRequest::default());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}
request!(ServerInfoRequest => "server_info", ServerInfoResult);

/// [`server_state`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/server_state):
/// the server's state in machine units.
///
/// ```
/// use xrpl_client::requests::{ServerStateRequest, Request};
///
/// assert_eq!(ServerStateRequest::METHOD, "server_state");
/// // It has no parameters.
/// assert_eq!(serde_json::to_value(ServerStateRequest::new())?, serde_json::json!({}));
/// # Ok::<(), serde_json::Error>(())
/// ```
///
/// ```no_run
/// # async fn run(client: impl xrpl_client::Client) -> Result<(), xrpl_client::ClientError> {
/// use xrpl_client::requests::ServerStateRequest;
///
/// let answer = client.request(&ServerStateRequest::new()).await?;
/// println!("load factor {:?}", answer.state.load_factor);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct ServerStateRequest {}
impl ServerStateRequest {
    /// The request with every optional parameter unset.
    ///
    /// ```
    /// use xrpl_client::requests::ServerStateRequest;
    ///
    /// assert_eq!(ServerStateRequest::new(), ServerStateRequest::default());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}
request!(ServerStateRequest => "server_state", ServerStateResult);

/// [`version`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/version):
/// the API versions the server supports.
///
/// ```
/// use xrpl_client::requests::{VersionRequest, Request};
///
/// assert_eq!(VersionRequest::METHOD, "version");
/// // It has no parameters.
/// assert_eq!(serde_json::to_value(VersionRequest::new())?, serde_json::json!({}));
/// # Ok::<(), serde_json::Error>(())
/// ```
///
/// ```no_run
/// # async fn run(client: impl xrpl_client::Client) -> Result<(), xrpl_client::ClientError> {
/// use xrpl_client::requests::VersionRequest;
///
/// let answer = client.request(&VersionRequest::new()).await?;
/// println!("API versions {} to {}", answer.version.first, answer.version.last);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct VersionRequest {}
impl VersionRequest {
    /// The request with every optional parameter unset.
    ///
    /// ```
    /// use xrpl_client::requests::VersionRequest;
    ///
    /// assert_eq!(VersionRequest::new(), VersionRequest::default());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}
request!(VersionRequest => "version", VersionResult);

// ---------------------------------------------------------------- utility

/// [`ping`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/utility-methods/ping).
///
/// ```
/// use xrpl_client::requests::{PingRequest, Request};
///
/// assert_eq!(PingRequest::METHOD, "ping");
/// // It has no parameters.
/// assert_eq!(serde_json::to_value(PingRequest::new())?, serde_json::json!({}));
/// # Ok::<(), serde_json::Error>(())
/// ```
///
/// ```no_run
/// # async fn run(client: impl xrpl_client::Client) -> Result<(), xrpl_client::ClientError> {
/// use xrpl_client::requests::PingRequest;
///
/// let answer = client.request(&PingRequest::new()).await?;
/// println!("role: {:?}", answer.role);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct PingRequest {}
impl PingRequest {
    /// The request with every optional parameter unset.
    ///
    /// ```
    /// use xrpl_client::requests::PingRequest;
    ///
    /// assert_eq!(PingRequest::new(), PingRequest::default());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}
request!(PingRequest => "ping", PingResult);

/// [`random`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/utility-methods/random).
///
/// ```
/// use xrpl_client::requests::{RandomRequest, Request};
///
/// assert_eq!(RandomRequest::METHOD, "random");
/// // It has no parameters.
/// assert_eq!(serde_json::to_value(RandomRequest::new())?, serde_json::json!({}));
/// # Ok::<(), serde_json::Error>(())
/// ```
///
/// ```no_run
/// # async fn run(client: impl xrpl_client::Client) -> Result<(), xrpl_client::ClientError> {
/// use xrpl_client::requests::RandomRequest;
///
/// let answer = client.request(&RandomRequest::new()).await?;
/// assert_eq!(answer.random.len(), 64);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct RandomRequest {}
impl RandomRequest {
    /// The request with every optional parameter unset.
    ///
    /// ```
    /// use xrpl_client::requests::RandomRequest;
    ///
    /// assert_eq!(RandomRequest::new(), RandomRequest::default());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}
request!(RandomRequest => "random", RandomResult);

// ------------------------------------------------------------------ vault

/// [`vault_info`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/vault-methods/vault_info):
/// a vault, by ID or by owner and sequence.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{Request, VaultInfoRequest};
///
/// // A vault by its owner and the sequence of the VaultCreate, instead of its ID.
/// let req = VaultInfoRequest::default()
///     .with_owner("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
///     .with_seq(5u32);
/// assert_eq!(VaultInfoRequest::METHOD, "vault_info");
/// assert_eq!(
///     serde_json::to_value(&req)?,
///     json!({
///         "owner": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "seq": 5, "ledger_index": "validated"
///     })
/// );
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use xrpl_client::requests::VaultInfoRequest;
    /// use xrpl_core::hashes::keylet;
    ///
    /// let id = keylet::vault("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", 5)?;
    /// let req = VaultInfoRequest::new(id.to_string());
    /// assert_eq!(req.vault_id, Some(id.to_string()));
    /// assert_eq!((req.owner, req.seq), (None, None));
    /// # Ok::<(), xrpl_core::CoreError>(())
    /// ```
    pub fn new(vault_id: impl Into<String>) -> Self {
        Self {
            vault_id: Some(vault_id.into()),
            ..Default::default()
        }
    }

    /// The account that owns the entry (`Owner`) (`owner`).
    #[must_use]
    pub fn with_owner(mut self, value: impl Into<String>) -> Self {
        self.owner = Some(value.into());
        self
    }

    /// The sequence of the transaction that created it (`seq`).
    #[must_use]
    pub fn with_seq(mut self, value: impl Into<u32>) -> Self {
        self.seq = Some(value.into());
        self
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }
}
request!(VaultInfoRequest => "vault_info", VaultInfoResult);

// ------------------------------------------------------------ Clio only

/// [`nft_info`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/nft_info)
/// (Clio): one NFT, including a burned one.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{NftInfoRequest, Request};
/// use xrpl_client::types::LedgerIndex;
///
/// let nft_id = "0000000047D6BDCD766717B67A937E83CFEF22B1760537E3EE0FC28306441448";
/// let req = NftInfoRequest::new(nft_id).with_ledger_index(LedgerIndex::Index(107830973));
/// assert_eq!(NftInfoRequest::METHOD, "nft_info");
/// assert_eq!(serde_json::to_value(&req)?, json!({"nft_id": nft_id, "ledger_index": 107830973}));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
pub struct NftInfoRequest {
    /// The NFT's ID.
    pub nft_id: String,
    /// The ledger the answer is from.
    pub ledger_index: LedgerIndex,
}
impl NftInfoRequest {
    /// The request with its required parameters set and every optional one unset.
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::NftInfoRequest;
    ///
    /// let nft_id = "0000000047D6BDCD766717B67A937E83CFEF22B1760537E3EE0FC28306441448";
    /// let req = NftInfoRequest::new(nft_id);
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "nft_id": nft_id, "ledger_index": "validated"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(nft_id: impl Into<String>) -> Self {
        Self {
            nft_id: nft_id.into(),
            ..Default::default()
        }
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }
}
request!(NftInfoRequest => "nft_info", NftInfoResult);

/// [`nft_history`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/nft_history)
/// (Clio): every transaction that touched an NFT.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{NftHistoryRequest, Request};
///
/// // The NFT's first two transactions, oldest first.
/// let nft_id = "0000000047D6BDCD766717B67A937E83CFEF22B1760537E3EE0FC28306441448";
/// let req = NftHistoryRequest::new(nft_id).with_forward(true).with_limit(2u32);
/// assert_eq!(NftHistoryRequest::METHOD, "nft_history");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "nft_id": nft_id, "ledger_index_min": -1, "ledger_index_max": -1,
///     "forward": true, "limit": 2
/// }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::NftHistoryRequest;
    ///
    /// // Every ledger the server has, newest first.
    /// let nft_id = "0000000047D6BDCD766717B67A937E83CFEF22B1760537E3EE0FC28306441448";
    /// let req = NftHistoryRequest::new(nft_id);
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "nft_id": nft_id, "ledger_index_min": -1, "ledger_index_max": -1, "forward": false
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(nft_id: impl Into<String>) -> Self {
        Self {
            nft_id: nft_id.into(),
            ..Default::default()
        }
    }

    /// The earliest ledger searched (`ledger_index_min`).
    #[must_use]
    pub fn with_ledger_index_min(mut self, value: impl Into<i32>) -> Self {
        self.ledger_index_min = value.into();
        self
    }

    /// The latest ledger searched (`ledger_index_max`).
    #[must_use]
    pub fn with_ledger_index_max(mut self, value: impl Into<i32>) -> Self {
        self.ledger_index_max = value.into();
        self
    }

    /// Hex instead of JSON (`binary`).
    #[must_use]
    pub fn with_binary(mut self, value: impl Into<bool>) -> Self {
        self.binary = Some(value.into());
        self
    }

    /// Oldest first (`forward`).
    #[must_use]
    pub fn with_forward(mut self, value: impl Into<bool>) -> Self {
        self.forward = value.into();
        self
    }

    /// The most entries per page (`limit`).
    #[must_use]
    pub fn with_limit(mut self, value: impl Into<u32>) -> Self {
        self.limit = Some(value.into());
        self
    }

    /// Where the next page starts: pass it back in the request; absent on the last page (`marker`).
    #[must_use]
    pub fn with_marker(mut self, value: impl Into<Value>) -> Self {
        self.marker = Some(value.into());
        self
    }
}
request!(NftHistoryRequest => "nft_history", NftHistoryResult);
paginated!(NftHistoryRequest, TxResult, transactions);

/// [`nfts_by_issuer`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/nfts_by_issuer)
/// (Clio): every NFT an account issued.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{NftsByIssuerRequest, Request};
///
/// let req = NftsByIssuerRequest::new("rfYiNfgLefTAZGfEyun1EjG68mTtC75vDe")
///     .with_nft_taxon(0u32)
///     .with_limit(2u32);
/// assert_eq!(NftsByIssuerRequest::METHOD, "nfts_by_issuer");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "issuer": "rfYiNfgLefTAZGfEyun1EjG68mTtC75vDe", "nft_taxon": 0, "ledger_index": "validated",
///     "limit": 2
/// }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::NftsByIssuerRequest;
    ///
    /// let req = NftsByIssuerRequest::new("rfYiNfgLefTAZGfEyun1EjG68mTtC75vDe");
    /// assert_eq!(serde_json::to_value(&req)?, json!({
    ///     "issuer": "rfYiNfgLefTAZGfEyun1EjG68mTtC75vDe", "ledger_index": "validated"
    /// }));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(issuer: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            ..Default::default()
        }
    }

    /// Only this taxon (`nft_taxon`).
    #[must_use]
    pub fn with_nft_taxon(mut self, value: impl Into<u32>) -> Self {
        self.nft_taxon = Some(value.into());
        self
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }

    /// The most entries per page (`limit`).
    #[must_use]
    pub fn with_limit(mut self, value: impl Into<u32>) -> Self {
        self.limit = Some(value.into());
        self
    }

    /// Where the next page starts: pass it back in the request; absent on the last page (`marker`).
    #[must_use]
    pub fn with_marker(mut self, value: impl Into<Value>) -> Self {
        self.marker = Some(value.into());
        self
    }
}
request!(NftsByIssuerRequest => "nfts_by_issuer", NftsByIssuerResult);
paginated!(NftsByIssuerRequest, NftInfo, nfts, pinned);

/// [`mpt_holders`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/mpt_holders)
/// (Clio): the holders of an MPT issuance.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::requests::{MptHoldersRequest, Request};
///
/// let req = MptHoldersRequest::new("014C29552B3DD7A7E525987A2FAFF91EECF53984A1FC7906")
///     .with_limit(50u32);
/// assert_eq!(MptHoldersRequest::METHOD, "mpt_holders");
/// assert_eq!(serde_json::to_value(&req)?, json!({
///     "mpt_issuance_id": "014C29552B3DD7A7E525987A2FAFF91EECF53984A1FC7906",
///     "ledger_index": "validated", "limit": 50
/// }));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[non_exhaustive]
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::requests::MptHoldersRequest;
    ///
    /// let req = MptHoldersRequest::new("014C29552B3DD7A7E525987A2FAFF91EECF53984A1FC7906");
    /// assert_eq!(
    ///     serde_json::to_value(&req)?,
    ///     json!({
    ///         "mpt_issuance_id": "014C29552B3DD7A7E525987A2FAFF91EECF53984A1FC7906",
    ///         "ledger_index": "validated"
    ///     })
    /// );
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn new(mpt_issuance_id: impl Into<String>) -> Self {
        Self {
            mpt_issuance_id: mpt_issuance_id.into(),
            ..Default::default()
        }
    }

    /// The ledger the answer is from (`ledger_index`).
    #[must_use]
    pub fn with_ledger_index(mut self, value: impl Into<LedgerIndex>) -> Self {
        self.ledger_index = value.into();
        self
    }

    /// The most entries per page (`limit`).
    #[must_use]
    pub fn with_limit(mut self, value: impl Into<u32>) -> Self {
        self.limit = Some(value.into());
        self
    }

    /// Where the next page starts: pass it back in the request; absent on the last page (`marker`).
    #[must_use]
    pub fn with_marker(mut self, value: impl Into<Value>) -> Self {
        self.marker = Some(value.into());
        self
    }
}
request!(MptHoldersRequest => "mpt_holders", MptHoldersResult);
paginated!(MptHoldersRequest, MptHolder, mptokens, pinned);
