//! Responses of the methods only [Clio](https://xrpl.org/docs/concepts/networks-and-servers/the-clio-server)
//! servers answer: `nft_info`, `nft_history`, `nfts_by_issuer` and
//! `mpt_holders`. Public Clio servers include `s1.ripple.com` and
//! `s2.ripple.com` (mainnet) and `clio.altnet.rippletest.net` (Testnet); a
//! `rippled` server answers these with an `unknownCmd` error.

use super::TxResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One NFT as Clio describes it (`nft_info`, and each entry of `nfts_by_issuer`).
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::NftInfo;
///
/// // One entry of `nfts_by_issuer`.
/// let nft: NftInfo = serde_json::from_value(json!({
///     "nft_id": "0000000047D6BDCD766717B67A937E83CFEF22B1760537E31280C5C306441428",
///     "owner": "rfYiNfgLefTAZGfEyun1EjG68mTtC75vDe",
///     "issuer": "rfYiNfgLefTAZGfEyun1EjG68mTtC75vDe",
///     "is_burned": true, "flags": 0, "transfer_fee": 0, "nft_taxon": 0, "nft_serial": 105124904,
///     "ledger_index": 105126663
/// }))?;
/// assert!(nft.is_burned);
/// assert_eq!(nft.nft_serial, 105124904);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NftInfo {
    /// The NFT's ID.
    pub nft_id: String,
    /// The NFT's current owner.
    pub owner: String,
    /// The NFT's issuer.
    pub issuer: String,
    /// Whether the NFT has been burned.
    pub is_burned: bool,
    /// The NFT's flags.
    pub flags: u32,
    /// The fee on secondary sales, in 1/100,000 (5000 = 5%); for tokens, see `TransferRate` (`TransferFee`).
    pub transfer_fee: u32,
    /// The NFT's taxon.
    pub nft_taxon: u32,
    /// The issuer's serial number for the NFT.
    pub nft_serial: u32,
    /// A URI, hex (`URI`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// The ledger the answer is from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
}

/// [`nft_info`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/nft_info):
/// an NFT, including burned ones.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::NftInfoResult;
///
/// let info: NftInfoResult = serde_json::from_value(json!({
///     "nft_id": "0000000047D6BDCD766717B67A937E83CFEF22B1760537E3EE0FC28306441448",
///     "owner": "rfYiNfgLefTAZGfEyun1EjG68mTtC75vDe",
///     "issuer": "rfYiNfgLefTAZGfEyun1EjG68mTtC75vDe",
///     "is_burned": false, "flags": 0, "transfer_fee": 0, "nft_taxon": 20260704,
///     "nft_serial": 105124936,
///     "ledger_index": 107830973, "validated": true
/// }))?;
/// // The NFT's fields sit beside `validated`, as Clio writes them.
/// assert!(!info.nft.is_burned);
/// assert_eq!(info.nft.owner, info.nft.issuer);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NftInfoResult {
    /// The NFT.
    #[serde(flatten)]
    pub nft: NftInfo,
    /// Whether the data is from a validated ledger (final).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// [`nft_history`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/nft_history):
/// every transaction that touched an NFT.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::NftHistoryResult;
///
/// let history: NftHistoryResult = serde_json::from_value(json!({
///     "nft_id": "0000000047D6BDCD766717B67A937E83CFEF22B1760537E3EE0FC28306441448",
///     "transactions": [{
///         "hash": "B2F50181DBCEB606C3E48AC2F58A688F1CD58DDBB0A1053F2E132CAAD475C6B0",
///         "ledger_index": 107830973, "validated": true,
///         "meta": {"AffectedNodes": [], "TransactionIndex": 0, "TransactionResult": "tesSUCCESS"}
///     }],
///     "ledger_index_min": 32570, "ledger_index_max": 107856070, "validated": true
/// }))?;
/// assert_eq!(history.transactions[0].ledger_index, Some(107830973));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NftHistoryResult {
    /// The NFT's ID.
    pub nft_id: String,
    /// The transactions.
    pub transactions: Vec<TxResult>,
    /// The earliest ledger searched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index_min: Option<i64>,
    /// The latest ledger searched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index_max: Option<i64>,
    /// The most entries per page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    /// Whether the data is from a validated ledger (final).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// [`nfts_by_issuer`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/nfts_by_issuer):
/// every NFT an account issued.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::NftsByIssuerResult;
///
/// let page: NftsByIssuerResult = serde_json::from_value(json!({
///     "issuer": "rfYiNfgLefTAZGfEyun1EjG68mTtC75vDe",
///     "nfts": [{
///         "nft_id": "0000000047D6BDCD766717B67A937E83CFEF22B1760537E31280C5C306441428",
///         "owner": "rfYiNfgLefTAZGfEyun1EjG68mTtC75vDe",
///         "issuer": "rfYiNfgLefTAZGfEyun1EjG68mTtC75vDe",
///         "is_burned": true, "flags": 0, "transfer_fee": 0, "nft_taxon": 0,
///         "nft_serial": 105124904
///     }],
///     "limit": 1, "marker": "0000000047D6BDCD766717B67A937E83CFEF22B1760537E31280C5C306441428",
///     "ledger_index": 107856070, "validated": true
/// }))?;
/// assert!(page.nfts[0].is_burned);
/// assert!(page.marker.is_some()); // more NFTs on the next page
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NftsByIssuerResult {
    /// The issuer (`Issuer`).
    pub issuer: String,
    /// The NFTs.
    pub nfts: Vec<NftInfo>,
    /// The NFT's taxon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nft_taxon: Option<u32>,
    /// The most entries per page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    /// The ledger the answer is from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    /// Whether the data is from a validated ledger (final).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// [`mpt_holders`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/mpt_holders):
/// the holders of an MPT issuance.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::MptHoldersResult;
///
/// let page: MptHoldersResult = serde_json::from_value(json!({
///     "mpt_issuance_id": "014C29552B3DD7A7E525987A2FAFF91EECF53984A1FC7906",
///     "mptokens": [{"account": "rMAdJ2PxkGkGtmt7XbTZiMczcZ4BA1E2wp", "flags": 0, "mpt_amount": "0",
///         "mptoken_index": "6FE2938199A423B96F259B803ACF1C0865CAC409395C42B482B6B2DF697E49DB"}],
///     "limit": 50, "ledger_index": 21768549, "validated": true
/// }))?;
/// assert_eq!(page.mptokens[0].account, "rMAdJ2PxkGkGtmt7XbTZiMczcZ4BA1E2wp");
/// assert_eq!(page.marker, None); // the last page
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MptHoldersResult {
    /// The MPT issuance's ID.
    pub mpt_issuance_id: String,
    /// The holders' MPToken entries.
    pub mptokens: Vec<MptHolder>,
    /// The most entries per page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Where the next page starts: pass it back in the request; absent on the last page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    /// The ledger the answer is from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    /// Whether the data is from a validated ledger (final).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// One holder's `MPToken`.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::MptHolder;
///
/// let holder: MptHolder = serde_json::from_value(json!({
///     "account": "rMAdJ2PxkGkGtmt7XbTZiMczcZ4BA1E2wp", "flags": 0, "mpt_amount": "0",
///     "mptoken_index": "6FE2938199A423B96F259B803ACF1C0865CAC409395C42B482B6B2DF697E49DB"
/// }))?;
/// assert_eq!(holder.mpt_amount.parse::<u64>()?, 0);
/// assert_eq!(holder.locked_amount, None);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MptHolder {
    /// The holder.
    pub account: String,
    /// The MPToken's flags.
    pub flags: u32,
    /// The balance, in the issuance's smallest unit (base 10).
    pub mpt_amount: String,
    /// The `MPToken` entry's ID.
    pub mptoken_index: String,
    /// The amount locked in escrow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locked_amount: Option<String>,
}
