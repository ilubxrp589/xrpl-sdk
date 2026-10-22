//! Responses of the path and order book methods: `book_offers`, `amm_info`,
//! `book_changes`, `deposit_authorized`, `get_aggregate_price`,
//! `nft_buy_offers` / `nft_sell_offers` and `ripple_path_find`.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use xrpl_core::types::Amount;

/// [`book_offers`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/book_offers):
/// an order book, best offer first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookOffersResult {
    pub offers: Vec<BookOffer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// One offer of an order book: its `Offer` entry plus what the owner can
/// actually fund.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookOffer {
    #[serde(rename = "Account")]
    pub account: String,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    #[serde(rename = "TakerGets")]
    pub taker_gets: Amount,
    #[serde(rename = "TakerPays")]
    pub taker_pays: Amount,
    /// TakerPays ÷ TakerGets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// What the owner holds of what it sells (on the first offer of each owner).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_funds: Option<String>,
    /// What the owner can actually deliver, when less than TakerGets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taker_gets_funded: Option<Amount>,
    /// TakerPays scaled to `taker_gets_funded`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taker_pays_funded: Option<Amount>,
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
    #[serde(
        rename = "BookDirectory",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub book_directory: Option<String>,
    /// The rest of the `Offer` entry (`BookNode`, `OwnerNode`, `PreviousTxnID`,
    /// `LedgerEntryType`, `index`, `DomainID`, …).
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

/// [`amm_info`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/amm_info):
/// an automated market maker's pool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmmInfoResult {
    pub amm: AmmInfo,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// An AMM pool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmmInfo {
    /// The AMM's own account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// The pool's balance of the first asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount2: Option<Amount>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_frozen: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset2_frozen: Option<bool>,
    /// LP tokens outstanding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lp_token: Option<Amount>,
    /// In units of 1/100,000 (1000 = 1%).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trading_fee: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auction_slot: Option<AuctionSlot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vote_slots: Option<Vec<VoteSlot>>,
}

/// The AMM's auction slot: who holds the discounted fee, until when.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuctionSlot {
    pub account: String,
    /// Accounts the holder lets trade at the discounted fee too.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_accounts: Option<Vec<AuthAccount>>,
    pub discounted_fee: u32,
    /// ISO 8601.
    pub expiration: String,
    pub price: Amount,
    pub time_interval: u32,
}

/// An account the auction slot holder authorised.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthAccount {
    pub account: String,
}

/// A liquidity provider's trading-fee vote.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoteSlot {
    pub account: String,
    pub trading_fee: u32,
    pub vote_weight: u32,
}

/// [`book_changes`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/book_changes):
/// each order book's open, high, low, close and volume in one ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookChangesResult {
    pub changes: Vec<BookChange>,
    /// Always `"bookChanges"`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_time: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// One book's trading in the ledger. Currencies are `XRP_drops` or
/// `issuer/currency`; rates are B per A.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookChange {
    pub currency_a: String,
    pub currency_b: String,
    pub volume_a: String,
    pub volume_b: String,
    pub high: String,
    pub low: String,
    pub open: String,
    pub close: String,
    /// The permissioned domain the book belongs to, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

/// [`deposit_authorized`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/deposit_authorized):
/// whether one account may pay another directly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositAuthorizedResult {
    pub deposit_authorized: bool,
    pub source_account: String,
    pub destination_account: String,
    /// Credentials the check took into account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credentials: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// [`get_aggregate_price`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/get_aggregate_price):
/// statistics over several price oracles' prices for one pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AggregatePriceResult {
    pub entire_set: PriceStats,
    /// The set without outliers (`trim` in the request).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trimmed_set: Option<PriceStats>,
    pub median: String,
    /// The latest `LastUpdateTime` among the prices used (Unix time).
    pub time: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// Mean, count and standard deviation of a set of prices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceStats {
    pub mean: String,
    pub size: u32,
    pub standard_deviation: String,
}

/// [`nft_buy_offers`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/nft_buy_offers)
/// and [`nft_sell_offers`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/nft_sell_offers):
/// the open offers for one NFT.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NftOffersResult {
    pub nft_id: String,
    pub offers: Vec<NftOffer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// One NFT offer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NftOffer {
    pub amount: Amount,
    /// 1 (`tfSellNFToken`) for a sell offer.
    pub flags: u32,
    /// The offer's ID, for NFTokenAcceptOffer / NFTokenCancelOffer.
    pub nft_offer_index: String,
    pub owner: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,
}

/// [`ripple_path_find`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/ripple_path_find):
/// ways to deliver an amount, and what each costs the sender.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RipplePathFindResult {
    pub alternatives: Vec<PathAlternative>,
    pub destination_account: String,
    pub destination_amount: Amount,
    /// Currencies the destination can receive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_currencies: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_account: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_reply: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// One way to pay: the paths to put in the Payment's `Paths`, and the most it
/// costs (`SendMax`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathAlternative {
    pub paths_computed: Vec<Vec<PathStep>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paths_canonical: Option<Vec<Value>>,
    /// What the sender spends. Kept as JSON: path finding can quote a token
    /// amount with more digits than a ledger amount holds (e.g.
    /// `"0.00001624579066564171"`), so it is not always an `Amount`.
    pub source_amount: Value,
    /// What the destination receives, when less than asked (partial payments).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_amount: Option<Value>,
}

/// One step of a payment path: an account to ripple through, or a currency
/// (and issuer) to convert into on the DEX.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathStep {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    /// Bits for which of the fields above are set (1 account, 16 currency, 32 issuer).
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub step_type: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_hex: Option<String>,
}
