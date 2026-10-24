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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::AccountInfo;
///
/// let info: AccountInfo = serde_json::from_value(json!({
///     "account_data": {
///         "Account": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "Balance": "99770260",
///         "Flags": 2174353408u32,
///         "OwnerCount": 19, "Sequence": 89926490
///     },
///     "account_flags": {"depositAuth": true, "requireDestinationTag": true},
///     "ledger_index": 107831520, "validated": true
/// }))?;
/// assert_eq!(info.account_data.balance_drops(), Some(99_770_260));
/// assert!(info.account_flags.unwrap().require_destination_tag);
/// assert_eq!(info.ledger_index, Some(107831520));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountInfo {
    /// The account's `AccountRoot` entry.
    pub account_data: AccountData,
    /// The `Flags` of `account_data`, one boolean each.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_flags: Option<AccountFlags>,
    /// Transactions this account has in the queue (`queue: true`, current ledger only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_data: Option<QueueData>,
    /// The account's signer list (`signer_lists: true`): none, or one.
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::AccountData;
///
/// let data: AccountData = serde_json::from_value(json!({
///     "Account": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "Balance": "99770260",
///     "Flags": 2174353408u32,
///     "OwnerCount": 19, "Sequence": 89926490, "LedgerEntryType": "AccountRoot",
///     "Domain": "68747470733A2F2F726970706C652E636F6D2F"
/// }))?;
/// assert_eq!((data.sequence, data.owner_count), (89926490, 19));
/// // The optional fields, under their `rippled` names.
/// assert_eq!(data.other["Domain"], "68747470733A2F2F726970706C652E636F6D2F");
/// # Ok::<(), serde_json::Error>(())
/// ```
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
    /// Every other field of the `AccountRoot` (`LedgerEntryType`, `Domain`,
    /// `RegularKey`, `TransferRate`, `TickSize`, `NFTokenMinter`, …).
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

impl AccountData {
    /// The XRP balance in drops (`None` only if the server sent a balance
    /// that is not XRP, which an account root never has).
    ///
    /// ```
    /// use serde_json::json;
    /// use xrpl_client::types::AccountData;
    ///
    /// let data: AccountData = serde_json::from_value(json!({
    ///     "Account": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "Balance": "99770260",
    ///     "Flags": 2174353408u32,
    ///     "OwnerCount": 19, "Sequence": 89926490
    /// }))?;
    /// assert_eq!(data.balance_drops(), Some(99_770_260)); // 99.77026 XRP
    /// # Ok::<(), serde_json::Error>(())
    /// ```
    pub fn balance_drops(&self) -> Option<u64> {
        match self.balance {
            xrpl_core::types::Amount::Xrp(drops) => Some(drops),
            _ => None,
        }
    }
}

/// The account's flags, one boolean each, as `rippled` spells them.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::AccountFlags;
///
/// let flags: AccountFlags = serde_json::from_value(json!({
///     "defaultRipple": true, "depositAuth": true, "disallowIncomingXRP": true, "noFreeze": false
/// }))?;
/// assert!(flags.default_ripple && flags.deposit_auth && flags.disallow_incoming_xrp);
/// // A flag the server leaves out reads as `false`.
/// assert!(!flags.global_freeze);
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::QueueData;
///
/// // Nothing queued: only the count.
/// let queue: QueueData = serde_json::from_value(json!({"txn_count": 0}))?;
/// assert_eq!(queue.txn_count, 0);
/// assert_eq!(queue.lowest_sequence, None);
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct QueueData {
    /// How many of the account's transactions are in the queue.
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::AccountLinesResult;
///
/// let page: AccountLinesResult = serde_json::from_value(json!({
///     "account": "rURtT5MMFzSUR54MAq9MKfzBoezTBZbKns",
///     "lines": [{"account": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "balance": "689748.581365565",
///         "currency": "524C555344000000000000000000000000000000", "limit": "10000000000",
///         "limit_peer": "0"}],
///     "marker": "2DE7C6A023569FD79077E9907D39289E2A8FBC2D3D8F24C42F05B1E13F455E7D,1",
///     "ledger_index": 107831520, "validated": true
/// }))?;
/// assert_eq!(page.lines[0].balance, "689748.581365565");
/// assert!(page.marker.is_some()); // more lines on the next page
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::TrustLine;
///
/// let line: TrustLine = serde_json::from_value(json!({
///     "account": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "balance": "689748.581365565",
///     "currency": "524C555344000000000000000000000000000000", "limit": "10000000000",
///     "limit_peer": "0",
///     "no_ripple": true, "quality_in": 0, "quality_out": 0
/// }))?;
/// // Positive: the peer (here the token's issuer) owes the requesting account.
/// assert!(line.balance.parse::<f64>()? > 0.0);
/// assert_eq!(line.no_ripple, Some(true));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::AccountOffersResult;
///
/// let page: AccountOffersResult = serde_json::from_value(json!({
///     "account": "rURtT5MMFzSUR54MAq9MKfzBoezTBZbKns",
///     "offers": [{"flags": 131072, "seq": 109300221, "quality": "81388.26",
///         "taker_gets": {
///             "currency": "BTC", "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B", "value": "0.12329"
///         },
///         "taker_pays": {
///             "currency": "524C555344000000000000000000000000000000",
///             "issuer": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "value": "10034.3585754"
///         }}],
///     "ledger_index": 107831520, "validated": true
/// }))?;
/// assert_eq!(page.offers[0].seq, 109300221);
/// assert_eq!(page.marker, None); // the last page
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::AccountOffer;
/// use xrpl_core::types::Amount;
///
/// let offer: AccountOffer =
///     serde_json::from_value(json!({"flags": 131072, "seq": 109300221, "quality": "81388.26",
///     "taker_gets": {
///         "currency": "BTC", "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B", "value": "0.12329"
///     },
///     "taker_pays": {
///         "currency": "524C555344000000000000000000000000000000",
///         "issuer": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "value": "10034.3585754"
///     }}))?;
/// assert_eq!(offer.seq, 109300221);
/// assert_eq!(
///     offer.taker_gets,
///     Amount::token("0.12329", "BTC", "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B")?,
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::AccountNftsResult;
///
/// let page: AccountNftsResult = serde_json::from_value(json!({
///     "account": "rfYiNfgLefTAZGfEyun1EjG68mTtC75vDe",
///     "account_nfts": [{"Flags": 0, "Issuer": "rfYiNfgLefTAZGfEyun1EjG68mTtC75vDe",
///         "NFTokenID": "0000000047D6BDCD766717B67A937E83CFEF22B1760537E30B759CB90644143E",
///         "NFTokenTaxon": 20260704, "nft_serial": 105124926}],
///     "limit": 1, "marker": "0000000047D6BDCD766717B67A937E83CFEF22B1760537E30B759CB90644143E",
///     "ledger_index": 107831520, "validated": true
/// }))?;
/// assert_eq!(page.account_nfts[0].nf_token_taxon, 20260704);
/// assert!(page.marker.is_some()); // more NFTs on the next page
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::NFToken;
/// use xrpl_core::utils::parse_nftoken_id;
///
/// let nft: NFToken =
///     serde_json::from_value(json!({"Flags": 0, "Issuer": "rfYiNfgLefTAZGfEyun1EjG68mTtC75vDe",
///     "NFTokenID": "0000000047D6BDCD766717B67A937E83CFEF22B1760537E30B759CB90644143E",
///     "NFTokenTaxon": 20260704, "nft_serial": 105124926}))?;
/// // The ID holds the issuer, the taxon and the serial.
/// let parts = parse_nftoken_id(&nft.nf_token_id)?;
/// assert_eq!(parts.issuer, nft.issuer);
/// assert_eq!((parts.taxon, parts.sequence), (nft.nf_token_taxon, nft.nft_serial));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::AccountTxResult;
///
/// let page: AccountTxResult = serde_json::from_value(json!({
///     "account": "rURtT5MMFzSUR54MAq9MKfzBoezTBZbKns",
///     "transactions": [{
///         "hash": "E3879013B23ED035EA7003ACE67552D7391CD3C8610446EFC3170F83259BD9AD",
///         "ledger_index": 107856059, "validated": true,
///         "meta": {"AffectedNodes": [], "TransactionIndex": 48, "TransactionResult": "tesSUCCESS"}
///     }],
///     "marker": {"ledger": 107856058, "seq": 39}, "validated": true
/// }))?;
/// assert_eq!(page.transactions[0].transaction_result(), Some("tesSUCCESS"));
/// assert!(page.marker.is_some()); // older transactions on the next page
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountTxResult {
    /// The account.
    #[serde(default)]
    pub account: String,
    /// The transactions, newest first unless `forward` was set.
    pub transactions: Vec<super::TxResult>,
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

/// [`account_objects`](https://xrpl.org/docs/references/http-websocket-apis/public-api-methods/account-methods/account_objects):
/// the ledger entries an account owns, typed.
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::{AccountObjectsResult, LedgerObject};
///
/// # let offer = json!({
/// #     "Account": "rURtT5MMFzSUR54MAq9MKfzBoezTBZbKns",
/// #     "BookDirectory": "DF621DB57C3AB113425E453CA295781CDA9A324848B162C8591CEA38189E6400",
/// #     "BookNode": "0", "Flags": 131072, "LedgerEntryType": "Offer", "OwnerNode": "1",
/// #     "PreviousTxnID": "5A12F3A0389812CAD8CF62C1FBDB7EECF1F8E547B7FD42F01A2101D89ABE4D94",
/// #     "PreviousTxnLgrSeq": 107831509, "Sequence": 109300221,
/// #     "TakerGets": {
/// #         "currency": "BTC", "issuer": "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B", "value": "0.12329"
/// #     },
/// #     "TakerPays": {
/// #         "currency": "524C555344000000000000000000000000000000",
/// #         "issuer": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De", "value": "10034.3585754"
/// #     },
/// #     "index": "0DE14C9E60C9A0C2CBA219DC9812888254214F490203C9BB618CA15667C66AD8"
/// # });
/// // `offer`: an `Offer` entry, as `rippled` writes it.
/// let page: AccountObjectsResult = serde_json::from_value(json!({
///     "account": "rURtT5MMFzSUR54MAq9MKfzBoezTBZbKns", "account_objects": [offer],
///     "ledger_index": 107831520, "validated": true
/// }))?;
/// assert_eq!(page.account_objects[0].entry_type(), "Offer");
/// assert!(matches!(page.account_objects[0], LedgerObject::Offer(_)));
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::AccountCurrenciesResult;
///
/// let currencies: AccountCurrenciesResult = serde_json::from_value(json!({
///     "receive_currencies": ["BTC", "ETH", "524C555344000000000000000000000000000000"],
///     "send_currencies": ["BTC", "ETH", "524C555344000000000000000000000000000000"],
///     "ledger_index": 107831520, "validated": true
/// }))?;
/// // Codes longer than three characters come as hex ("RLUSD").
/// assert!(currencies.send_currencies.iter().any(|c| c == "524C555344000000000000000000000000000000"));
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::{AccountChannelsResult, PaymentChannel};
///
/// # let channel = json!({
/// #     "account": "rhAe93ohNJkzvgD2rutsfnayqccnnJgS8W", "amount": "2000000", "balance": "500000",
/// #     "channel_id": "0433E751D6847198DD4D8226D9EACD609D80597AF575FA05F9AAA5C8A882CDF5",
/// #     "destination_account": "rMAdJ2PxkGkGtmt7XbTZiMczcZ4BA1E2wp", "settle_delay": 60,
/// #     "public_key_hex": "ED5392518D18161CC0501B8FD2B8AEFA959E81DB43DD5A4B4810AAEC38AD42A29B"
/// # });
/// // `channel`: one payment channel (see `PaymentChannel`).
/// let page: AccountChannelsResult = serde_json::from_value(json!({
///     "account": "rhAe93ohNJkzvgD2rutsfnayqccnnJgS8W", "channels": [channel],
///     "ledger_index": 21768537, "validated": true
/// }))?;
/// assert_eq!(page.channels[0].destination_account, "rMAdJ2PxkGkGtmt7XbTZiMczcZ4BA1E2wp");
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::PaymentChannel;
///
/// let channel: PaymentChannel = serde_json::from_value(json!({
///     "account": "rhAe93ohNJkzvgD2rutsfnayqccnnJgS8W", "amount": "2000000", "balance": "500000",
///     "channel_id": "0433E751D6847198DD4D8226D9EACD609D80597AF575FA05F9AAA5C8A882CDF5",
///     "destination_account": "rMAdJ2PxkGkGtmt7XbTZiMczcZ4BA1E2wp", "settle_delay": 60,
///     "public_key_hex": "ED5392518D18161CC0501B8FD2B8AEFA959E81DB43DD5A4B4810AAEC38AD42A29B"
/// }))?;
/// // What is left to claim, in drops.
/// let left = channel.amount.parse::<u64>()? - channel.balance.parse::<u64>()?;
/// assert_eq!(left, 1_500_000);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::GatewayBalancesResult;
///
/// let issuer: GatewayBalancesResult = serde_json::from_value(json!({
///     "account": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De",
///     "obligations": {"524C555344000000000000000000000000000000": "921782606.8615378"},
///     "ledger_index": 107831520, "validated": true
/// }))?;
/// let issued = issuer.obligations.unwrap();
/// assert_eq!(issued["524C555344000000000000000000000000000000"], "921782606.8615378");
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::GatewayBalance;
///
/// let balance: GatewayBalance = serde_json::from_value(json!({
///     "currency": "524C555344000000000000000000000000000000", "value": "689748.581365565"
/// }))?;
/// assert_eq!(balance.value, "689748.581365565");
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// ```
/// use serde_json::json;
/// use xrpl_client::types::NoRippleCheckResult;
///
/// let check: NoRippleCheckResult = serde_json::from_value(json!({
///     "problems": [], "transactions": [], "ledger_index": 107831520, "validated": true
/// }))?;
/// assert!(check.problems.is_empty()); // nothing to fix
/// # Ok::<(), serde_json::Error>(())
/// ```
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
