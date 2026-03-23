use serde::{Deserialize, Serialize};

/// Ledger index specifier for API requests.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum LedgerIndex {
    Validated,
    Current,
    Closed,
    Index(u32),
}

impl LedgerIndex {
    pub fn as_value(&self) -> serde_json::Value {
        match self {
            LedgerIndex::Validated => serde_json::Value::String("validated".into()),
            LedgerIndex::Current => serde_json::Value::String("current".into()),
            LedgerIndex::Closed => serde_json::Value::String("closed".into()),
            LedgerIndex::Index(n) => serde_json::Value::Number((*n).into()),
        }
    }
}

/// Account info response.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountInfo {
    pub account_data: AccountData,
    pub ledger_current_index: Option<u32>,
    pub ledger_index: Option<u32>,
    pub validated: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccountData {
    pub account: String,
    pub balance: String,
    pub sequence: u32,
    pub flags: u32,
    pub owner_count: u32,
}

/// Fee response.
#[derive(Debug, Clone, Deserialize)]
pub struct FeeResult {
    pub drops: FeeDrops,
    pub current_ledger_size: Option<u32>,
    pub current_queue_size: Option<u32>,
    pub expected_ledger_size: Option<u32>,
    pub ledger_current_index: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeeDrops {
    pub base_fee: String,
    pub median_fee: String,
    pub minimum_fee: String,
    pub open_ledger_fee: String,
}

/// Submit response.
#[derive(Debug, Clone, Deserialize)]
pub struct SubmitResult {
    pub engine_result: String,
    pub engine_result_code: i32,
    pub engine_result_message: String,
    pub tx_blob: Option<String>,
    pub tx_json: Option<serde_json::Value>,
}

/// Transaction lookup response.
#[derive(Debug, Clone, Deserialize)]
pub struct TxResult {
    pub meta: Option<serde_json::Value>,
    pub validated: Option<bool>,
    pub ledger_index: Option<u32>,
    pub ledger_hash: Option<String>,
    #[serde(flatten)]
    pub tx_fields: std::collections::HashMap<String, serde_json::Value>,
}

/// Ledger info response.
#[derive(Debug, Clone, Deserialize)]
pub struct LedgerResult {
    pub ledger: Option<LedgerInfo>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LedgerInfo {
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub close_time: Option<u64>,
    pub total_coins: Option<String>,
}

/// Ledger current response.
#[derive(Debug, Clone, Deserialize)]
pub struct LedgerCurrentResult {
    pub ledger_current_index: u32,
}

/// Account trust lines response.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountLinesResult {
    pub lines: Vec<TrustLine>,
    pub marker: Option<Value>,
}

/// A single trust line.
#[derive(Debug, Clone, Deserialize)]
pub struct TrustLine {
    pub account: String,
    pub balance: String,
    pub currency: String,
    pub limit: String,
    pub limit_peer: String,
    pub quality_in: Option<u32>,
    pub quality_out: Option<u32>,
    pub no_ripple: Option<bool>,
    pub no_ripple_peer: Option<bool>,
    pub freeze: Option<bool>,
    pub freeze_peer: Option<bool>,
}

/// Account offers response.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountOffersResult {
    pub offers: Vec<AccountOffer>,
    pub marker: Option<Value>,
}

/// A single offer from account_offers.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountOffer {
    pub flags: u32,
    pub seq: u32,
    pub taker_gets: Value,
    pub taker_pays: Value,
    pub quality: Option<String>,
    pub expiration: Option<u32>,
}

/// Account NFTs response.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountNftsResult {
    pub account_nfts: Vec<NFToken>,
    pub marker: Option<Value>,
}

/// A single NFToken.
#[derive(Debug, Clone, Deserialize)]
pub struct NFToken {
    pub flags: u32,
    pub issuer: String,
    #[serde(rename = "NFTokenID")]
    pub nf_token_id: String,
    #[serde(rename = "NFTokenTaxon")]
    pub nf_token_taxon: u32,
    pub nft_serial: Option<u32>,
    #[serde(rename = "URI")]
    pub uri: Option<String>,
    pub transfer_fee: Option<u32>,
}

/// Book offers response (order book).
#[derive(Debug, Clone, Deserialize)]
pub struct BookOffersResult {
    pub offers: Vec<BookOffer>,
}

/// A single order book offer.
#[derive(Debug, Clone, Deserialize)]
pub struct BookOffer {
    #[serde(rename = "Account")]
    pub account: String,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    #[serde(rename = "TakerGets")]
    pub taker_gets: Value,
    #[serde(rename = "TakerPays")]
    pub taker_pays: Value,
    pub quality: Option<String>,
    pub owner_funds: Option<String>,
    #[serde(rename = "Expiration")]
    pub expiration: Option<u32>,
    #[serde(rename = "BookDirectory")]
    pub book_directory: Option<String>,
}

/// AMM info response.
#[derive(Debug, Clone, Deserialize)]
pub struct AmmInfoResult {
    pub amm: AmmInfo,
}

/// AMM pool information.
#[derive(Debug, Clone, Deserialize)]
pub struct AmmInfo {
    pub account: Option<String>,
    pub amount: Option<Value>,
    pub amount2: Option<Value>,
    pub lp_token: Option<Value>,
    pub trading_fee: Option<u32>,
    pub vote_slots: Option<Vec<Value>>,
    pub auction_slot: Option<Value>,
}

use serde_json::Value;

/// Account transaction history response.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountTxResult {
    pub account: String,
    pub transactions: Vec<AccountTxEntry>,
    pub marker: Option<Value>,
}

/// A single entry from account_tx.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountTxEntry {
    pub tx: Option<Value>,
    pub meta: Option<Value>,
    pub validated: Option<bool>,
}

/// Account objects response.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountObjectsResult {
    pub account: String,
    pub account_objects: Vec<Value>,
    pub marker: Option<Value>,
}

/// Account currencies response.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountCurrenciesResult {
    pub receive_currencies: Vec<String>,
    pub send_currencies: Vec<String>,
    pub ledger_index: Option<u32>,
    pub validated: Option<bool>,
}

/// Account channels response.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountChannelsResult {
    pub account: String,
    pub channels: Vec<PaymentChannel>,
    pub marker: Option<Value>,
}

/// A single payment channel.
#[derive(Debug, Clone, Deserialize)]
pub struct PaymentChannel {
    pub channel_id: String,
    pub account: String,
    pub destination_account: String,
    pub amount: String,
    pub balance: String,
    pub public_key: Option<String>,
    pub settle_delay: u32,
    pub expiration: Option<u32>,
    pub cancel_after: Option<u32>,
    pub source_tag: Option<u32>,
    pub destination_tag: Option<u32>,
}

/// Ledger entry response.
#[derive(Debug, Clone, Deserialize)]
pub struct LedgerEntryResult {
    pub index: String,
    pub node: Value,
    pub ledger_index: Option<u32>,
    pub validated: Option<bool>,
}

/// Server info response.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerInfoResult {
    pub info: ServerInfo,
}

/// Server information.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerInfo {
    pub build_version: Option<String>,
    pub complete_ledgers: Option<String>,
    pub hostid: Option<String>,
    pub load_factor: Option<f64>,
    pub peers: Option<u32>,
    pub pubkey_node: Option<String>,
    pub server_state: Option<String>,
    pub uptime: Option<u64>,
    pub validated_ledger: Option<ValidatedLedger>,
    pub reserve_base_xrp: Option<f64>,
    pub reserve_inc_xrp: Option<f64>,
}

/// Validated ledger info from server_info.
#[derive(Debug, Clone, Deserialize)]
pub struct ValidatedLedger {
    pub age: Option<u32>,
    pub base_fee_xrp: Option<f64>,
    pub hash: Option<String>,
    pub reserve_base_xrp: Option<f64>,
    pub reserve_inc_xrp: Option<f64>,
    pub seq: Option<u32>,
}

/// Gateway balances response.
#[derive(Debug, Clone, Deserialize)]
pub struct GatewayBalancesResult {
    pub account: String,
    pub obligations: Option<std::collections::HashMap<String, String>>,
    pub balances: Option<std::collections::HashMap<String, Vec<GatewayBalance>>>,
    pub assets: Option<std::collections::HashMap<String, Vec<GatewayBalance>>>,
}

/// A single gateway balance entry.
#[derive(Debug, Clone, Deserialize)]
pub struct GatewayBalance {
    pub currency: Option<String>,
    pub value: String,
}
