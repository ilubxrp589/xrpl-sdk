//! Responses of the methods only [Clio](https://xrpl.org/docs/concepts/networks-and-servers/the-clio-server)
//! servers answer: `nft_info`, `nft_history`, `nfts_by_issuer` and
//! `mpt_holders`. Public Clio servers include `s1.ripple.com` and
//! `s2.ripple.com` (mainnet) and `clio.altnet.rippletest.net` (Testnet); a
//! `rippled` server answers these with an `unknownCmd` error.

use super::AccountTxEntry;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One NFT as Clio describes it (`nft_info`, and each entry of `nfts_by_issuer`).
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NftHistoryResult {
    /// The NFT's ID.
    pub nft_id: String,
    /// The transactions.
    pub transactions: Vec<AccountTxEntry>,
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
