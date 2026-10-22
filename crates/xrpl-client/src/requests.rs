//! Typed rippled requests. Each names its method and the type its `result`
//! decodes into; [`crate::Client::request`] sends any of them over HTTP or
//! WebSocket alike.

use crate::types::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use xrpl_core::types::LedgerObject;

/// A typed rippled request: the method it calls and the type of its `result`.
pub trait Request: Serialize {
    /// The rippled method (`account_info`, `fee`, …).
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
}

/// `account_info`
#[derive(Debug, Clone, Serialize)]
pub struct AccountInfoRequest {
    pub account: String,
    pub ledger_index: LedgerIndex,
}
request!(AccountInfoRequest => "account_info", AccountInfo);

/// `fee`
#[derive(Debug, Clone, Default, Serialize)]
pub struct FeeRequest {}
request!(FeeRequest => "fee", FeeResult);

/// `ledger_current`
#[derive(Debug, Clone, Default, Serialize)]
pub struct LedgerCurrentRequest {}
request!(LedgerCurrentRequest => "ledger_current", LedgerCurrentResult);

/// `ledger`
#[derive(Debug, Clone, Serialize)]
pub struct LedgerRequest {
    pub ledger_index: LedgerIndex,
}
request!(LedgerRequest => "ledger", LedgerResult);

/// `submit` (a signed blob)
#[derive(Debug, Clone, Serialize)]
pub struct SubmitRequest {
    pub tx_blob: String,
}
request!(SubmitRequest => "submit", SubmitResult);

/// `tx`
#[derive(Debug, Clone, Serialize)]
pub struct TxRequest {
    pub transaction: String,
    pub binary: bool,
}
request!(TxRequest => "tx", TxResult);

/// `account_lines`
#[derive(Debug, Clone, Serialize)]
pub struct AccountLinesRequest {
    pub account: String,
    pub ledger_index: LedgerIndex,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
request!(AccountLinesRequest => "account_lines", AccountLinesResult);
paginated!(AccountLinesRequest, TrustLine, lines);

/// `account_offers`
#[derive(Debug, Clone, Serialize)]
pub struct AccountOffersRequest {
    pub account: String,
    pub ledger_index: LedgerIndex,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
request!(AccountOffersRequest => "account_offers", AccountOffersResult);
paginated!(AccountOffersRequest, AccountOffer, offers);

/// `account_nfts`
#[derive(Debug, Clone, Serialize)]
pub struct AccountNftsRequest {
    pub account: String,
    pub ledger_index: LedgerIndex,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
request!(AccountNftsRequest => "account_nfts", AccountNftsResult);
paginated!(AccountNftsRequest, NFToken, account_nfts);

/// `book_offers`
#[derive(Debug, Clone, Serialize)]
pub struct BookOffersRequest {
    pub taker_pays: Value,
    pub taker_gets: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}
request!(BookOffersRequest => "book_offers", BookOffersResult);

/// `amm_info`
#[derive(Debug, Clone, Serialize)]
pub struct AmmInfoRequest {
    pub asset: Value,
    pub asset2: Value,
}
request!(AmmInfoRequest => "amm_info", AmmInfoResult);

/// `account_tx` (`-1` bounds mean the earliest / latest validated ledger)
#[derive(Debug, Clone, Serialize)]
pub struct AccountTxRequest {
    pub account: String,
    pub ledger_index_min: i32,
    pub ledger_index_max: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    pub forward: bool,
}
request!(AccountTxRequest => "account_tx", AccountTxResult);
paginated!(AccountTxRequest, AccountTxEntry, transactions);

/// `account_objects`
#[derive(Debug, Clone, Serialize)]
pub struct AccountObjectsRequest {
    pub account: String,
    /// Only objects of this type (`offer`, `state`, `check`, …).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    pub ledger_index: LedgerIndex,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
}
request!(AccountObjectsRequest => "account_objects", AccountObjectsResult);
paginated!(AccountObjectsRequest, LedgerObject, account_objects);

/// `account_currencies`
#[derive(Debug, Clone, Serialize)]
pub struct AccountCurrenciesRequest {
    pub account: String,
    pub ledger_index: LedgerIndex,
}
request!(AccountCurrenciesRequest => "account_currencies", AccountCurrenciesResult);

/// `account_channels`
#[derive(Debug, Clone, Serialize)]
pub struct AccountChannelsRequest {
    pub account: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_account: Option<String>,
    pub ledger_index: LedgerIndex,
}
request!(AccountChannelsRequest => "account_channels", AccountChannelsResult);

/// `ledger_entry` by object ID
#[derive(Debug, Clone, Serialize)]
pub struct LedgerEntryRequest {
    pub index: String,
    pub ledger_index: LedgerIndex,
}
request!(LedgerEntryRequest => "ledger_entry", LedgerEntryResult);

/// `server_info`
#[derive(Debug, Clone, Default, Serialize)]
pub struct ServerInfoRequest {}
request!(ServerInfoRequest => "server_info", ServerInfoResult);

/// `gateway_balances`
#[derive(Debug, Clone, Serialize)]
pub struct GatewayBalancesRequest {
    pub account: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hotwallet: Option<Vec<String>>,
    pub ledger_index: LedgerIndex,
}
request!(GatewayBalancesRequest => "gateway_balances", GatewayBalancesResult);
