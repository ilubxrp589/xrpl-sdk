//! Responses of the account methods: `account_info`, `account_lines`,
//! `account_offers`, `account_nfts`, `account_tx`, `account_objects`,
//! `account_currencies`, `account_channels`, `gateway_balances` and
//! `noripple_check`.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use xrpl_core::types::{Amount, LedgerObject};

/// [`account_info`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_info):
/// an account's `AccountRoot`, its flags spelled out, and (when asked for) its
/// signer lists and queued transactions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountInfo {
    /// The account's `AccountRoot` entry (with `signer_lists` inside it when
    /// requested under API version 1, as `rippled` does).
    pub account_data: AccountData,
    /// The `Flags` of `account_data`, one boolean each.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_flags: Option<AccountFlags>,
    /// Transactions this account has in the queue (`queue: true`, current ledger only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_data: Option<QueueData>,
    /// Signer lists, under API version 2 (version 1 puts them in `account_data`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer_lists: Option<Vec<LedgerObject>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// An account's `AccountRoot`, as `account_info` returns it. The fields every
/// account has are typed; the optional ones (`Domain`, `RegularKey`,
/// `TransferRate`, `NFTokenMinter`, …) are in `other`, under their
/// `rippled` names. For the whole entry typed, parse it as
/// [`xrpl_core::types::LedgerObject`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountData {
    #[serde(rename = "Account")]
    pub account: String,
    /// The XRP balance, in drops (see [`AccountData::balance_drops`]).
    #[serde(rename = "Balance")]
    pub balance: String,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "OwnerCount")]
    pub owner_count: u32,
    /// The next sequence number this account can use.
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    #[serde(
        rename = "PreviousTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_id: Option<String>,
    #[serde(
        rename = "PreviousTxnLgrSeq",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_lgr_seq: Option<u32>,
    /// The entry's ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<String>,
    /// `SignerList` entries (API version 1, with `signer_lists: true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer_lists: Option<Vec<LedgerObject>>,
    /// Every other field of the `AccountRoot` (`LedgerEntryType`, `Domain`,
    /// `RegularKey`, `TransferRate`, `TickSize`, `NFTokenMinter`, …).
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

impl AccountData {
    /// The XRP balance in drops, as a number.
    pub fn balance_drops(&self) -> Option<u64> {
        self.balance.parse().ok()
    }
}

/// The account's flags, one boolean each, as `rippled` spells them.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountFlags {
    #[serde(default)]
    pub default_ripple: bool,
    #[serde(default)]
    pub deposit_auth: bool,
    #[serde(default)]
    pub disable_master_key: bool,
    #[serde(default, rename = "disallowIncomingCheck")]
    pub disallow_incoming_check: bool,
    #[serde(default, rename = "disallowIncomingNFTokenOffer")]
    pub disallow_incoming_nftoken_offer: bool,
    #[serde(default)]
    pub disallow_incoming_pay_chan: bool,
    #[serde(default)]
    pub disallow_incoming_trustline: bool,
    #[serde(default, rename = "disallowIncomingXRP")]
    pub disallow_incoming_xrp: bool,
    #[serde(default)]
    pub global_freeze: bool,
    #[serde(default)]
    pub no_freeze: bool,
    #[serde(default)]
    pub password_spent: bool,
    #[serde(default)]
    pub require_authorization: bool,
    #[serde(default)]
    pub require_destination_tag: bool,
    #[serde(default)]
    pub allow_trust_line_clawback: bool,
    #[serde(default)]
    pub allow_trust_line_locking: bool,
}

/// What the account has in the transaction queue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueueData {
    pub txn_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_change_queued: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lowest_sequence: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highest_sequence: Option<u32>,
    /// The XRP all queued transactions could spend, in drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_spend_drops_total: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<Value>>,
}

/// [`account_lines`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_lines):
/// an account's trust lines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountLinesResult {
    #[serde(default)]
    pub account: String,
    pub lines: Vec<TrustLine>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pass back in the request for the next page; `None` on the last.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// One trust line, from the requesting account's side: `balance` is positive
/// when the peer (`account`) owes the requester.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustLine {
    /// The peer on the other side of the line.
    pub account: String,
    pub balance: String,
    pub currency: String,
    /// The most the requester will hold of the peer's token.
    pub limit: String,
    /// The most the peer will hold of the requester's token.
    pub limit_peer: String,
    /// Incoming quality, in billionths (0 = 1:1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_in: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_out: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_ripple: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_ripple_peer: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorized: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_authorized: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freeze: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freeze_peer: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deep_freeze: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deep_freeze_peer: Option<bool>,
}

/// [`account_offers`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_offers):
/// an account's open DEX offers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountOffersResult {
    #[serde(default)]
    pub account: String,
    pub offers: Vec<AccountOffer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// One offer of `account_offers`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountOffer {
    pub flags: u32,
    /// The sequence of the OfferCreate that placed it (its `keylet::offer`).
    pub seq: u32,
    pub taker_gets: Amount,
    pub taker_pays: Amount,
    /// TakerPays ÷ TakerGets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,
}

/// [`account_nfts`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_nfts):
/// the NFTs an account holds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountNftsResult {
    #[serde(default)]
    pub account: String,
    pub account_nfts: Vec<NFToken>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// One NFT held by the account. `xrpl_core::utils::parse_nftoken_id` splits
/// its ID into issuer, taxon, flags, fee and serial.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NFToken {
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Issuer")]
    pub issuer: String,
    #[serde(rename = "NFTokenID")]
    pub nf_token_id: String,
    #[serde(rename = "NFTokenTaxon")]
    pub nf_token_taxon: u32,
    pub nft_serial: u32,
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(
        rename = "TransferFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transfer_fee: Option<u32>,
}

/// [`account_tx`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_tx)
/// (also the shape of Clio's `nft_history`): transactions that affected an account.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountTxResult {
    #[serde(default)]
    pub account: String,
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

/// One transaction of `account_tx`: JSON (`tx` and `meta` objects) or, with
/// `binary: true`, hex (`tx_blob` and `meta` as a string).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountTxEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_blob: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// [`account_objects`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_objects):
/// the ledger entries an account owns, typed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountObjectsResult {
    pub account: String,
    pub account_objects: Vec<LedgerObject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// [`account_currencies`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_currencies):
/// currencies the account can send and receive, from its trust lines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountCurrenciesResult {
    pub receive_currencies: Vec<String>,
    pub send_currencies: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// [`account_channels`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_channels):
/// payment channels where the account is the source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountChannelsResult {
    pub account: String,
    pub channels: Vec<PaymentChannel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// One payment channel. Amounts are XRP in drops.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentChannel {
    pub channel_id: String,
    pub account: String,
    pub destination_account: String,
    /// Total XRP put into the channel.
    pub amount: String,
    /// XRP already paid out.
    pub balance: String,
    pub settle_delay: u32,
    /// The key that signs claims, in base58 (`a…` / `n…`) form.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    /// The same key in hex, as claims are verified with.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancel_after: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_tag: Option<u32>,
}

/// [`gateway_balances`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/gateway_balances):
/// what an issuer owes (`obligations`, by currency) and holds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayBalancesResult {
    pub account: String,
    /// Total issued, by currency code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obligations: Option<BTreeMap<String, String>>,
    /// Balances held by the `hotwallet` accounts of the request, by account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balances: Option<BTreeMap<String, Vec<GatewayBalance>>>,
    /// Balances held by frozen accounts, by account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frozen_balances: Option<BTreeMap<String, Vec<GatewayBalance>>>,
    /// Tokens of other issuers the account holds, by issuer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<BTreeMap<String, Vec<GatewayBalance>>>,
    /// Escrowed or locked tokens, by issuer (TokenEscrow).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locked: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// One balance of `gateway_balances`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayBalance {
    pub currency: String,
    pub value: String,
}

/// [`noripple_check`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/noripple_check):
/// trust-line settings that do not suit the account's role, and (when asked)
/// the transactions that would fix them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoRippleCheckResult {
    /// One sentence per problem.
    pub problems: Vec<String>,
    /// Unsigned transactions that fix the problems (`transactions: true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_current_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}
