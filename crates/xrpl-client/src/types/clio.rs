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
pub struct NftInfo {
    pub nft_id: String,
    pub owner: String,
    pub issuer: String,
    pub is_burned: bool,
    pub flags: u32,
    pub transfer_fee: u32,
    pub nft_taxon: u32,
    pub nft_serial: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
}

/// [`nft_info`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/nft_info):
/// an NFT, including burned ones.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NftInfoResult {
    #[serde(flatten)]
    pub nft: NftInfo,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// [`nft_history`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/nft_history):
/// every transaction that touched an NFT.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NftHistoryResult {
    pub nft_id: String,
    pub transactions: Vec<AccountTxEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index_min: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index_max: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// [`nfts_by_issuer`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/nfts_by_issuer):
/// every NFT an account issued.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NftsByIssuerResult {
    pub issuer: String,
    pub nfts: Vec<NftInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nft_taxon: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// [`mpt_holders`](https://xrpl.org/docs/references/http-websocket-apis/clio-methods/mpt_holders):
/// the holders of an MPT issuance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MptHoldersResult {
    pub mpt_issuance_id: String,
    pub mptokens: Vec<MptHolder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// One holder's `MPToken`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MptHolder {
    pub account: String,
    pub flags: u32,
    /// The balance, in the issuance's smallest unit (base 10).
    pub mpt_amount: String,
    /// The `MPToken` entry's ID.
    pub mptoken_index: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locked_amount: Option<String>,
}
