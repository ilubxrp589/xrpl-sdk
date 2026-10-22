//! Response of `vault_info` (the SingleAssetVault amendment).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// [`vault_info`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/vault-methods/vault_info):
/// a single-asset vault and its share issuance (SingleAssetVault amendment).
/// The vault is kept as JSON until mainnet data lets it be checked typed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultInfoResult {
    /// The vault and its share issuance.
    pub vault: Value,
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
