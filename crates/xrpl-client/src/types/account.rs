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
#[non_exhaustive]
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

/// An account's `AccountRoot`, as `account_info` returns it. The fields every
/// account has are typed; the optional ones (`Domain`, `RegularKey`,
/// `TransferRate`, `NFTokenMinter`, …) are in `other`, under their
/// `rippled` names. For the whole entry typed, parse it as
/// [`xrpl_core::types::LedgerObject`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountData {
    /// The account that sends the transaction or owns the entry (`Account`).
    #[serde(rename = "Account")]
    pub account: String,
    /// The XRP balance (an [`Amount::Xrp`](xrpl_core::types::Amount::Xrp), in
    /// drops; shown as XRP). See [`AccountData::balance_drops`].
    #[serde(rename = "Balance")]
    pub balance: xrpl_core::types::Amount,
    /// Flag bits (`Flags`).
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// How many entries the account owns (each needs reserve) (`OwnerCount`).
    #[serde(rename = "OwnerCount")]
    pub owner_count: u32,
    /// The next sequence number this account can use.
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    /// The last transaction that changed this entry (`PreviousTxnID`).
    #[serde(
        rename = "PreviousTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_id: Option<String>,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`).
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
    /// The XRP balance in drops (`None` only if the server sent a balance
    /// that is not XRP, which an account root never has).
    pub fn balance_drops(&self) -> Option<u64> {
        match self.balance {
            xrpl_core::types::Amount::Xrp(drops) => Some(drops),
            _ => None,
        }
    }
}

/// The account's flags, one boolean each, as `rippled` spells them.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AccountFlags {
    /// Rippling is on by default for the account's trust lines.
    #[serde(default)]
    pub default_ripple: bool,
    /// Only preauthorized accounts may pay it.
    #[serde(default)]
    pub deposit_auth: bool,
    /// The master key cannot sign.
    #[serde(default)]
    pub disable_master_key: bool,
    /// It refuses checks (`disallowIncomingCheck`).
    #[serde(default, rename = "disallowIncomingCheck")]
    pub disallow_incoming_check: bool,
    /// It refuses NFT offers (`disallowIncomingNFTokenOffer`).
    #[serde(default, rename = "disallowIncomingNFTokenOffer")]
    pub disallow_incoming_nftoken_offer: bool,
    /// It refuses payment channels.
    #[serde(default)]
    pub disallow_incoming_pay_chan: bool,
    /// It refuses trust lines.
    #[serde(default)]
    pub disallow_incoming_trustline: bool,
    /// It asks not to be sent XRP (`disallowIncomingXRP`).
    #[serde(default, rename = "disallowIncomingXRP")]
    pub disallow_incoming_xrp: bool,
    /// All its tokens are frozen.
    #[serde(default)]
    pub global_freeze: bool,
    /// It gave up the power to freeze.
    #[serde(default)]
    pub no_freeze: bool,
    /// Its one free key change is used.
    #[serde(default)]
    pub password_spent: bool,
    /// Its tokens need its authorization to hold.
    #[serde(default)]
    pub require_authorization: bool,
    /// Payments to it need a destination tag.
    #[serde(default)]
    pub require_destination_tag: bool,
    /// It may claw back its tokens.
    #[serde(default)]
    pub allow_trust_line_clawback: bool,
    /// Its tokens may be locked in escrow.
    #[serde(default)]
    pub allow_trust_line_locking: bool,
}

/// What the account has in the transaction queue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct QueueData {
    /// Transactions in the ledger.
    pub txn_count: u32,
    /// Whether a queued transaction changes the account's keys.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_change_queued: Option<bool>,
    /// The lowest queued sequence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lowest_sequence: Option<u32>,
    /// The highest queued sequence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highest_sequence: Option<u32>,
    /// The XRP all queued transactions could spend, in drops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_spend_drops_total: Option<String>,
    /// The transactions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<Value>>,
}

/// [`account_lines`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_lines):
/// an account's trust lines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountLinesResult {
    /// The account.
    #[serde(default)]
    pub account: String,
    /// The trust lines.
    pub lines: Vec<TrustLine>,
    /// The most entries per page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pass back in the request for the next page; `None` on the last.
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

/// One trust line, from the requesting account's side: `balance` is positive
/// when the peer (`account`) owes the requester.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TrustLine {
    /// The peer on the other side of the line.
    pub account: String,
    /// The balance (`Balance`).
    pub balance: String,
    /// The currency code.
    pub currency: String,
    /// The most the requester will hold of the peer's token.
    pub limit: String,
    /// The most the peer will hold of the requester's token.
    pub limit_peer: String,
    /// Incoming quality, in billionths (0 = 1:1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_in: Option<u32>,
    /// The value of outgoing balances, in billionths (`QualityOut`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_out: Option<u32>,
    /// Whether rippling is off on this side.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_ripple: Option<bool>,
    /// Whether rippling is off on the peer's side.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_ripple_peer: Option<bool>,
    /// Whether this side authorized the line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorized: Option<bool>,
    /// Whether the peer authorized the line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_authorized: Option<bool>,
    /// Whether this side froze the line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freeze: Option<bool>,
    /// Whether the peer froze the line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freeze_peer: Option<bool>,
    /// Whether this side deep-froze the line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deep_freeze: Option<bool>,
    /// Whether the peer deep-froze the line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deep_freeze_peer: Option<bool>,
}

/// [`account_offers`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_offers):
/// an account's open DEX offers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountOffersResult {
    /// The account.
    #[serde(default)]
    pub account: String,
    /// The offers.
    pub offers: Vec<AccountOffer>,
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

/// One offer of `account_offers`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountOffer {
    /// The offer's flags.
    pub flags: u32,
    /// The sequence of the OfferCreate that placed it (its `keylet::offer`).
    pub seq: u32,
    /// What the offer gives (`TakerGets`).
    pub taker_gets: Amount,
    /// What the offer wants in return (`TakerPays`).
    pub taker_pays: Amount,
    /// TakerPays ÷ TakerGets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// When the offer expires (ledger time).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,
}

/// [`account_nfts`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_nfts):
/// the NFTs an account holds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountNftsResult {
    /// The account.
    #[serde(default)]
    pub account: String,
    /// The NFTs.
    pub account_nfts: Vec<NFToken>,
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

/// One NFT held by the account. `xrpl_core::utils::parse_nftoken_id` splits
/// its ID into issuer, taxon, flags, fee and serial.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NFToken {
    /// The NFT's flags.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// The issuer (`Issuer`).
    #[serde(rename = "Issuer")]
    pub issuer: String,
    /// An NFT's ID (`NFTokenID`).
    #[serde(rename = "NFTokenID")]
    pub nf_token_id: String,
    /// A number the issuer uses to group its NFTs (`NFTokenTaxon`).
    #[serde(rename = "NFTokenTaxon")]
    pub nf_token_taxon: u32,
    /// The issuer's serial number for the NFT.
    pub nft_serial: u32,
    /// A URI, hex (`URI`).
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// The fee on secondary sales, in 1/100,000 (5000 = 5%); for tokens, see `TransferRate` (`TransferFee`).
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
#[non_exhaustive]
pub struct AccountTxResult {
    /// The account.
    #[serde(default)]
    pub account: String,
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

/// One transaction of `account_tx`: JSON (`tx` and `meta` objects) or, with
/// `binary: true`, hex (`tx_blob` and `meta` as a string).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountTxEntry {
    /// The transaction, as JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx: Option<Value>,
    /// The transaction, as hex.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_blob: Option<String>,
    /// The transaction's metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
    /// The ledger the answer is from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger_index: Option<u32>,
    /// Whether the data is from a validated ledger (final).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validated: Option<bool>,
}

/// [`account_objects`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_objects):
/// the ledger entries an account owns, typed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountObjectsResult {
    /// The account.
    pub account: String,
    /// The ledger entries, typed.
    pub account_objects: Vec<LedgerObject>,
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

/// [`account_currencies`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_currencies):
/// currencies the account can send and receive, from its trust lines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountCurrenciesResult {
    /// Currencies the account can receive.
    pub receive_currencies: Vec<String>,
    /// Currencies the account can send.
    pub send_currencies: Vec<String>,
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

/// [`account_channels`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_channels):
/// payment channels where the account is the source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountChannelsResult {
    /// The account.
    pub account: String,
    /// The payment channels.
    pub channels: Vec<PaymentChannel>,
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

/// One payment channel. Amounts are XRP in drops.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PaymentChannel {
    /// The channel's ID.
    pub channel_id: String,
    /// The channel's source.
    pub account: String,
    /// The receiving account.
    pub destination_account: String,
    /// Total XRP put into the channel.
    pub amount: String,
    /// XRP already paid out.
    pub balance: String,
    /// Seconds the source must wait before closing a channel with unclaimed XRP (`SettleDelay`).
    pub settle_delay: u32,
    /// The key that signs claims, in base58 (`a…` / `n…`) form.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    /// The same key in hex, as claims are verified with.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key_hex: Option<String>,
    /// When the channel expires (ledger time).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration: Option<u32>,
    /// The time after which it can be cancelled (ledger time) (`CancelAfter`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancel_after: Option<u32>,
    /// A tag the sender uses to tell its own payments apart (`SourceTag`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,
    /// A tag the destination uses to tell its senders apart (`DestinationTag`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_tag: Option<u32>,
}

/// [`gateway_balances`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/gateway_balances):
/// what an issuer owes (`obligations`, by currency) and holds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GatewayBalancesResult {
    /// The issuing account.
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

/// One balance of `gateway_balances`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GatewayBalance {
    /// The currency code.
    pub currency: String,
    /// The balance.
    pub value: String,
}

/// [`noripple_check`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/noripple_check):
/// trust-line settings that do not suit the account's role, and (when asked)
/// the transactions that would fix them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NoRippleCheckResult {
    /// One sentence per problem.
    pub problems: Vec<String>,
    /// Unsigned transactions that fix the problems (`transactions: true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transactions: Option<Vec<Value>>,
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
