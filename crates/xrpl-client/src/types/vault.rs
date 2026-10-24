//! Response of `vault_info` (the SingleAssetVault amendment).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// [`vault_info`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/vault-methods/vault_info):
/// a single-asset vault and its share issuance (SingleAssetVault amendment).
/// The vault is kept as JSON until mainnet data lets it be checked typed.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::VaultInfoResult;
///
/// let answer: VaultInfoResult = serde_json::from_value(json!({
///     "vault": {"LedgerEntryType": "Vault", "Owner": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
///         "Asset": {"currency": "XRP"}, "Sequence": 5},
///     "validated": true
/// }))?;
/// // The vault stays JSON: read its fields by their `rippled` names.
/// assert_eq!(answer.vault["Asset"], json!({"currency": "XRP"}));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
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
