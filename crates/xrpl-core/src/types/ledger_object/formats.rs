// @generated from rippled 3.4.1's server_definitions (LEDGER_ENTRY_FORMATS) by
// crates/xrpl-core/scripts/gen_ledger_objects.py: one struct per ledger entry type, its fields as
// rippled lists them, and its ledger entry flags (LEDGER_ENTRY_FLAGS) as LSF_ constants.
// Regenerate: python3 gen_ledger_objects.py <server_definitions.json> <this file>
// Required fields are plain; optional and default ones (omitted from JSON at their default) are
// Option. UInt64, Blob and Number stay the strings rippled writes; objects and arrays stay JSON.

#![allow(clippy::upper_case_acronyms)]

use crate::types::{AccountId, Amount, Hash128, Hash160, Hash192, Hash256};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `AMM` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AMM {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// The trading fee, in 1/100,000 (1000 = 1%) (`TradingFee`: a number). Optional.
    #[serde(
        rename = "TradingFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub trading_fee: Option<u16>,
    /// `VoteSlots`: a list. Optional.
    #[serde(rename = "VoteSlots", default, skip_serializing_if = "Option::is_none")]
    pub vote_slots: Option<Vec<Value>>,
    /// `AuctionSlot`: an object. Optional.
    #[serde(
        rename = "AuctionSlot",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auction_slot: Option<Value>,
    /// `LPTokenBalance`: XRP in drops, a token, or an MPT. Required.
    #[serde(rename = "LPTokenBalance")]
    pub lp_token_balance: Amount,
    /// The pool's first asset (`Asset`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset")]
    pub asset: Value,
    /// The pool's second asset (`Asset2`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset2")]
    pub asset2: Value,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Optional.
    #[serde(
        rename = "PreviousTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_id: Option<Hash256>,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Optional.
    #[serde(
        rename = "PreviousTxnLgrSeq",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_lgr_seq: Option<u32>,
}

/// `AccountRoot` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountRoot {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// The sequence number (`Sequence`: a number). Required.
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    /// The balance (`Balance`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Balance")]
    pub balance: Amount,
    /// How many entries the account owns (each needs reserve) (`OwnerCount`: a number). Required.
    #[serde(rename = "OwnerCount")]
    pub owner_count: u32,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    /// `AccountTxnID`: 256 bits, hex. Optional.
    #[serde(
        rename = "AccountTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub account_txn_id: Option<Hash256>,
    /// The regular key pair's account; absent to remove it (`RegularKey`: a classic address). Optional.
    #[serde(
        rename = "RegularKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub regular_key: Option<AccountId>,
    /// The MD5 of an email address, for an avatar (`EmailHash`: 128 bits, hex). Optional.
    #[serde(rename = "EmailHash", default, skip_serializing_if = "Option::is_none")]
    pub email_hash: Option<Hash128>,
    /// `WalletLocator`: 256 bits, hex. Optional.
    #[serde(
        rename = "WalletLocator",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub wallet_locator: Option<Hash256>,
    /// `WalletSize`: a number. Optional.
    #[serde(
        rename = "WalletSize",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub wallet_size: Option<u32>,
    /// A public key for sending the account encrypted messages (`MessageKey`: bytes, hex). Optional.
    #[serde(
        rename = "MessageKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub message_key: Option<String>,
    /// The fee on transfers of the account's tokens, in billionths over 1,000,000,000 (`TransferRate`: a number). Optional.
    #[serde(
        rename = "TransferRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transfer_rate: Option<u32>,
    /// A domain the account claims, hex (ASCII, lower case) (`Domain`: bytes, hex). Optional.
    #[serde(rename = "Domain", default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Significant digits for offers of the account's tokens (3–15; 0 clears) (`TickSize`: a number). Optional.
    #[serde(rename = "TickSize", default, skip_serializing_if = "Option::is_none")]
    pub tick_size: Option<u8>,
    /// How many tickets to create (`TicketCount`: a number). Optional.
    #[serde(
        rename = "TicketCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub ticket_count: Option<u32>,
    /// An account allowed to mint NFTs for this one (`NFTokenMinter`: a classic address). Optional.
    #[serde(
        rename = "NFTokenMinter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub nf_token_minter: Option<AccountId>,
    /// `MintedNFTokens`: a number. Optional.
    #[serde(
        rename = "MintedNFTokens",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub minted_nf_tokens: Option<u32>,
    /// `BurnedNFTokens`: a number. Optional.
    #[serde(
        rename = "BurnedNFTokens",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub burned_nf_tokens: Option<u32>,
    /// `FirstNFTokenSequence`: a number. Optional.
    #[serde(
        rename = "FirstNFTokenSequence",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub first_nf_token_sequence: Option<u32>,
    /// `SponsoredOwnerCount`: a number. Optional.
    #[serde(
        rename = "SponsoredOwnerCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sponsored_owner_count: Option<u32>,
    /// `SponsoringOwnerCount`: a number. Optional.
    #[serde(
        rename = "SponsoringOwnerCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sponsoring_owner_count: Option<u32>,
    /// `SponsoringAccountCount`: a number. Optional.
    #[serde(
        rename = "SponsoringAccountCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sponsoring_account_count: Option<u32>,
    /// `AMMID`: 256 bits, hex. Optional.
    #[serde(rename = "AMMID", default, skip_serializing_if = "Option::is_none")]
    pub amm_id: Option<Hash256>,
    /// `VaultID`: 256 bits, hex. Optional.
    #[serde(rename = "VaultID", default, skip_serializing_if = "Option::is_none")]
    pub vault_id: Option<Hash256>,
    /// `LoanBrokerID`: 256 bits, hex. Optional.
    #[serde(
        rename = "LoanBrokerID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub loan_broker_id: Option<Hash256>,
}

impl AccountRoot {
    /// `lsfPasswordSpent`: The account has used its one free regular-key change (a SetRegularKey signed with the master key costs no fee until then).
    pub const LSF_PASSWORD_SPENT: u32 = 0x00010000;
    /// `lsfRequireDestTag`: Payments to the account must carry a DestinationTag (`asfRequireDest`).
    pub const LSF_REQUIRE_DEST_TAG: u32 = 0x00020000;
    /// `lsfRequireAuth`: The account must authorize each trust line that holds its tokens (`asfRequireAuth`).
    pub const LSF_REQUIRE_AUTH: u32 = 0x00040000;
    /// `lsfDisallowXRP`: The account asks not to be sent XRP (`asfDisallowXRP`; advisory, not enforced).
    pub const LSF_DISALLOW_XRP: u32 = 0x00080000;
    /// `lsfDisableMaster`: The master key cannot sign for the account (`asfDisableMaster`).
    pub const LSF_DISABLE_MASTER: u32 = 0x00100000;
    /// `lsfNoFreeze`: The account has given up freezing its tokens, for good (`asfNoFreeze`).
    pub const LSF_NO_FREEZE: u32 = 0x00200000;
    /// `lsfGlobalFreeze`: All tokens the account issues are frozen (`asfGlobalFreeze`).
    pub const LSF_GLOBAL_FREEZE: u32 = 0x00400000;
    /// `lsfDefaultRipple`: New trust lines to the account allow rippling by default (`asfDefaultRipple`).
    pub const LSF_DEFAULT_RIPPLE: u32 = 0x00800000;
    /// `lsfDepositAuth`: Only preauthorized accounts may send the account funds (`asfDepositAuth`).
    pub const LSF_DEPOSIT_AUTH: u32 = 0x01000000;
    /// `lsfDisallowIncomingNFTokenOffer`: Others may not make NFT offers to the account (`asfDisallowIncomingNFTokenOffer`).
    pub const LSF_DISALLOW_INCOMING_NFTOKEN_OFFER: u32 = 0x04000000;
    /// `lsfDisallowIncomingCheck`: Others may not create checks to the account (`asfDisallowIncomingCheck`).
    pub const LSF_DISALLOW_INCOMING_CHECK: u32 = 0x08000000;
    /// `lsfDisallowIncomingPayChan`: Others may not open payment channels to the account (`asfDisallowIncomingPayChan`).
    pub const LSF_DISALLOW_INCOMING_PAY_CHAN: u32 = 0x10000000;
    /// `lsfDisallowIncomingTrustline`: Others may not open trust lines to the account (`asfDisallowIncomingTrustline`).
    pub const LSF_DISALLOW_INCOMING_TRUSTLINE: u32 = 0x20000000;
    /// `lsfAllowTrustLineLocking`: The account's tokens may be locked in escrow (`asfAllowTrustLineLocking`).
    pub const LSF_ALLOW_TRUST_LINE_LOCKING: u32 = 0x40000000;
    /// `lsfAllowTrustLineClawback`: The account may claw back the tokens it issues (`asfAllowTrustLineClawback`).
    pub const LSF_ALLOW_TRUST_LINE_CLAWBACK: u32 = 0x80000000;
    /// Every flag `rippled` 3.4.1 defines for this entry: a bit outside it
    /// is one this version does not know.
    pub const ALL_FLAGS: u32 = 0xFDFF0000;

    /// Whether `flag` (one of the `LSF_` constants above) is set.
    #[must_use]
    pub fn has_flag(&self, flag: u32) -> bool {
        self.flags & flag == flag
    }
}

/// `Amendments` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Amendments {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// `Amendments`: a list of 256-bit IDs. Optional.
    #[serde(
        rename = "Amendments",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub amendments: Option<Vec<Hash256>>,
    /// `Majorities`: a list. Optional.
    #[serde(
        rename = "Majorities",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub majorities: Option<Vec<Value>>,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Optional.
    #[serde(
        rename = "PreviousTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_id: Option<Hash256>,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Optional.
    #[serde(
        rename = "PreviousTxnLgrSeq",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_lgr_seq: Option<u32>,
}

/// `Bridge` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bridge {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// `SignatureReward`: XRP in drops, a token, or an MPT. Required.
    #[serde(rename = "SignatureReward")]
    pub signature_reward: Amount,
    /// `MinAccountCreateAmount`: XRP in drops, a token, or an MPT. Optional.
    #[serde(
        rename = "MinAccountCreateAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub min_account_create_amount: Option<Amount>,
    /// `XChainBridge`: a cross-chain bridge. Required.
    #[serde(rename = "XChainBridge")]
    pub x_chain_bridge: Value,
    /// `XChainClaimID`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "XChainClaimID")]
    pub x_chain_claim_id: String,
    /// `XChainAccountCreateCount`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "XChainAccountCreateCount")]
    pub x_chain_account_create_count: String,
    /// `XChainAccountClaimCount`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "XChainAccountClaimCount")]
    pub x_chain_account_claim_count: String,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `Check` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Check {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// The account that receives it (`Destination`: a classic address). Required.
    #[serde(rename = "Destination")]
    pub destination: AccountId,
    /// The most the sender will spend, in any currency (`SendMax`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "SendMax")]
    pub send_max: Amount,
    /// The sequence number (`Sequence`: a number). Required.
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// `DestinationNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "DestinationNode")]
    pub destination_node: String,
    /// When it expires (ledger time: seconds since 2000-01-01) (`Expiration`: a number). Optional.
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
    /// An ID of the sender's choosing, such as an invoice number (`InvoiceID`: 256 bits, hex). Optional.
    #[serde(rename = "InvoiceID", default, skip_serializing_if = "Option::is_none")]
    pub invoice_id: Option<Hash256>,
    /// A tag the sender uses to tell its own payments apart (`SourceTag`: a number). Optional.
    #[serde(rename = "SourceTag", default, skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,
    /// A tag the destination uses to tell its senders apart (`DestinationTag`: a number). Optional.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `Credential` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Credential {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account a credential is about (`Subject`: a classic address). Required.
    #[serde(rename = "Subject")]
    pub subject: AccountId,
    /// The issuer (`Issuer`: a classic address). Required.
    #[serde(rename = "Issuer")]
    pub issuer: AccountId,
    /// The credential's type, hex (`CredentialType`: bytes, hex). Required.
    #[serde(rename = "CredentialType")]
    pub credential_type: String,
    /// When it expires (ledger time: seconds since 2000-01-01) (`Expiration`: a number). Optional.
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
    /// A URI, hex (`URI`: bytes, hex). Optional.
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// `IssuerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "IssuerNode")]
    pub issuer_node: String,
    /// `SubjectNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Optional.
    #[serde(
        rename = "SubjectNode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub subject_node: Option<String>,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

impl Credential {
    /// `lsfAccepted`: The subject has accepted the credential.
    pub const LSF_ACCEPTED: u32 = 0x00010000;
    /// Every flag `rippled` 3.4.1 defines for this entry: a bit outside it
    /// is one this version does not know.
    pub const ALL_FLAGS: u32 = 0x00010000;

    /// Whether `flag` (one of the `LSF_` constants above) is set.
    #[must_use]
    pub fn has_flag(&self, flag: u32) -> bool {
        self.flags & flag == flag
    }
}

/// `DID` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DID {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// The DID document, hex (`DIDDocument`: bytes, hex). Optional.
    #[serde(
        rename = "DIDDocument",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub did_document: Option<String>,
    /// A URI, hex (`URI`: bytes, hex). Optional.
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Data, hex (`Data`: bytes, hex). Optional.
    #[serde(rename = "Data", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `Delegate` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Delegate {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// The account to authorize (`Authorize`: a classic address). Required.
    #[serde(rename = "Authorize")]
    pub authorize: AccountId,
    /// The transaction types or granular permissions to grant (`Permissions`: a list). Required.
    #[serde(rename = "Permissions")]
    pub permissions: Vec<Value>,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// `DestinationNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Optional.
    #[serde(
        rename = "DestinationNode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_node: Option<String>,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `DepositPreauth` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositPreauth {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// The account to authorize (`Authorize`: a classic address). Optional.
    #[serde(rename = "Authorize", default, skip_serializing_if = "Option::is_none")]
    pub authorize: Option<AccountId>,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    /// Credentials to authorize (`AuthorizeCredentials`: a list). Optional.
    #[serde(
        rename = "AuthorizeCredentials",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub authorize_credentials: Option<Vec<Value>>,
}

/// `DirectoryNode` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectoryNode {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that owns the entry (`Owner`: a classic address). Optional.
    #[serde(rename = "Owner", default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<AccountId>,
    /// `TakerPaysCurrency`: 160 bits, hex. Optional.
    #[serde(
        rename = "TakerPaysCurrency",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_pays_currency: Option<Hash160>,
    /// `TakerPaysIssuer`: 160 bits, hex. Optional.
    #[serde(
        rename = "TakerPaysIssuer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_pays_issuer: Option<Hash160>,
    /// `TakerPaysMPT`: 192 bits, hex. Optional.
    #[serde(
        rename = "TakerPaysMPT",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_pays_mpt: Option<Hash192>,
    /// `TakerGetsCurrency`: 160 bits, hex. Optional.
    #[serde(
        rename = "TakerGetsCurrency",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_gets_currency: Option<Hash160>,
    /// `TakerGetsIssuer`: 160 bits, hex. Optional.
    #[serde(
        rename = "TakerGetsIssuer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_gets_issuer: Option<Hash160>,
    /// `TakerGetsMPT`: 192 bits, hex. Optional.
    #[serde(
        rename = "TakerGetsMPT",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_gets_mpt: Option<Hash192>,
    /// `ExchangeRate`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Optional.
    #[serde(
        rename = "ExchangeRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub exchange_rate: Option<String>,
    /// `Indexes`: a list of 256-bit IDs. Required.
    #[serde(rename = "Indexes")]
    pub indexes: Vec<Hash256>,
    /// `RootIndex`: 256 bits, hex. Required.
    #[serde(rename = "RootIndex")]
    pub root_index: Hash256,
    /// `IndexNext`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Optional.
    #[serde(rename = "IndexNext", default, skip_serializing_if = "Option::is_none")]
    pub index_next: Option<String>,
    /// `IndexPrevious`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Optional.
    #[serde(
        rename = "IndexPrevious",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub index_previous: Option<String>,
    /// An NFT's ID (`NFTokenID`: 256 bits, hex). Optional.
    #[serde(rename = "NFTokenID", default, skip_serializing_if = "Option::is_none")]
    pub nf_token_id: Option<Hash256>,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Optional.
    #[serde(
        rename = "PreviousTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_id: Option<Hash256>,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Optional.
    #[serde(
        rename = "PreviousTxnLgrSeq",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_lgr_seq: Option<u32>,
    /// A permissioned domain's ID (`DomainID`: 256 bits, hex). Optional.
    #[serde(rename = "DomainID", default, skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<Hash256>,
}

impl DirectoryNode {
    /// `lsfNFTokenBuyOffers`: The directory lists buy offers for an NFT.
    pub const LSF_NFTOKEN_BUY_OFFERS: u32 = 0x00000001;
    /// `lsfNFTokenSellOffers`: The directory lists sell offers for an NFT.
    pub const LSF_NFTOKEN_SELL_OFFERS: u32 = 0x00000002;
    /// Every flag `rippled` 3.4.1 defines for this entry: a bit outside it
    /// is one this version does not know.
    pub const ALL_FLAGS: u32 = 0x00000003;

    /// Whether `flag` (one of the `LSF_` constants above) is set.
    #[must_use]
    pub fn has_flag(&self, flag: u32) -> bool {
        self.flags & flag == flag
    }
}

/// `Escrow` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Escrow {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// The sequence number (`Sequence`: a number). Optional.
    #[serde(rename = "Sequence", default, skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u32>,
    /// The account that receives it (`Destination`: a classic address). Required.
    #[serde(rename = "Destination")]
    pub destination: AccountId,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// A PREIMAGE-SHA-256 crypto-condition that must be fulfilled (`Condition`: bytes, hex). Optional.
    #[serde(rename = "Condition", default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// The time after which it can be cancelled (ledger time) (`CancelAfter`: a number). Optional.
    #[serde(
        rename = "CancelAfter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cancel_after: Option<u32>,
    /// The time after which it can be finished (ledger time) (`FinishAfter`: a number). Optional.
    #[serde(
        rename = "FinishAfter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub finish_after: Option<u32>,
    /// A tag the sender uses to tell its own payments apart (`SourceTag`: a number). Optional.
    #[serde(rename = "SourceTag", default, skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,
    /// A tag the destination uses to tell its senders apart (`DestinationTag`: a number). Optional.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    /// `DestinationNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Optional.
    #[serde(
        rename = "DestinationNode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_node: Option<String>,
    /// The fee on transfers of the account's tokens, in billionths over 1,000,000,000 (`TransferRate`: a number). Optional.
    #[serde(
        rename = "TransferRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transfer_rate: Option<u32>,
    /// `IssuerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Optional.
    #[serde(
        rename = "IssuerNode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub issuer_node: Option<String>,
}

/// `FeeSettings` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeeSettings {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// `BaseFee`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Optional.
    #[serde(rename = "BaseFee", default, skip_serializing_if = "Option::is_none")]
    pub base_fee: Option<String>,
    /// `ReferenceFeeUnits`: a number. Optional.
    #[serde(
        rename = "ReferenceFeeUnits",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reference_fee_units: Option<u32>,
    /// `ReserveBase`: a number. Optional.
    #[serde(
        rename = "ReserveBase",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reserve_base: Option<u32>,
    /// `ReserveIncrement`: a number. Optional.
    #[serde(
        rename = "ReserveIncrement",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reserve_increment: Option<u32>,
    /// The reference transaction cost, in drops (`BaseFeeDrops`: XRP in drops, a token, or an MPT). Optional.
    #[serde(
        rename = "BaseFeeDrops",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub base_fee_drops: Option<Amount>,
    /// The account reserve, in drops (`ReserveBaseDrops`: XRP in drops, a token, or an MPT). Optional.
    #[serde(
        rename = "ReserveBaseDrops",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reserve_base_drops: Option<Amount>,
    /// The reserve per owned entry, in drops (`ReserveIncrementDrops`: XRP in drops, a token, or an MPT). Optional.
    #[serde(
        rename = "ReserveIncrementDrops",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reserve_increment_drops: Option<Amount>,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Optional.
    #[serde(
        rename = "PreviousTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_id: Option<Hash256>,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Optional.
    #[serde(
        rename = "PreviousTxnLgrSeq",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_lgr_seq: Option<u32>,
}

/// `LedgerHashes` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerHashes {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// `FirstLedgerSequence`: a number. Optional.
    #[serde(
        rename = "FirstLedgerSequence",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub first_ledger_sequence: Option<u32>,
    /// `LastLedgerSequence`: a number. Optional.
    #[serde(
        rename = "LastLedgerSequence",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub last_ledger_sequence: Option<u32>,
    /// `Hashes`: a list of 256-bit IDs. Required.
    #[serde(rename = "Hashes")]
    pub hashes: Vec<Hash256>,
}

/// `Loan` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Loan {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// `LoanBrokerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "LoanBrokerNode")]
    pub loan_broker_node: String,
    /// `LoanBrokerID`: 256 bits, hex. Required.
    #[serde(rename = "LoanBrokerID")]
    pub loan_broker_id: Hash256,
    /// `LoanSequence`: a number. Required.
    #[serde(rename = "LoanSequence")]
    pub loan_sequence: u32,
    /// `Borrower`: a classic address. Required.
    #[serde(rename = "Borrower")]
    pub borrower: AccountId,
    /// `LoanOriginationFee`: a decimal number, as a string. Optional.
    #[serde(
        rename = "LoanOriginationFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub loan_origination_fee: Option<String>,
    /// `LoanServiceFee`: a decimal number, as a string. Optional.
    #[serde(
        rename = "LoanServiceFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub loan_service_fee: Option<String>,
    /// `LatePaymentFee`: a decimal number, as a string. Optional.
    #[serde(
        rename = "LatePaymentFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub late_payment_fee: Option<String>,
    /// `ClosePaymentFee`: a decimal number, as a string. Optional.
    #[serde(
        rename = "ClosePaymentFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub close_payment_fee: Option<String>,
    /// `OverpaymentFee`: a number. Optional.
    #[serde(
        rename = "OverpaymentFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub overpayment_fee: Option<u32>,
    /// `InterestRate`: a number. Optional.
    #[serde(
        rename = "InterestRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub interest_rate: Option<u32>,
    /// `LateInterestRate`: a number. Optional.
    #[serde(
        rename = "LateInterestRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub late_interest_rate: Option<u32>,
    /// `CloseInterestRate`: a number. Optional.
    #[serde(
        rename = "CloseInterestRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub close_interest_rate: Option<u32>,
    /// `OverpaymentInterestRate`: a number. Optional.
    #[serde(
        rename = "OverpaymentInterestRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub overpayment_interest_rate: Option<u32>,
    /// `StartDate`: a number. Required.
    #[serde(rename = "StartDate")]
    pub start_date: u32,
    /// `PaymentInterval`: a number. Required.
    #[serde(rename = "PaymentInterval")]
    pub payment_interval: u32,
    /// `GracePeriod`: a number. Optional.
    #[serde(
        rename = "GracePeriod",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub grace_period: Option<u32>,
    /// `PreviousPaymentDueDate`: a number. Optional.
    #[serde(
        rename = "PreviousPaymentDueDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_payment_due_date: Option<u32>,
    /// `NextPaymentDueDate`: a number. Optional.
    #[serde(
        rename = "NextPaymentDueDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub next_payment_due_date: Option<u32>,
    /// `PaymentRemaining`: a number. Optional.
    #[serde(
        rename = "PaymentRemaining",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub payment_remaining: Option<u32>,
    /// `PeriodicPayment`: a decimal number, as a string. Required.
    #[serde(rename = "PeriodicPayment")]
    pub periodic_payment: String,
    /// `PrincipalOutstanding`: a decimal number, as a string. Optional.
    #[serde(
        rename = "PrincipalOutstanding",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub principal_outstanding: Option<String>,
    /// `TotalValueOutstanding`: a decimal number, as a string. Optional.
    #[serde(
        rename = "TotalValueOutstanding",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub total_value_outstanding: Option<String>,
    /// `ManagementFeeOutstanding`: a decimal number, as a string. Optional.
    #[serde(
        rename = "ManagementFeeOutstanding",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub management_fee_outstanding: Option<String>,
    /// `LoanScale`: a signed number. Optional.
    #[serde(rename = "LoanScale", default, skip_serializing_if = "Option::is_none")]
    pub loan_scale: Option<i32>,
}

impl Loan {
    /// `lsfLoanDefault`: The loan has defaulted.
    pub const LSF_LOAN_DEFAULT: u32 = 0x00010000;
    /// `lsfLoanImpaired`: The loan is impaired.
    pub const LSF_LOAN_IMPAIRED: u32 = 0x00020000;
    /// `lsfLoanOverpayment`: The loan accepts overpayments.
    pub const LSF_LOAN_OVERPAYMENT: u32 = 0x00040000;
    /// Every flag `rippled` 3.4.1 defines for this entry: a bit outside it
    /// is one this version does not know.
    pub const ALL_FLAGS: u32 = 0x00070000;

    /// Whether `flag` (one of the `LSF_` constants above) is set.
    #[must_use]
    pub fn has_flag(&self, flag: u32) -> bool {
        self.flags & flag == flag
    }
}

/// `LoanBroker` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoanBroker {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    /// The sequence number (`Sequence`: a number). Required.
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// `VaultNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "VaultNode")]
    pub vault_node: String,
    /// `VaultID`: 256 bits, hex. Required.
    #[serde(rename = "VaultID")]
    pub vault_id: Hash256,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// The account that owns the entry (`Owner`: a classic address). Required.
    #[serde(rename = "Owner")]
    pub owner: AccountId,
    /// `LoanSequence`: a number. Required.
    #[serde(rename = "LoanSequence")]
    pub loan_sequence: u32,
    /// Data, hex (`Data`: bytes, hex). Optional.
    #[serde(rename = "Data", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// `ManagementFeeRate`: a number. Optional.
    #[serde(
        rename = "ManagementFeeRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub management_fee_rate: Option<u16>,
    /// How many entries the account owns (each needs reserve) (`OwnerCount`: a number). Optional.
    #[serde(
        rename = "OwnerCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub owner_count: Option<u32>,
    /// `DebtTotal`: a decimal number, as a string. Optional.
    #[serde(rename = "DebtTotal", default, skip_serializing_if = "Option::is_none")]
    pub debt_total: Option<String>,
    /// `DebtMaximum`: a decimal number, as a string. Optional.
    #[serde(
        rename = "DebtMaximum",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub debt_maximum: Option<String>,
    /// `CoverAvailable`: a decimal number, as a string. Optional.
    #[serde(
        rename = "CoverAvailable",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cover_available: Option<String>,
    /// `CoverRateMinimum`: a number. Optional.
    #[serde(
        rename = "CoverRateMinimum",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cover_rate_minimum: Option<u32>,
    /// `CoverRateLiquidation`: a number. Optional.
    #[serde(
        rename = "CoverRateLiquidation",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cover_rate_liquidation: Option<u32>,
}

/// `MPToken` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MPToken {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// The MPT issuance's ID (`MPTokenIssuanceID`: 192 bits, hex). Required.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mp_token_issuance_id: Hash192,
    /// `MPTAmount`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Optional.
    #[serde(rename = "MPTAmount", default, skip_serializing_if = "Option::is_none")]
    pub mpt_amount: Option<String>,
    /// `LockedAmount`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Optional.
    #[serde(
        rename = "LockedAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub locked_amount: Option<String>,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    /// `ConfidentialBalanceInbox`: bytes, hex. Optional.
    #[serde(
        rename = "ConfidentialBalanceInbox",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub confidential_balance_inbox: Option<String>,
    /// `ConfidentialBalanceSpending`: bytes, hex. Optional.
    #[serde(
        rename = "ConfidentialBalanceSpending",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub confidential_balance_spending: Option<String>,
    /// `ConfidentialBalanceVersion`: a number. Optional.
    #[serde(
        rename = "ConfidentialBalanceVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub confidential_balance_version: Option<u32>,
    /// `IssuerEncryptedBalance`: bytes, hex. Optional.
    #[serde(
        rename = "IssuerEncryptedBalance",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub issuer_encrypted_balance: Option<String>,
    /// `AuditorEncryptedBalance`: bytes, hex. Optional.
    #[serde(
        rename = "AuditorEncryptedBalance",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auditor_encrypted_balance: Option<String>,
    /// `HolderEncryptionKey`: bytes, hex. Optional.
    #[serde(
        rename = "HolderEncryptionKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub holder_encryption_key: Option<String>,
}

impl MPToken {
    /// `lsfMPTLocked`: Locked: the tokens cannot move.
    pub const LSF_MPT_LOCKED: u32 = 0x00000001;
    /// `lsfMPTAuthorized`: The issuer has authorized the holder.
    pub const LSF_MPT_AUTHORIZED: u32 = 0x00000002;
    /// `lsfMPTAMM`: The holder is an AMM.
    pub const LSF_MPTAMM: u32 = 0x00000004;
    /// Every flag `rippled` 3.4.1 defines for this entry: a bit outside it
    /// is one this version does not know.
    pub const ALL_FLAGS: u32 = 0x00000007;

    /// Whether `flag` (one of the `LSF_` constants above) is set.
    #[must_use]
    pub fn has_flag(&self, flag: u32) -> bool {
        self.flags & flag == flag
    }
}

/// `MPTokenIssuance` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MPTokenIssuance {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The issuer (`Issuer`: a classic address). Required.
    #[serde(rename = "Issuer")]
    pub issuer: AccountId,
    /// The sequence number (`Sequence`: a number). Required.
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    /// The fee on secondary sales, in 1/100,000 (5000 = 5%); for tokens, see `TransferRate` (`TransferFee`: a number). Optional.
    #[serde(
        rename = "TransferFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transfer_fee: Option<u16>,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// Decimal places: an MPT amount of 12345 at scale 2 is 123.45 (`AssetScale`: a number). Optional.
    #[serde(
        rename = "AssetScale",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub asset_scale: Option<u8>,
    /// The most of the MPT that can ever exist, in its smallest unit (`MaximumAmount`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Optional.
    #[serde(
        rename = "MaximumAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub maximum_amount: Option<String>,
    /// `OutstandingAmount`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "OutstandingAmount")]
    pub outstanding_amount: String,
    /// `LockedAmount`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Optional.
    #[serde(
        rename = "LockedAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub locked_amount: Option<String>,
    /// Metadata, hex (XLS-89 JSON) (`MPTokenMetadata`: bytes, hex). Optional.
    #[serde(
        rename = "MPTokenMetadata",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mp_token_metadata: Option<String>,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    /// A permissioned domain's ID (`DomainID`: 256 bits, hex). Optional.
    #[serde(rename = "DomainID", default, skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<Hash256>,
    /// `ImmutableFlags`: a number. Optional.
    #[serde(
        rename = "ImmutableFlags",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub immutable_flags: Option<u32>,
    /// `ReferenceHolding`: 256 bits, hex. Optional.
    #[serde(
        rename = "ReferenceHolding",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reference_holding: Option<Hash256>,
    /// `IssuerEncryptionKey`: bytes, hex. Optional.
    #[serde(
        rename = "IssuerEncryptionKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub issuer_encryption_key: Option<String>,
    /// `AuditorEncryptionKey`: bytes, hex. Optional.
    #[serde(
        rename = "AuditorEncryptionKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auditor_encryption_key: Option<String>,
    /// `ConfidentialOutstandingAmount`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Optional.
    #[serde(
        rename = "ConfidentialOutstandingAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub confidential_outstanding_amount: Option<String>,
}

impl MPTokenIssuance {
    /// `lsfMPTLocked`: Locked: the tokens cannot move.
    pub const LSF_MPT_LOCKED: u32 = 0x00000001;
    /// `lsfMPTCanLock`: The issuer may lock the tokens, for one holder or all.
    pub const LSF_MPT_CAN_LOCK: u32 = 0x00000002;
    /// `lsfMPTRequireAuth`: Holders must be authorized by the issuer.
    pub const LSF_MPT_REQUIRE_AUTH: u32 = 0x00000004;
    /// `lsfMPTCanEscrow`: The tokens may be put in escrow.
    pub const LSF_MPT_CAN_ESCROW: u32 = 0x00000008;
    /// `lsfMPTCanTrade`: The tokens may be traded on the DEX.
    pub const LSF_MPT_CAN_TRADE: u32 = 0x00000010;
    /// `lsfMPTCanTransfer`: The tokens may be sent between holders, not only to and from the issuer.
    pub const LSF_MPT_CAN_TRANSFER: u32 = 0x00000020;
    /// `lsfMPTCanClawback`: The issuer may claw the tokens back.
    pub const LSF_MPT_CAN_CLAWBACK: u32 = 0x00000040;
    /// `lsfMPTCanHoldConfidentialBalance`: Holders may keep confidential balances of the token.
    pub const LSF_MPT_CAN_HOLD_CONFIDENTIAL_BALANCE: u32 = 0x00000080;
    /// Every flag `rippled` 3.4.1 defines for this entry: a bit outside it
    /// is one this version does not know.
    pub const ALL_FLAGS: u32 = 0x000000FF;

    /// Whether `flag` (one of the `LSF_` constants above) is set.
    #[must_use]
    pub fn has_flag(&self, flag: u32) -> bool {
        self.flags & flag == flag
    }
}

/// `NFTokenOffer` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NFTokenOffer {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that owns the entry (`Owner`: a classic address). Required.
    #[serde(rename = "Owner")]
    pub owner: AccountId,
    /// An NFT's ID (`NFTokenID`: 256 bits, hex). Required.
    #[serde(rename = "NFTokenID")]
    pub nf_token_id: Hash256,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// `NFTokenOfferNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "NFTokenOfferNode")]
    pub nf_token_offer_node: String,
    /// The account that receives it (`Destination`: a classic address). Optional.
    #[serde(
        rename = "Destination",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination: Option<AccountId>,
    /// When it expires (ledger time: seconds since 2000-01-01) (`Expiration`: a number). Optional.
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

impl NFTokenOffer {
    /// `lsfSellNFToken`: The offer sells the NFT (otherwise it is a buy offer).
    pub const LSF_SELL_NFTOKEN: u32 = 0x00000001;
    /// Every flag `rippled` 3.4.1 defines for this entry: a bit outside it
    /// is one this version does not know.
    pub const ALL_FLAGS: u32 = 0x00000001;

    /// Whether `flag` (one of the `LSF_` constants above) is set.
    #[must_use]
    pub fn has_flag(&self, flag: u32) -> bool {
        self.flags & flag == flag
    }
}

/// `NFTokenPage` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NFTokenPage {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// `PreviousPageMin`: 256 bits, hex. Optional.
    #[serde(
        rename = "PreviousPageMin",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_page_min: Option<Hash256>,
    /// `NextPageMin`: 256 bits, hex. Optional.
    #[serde(
        rename = "NextPageMin",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub next_page_min: Option<Hash256>,
    /// `NFTokens`: a list. Required.
    #[serde(rename = "NFTokens")]
    pub nf_tokens: Vec<Value>,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `NegativeUNL` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NegativeUNL {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// `DisabledValidators`: a list. Optional.
    #[serde(
        rename = "DisabledValidators",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub disabled_validators: Option<Vec<Value>>,
    /// `ValidatorToDisable`: bytes, hex. Optional.
    #[serde(
        rename = "ValidatorToDisable",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub validator_to_disable: Option<String>,
    /// `ValidatorToReEnable`: bytes, hex. Optional.
    #[serde(
        rename = "ValidatorToReEnable",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub validator_to_re_enable: Option<String>,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Optional.
    #[serde(
        rename = "PreviousTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_id: Option<Hash256>,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Optional.
    #[serde(
        rename = "PreviousTxnLgrSeq",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_lgr_seq: Option<u32>,
}

/// `Offer` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Offer {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// The sequence number (`Sequence`: a number). Required.
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    /// What the offer wants in return (`TakerPays`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "TakerPays")]
    pub taker_pays: Amount,
    /// What the offer gives (`TakerGets`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "TakerGets")]
    pub taker_gets: Amount,
    /// The order book directory page holding the offer (`BookDirectory`: 256 bits, hex). Required.
    #[serde(rename = "BookDirectory")]
    pub book_directory: Hash256,
    /// Which page of the book directory lists the offer (`BookNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "BookNode")]
    pub book_node: String,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    /// When it expires (ledger time: seconds since 2000-01-01) (`Expiration`: a number). Optional.
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
    /// A permissioned domain's ID (`DomainID`: 256 bits, hex). Optional.
    #[serde(rename = "DomainID", default, skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<Hash256>,
    /// `AdditionalBooks`: a list. Optional.
    #[serde(
        rename = "AdditionalBooks",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_books: Option<Vec<Value>>,
}

impl Offer {
    /// `lsfPassive`: A passive offer: it does not consume offers that match it exactly.
    pub const LSF_PASSIVE: u32 = 0x00010000;
    /// `lsfSell`: A sell offer: it exchanges all of TakerGets, even for more than TakerPays.
    pub const LSF_SELL: u32 = 0x00020000;
    /// `lsfHybrid`: A hybrid offer: it trades in a permissioned domain's book and the open one.
    pub const LSF_HYBRID: u32 = 0x00040000;
    /// Every flag `rippled` 3.4.1 defines for this entry: a bit outside it
    /// is one this version does not know.
    pub const ALL_FLAGS: u32 = 0x00070000;

    /// Whether `flag` (one of the `LSF_` constants above) is set.
    #[must_use]
    pub fn has_flag(&self, flag: u32) -> bool {
        self.flags & flag == flag
    }
}

/// `Oracle` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Oracle {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that owns the entry (`Owner`: a classic address). Required.
    #[serde(rename = "Owner")]
    pub owner: AccountId,
    /// The oracle's number among its owner's oracles (`OracleDocumentID`: a number). Optional.
    #[serde(
        rename = "OracleDocumentID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub oracle_document_id: Option<u32>,
    /// The data provider, hex (`Provider`: bytes, hex). Required.
    #[serde(rename = "Provider")]
    pub provider: String,
    /// The prices (`PriceDataSeries`: a list). Required.
    #[serde(rename = "PriceDataSeries")]
    pub price_data_series: Vec<Value>,
    /// The kind of asset priced, hex (e.g. "currency") (`AssetClass`: bytes, hex). Required.
    #[serde(rename = "AssetClass")]
    pub asset_class: String,
    /// When the prices were last updated (Unix time) (`LastUpdateTime`: a number). Required.
    #[serde(rename = "LastUpdateTime")]
    pub last_update_time: u32,
    /// A URI, hex (`URI`: bytes, hex). Optional.
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `PayChannel` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayChannel {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// The account that receives it (`Destination`: a classic address). Required.
    #[serde(rename = "Destination")]
    pub destination: AccountId,
    /// The sequence number (`Sequence`: a number). Optional.
    #[serde(rename = "Sequence", default, skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u32>,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// The balance (`Balance`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Balance")]
    pub balance: Amount,
    /// A public key, hex (`PublicKey`: bytes, hex). Required.
    #[serde(rename = "PublicKey")]
    pub public_key: String,
    /// Seconds the source must wait before closing a channel with unclaimed XRP (`SettleDelay`: a number). Required.
    #[serde(rename = "SettleDelay")]
    pub settle_delay: u32,
    /// When it expires (ledger time: seconds since 2000-01-01) (`Expiration`: a number). Optional.
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
    /// The time after which it can be cancelled (ledger time) (`CancelAfter`: a number). Optional.
    #[serde(
        rename = "CancelAfter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cancel_after: Option<u32>,
    /// A tag the sender uses to tell its own payments apart (`SourceTag`: a number). Optional.
    #[serde(rename = "SourceTag", default, skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,
    /// A tag the destination uses to tell its senders apart (`DestinationTag`: a number). Optional.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    /// `DestinationNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Optional.
    #[serde(
        rename = "DestinationNode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_node: Option<String>,
}

/// `PermissionedDomain` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionedDomain {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that owns the entry (`Owner`: a classic address). Required.
    #[serde(rename = "Owner")]
    pub owner: AccountId,
    /// The sequence number (`Sequence`: a number). Required.
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    /// The credentials the domain accepts (`AcceptedCredentials`: a list). Required.
    #[serde(rename = "AcceptedCredentials")]
    pub accepted_credentials: Vec<Value>,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `RippleState` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RippleState {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The balance (`Balance`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Balance")]
    pub balance: Amount,
    /// The low account's limit (the lower account ID) (`LowLimit`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "LowLimit")]
    pub low_limit: Amount,
    /// The high account's limit (the higher account ID) (`HighLimit`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "HighLimit")]
    pub high_limit: Amount,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    /// Which page of the low account's directory lists this line (`LowNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Optional.
    #[serde(rename = "LowNode", default, skip_serializing_if = "Option::is_none")]
    pub low_node: Option<String>,
    /// `LowQualityIn`: a number. Optional.
    #[serde(
        rename = "LowQualityIn",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub low_quality_in: Option<u32>,
    /// `LowQualityOut`: a number. Optional.
    #[serde(
        rename = "LowQualityOut",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub low_quality_out: Option<u32>,
    /// Which page of the high account's directory lists this line (`HighNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Optional.
    #[serde(rename = "HighNode", default, skip_serializing_if = "Option::is_none")]
    pub high_node: Option<String>,
    /// `HighQualityIn`: a number. Optional.
    #[serde(
        rename = "HighQualityIn",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub high_quality_in: Option<u32>,
    /// `HighQualityOut`: a number. Optional.
    #[serde(
        rename = "HighQualityOut",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub high_quality_out: Option<u32>,
    /// `HighSponsor`: a classic address. Optional.
    #[serde(
        rename = "HighSponsor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub high_sponsor: Option<AccountId>,
    /// `LowSponsor`: a classic address. Optional.
    #[serde(
        rename = "LowSponsor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub low_sponsor: Option<AccountId>,
}

impl RippleState {
    /// `lsfLowReserve`: The trust line counts toward the low account's owner reserve.
    pub const LSF_LOW_RESERVE: u32 = 0x00010000;
    /// `lsfHighReserve`: The trust line counts toward the high account's owner reserve.
    pub const LSF_HIGH_RESERVE: u32 = 0x00020000;
    /// `lsfLowAuth`: The low account has authorized the high account to hold its tokens.
    pub const LSF_LOW_AUTH: u32 = 0x00040000;
    /// `lsfHighAuth`: The high account has authorized the low account to hold its tokens.
    pub const LSF_HIGH_AUTH: u32 = 0x00080000;
    /// `lsfLowNoRipple`: The low account has disabled rippling through this line.
    pub const LSF_LOW_NO_RIPPLE: u32 = 0x00100000;
    /// `lsfHighNoRipple`: The high account has disabled rippling through this line.
    pub const LSF_HIGH_NO_RIPPLE: u32 = 0x00200000;
    /// `lsfLowFreeze`: The low account has frozen the line.
    pub const LSF_LOW_FREEZE: u32 = 0x00400000;
    /// `lsfHighFreeze`: The high account has frozen the line.
    pub const LSF_HIGH_FREEZE: u32 = 0x00800000;
    /// `lsfAMMNode`: The trust line belongs to an AMM.
    pub const LSF_AMM_NODE: u32 = 0x01000000;
    /// `lsfLowDeepFreeze`: The low account has deep-frozen the line.
    pub const LSF_LOW_DEEP_FREEZE: u32 = 0x02000000;
    /// `lsfHighDeepFreeze`: The high account has deep-frozen the line.
    pub const LSF_HIGH_DEEP_FREEZE: u32 = 0x04000000;
    /// Every flag `rippled` 3.4.1 defines for this entry: a bit outside it
    /// is one this version does not know.
    pub const ALL_FLAGS: u32 = 0x07FF0000;

    /// Whether `flag` (one of the `LSF_` constants above) is set.
    #[must_use]
    pub fn has_flag(&self, flag: u32) -> bool {
        self.flags & flag == flag
    }
}

/// `SignerList` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignerList {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that owns the entry (`Owner`: a classic address). Optional.
    #[serde(rename = "Owner", default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<AccountId>,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// The total signer weight a multi-signature needs; 0 deletes the list (`SignerQuorum`: a number). Required.
    #[serde(rename = "SignerQuorum")]
    pub signer_quorum: u32,
    /// The signers and their weights (`SignerEntries`: a list). Required.
    #[serde(rename = "SignerEntries")]
    pub signer_entries: Vec<Value>,
    /// `SignerListID`: a number. Required.
    #[serde(rename = "SignerListID")]
    pub signer_list_id: u32,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

impl SignerList {
    /// `lsfOneOwnerCount`: The signer list counts as one entry toward the owner reserve.
    pub const LSF_ONE_OWNER_COUNT: u32 = 0x00010000;
    /// Every flag `rippled` 3.4.1 defines for this entry: a bit outside it
    /// is one this version does not know.
    pub const ALL_FLAGS: u32 = 0x00010000;

    /// Whether `flag` (one of the `LSF_` constants above) is set.
    #[must_use]
    pub fn has_flag(&self, flag: u32) -> bool {
        self.flags & flag == flag
    }
}

/// `Sponsorship` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sponsorship {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    /// The account that owns the entry (`Owner`: a classic address). Required.
    #[serde(rename = "Owner")]
    pub owner: AccountId,
    /// `Sponsee`: a classic address. Required.
    #[serde(rename = "Sponsee")]
    pub sponsee: AccountId,
    /// `FeeAmount`: XRP in drops, a token, or an MPT. Optional.
    #[serde(rename = "FeeAmount", default, skip_serializing_if = "Option::is_none")]
    pub fee_amount: Option<Amount>,
    /// `MaxFee`: XRP in drops, a token, or an MPT. Optional.
    #[serde(rename = "MaxFee", default, skip_serializing_if = "Option::is_none")]
    pub max_fee: Option<Amount>,
    /// `RemainingOwnerCount`: a number. Optional.
    #[serde(
        rename = "RemainingOwnerCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub remaining_owner_count: Option<u32>,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// `SponseeNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "SponseeNode")]
    pub sponsee_node: String,
}

impl Sponsorship {
    /// `lsfSponsorshipRequireSignForFee`: The sponsor must sign each transaction whose fee it pays.
    pub const LSF_SPONSORSHIP_REQUIRE_SIGN_FOR_FEE: u32 = 0x00010000;
    /// `lsfSponsorshipRequireSignForReserve`: The sponsor must sign each transaction whose reserve it covers.
    pub const LSF_SPONSORSHIP_REQUIRE_SIGN_FOR_RESERVE: u32 = 0x00020000;
    /// Every flag `rippled` 3.4.1 defines for this entry: a bit outside it
    /// is one this version does not know.
    pub const ALL_FLAGS: u32 = 0x00030000;

    /// Whether `flag` (one of the `LSF_` constants above) is set.
    #[must_use]
    pub fn has_flag(&self, flag: u32) -> bool {
        self.flags & flag == flag
    }
}

/// `Ticket` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ticket {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// `TicketSequence`: a number. Required.
    #[serde(rename = "TicketSequence")]
    pub ticket_sequence: u32,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `Vault` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vault {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    /// The sequence number (`Sequence`: a number). Required.
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// The account that owns the entry (`Owner`: a classic address). Required.
    #[serde(rename = "Owner")]
    pub owner: AccountId,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// Data, hex (`Data`: bytes, hex). Optional.
    #[serde(rename = "Data", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// The pool's first asset (`Asset`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset")]
    pub asset: Value,
    /// `AssetsTotal`: a decimal number, as a string. Optional.
    #[serde(
        rename = "AssetsTotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub assets_total: Option<String>,
    /// `AssetsAvailable`: a decimal number, as a string. Optional.
    #[serde(
        rename = "AssetsAvailable",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub assets_available: Option<String>,
    /// `AssetsMaximum`: a decimal number, as a string. Optional.
    #[serde(
        rename = "AssetsMaximum",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub assets_maximum: Option<String>,
    /// `LossUnrealized`: a decimal number, as a string. Optional.
    #[serde(
        rename = "LossUnrealized",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub loss_unrealized: Option<String>,
    /// `ShareMPTID`: 192 bits, hex. Required.
    #[serde(rename = "ShareMPTID")]
    pub share_mpt_id: Hash192,
    /// `WithdrawalPolicy`: a number. Required.
    #[serde(rename = "WithdrawalPolicy")]
    pub withdrawal_policy: u8,
    /// `Scale`: a number. Optional.
    #[serde(rename = "Scale", default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<u8>,
    /// `LEVersion`: a number. Optional.
    #[serde(rename = "LEVersion", default, skip_serializing_if = "Option::is_none")]
    pub le_version: Option<u8>,
    /// `VaultKind`: a number. Optional.
    #[serde(rename = "VaultKind", default, skip_serializing_if = "Option::is_none")]
    pub vault_kind: Option<u8>,
    /// `SubscriptionDate`: a number. Optional.
    #[serde(
        rename = "SubscriptionDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub subscription_date: Option<u32>,
    /// `RedemptionDate`: a number. Optional.
    #[serde(
        rename = "RedemptionDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub redemption_date: Option<u32>,
}

impl Vault {
    /// `lsfVaultPrivate`: A private vault: only authorized accounts may deposit.
    pub const LSF_VAULT_PRIVATE: u32 = 0x00010000;
    /// Every flag `rippled` 3.4.1 defines for this entry: a bit outside it
    /// is one this version does not know.
    pub const ALL_FLAGS: u32 = 0x00010000;

    /// Whether `flag` (one of the `LSF_` constants above) is set.
    #[must_use]
    pub fn has_flag(&self, flag: u32) -> bool {
        self.flags & flag == flag
    }
}

/// `XChainOwnedClaimID` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XChainOwnedClaimID {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// `XChainBridge`: a cross-chain bridge. Required.
    #[serde(rename = "XChainBridge")]
    pub x_chain_bridge: Value,
    /// `XChainClaimID`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "XChainClaimID")]
    pub x_chain_claim_id: String,
    /// `OtherChainSource`: a classic address. Required.
    #[serde(rename = "OtherChainSource")]
    pub other_chain_source: AccountId,
    /// `XChainClaimAttestations`: a list. Required.
    #[serde(rename = "XChainClaimAttestations")]
    pub x_chain_claim_attestations: Vec<Value>,
    /// `SignatureReward`: XRP in drops, a token, or an MPT. Required.
    #[serde(rename = "SignatureReward")]
    pub signature_reward: Amount,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `XChainOwnedCreateAccountClaimID` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XChainOwnedCreateAccountClaimID {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    /// Flag bits (`Flags`: a number). Required.
    #[serde(rename = "Flags")]
    pub flags: u32,
    /// `Sponsor`: a classic address. Optional.
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    /// The account that sends the transaction or owns the entry (`Account`: a classic address). Required.
    #[serde(rename = "Account")]
    pub account: AccountId,
    /// `XChainBridge`: a cross-chain bridge. Required.
    #[serde(rename = "XChainBridge")]
    pub x_chain_bridge: Value,
    /// `XChainAccountCreateCount`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "XChainAccountCreateCount")]
    pub x_chain_account_create_count: String,
    /// `XChainCreateAccountAttestations`: a list. Required.
    #[serde(rename = "XChainCreateAccountAttestations")]
    pub x_chain_create_account_attestations: Vec<Value>,
    /// Which page of the owner's directory lists this entry (`OwnerNode`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Required.
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    /// The last transaction that changed this entry (`PreviousTxnID`: 256 bits, hex). Required.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    /// The ledger of the last transaction that changed this entry (`PreviousTxnLgrSeq`: a number). Required.
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// Every ledger entry type rippled 3.4.1 defines, by `LedgerEntryType`.
pub const NAMES: &[&str] = &[
    "AMM",
    "AccountRoot",
    "Amendments",
    "Bridge",
    "Check",
    "Credential",
    "DID",
    "Delegate",
    "DepositPreauth",
    "DirectoryNode",
    "Escrow",
    "FeeSettings",
    "LedgerHashes",
    "Loan",
    "LoanBroker",
    "MPToken",
    "MPTokenIssuance",
    "NFTokenOffer",
    "NFTokenPage",
    "NegativeUNL",
    "Offer",
    "Oracle",
    "PayChannel",
    "PermissionedDomain",
    "RippleState",
    "SignerList",
    "Sponsorship",
    "Ticket",
    "Vault",
    "XChainOwnedClaimID",
    "XChainOwnedCreateAccountClaimID",
];

macro_rules! each_entry_type {
    ($m:ident) => {
        $m!(
            AMM,
            AccountRoot,
            Amendments,
            Bridge,
            Check,
            Credential,
            DID,
            Delegate,
            DepositPreauth,
            DirectoryNode,
            Escrow,
            FeeSettings,
            LedgerHashes,
            Loan,
            LoanBroker,
            MPToken,
            MPTokenIssuance,
            NFTokenOffer,
            NFTokenPage,
            NegativeUNL,
            Offer,
            Oracle,
            PayChannel,
            PermissionedDomain,
            RippleState,
            SignerList,
            Sponsorship,
            Ticket,
            Vault,
            XChainOwnedClaimID,
            XChainOwnedCreateAccountClaimID
        );
    };
}
pub(super) use each_entry_type;
