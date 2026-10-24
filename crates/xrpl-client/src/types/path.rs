//! Responses of the path and order book methods: `book_offers`, `amm_info`,
//! `book_changes`, `deposit_authorized`, `get_aggregate_price`,
//! `nft_buy_offers` / `nft_sell_offers` and `ripple_path_find`.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use xrpl_core::types::Amount;

/// [`book_offers`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/path-and-order-book-methods/book_offers):
/// an order book, best offer first.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::BookOffersResult;
/// use xrpl_core::types::Amount;
///
/// let book: BookOffersResult = serde_json::from_value(json!({
///     "offers": [{
///         "Account": "rLhB6qhWbVsRSRePxPLhXJFtpk2uLqpsQD", "Flags": 0, "Sequence": 107449413,
///         "TakerGets": "14120000",
///         "TakerPays": {
///             "currency": "524C555344000000000000000000000000000000",
///             "issuer": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "value": "18.385652"
///         },
///         "owner_funds": "559663972", "quality": "0.0000013021"
///     }],
///     "ledger_index": 107856070, "validated": true
/// }))?;
/// assert_eq!(book.offers[0].taker_gets, Amount::Xrp(14_120_000));
/// # Ok::<(), serde_json::Error>(())
/// ```
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
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::types::{BookOffer, OrderBook};
    ///
    /// // Selling 10 XRP for 5 USD: 0.5 USD per XRP.
    /// let ask: BookOffer = serde_json::from_value(json!({
    ///     "Account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "Flags": 0, "Sequence": 1,
    ///     "TakerGets": "10000000",
    ///     "TakerPays": {
    ///         "currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "value": "5"
    ///     }
    /// }))?;
    /// assert_eq!(OrderBook::ask_price(&ask), 0.5);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn ask_price(offer: &BookOffer) -> f64 {
        offer.taker_pays.to_f64() / offer.taker_gets.to_f64()
    }

    /// A bid's price: what it pays in `quote` per `base` it buys.
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::types::{BookOffer, OrderBook};
    ///
    /// // Paying 4.5 USD for 10 XRP: 0.45 USD per XRP.
    /// let bid: BookOffer = serde_json::from_value(json!({
    ///     "Account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "Flags": 0, "Sequence": 1,
    ///     "TakerGets": {
    ///         "currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "value": "4.5"
    ///     },
    ///     "TakerPays": "10000000"
    /// }))?;
    /// assert_eq!(OrderBook::bid_price(&bid), 0.45);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn bid_price(offer: &BookOffer) -> f64 {
        offer.taker_gets.to_f64() / offer.taker_pays.to_f64()
    }

    /// The lowest price anyone sells `base` at.
    ///
    /// ```
    /// # use serde_json::{json, Value};
    /// # use xrpl_client::types::OrderBook;
    /// # let usd = |v: &str| json!({
    /// #     "currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "value": v
    /// # });
    /// # let offer = |gets: Value, pays: Value| {
    /// #     json!({"Account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "Flags": 0, "Sequence": 1,
    /// #         "TakerGets": gets, "TakerPays": pays})
    /// # };
    /// # let book: OrderBook = serde_json::from_value(json!({
    /// #     "base": {"currency": "XRP"},
    /// #     "quote": {"currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"},
    /// #     "asks": [offer(json!("10000000"), usd("5")), offer(json!("10000000"), usd("6"))],
    /// #     "bids": [offer(usd("4.5"), json!("10000000"))],
    /// #     "ledger_index": 107856070, "validated": true
    /// # }))?;
    /// // `book`: XRP priced in USD. Its best ask sells 10 XRP for 5 USD, its
    /// // best bid buys 10 XRP for 4.5 USD.
    /// assert_eq!(book.best_ask(), Some(0.5));
    /// assert_eq!(book.best_ask(), book.asks.first().map(OrderBook::ask_price));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.first().map(Self::ask_price)
    }

    /// The highest price anyone buys `base` at.
    ///
    /// ```
    /// # use serde_json::{json, Value};
    /// # use xrpl_client::types::OrderBook;
    /// # let usd = |v: &str| json!({
    /// #     "currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "value": v
    /// # });
    /// # let offer = |gets: Value, pays: Value| {
    /// #     json!({"Account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "Flags": 0, "Sequence": 1,
    /// #         "TakerGets": gets, "TakerPays": pays})
    /// # };
    /// # let book: OrderBook = serde_json::from_value(json!({
    /// #     "base": {"currency": "XRP"},
    /// #     "quote": {"currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"},
    /// #     "asks": [offer(json!("10000000"), usd("5")), offer(json!("10000000"), usd("6"))],
    /// #     "bids": [offer(usd("4.5"), json!("10000000"))],
    /// #     "ledger_index": 107856070, "validated": true
    /// # }))?;
    /// // `book`: XRP priced in USD. Its best ask sells 10 XRP for 5 USD, its
    /// // best bid buys 10 XRP for 4.5 USD.
    /// assert_eq!(book.best_bid(), Some(0.45));
    /// assert_eq!(book.best_bid(), book.bids.first().map(OrderBook::bid_price));
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.first().map(Self::bid_price)
    }

    /// The best ask less the best bid (negative when the book is crossed,
    /// which the ledger settles as soon as an offer crosses).
    ///
    /// ```
    /// # use serde_json::{json, Value};
    /// # use xrpl_client::types::OrderBook;
    /// # let usd = |v: &str| json!({
    /// #     "currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "value": v
    /// # });
    /// # let offer = |gets: Value, pays: Value| {
    /// #     json!({"Account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "Flags": 0, "Sequence": 1,
    /// #         "TakerGets": gets, "TakerPays": pays})
    /// # };
    /// # let book: OrderBook = serde_json::from_value(json!({
    /// #     "base": {"currency": "XRP"},
    /// #     "quote": {"currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"},
    /// #     "asks": [offer(json!("10000000"), usd("5")), offer(json!("10000000"), usd("6"))],
    /// #     "bids": [offer(usd("4.5"), json!("10000000"))],
    /// #     "ledger_index": 107856070, "validated": true
    /// # }))?;
    /// // `book`: XRP priced in USD. Its best ask sells 10 XRP for 5 USD, its
    /// // best bid buys 10 XRP for 4.5 USD.
    /// assert!((book.spread().unwrap() - 0.05).abs() < 1e-12);
    /// // No spread while either side is empty.
    /// let mut one_sided = book.clone();
    /// one_sided.bids.clear();
    /// assert_eq!(one_sided.spread(), None);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn spread(&self) -> Option<f64> {
        Some(self.best_ask()? - self.best_bid()?)
    }

    /// Halfway between the best bid and the best ask.
    ///
    /// ```
    /// # use serde_json::{json, Value};
    /// # use xrpl_client::types::OrderBook;
    /// # let usd = |v: &str| json!({
    /// #     "currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "value": v
    /// # });
    /// # let offer = |gets: Value, pays: Value| {
    /// #     json!({"Account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "Flags": 0, "Sequence": 1,
    /// #         "TakerGets": gets, "TakerPays": pays})
    /// # };
    /// # let book: OrderBook = serde_json::from_value(json!({
    /// #     "base": {"currency": "XRP"},
    /// #     "quote": {"currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"},
    /// #     "asks": [offer(json!("10000000"), usd("5")), offer(json!("10000000"), usd("6"))],
    /// #     "bids": [offer(usd("4.5"), json!("10000000"))],
    /// #     "ledger_index": 107856070, "validated": true
    /// # }))?;
    /// // `book`: XRP priced in USD. Its best ask sells 10 XRP for 5 USD, its
    /// // best bid buys 10 XRP for 4.5 USD.
    /// assert!((book.midpoint().unwrap() - 0.475).abs() < 1e-12);
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn midpoint(&self) -> Option<f64> {
        Some((self.best_ask()? + self.best_bid()?) / 2.0)
    }
}

/// One offer of an order book: its `Offer` entry plus what the owner can
/// actually fund.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::{BookOffer, OrderBook};
///
/// let offer: BookOffer = serde_json::from_value(json!({
///     "Account": "rLhB6qhWbVsRSRePxPLhXJFtpk2uLqpsQD", "Flags": 0, "Sequence": 107449413,
///     "TakerGets": "14120000",
///     "TakerPays": {
///         "currency": "524C555344000000000000000000000000000000",
///         "issuer": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "value": "18.385652"
///     },
///     "owner_funds": "559663972", "quality": "0.0000013021", "BookNode": "0"
/// }))?;
/// // It sells 14.12 XRP for 18.385652 RLUSD: 1.3021 RLUSD per XRP.
/// assert!((OrderBook::ask_price(&offer) - 1.3021).abs() < 1e-9);
/// assert_eq!(offer.owner_funds.as_deref(), Some("559663972"));
/// assert_eq!(offer.other["BookNode"], "0"); // the rest of the `Offer` entry
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::AmmInfoResult;
///
/// let answer: AmmInfoResult = serde_json::from_value(json!({
///     "amm": {
///         "account": "rhWTXC2m2gGGA9WozUaoMm6kLAVPb1tcS3",
///         "amount": "1849156969805",
///         "amount2": {
///             "currency": "524C555344000000000000000000000000000000",
///             "issuer": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "value": "2406585.019629636"
///         },
///         "trading_fee": 205
///     },
///     "ledger_index": 107856070, "validated": true
/// }))?;
/// assert_eq!(answer.amm.trading_fee, Some(205)); // 0.205%
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::AmmInfo;
/// use xrpl_core::types::Amount;
///
/// let pool: AmmInfo = serde_json::from_value(json!({
///     "account": "rhWTXC2m2gGGA9WozUaoMm6kLAVPb1tcS3",
///     "amount": "1849156969805",
///     "amount2": {
///         "currency": "524C555344000000000000000000000000000000",
///         "issuer": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "value": "2406585.019629636"
///     },
///     "trading_fee": 205
/// }))?;
/// assert_eq!(pool.amount, Some(Amount::Xrp(1_849_156_969_805)));
/// // The pool's price of XRP: about 1.30 RLUSD.
/// let price = pool.amount2.unwrap().to_f64() / pool.amount.unwrap().to_f64();
/// assert!((1.30..1.31).contains(&price));
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::AuctionSlot;
///
/// let slot: AuctionSlot = serde_json::from_value(json!({
///     "account": "r4uKh2rHuarMZdAgSBYqqicX7MxPBegttw",
///     "auth_accounts": [{"account": "rNy92d5CrBQ1eC5HUokq1LDfcuLPvTfh4X"}],
///     "discounted_fee": 20, "expiration": "2026-03-02T17:05:00+0000",
///     "price": {"currency": "03B245BE580EC4F4386A751C084489EC4B514A2F",
///         "issuer": "rhWTXC2m2gGGA9WozUaoMm6kLAVPb1tcS3", "value": "131482.3939333034"},
///     "time_interval": 20
/// }))?;
/// assert_eq!(slot.discounted_fee, 20); // 0.02% for the holder
/// assert_eq!(slot.auth_accounts.unwrap()[0].account, "rNy92d5CrBQ1eC5HUokq1LDfcuLPvTfh4X");
/// # Ok::<(), serde_json::Error>(())
/// ```
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
    /// Which 72-minute interval of the 24-hour slot is current: 0–19 while
    /// the slot runs, 20 once it has expired.
    pub time_interval: u32,
}

/// An account the auction slot holder authorised, as `amm_info` writes it
/// (`{"account": …}`; a transaction's `AuthAccounts` entry is
/// [`xrpl_core::transaction::models::AuthAccount`]).
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::AuctionSlotAccount;
///
/// let authorized: AuctionSlotAccount =
///     serde_json::from_value(json!({"account": "rNy92d5CrBQ1eC5HUokq1LDfcuLPvTfh4X"}))?;
/// assert_eq!(authorized.account, "rNy92d5CrBQ1eC5HUokq1LDfcuLPvTfh4X");
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AuctionSlotAccount {
    /// The account.
    pub account: String,
}

/// A liquidity provider's trading-fee vote.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::VoteSlot;
///
/// let vote: VoteSlot = serde_json::from_value(json!({
///     "account": "rJ4XeEzorvsQp6un4h9CjtANEsB3Qcc8iA", "trading_fee": 300, "vote_weight": 2936
/// }))?;
/// // A vote for 0.3%, from 2.936% of the LP tokens.
/// assert_eq!((vote.trading_fee, vote.vote_weight), (300, 2936));
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::BookChangesResult;
///
/// let changes: BookChangesResult = serde_json::from_value(json!({
///     "type": "bookChanges", "changes": [{
///         "currency_a": "XRP_drops",
///         "currency_b": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De/524C555344000000000000000000000000000000",
///         "volume_a": "862124", "volume_b": "1.13149388782",
///         "open": "762031", "high": "762031", "low": "761187.2459410326",
///         "close": "761202.9106741559"
///     }],
///     "ledger_index": 107831526, "ledger_time": 846004322, "validated": true
/// }))?;
/// assert_eq!(changes.kind.as_deref(), Some("bookChanges"));
/// assert_eq!(changes.changes[0].currency_a, "XRP_drops");
/// # Ok::<(), serde_json::Error>(())
/// ```
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
/// `issuer/currency`; rates (`open`, `high`, `low`, `close`) are A per B
/// (the volume of A traded for one of B).
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::BookChange;
///
/// let change: BookChange = serde_json::from_value(json!({
///     "currency_a": "XRP_drops",
///     "currency_b": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De/524C555344000000000000000000000000000000",
///     "volume_a": "862124", "volume_b": "1.13149388782",
///     "open": "762031", "high": "762031", "low": "761187.2459410326", "close": "761202.9106741559"
/// }))?;
/// // XRP against RLUSD (`issuer/currency`), in this ledger.
/// assert_eq!(
///     change.currency_b,
///     "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De/524C555344000000000000000000000000000000",
/// );
/// assert_eq!(change.volume_a, "862124"); // drops
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::DepositAuthorizedResult;
///
/// let answer: DepositAuthorizedResult = serde_json::from_value(json!({
///     "deposit_authorized": false,
///     "source_account": "rURtT5MMFzSUR54MAq9MKfzBoezTBZbKns",
///     "destination_account": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De",
///     "ledger_index": 107831520, "validated": true
/// }))?;
/// // The destination has DepositAuth set, and has not preauthorized this source.
/// assert!(!answer.deposit_authorized);
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::AggregatePriceResult;
///
/// let price: AggregatePriceResult = serde_json::from_value(json!({
///     "entire_set": {"mean": "0.74", "size": 1, "standard_deviation": "0"},
///     "median": "0.74", "time": 1792785775, "ledger_current_index": 21768546, "validated": false
/// }))?;
/// assert_eq!(price.median, "0.74");
/// assert!(price.trimmed_set.is_none()); // the request had no `trim`
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::PriceStats;
///
/// let stats: PriceStats = serde_json::from_value(json!({
///     "mean": "0.74", "size": 1, "standard_deviation": "0"
/// }))?;
/// assert_eq!((stats.mean.as_str(), stats.size), ("0.74", 1));
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::NftOffersResult;
///
/// let page: NftOffersResult = serde_json::from_value(json!({
///     "nft_id": "000801F42B3DD7A7E525987A2FAFF91EECF53984A1FC79061A7C53EB014C2951",
///     "offers": [{
///         "amount": "1", "flags": 1, "owner": "rhAe93ohNJkzvgD2rutsfnayqccnnJgS8W",
///         "nft_offer_index": "83C136C3A2BE2115D5C9D0B36B3BD99B71F6580B824D6FF7705A018344B97E37"
///     }]
/// }))?;
/// assert_eq!(page.offers[0].owner, "rhAe93ohNJkzvgD2rutsfnayqccnnJgS8W");
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::NftOffer;
/// use xrpl_core::types::Amount;
///
/// let offer: NftOffer = serde_json::from_value(json!({
///     "amount": "1", "flags": 1, "owner": "rhAe93ohNJkzvgD2rutsfnayqccnnJgS8W",
///     "nft_offer_index": "83C136C3A2BE2115D5C9D0B36B3BD99B71F6580B824D6FF7705A018344B97E37"
/// }))?;
/// assert_eq!(offer.amount, Amount::Xrp(1));
/// assert_eq!(offer.flags & 1, 1); // a sell offer
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::RipplePathFindResult;
/// use xrpl_core::types::Amount;
///
/// # let alternative = json!({
/// #     "paths_computed": [
/// #         [
/// #             {"account": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "type": 1},
/// #             {"currency": "XRP", "type": 16}
/// #         ]
/// #     ],
/// #     "source_amount": {
/// #         "currency": "524C555344000000000000000000000000000000",
/// #         "issuer": "rURtT5MMFzSUR54MAq9MKfzBoezTBZbKns", "value": "1.31362"
/// #     }
/// # });
/// // `alternative`: one way to pay, 1.31362 RLUSD (see `PathAlternative`).
/// let found: RipplePathFindResult = serde_json::from_value(json!({
///     "alternatives": [alternative],
///     "destination_account": "rfYiNfgLefTAZGfEyun1EjG68mTtC75vDe", "destination_amount": "1000000",
///     "full_reply": true, "ledger_index": 107831520, "validated": true
/// }))?;
/// assert_eq!(found.destination_amount, Amount::Xrp(1_000_000));
/// assert_eq!(found.alternatives[0].source_amount["value"], "1.31362");
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::PathFindResult;
///
/// # let alternative = json!({
/// #     "paths_computed": [
/// #         [{
/// #             "currency": "524C555344000000000000000000000000000000",
/// #             "issuer": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "type": 48
/// #         }]
/// #     ],
/// #     "source_amount": "767241"
/// # });
/// // `alternative`: one way to pay, 0.767241 XRP (see `PathAlternative`).
/// let first: PathFindResult = serde_json::from_value(json!({
///     "alternatives": [alternative],
///     "source_account": "rURtT5MMFzSUR54MAq9MKfzBoezTBZbKns",
///     "destination_account": "rpYNm4xpK7aPC9EVqdpkdrTZjCLUVGc4kA",
///     "destination_amount": {
///         "currency": "524C555344000000000000000000000000000000",
///         "issuer": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "value": "1"
///     },
///     "full_reply": false, "id": 2
/// }))?;
/// assert!(!first.full_reply); // an update completes the search
/// assert_eq!(first.alternatives[0].source_amount, json!("767241")); // drops
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::PathAlternative;
///
/// let way: PathAlternative = serde_json::from_value(json!({
///     "paths_computed": [[
///         {"account": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B", "type": 1},
///         {"currency": "XRP", "type": 16}
///     ]],
///     "source_amount": {
///         "currency": "BTC", "issuer": "rURtT5MMFzSUR54MAq9MKfzBoezTBZbKns",
///         "value": "0.00001624579066564171"
///     }
/// }))?;
/// let step = &way.paths_computed[0][0];
/// assert_eq!(step.account.as_deref(), Some("rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B"));
/// // More digits than a ledger amount holds: kept as JSON.
/// assert_eq!(way.source_amount["value"], "0.00001624579066564171");
/// # Ok::<(), serde_json::Error>(())
/// ```
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
