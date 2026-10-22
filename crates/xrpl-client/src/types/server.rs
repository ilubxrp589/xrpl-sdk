//! Responses of the server methods: `fee`, `server_info`, `server_state`,
//! `server_definitions`, `feature`, `manifest` and `version`; the utility
//! methods `ping` and `random`; and `channel_verify`.

use super::count_as_string;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// [`fee`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/fee):
/// the transaction cost now, and how full the open ledger and queue are.
/// `rippled` writes the sizes as strings; they are read as numbers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeeResult {
    pub drops: FeeDrops,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "count_as_string"
    )]
    pub current_ledger_size: Option<u32>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "count_as_string"
    )]
    pub current_queue_size: Option<u32>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "count_as_string"
    )]
    pub expected_ledger_size: Option<u32>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "count_as_string"
    )]
    pub max_queue_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub levels: Option<FeeLevels>,
}

/// Transaction costs, in drops.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeeDrops {
    /// The cost of a reference transaction with no load.
    pub base_fee: String,
    /// The median fee in the last ledger.
    pub median_fee: String,
    /// The least that gets a transaction into the queue.
    pub minimum_fee: String,
    /// The least that gets a transaction into the open ledger now; what
    /// `autofill` uses.
    pub open_ledger_fee: String,
}

/// The same costs as fee levels (256 = the reference transaction cost).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeeLevels {
    pub median_level: String,
    pub minimum_level: String,
    pub open_ledger_level: String,
    pub reference_level: String,
}

/// [`server_info`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/server_info):
/// the server's state, in human units (XRP, seconds).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerInfoResult {
    pub info: ServerInfo,
}

/// What `server_info` reports. Which fields appear depends on the server's
/// state, its role and whether the request is from an admin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amendment_blocked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_version: Option<String>,
    /// Ranges of ledgers the server has, e.g. `"32570-107831520"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub complete_ledgers: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_ledger: Option<ValidatedLedger>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<GitInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_sync_duration_us: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io_latency_ms: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jq_trans_overflow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_close: Option<LastClose>,
    /// Job queue detail (admin).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load: Option<Value>,
    /// The load multiplier on the transaction cost (1 = no load).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_local: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_net: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_cluster: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_escalation: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_queue: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_server: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_id: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_ledger: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_disconnects: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_disconnects_resources: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peers: Option<u32>,
    /// Listening ports (admin).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<Port>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubkey_node: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubkey_validator: Option<String>,
    /// `full`, `proposing`, `validating`, `tracking`, `syncing`, `connected`, `disconnected`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_state_duration_us: Option<String>,
    /// Time spent in each server state (admin).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_accounting: Option<BTreeMap<String, StateAccounting>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uptime: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated_ledger: Option<ValidatedLedger>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_quorum: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validator_list: Option<ValidatorListInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validator_list_expires: Option<String>,
}

/// The latest validated (or closed) ledger, in `server_info`'s XRP units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidatedLedger {
    /// Seconds since it closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
    /// The reference transaction cost, in XRP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_fee_xrp: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// The account reserve, in XRP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve_base_xrp: Option<f64>,
    /// The reserve per owned object, in XRP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve_inc_xrp: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq: Option<u32>,
}

/// The build's git branch and commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// How the last consensus round went.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LastClose {
    /// Seconds (`server_info`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub converge_time_s: Option<f64>,
    /// Milliseconds (`server_state`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub converge_time: Option<u64>,
    pub proposers: u32,
}

/// A port the server listens on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Port {
    pub port: String,
    pub protocol: Vec<String>,
}

/// Time in one server state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateAccounting {
    pub duration_us: String,
    pub transitions: String,
}

/// The validator list the server uses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorListInfo {
    pub count: u32,
    pub expiration: String,
    pub status: String,
}

/// [`server_state`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/server_state):
/// `server_info` in machine units (drops, fee levels, milliseconds).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerStateResult {
    pub state: ServerState,
}

/// What `server_state` reports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amendment_blocked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub complete_ledgers: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_ledger: Option<StateLedger>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<GitInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_sync_duration_us: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io_latency_ms: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jq_trans_overflow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_close: Option<LastClose>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load: Option<Value>,
    /// The fee level with no load (256).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_base: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_escalation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_queue: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_reference: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_server: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_id: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_disconnects: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_disconnects_resources: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peers: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<Port>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubkey_node: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubkey_validator: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_state_duration_us: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_accounting: Option<BTreeMap<String, StateAccounting>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uptime: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated_ledger: Option<StateLedger>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_quorum: Option<u32>,
    /// Ledger time when the validator list expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validator_list_expires: Option<u64>,
}

/// The latest validated (or closed) ledger, in `server_state`'s units (drops).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateLedger {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_fee: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve_base: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve_inc: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq: Option<u32>,
}

/// [`server_definitions`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/server_definitions):
/// the server's binary-format definitions: what `xrpl_core::codec` embeds
/// from `rippled` 3.4.1, live from the server (compare `hash` to see if a
/// server has new fields or types).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct ServerDefinitionsResult {
    /// `[name, { nth, type, isVLEncoded, isSerialized, isSigningField }]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<(String, FieldInfo)>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_entry_types: Option<BTreeMap<String, i32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_results: Option<BTreeMap<String, i32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_types: Option<BTreeMap<String, i32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<BTreeMap<String, i32>>,
    /// AccountSet `SetFlag` values, by name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_set_flags: Option<Value>,
    /// Ledger entry flags, by entry type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_entry_flags: Option<Value>,
    /// The fields of each ledger entry type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_entry_formats: Option<Value>,
    /// Transaction flags, by transaction type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_flags: Option<Value>,
    /// The fields of each transaction type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_formats: Option<Value>,
    /// A hash of all of the above.
    #[serde(rename = "hash")]
    pub hash: String,
}

/// One field's definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldInfo {
    pub nth: i32,
    #[serde(rename = "isVLEncoded")]
    pub is_vl_encoded: bool,
    pub is_serialized: bool,
    pub is_signing_field: bool,
    #[serde(rename = "type")]
    pub field_type: String,
}

/// [`feature`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/feature):
/// amendments the server knows, by ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureResult {
    pub features: BTreeMap<String, Amendment>,
}

/// One amendment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Amendment {
    pub name: String,
    pub enabled: bool,
    pub supported: bool,
    /// Whether this server votes against it (admin).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vetoed: Option<Value>,
    /// Validators voting for it, of `threshold` needed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validations: Option<u32>,
    /// Ledger time it gained a majority (it enables two weeks later).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub majority: Option<u64>,
}

/// [`manifest`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/manifest):
/// a validator's current manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManifestResult {
    /// The key asked about.
    pub requested: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<ManifestDetails>,
    /// The manifest, base64.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest: Option<String>,
}

/// What a validator's manifest says.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    pub ephemeral_key: String,
    pub master_key: String,
    pub seq: u32,
}

/// [`version`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/version):
/// the API versions the server supports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionResult {
    pub version: ApiVersions,
}

/// Lowest, default and highest API versions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiVersions {
    pub first: String,
    pub good: String,
    pub last: String,
}

/// [`ping`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/utility-methods/ping):
/// the server is up; and how it sees the caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PingResult {
    /// `admin`, `identified`, `proxied` or `user`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unlimited: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

/// [`random`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/utility-methods/random):
/// 256 random bits from the server, in hex. Do not use them for keys: they
/// come from someone else's machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RandomResult {
    pub random: String,
}

/// [`channel_verify`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/payment-channel-methods/channel_verify):
/// whether a claim's signature is valid. `xrpl_core::crypto::verify_payment_channel_claim`
/// does the same offline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelVerifyResult {
    pub signature_verified: bool,
}
