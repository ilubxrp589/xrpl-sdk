//! Responses of the path and order book methods: `book_offers`, `amm_info`,
//! `book_changes`, `deposit_authorized`, `get_aggregate_price`,
//! `nft_buy_offers` / `nft_sell_offers` and `ripple_path_find`.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use xrpl_core::types::Amount;

/// [`book_offers`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/book_offers):
/// an order book, best offer first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BookOffersResult {
    /// The offers.
    pub offers: Vec<BookOffer>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
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

/// Both sides of an order book, from one ledger:
/// [`Client::order_book`](crate::Client::order_book). Prices are in `quote`
/// per `base`, in display units (XRP, not drops), as approximate `f64`s for
/// display and estimates; the offers themselves are exact.
///
/// ```no_run
/// # async fn run(client: impl xrpl_client::Client) -> Result<(), Box<dyn std::error::Error>> {
/// use xrpl_core::types::Asset;
///
/// let rlusd = Asset::token("RLUSD", "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De")?;
/// let book = client.order_book(&Asset::xrp(), &rlusd, 20).await?;
/// println!(
///     "XRP/RLUSD bid {:?} ask {:?} spread {:?}",
///     book.best_bid(),
///     book.best_ask(),
///     book.spread()
/// );
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OrderBook {
    /// What is bought and sold.
    pub base: xrpl_core::types::Asset,
    /// What it is priced in.
    pub quote: xrpl_core::types::Asset,
    /// Offers selling `base` for `quote` (their `TakerGets` is `base`),
    /// lowest price first.
    pub asks: Vec<BookOffer>,
    /// Offers buying `base` with `quote` (their `TakerPays` is `base`),
    /// highest price first.
    pub bids: Vec<BookOffer>,
    /// The ledger both sides are from.
    pub ledger_index: u32,
    /// Whether that ledger is validated.
    pub validated: bool,
}

impl OrderBook {
    /// An ask's price: what it asks in `quote` per `base` it sells.
    pub fn ask_price(offer: &BookOffer) -> f64 {
        offer.taker_pays.to_f64() / offer.taker_gets.to_f64()
    }

    /// A bid's price: what it pays in `quote` per `base` it buys.
    pub fn bid_price(offer: &BookOffer) -> f64 {
        offer.taker_gets.to_f64() / offer.taker_pays.to_f64()
    }

    /// The lowest price anyone sells `base` at.
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.first().map(Self::ask_price)
    }

    /// The highest price anyone buys `base` at.
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.first().map(Self::bid_price)
    }

    /// The best ask less the best bid (negative when the book is crossed,
    /// which the ledger settles as soon as an offer crosses).
    pub fn spread(&self) -> Option<f64> {
        Some(self.best_ask()? - self.best_bid()?)
    }

    /// Halfway between the best bid and the best ask.
    pub fn midpoint(&self) -> Option<f64> {
        Some((self.best_ask()? + self.best_bid()?) / 2.0)
    }
}

/// One offer of an order book: its `Offer` entry plus what the owner can
/// actually fund.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BookOffer {
    /// The offer's owner.
    #[serde(rename = "Account")]
    pub account: String,
    /// Flag bits (`Flags`).
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// The sequence number (`Sequence`).
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    /// What the offer gives (`TakerGets`).
    #[serde(rename = "TakerGets")]
    pub taker_gets: Amount,
    /// What the offer wants in return (`TakerPays`).
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
    /// When it expires (ledger time: seconds since 2000-01-01) (`Expiration`).
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
    /// The order book directory page holding the offer (`BookDirectory`).
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
#[non_exhaustive]
pub struct AmmInfoResult {
    /// The pool.
    pub amm: AmmInfo,
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

/// An AMM pool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AmmInfo {
    /// The AMM's own account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// The pool's balance of the first asset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
    /// The second asset's amount (`Amount2`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount2: Option<Amount>,
    /// Whether the first asset is frozen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_frozen: Option<bool>,
    /// Whether the second asset is frozen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset2_frozen: Option<bool>,
    /// LP tokens outstanding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lp_token: Option<Amount>,
    /// In units of 1/100,000 (1000 = 1%).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trading_fee: Option<u32>,
    /// The auction slot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auction_slot: Option<AuctionSlot>,
    /// The fee votes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vote_slots: Option<Vec<VoteSlot>>,
}

/// The AMM's auction slot: who holds the discounted fee, until when.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AuctionSlot {
    /// The slot holder.
    pub account: String,
    /// Accounts the holder lets trade at the discounted fee too.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_accounts: Option<Vec<AuctionSlotAccount>>,
    /// The slot holder's trading fee, in 1/100,000.
    pub discounted_fee: u32,
    /// ISO 8601.
    pub expiration: String,
    /// What the slot cost.
    pub price: Amount,
    /// Which 72-minute interval of the 24-hour slot is current (0–19).
    pub time_interval: u32,
}

/// An account the auction slot holder authorised, as `amm_info` writes it
/// (`{"account": …}`; a transaction's `AuthAccounts` entry is
/// [`xrpl_core::transaction::models::AuthAccount`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AuctionSlotAccount {
    /// The account.
    pub account: String,
}

/// A liquidity provider's trading-fee vote.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VoteSlot {
    /// The voting liquidity provider.
    pub account: String,
    /// The trading fee, in 1/100,000 (1000 = 1%) (`TradingFee`).
    pub trading_fee: u32,
    /// The voter's weight, in 1/100,000 of the LP tokens.
    pub vote_weight: u32,
}

/// [`book_changes`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/book_changes):
/// each order book's open, high, low, close and volume in one ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BookChangesResult {
    /// Each book's trading.
    pub changes: Vec<BookChange>,
    /// Always `"bookChanges"`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// That ledger's hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    /// The ledger the answer is from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    /// When the ledger closed (ledger time).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_time: Option<u64>,
    /// Whether the data is from a validated ledger (final).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// One book's trading in the ledger. Currencies are `XRP_drops` or
/// `issuer/currency`; rates are B per A.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BookChange {
    /// The first currency (`XRP_drops` or `issuer/currency`).
    pub currency_a: String,
    /// The second currency.
    pub currency_b: String,
    /// How much of the first currency traded.
    pub volume_a: String,
    /// How much of the second currency traded.
    pub volume_b: String,
    /// The highest rate.
    pub high: String,
    /// The lowest rate.
    pub low: String,
    /// The first rate.
    pub open: String,
    /// The last rate.
    pub close: String,
    /// The permissioned domain the book belongs to, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

/// [`deposit_authorized`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/deposit_authorized):
/// whether one account may pay another directly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DepositAuthorizedResult {
    /// Whether the source may pay the destination.
    pub deposit_authorized: bool,
    /// The sending account.
    pub source_account: String,
    /// The receiving account.
    pub destination_account: String,
    /// Credentials the check took into account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credentials: Option<Vec<String>>,
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

/// [`get_aggregate_price`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/get_aggregate_price):
/// statistics over several price oracles' prices for one pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AggregatePriceResult {
    /// Statistics over every price.
    pub entire_set: PriceStats,
    /// The set without outliers (`trim` in the request).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trimmed_set: Option<PriceStats>,
    /// The median price.
    pub median: String,
    /// The latest `LastUpdateTime` among the prices used (Unix time).
    pub time: u64,
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

/// Mean, count and standard deviation of a set of prices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PriceStats {
    /// The mean price.
    pub mean: String,
    /// How many prices.
    pub size: u32,
    /// The standard deviation.
    pub standard_deviation: String,
}

/// [`nft_buy_offers`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/nft_buy_offers)
/// and [`nft_sell_offers`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/nft_sell_offers):
/// the open offers for one NFT.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NftOffersResult {
    /// The NFT's ID.
    pub nft_id: String,
    /// The offers.
    pub offers: Vec<NftOffer>,
    /// The most entries per page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
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

/// One NFT offer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NftOffer {
    /// The price.
    pub amount: Amount,
    /// 1 (`tfSellNFToken`) for a sell offer.
    pub flags: u32,
    /// The offer's ID, for NFTokenAcceptOffer / NFTokenCancelOffer.
    pub nft_offer_index: String,
    /// The offer's owner.
    pub owner: String,
    /// The account that receives it (`Destination`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    /// When it expires (ledger time: seconds since 2000-01-01) (`Expiration`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,
}

/// [`ripple_path_find`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/ripple_path_find):
/// ways to deliver an amount, and what each costs the sender.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RipplePathFindResult {
    /// The ways to pay.
    pub alternatives: Vec<PathAlternative>,
    /// The receiving account.
    pub destination_account: String,
    /// What the destination receives.
    pub destination_amount: Amount,
    /// Currencies the destination can receive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_currencies: Option<Vec<String>>,
    /// The sending account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_account: Option<String>,
    /// Whether the search finished.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_reply: Option<bool>,
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

/// An answer of [`path_find`](crate::requests::PathFindRequest): the first
/// one, each update (as [`WsEvent::PathFind`](crate::stream::WsEvent::PathFind)),
/// and the last (`closed`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PathFindResult {
    /// The ways to pay, best first.
    pub alternatives: Vec<PathAlternative>,
    /// The receiving account.
    pub destination_account: String,
    /// What the destination receives.
    pub destination_amount: Amount,
    /// The sending account.
    pub source_account: String,
    /// Whether the search is complete (the first answer often is not; an
    /// update completes it).
    pub full_reply: bool,
    /// The request's `id`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    /// Whether the search was closed (the answer to `close`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
}

/// One way to pay: the paths to put in the Payment's `Paths`, and the most it
/// costs (`SendMax`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PathAlternative {
    /// The paths to use.
    pub paths_computed: Vec<Vec<PathStep>>,
    /// Rippled's canonical paths (usually empty).
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

/// One step of a payment path: the transaction models' type, so a path
/// `ripple_path_find` returns goes into `Payment::paths` as it is.
pub use xrpl_core::transaction::models::PathStep;
