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
#[non_exhaustive]
pub struct FeeResult {
    /// The costs, in drops.
    pub drops: FeeDrops,
    /// Transactions in the open ledger.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "count_as_string"
    )]
    pub current_ledger_size: Option<u32>,
    /// Transactions in the queue.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "count_as_string"
    )]
    pub current_queue_size: Option<u32>,
    /// Transactions the network expects in a ledger.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "count_as_string"
    )]
    pub expected_ledger_size: Option<u32>,
    /// The most transactions the queue holds.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "count_as_string"
    )]
    pub max_queue_size: Option<u32>,
    /// The open ledger the answer is from (when not a validated one).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    /// The costs, as fee levels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub levels: Option<FeeLevels>,
}

/// Transaction costs, in drops.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
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
#[non_exhaustive]
pub struct FeeLevels {
    /// The median fee level in the last ledger.
    pub median_level: String,
    /// The least fee level that gets into the queue.
    pub minimum_level: String,
    /// The least fee level that gets into the open ledger now.
    pub open_ledger_level: String,
    /// The reference transaction's fee level (256).
    pub reference_level: String,
}

/// [`server_info`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/server_info):
/// the server's state, in human units (XRP, seconds).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ServerInfoResult {
    /// What the server reports.
    pub info: ServerInfo,
}

/// What `server_info` reports. Which fields appear depends on the server's
/// state, its role and whether the request is from an admin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ServerInfo {
    /// Whether the server lacks an enabled amendment and so cannot follow the network.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amendment_blocked: Option<bool>,
    /// The server's version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_version: Option<String>,
    /// Ranges of ledgers the server has, e.g. `"32570-107831520"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub complete_ledgers: Option<String>,
    /// The latest closed ledger, when no validated one is known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_ledger: Option<ValidatedLedger>,
    /// The build's git branch and commit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<GitInfo>,
    /// The server's host name (or a short code for non-admins).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostid: Option<String>,
    /// Microseconds the first sync took.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_sync_duration_us: Option<String>,
    /// How long the server waits for I/O, in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io_latency_ms: Option<u32>,
    /// How often the job queue overflowed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jq_trans_overflow: Option<String>,
    /// How the last consensus round went.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_close: Option<LastClose>,
    /// Job queue detail (admin).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load: Option<Value>,
    /// The load multiplier on the transaction cost (1 = no load).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor: Option<f64>,
    /// The server's own load factor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_local: Option<f64>,
    /// The network's load factor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_net: Option<f64>,
    /// The cluster's load factor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_cluster: Option<f64>,
    /// The open-ledger fee escalation, as a fee level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_escalation: Option<f64>,
    /// The fee level to get into the queue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_queue: Option<f64>,
    /// The server's load factor, as a fee level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_server: Option<f64>,
    /// The network's ID (0 mainnet, 1 Testnet, 2 Devnet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_id: Option<u32>,
    /// Whether the server has heard of the network's ledgers yet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_ledger: Option<String>,
    /// The server's configured size (`tiny` … `huge`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_size: Option<String>,
    /// Peers disconnected since the server started.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_disconnects: Option<String>,
    /// Peers disconnected for using too much.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_disconnects_resources: Option<String>,
    /// Connected peers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peers: Option<u32>,
    /// Listening ports (admin).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<Port>>,
    /// The server's node public key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubkey_node: Option<String>,
    /// The validator's public key, if the server validates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubkey_validator: Option<String>,
    /// `full`, `proposing`, `validating`, `tracking`, `syncing`, `connected`, `disconnected`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_state: Option<String>,
    /// Microseconds in the current server state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_state_duration_us: Option<String>,
    /// Time spent in each server state (admin).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_accounting: Option<BTreeMap<String, StateAccounting>>,
    /// The server's time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    /// Seconds since the server started.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uptime: Option<u64>,
    /// The latest validated ledger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated_ledger: Option<ValidatedLedger>,
    /// Validations needed to validate a ledger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_quorum: Option<u32>,
    /// The validator list in use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validator_list: Option<ValidatorListInfo>,
    /// When the validator list expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validator_list_expires: Option<String>,
}

/// The latest validated (or closed) ledger, in `server_info`'s XRP units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ValidatedLedger {
    /// Seconds since it closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
    /// The reference transaction cost, in XRP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_fee_xrp: Option<f64>,
    /// The ledger's hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// The account reserve, in XRP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve_base_xrp: Option<f64>,
    /// The reserve per owned object, in XRP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve_inc_xrp: Option<f64>,
    /// The sequence of the transaction that created it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq: Option<u32>,
}

/// The build's git branch and commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GitInfo {
    /// The git branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// The commit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// How the last consensus round went.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LastClose {
    /// Seconds (`server_info`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub converge_time_s: Option<f64>,
    /// Milliseconds (`server_state`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub converge_time: Option<u64>,
    /// Validators that proposed in the round.
    pub proposers: u32,
}

/// A port the server listens on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Port {
    /// The port number.
    pub port: String,
    /// Its protocols.
    pub protocol: Vec<String>,
}

/// Time in one server state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StateAccounting {
    /// Microseconds in this state.
    pub duration_us: String,
    /// Times the server entered this state.
    pub transitions: String,
}

/// The validator list the server uses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ValidatorListInfo {
    /// How many.
    pub count: u32,
    /// When it expires (ledger time: seconds since 2000-01-01) (`Expiration`).
    pub expiration: String,
    /// The status.
    pub status: String,
}

/// [`server_state`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/server_state):
/// `server_info` in machine units (drops, fee levels, milliseconds).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ServerStateResult {
    /// What the server reports.
    pub state: ServerState,
}

/// What `server_state` reports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ServerState {
    /// Whether the server lacks an enabled amendment and so cannot follow the network.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amendment_blocked: Option<bool>,
    /// The server's version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_version: Option<String>,
    /// The ledgers the server has, e.g. `"32570-107831520"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub complete_ledgers: Option<String>,
    /// The latest closed ledger, when no validated one is known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_ledger: Option<StateLedger>,
    /// The build's git branch and commit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<GitInfo>,
    /// Microseconds the first sync took.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_sync_duration_us: Option<String>,
    /// How long the server waits for I/O, in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub io_latency_ms: Option<u32>,
    /// How often the job queue overflowed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jq_trans_overflow: Option<String>,
    /// How the last consensus round went.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_close: Option<LastClose>,
    /// Job queue detail (admin).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load: Option<Value>,
    /// The fee level with no load (256).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_base: Option<u64>,
    /// The load multiplier on the transaction cost.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor: Option<u64>,
    /// The open-ledger fee escalation, as a fee level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_escalation: Option<u64>,
    /// The fee level to get into the queue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_queue: Option<u64>,
    /// The reference fee level (256).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_fee_reference: Option<u64>,
    /// The server's load factor, as a fee level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor_server: Option<u64>,
    /// The network's ID (0 mainnet, 1 Testnet, 2 Devnet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_id: Option<u32>,
    /// The server's configured size (`tiny` … `huge`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_size: Option<String>,
    /// Peers disconnected since the server started.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_disconnects: Option<String>,
    /// Peers disconnected for using too much.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_disconnects_resources: Option<String>,
    /// Connected peers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peers: Option<u32>,
    /// Listening ports (admin).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<Port>>,
    /// The server's node public key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubkey_node: Option<String>,
    /// The validator's public key, if the server validates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubkey_validator: Option<String>,
    /// The server's state (`full`, `syncing`, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_state: Option<String>,
    /// Microseconds in the current server state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_state_duration_us: Option<String>,
    /// Time spent in each server state (admin).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_accounting: Option<BTreeMap<String, StateAccounting>>,
    /// The server's time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    /// Seconds since the server started.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uptime: Option<u64>,
    /// The latest validated ledger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated_ledger: Option<StateLedger>,
    /// Validations needed to validate a ledger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_quorum: Option<u32>,
    /// Ledger time when the validator list expires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validator_list_expires: Option<u64>,
}

/// The latest validated (or closed) ledger, in `server_state`'s units (drops).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StateLedger {
    /// The reference transaction cost, in drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_fee: Option<u64>,
    /// When the ledger closed (ledger time).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time: Option<u64>,
    /// The ledger's hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// The account reserve, in drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve_base: Option<u64>,
    /// The reserve per owned entry, in drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve_inc: Option<u64>,
    /// The sequence of the transaction that created it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq: Option<u32>,
}

/// [`server_definitions`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/server_definitions):
/// the server's binary-format definitions: what `xrpl_core::codec` embeds
/// from `rippled` 3.4.1, live from the server (compare `hash` to see if a
/// server has new fields or types).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub struct ServerDefinitionsResult {
    /// `[name, { nth, type, isVLEncoded, isSerialized, isSigningField }]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<(String, FieldInfo)>>,
    /// Ledger entry types, by name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_entry_types: Option<BTreeMap<String, i32>>,
    /// Engine results, by name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_results: Option<BTreeMap<String, i32>>,
    /// Transaction types, by name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_types: Option<BTreeMap<String, i32>>,
    /// Field types, by name.
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
#[non_exhaustive]
pub struct FieldInfo {
    /// The field's code within its type.
    pub nth: i32,
    /// Whether the value has a length prefix (`isVLEncoded`).
    #[serde(rename = "isVLEncoded")]
    pub is_vl_encoded: bool,
    /// Whether the field is serialized.
    pub is_serialized: bool,
    /// Whether signatures cover the field.
    pub is_signing_field: bool,
    /// The field's type (`type`).
    #[serde(rename = "type")]
    pub field_type: String,
}

/// [`feature`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/feature):
/// amendments the server knows, by ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FeatureResult {
    /// Amendments, by ID.
    pub features: BTreeMap<String, Amendment>,
}

/// One amendment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Amendment {
    /// The name.
    pub name: String,
    /// Whether it is enabled.
    pub enabled: bool,
    /// Whether this server supports it.
    pub supported: bool,
    /// Whether this server votes against it (admin).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vetoed: Option<Value>,
    /// Validators voting for it, of `threshold` needed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    /// Votes needed for a majority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<u32>,
    /// Validators heard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validations: Option<u32>,
    /// Ledger time it gained a majority (it enables two weeks later).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub majority: Option<u64>,
}

/// [`manifest`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/manifest):
/// a validator's current manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ManifestResult {
    /// The key asked about.
    pub requested: String,
    /// What the manifest says.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<ManifestDetails>,
    /// The manifest, base64.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest: Option<String>,
}

/// What a validator's manifest says.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ManifestDetails {
    /// A domain the account claims, hex (ASCII, lower case) (`Domain`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// The validator's current signing key.
    pub ephemeral_key: String,
    /// The validator's master key.
    pub master_key: String,
    /// The sequence of the transaction that created it.
    pub seq: u32,
}

/// [`version`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/server-info-methods/version):
/// the API versions the server supports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VersionResult {
    /// The API versions.
    pub version: ApiVersions,
}

/// The lowest and highest API versions the server takes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ApiVersions {
    /// The lowest supported API version.
    pub first: u32,
    /// The highest supported API version.
    pub last: u32,
}

/// [`ping`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/utility-methods/ping):
/// the server is up; and how it sees the caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PingResult {
    /// `admin`, `identified`, `proxied` or `user`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Whether the caller is exempt from rate limits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unlimited: Option<bool>,
    /// The caller's IP address, as the server sees it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    /// The caller's user name, if identified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

/// [`random`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/utility-methods/random):
/// 256 random bits from the server, in hex. Do not use them for keys: they
/// come from someone else's machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RandomResult {
    /// 256 random bits, hex.
    pub random: String,
}

/// [`channel_verify`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/payment-channel-methods/channel_verify):
/// whether a claim's signature is valid. `xrpl_core::crypto::verify_payment_channel_claim`
/// does the same offline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChannelVerifyResult {
    /// Whether the signature is valid.
    pub signature_verified: bool,
}
