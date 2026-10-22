// @generated from rippled 3.4.1's server_definitions (LEDGER_ENTRY_FORMATS) by
// crates/xrpl-core/scripts/gen_ledger_objects.py: one struct per ledger entry type, its fields as
// rippled lists them. Regenerate: python3 gen_ledger_objects.py <server_definitions.json> <this file>
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
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(
        rename = "TradingFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub trading_fee: Option<u16>,
    #[serde(rename = "VoteSlots", default, skip_serializing_if = "Option::is_none")]
    pub vote_slots: Option<Vec<Value>>,
    #[serde(
        rename = "AuctionSlot",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auction_slot: Option<Value>,
    #[serde(rename = "LPTokenBalance")]
    pub lp_token_balance: Amount,
    #[serde(rename = "Asset")]
    pub asset: Value,
    #[serde(rename = "Asset2")]
    pub asset2: Value,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(
        rename = "PreviousTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_id: Option<Hash256>,
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
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    #[serde(rename = "Balance")]
    pub balance: Amount,
    #[serde(rename = "OwnerCount")]
    pub owner_count: u32,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    #[serde(
        rename = "AccountTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub account_txn_id: Option<Hash256>,
    #[serde(
        rename = "RegularKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub regular_key: Option<AccountId>,
    #[serde(rename = "EmailHash", default, skip_serializing_if = "Option::is_none")]
    pub email_hash: Option<Hash128>,
    #[serde(
        rename = "WalletLocator",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub wallet_locator: Option<Hash256>,
    #[serde(
        rename = "WalletSize",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub wallet_size: Option<u32>,
    #[serde(
        rename = "MessageKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub message_key: Option<String>,
    #[serde(
        rename = "TransferRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transfer_rate: Option<u32>,
    #[serde(rename = "Domain", default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(rename = "TickSize", default, skip_serializing_if = "Option::is_none")]
    pub tick_size: Option<u8>,
    #[serde(
        rename = "TicketCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub ticket_count: Option<u32>,
    #[serde(
        rename = "NFTokenMinter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub nf_token_minter: Option<AccountId>,
    #[serde(
        rename = "MintedNFTokens",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub minted_nf_tokens: Option<u32>,
    #[serde(
        rename = "BurnedNFTokens",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub burned_nf_tokens: Option<u32>,
    #[serde(
        rename = "FirstNFTokenSequence",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub first_nf_token_sequence: Option<u32>,
    #[serde(
        rename = "SponsoredOwnerCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sponsored_owner_count: Option<u32>,
    #[serde(
        rename = "SponsoringOwnerCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sponsoring_owner_count: Option<u32>,
    #[serde(
        rename = "SponsoringAccountCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sponsoring_account_count: Option<u32>,
    #[serde(rename = "AMMID", default, skip_serializing_if = "Option::is_none")]
    pub amm_id: Option<Hash256>,
    #[serde(rename = "VaultID", default, skip_serializing_if = "Option::is_none")]
    pub vault_id: Option<Hash256>,
    #[serde(
        rename = "LoanBrokerID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub loan_broker_id: Option<Hash256>,
}

/// `Amendments` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Amendments {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(
        rename = "Amendments",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub amendments: Option<Vec<Hash256>>,
    #[serde(
        rename = "Majorities",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub majorities: Option<Vec<Value>>,
    #[serde(
        rename = "PreviousTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_id: Option<Hash256>,
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
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(rename = "SignatureReward")]
    pub signature_reward: Amount,
    #[serde(
        rename = "MinAccountCreateAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub min_account_create_amount: Option<Amount>,
    #[serde(rename = "XChainBridge")]
    pub x_chain_bridge: Value,
    #[serde(rename = "XChainClaimID")]
    pub x_chain_claim_id: String,
    #[serde(rename = "XChainAccountCreateCount")]
    pub x_chain_account_create_count: String,
    #[serde(rename = "XChainAccountClaimCount")]
    pub x_chain_account_claim_count: String,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `Check` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Check {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(rename = "Destination")]
    pub destination: AccountId,
    #[serde(rename = "SendMax")]
    pub send_max: Amount,
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "DestinationNode")]
    pub destination_node: String,
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
    #[serde(rename = "InvoiceID", default, skip_serializing_if = "Option::is_none")]
    pub invoice_id: Option<Hash256>,
    #[serde(rename = "SourceTag", default, skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `Credential` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Credential {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Subject")]
    pub subject: AccountId,
    #[serde(rename = "Issuer")]
    pub issuer: AccountId,
    #[serde(rename = "CredentialType")]
    pub credential_type: String,
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(rename = "IssuerNode")]
    pub issuer_node: String,
    #[serde(
        rename = "SubjectNode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub subject_node: Option<String>,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `DID` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DID {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(
        rename = "DIDDocument",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub did_document: Option<String>,
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(rename = "Data", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `Delegate` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Delegate {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(rename = "Authorize")]
    pub authorize: AccountId,
    #[serde(rename = "Permissions")]
    pub permissions: Vec<Value>,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(
        rename = "DestinationNode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_node: Option<String>,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `DepositPreauth` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositPreauth {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(rename = "Authorize", default, skip_serializing_if = "Option::is_none")]
    pub authorize: Option<AccountId>,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
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
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Owner", default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<AccountId>,
    #[serde(
        rename = "TakerPaysCurrency",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_pays_currency: Option<Hash160>,
    #[serde(
        rename = "TakerPaysIssuer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_pays_issuer: Option<Hash160>,
    #[serde(
        rename = "TakerPaysMPT",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_pays_mpt: Option<Hash192>,
    #[serde(
        rename = "TakerGetsCurrency",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_gets_currency: Option<Hash160>,
    #[serde(
        rename = "TakerGetsIssuer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_gets_issuer: Option<Hash160>,
    #[serde(
        rename = "TakerGetsMPT",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_gets_mpt: Option<Hash192>,
    #[serde(
        rename = "ExchangeRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub exchange_rate: Option<String>,
    #[serde(rename = "Indexes")]
    pub indexes: Vec<Hash256>,
    #[serde(rename = "RootIndex")]
    pub root_index: Hash256,
    #[serde(rename = "IndexNext", default, skip_serializing_if = "Option::is_none")]
    pub index_next: Option<String>,
    #[serde(
        rename = "IndexPrevious",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub index_previous: Option<String>,
    #[serde(rename = "NFTokenID", default, skip_serializing_if = "Option::is_none")]
    pub nf_token_id: Option<Hash256>,
    #[serde(
        rename = "PreviousTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_id: Option<Hash256>,
    #[serde(
        rename = "PreviousTxnLgrSeq",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_lgr_seq: Option<u32>,
    #[serde(rename = "DomainID", default, skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<Hash256>,
}

/// `Escrow` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Escrow {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(rename = "Sequence", default, skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u32>,
    #[serde(rename = "Destination")]
    pub destination: AccountId,
    #[serde(rename = "Amount")]
    pub amount: Amount,
    #[serde(rename = "Condition", default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(
        rename = "CancelAfter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cancel_after: Option<u32>,
    #[serde(
        rename = "FinishAfter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub finish_after: Option<u32>,
    #[serde(rename = "SourceTag", default, skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    #[serde(
        rename = "DestinationNode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_node: Option<String>,
    #[serde(
        rename = "TransferRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transfer_rate: Option<u32>,
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
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "BaseFee", default, skip_serializing_if = "Option::is_none")]
    pub base_fee: Option<String>,
    #[serde(
        rename = "ReferenceFeeUnits",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reference_fee_units: Option<u32>,
    #[serde(
        rename = "ReserveBase",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reserve_base: Option<u32>,
    #[serde(
        rename = "ReserveIncrement",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reserve_increment: Option<u32>,
    #[serde(
        rename = "BaseFeeDrops",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub base_fee_drops: Option<Amount>,
    #[serde(
        rename = "ReserveBaseDrops",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reserve_base_drops: Option<Amount>,
    #[serde(
        rename = "ReserveIncrementDrops",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reserve_increment_drops: Option<Amount>,
    #[serde(
        rename = "PreviousTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_id: Option<Hash256>,
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
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(
        rename = "FirstLedgerSequence",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub first_ledger_sequence: Option<u32>,
    #[serde(
        rename = "LastLedgerSequence",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub last_ledger_sequence: Option<u32>,
    #[serde(rename = "Hashes")]
    pub hashes: Vec<Hash256>,
}

/// `Loan` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Loan {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "LoanBrokerNode")]
    pub loan_broker_node: String,
    #[serde(rename = "LoanBrokerID")]
    pub loan_broker_id: Hash256,
    #[serde(rename = "LoanSequence")]
    pub loan_sequence: u32,
    #[serde(rename = "Borrower")]
    pub borrower: AccountId,
    #[serde(
        rename = "LoanOriginationFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub loan_origination_fee: Option<String>,
    #[serde(
        rename = "LoanServiceFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub loan_service_fee: Option<String>,
    #[serde(
        rename = "LatePaymentFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub late_payment_fee: Option<String>,
    #[serde(
        rename = "ClosePaymentFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub close_payment_fee: Option<String>,
    #[serde(
        rename = "OverpaymentFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub overpayment_fee: Option<u32>,
    #[serde(
        rename = "InterestRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub interest_rate: Option<u32>,
    #[serde(
        rename = "LateInterestRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub late_interest_rate: Option<u32>,
    #[serde(
        rename = "CloseInterestRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub close_interest_rate: Option<u32>,
    #[serde(
        rename = "OverpaymentInterestRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub overpayment_interest_rate: Option<u32>,
    #[serde(rename = "StartDate")]
    pub start_date: u32,
    #[serde(rename = "PaymentInterval")]
    pub payment_interval: u32,
    #[serde(
        rename = "GracePeriod",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub grace_period: Option<u32>,
    #[serde(
        rename = "PreviousPaymentDueDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_payment_due_date: Option<u32>,
    #[serde(
        rename = "NextPaymentDueDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub next_payment_due_date: Option<u32>,
    #[serde(
        rename = "PaymentRemaining",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub payment_remaining: Option<u32>,
    #[serde(rename = "PeriodicPayment")]
    pub periodic_payment: String,
    #[serde(
        rename = "PrincipalOutstanding",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub principal_outstanding: Option<String>,
    #[serde(
        rename = "TotalValueOutstanding",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub total_value_outstanding: Option<String>,
    #[serde(
        rename = "ManagementFeeOutstanding",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub management_fee_outstanding: Option<String>,
    #[serde(rename = "LoanScale", default, skip_serializing_if = "Option::is_none")]
    pub loan_scale: Option<i32>,
}

/// `LoanBroker` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoanBroker {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "VaultNode")]
    pub vault_node: String,
    #[serde(rename = "VaultID")]
    pub vault_id: Hash256,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(rename = "Owner")]
    pub owner: AccountId,
    #[serde(rename = "LoanSequence")]
    pub loan_sequence: u32,
    #[serde(rename = "Data", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(
        rename = "ManagementFeeRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub management_fee_rate: Option<u16>,
    #[serde(
        rename = "OwnerCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub owner_count: Option<u32>,
    #[serde(rename = "DebtTotal", default, skip_serializing_if = "Option::is_none")]
    pub debt_total: Option<String>,
    #[serde(
        rename = "DebtMaximum",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub debt_maximum: Option<String>,
    #[serde(
        rename = "CoverAvailable",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cover_available: Option<String>,
    #[serde(
        rename = "CoverRateMinimum",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cover_rate_minimum: Option<u32>,
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
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(rename = "MPTokenIssuanceID")]
    pub mp_token_issuance_id: Hash192,
    #[serde(rename = "MPTAmount", default, skip_serializing_if = "Option::is_none")]
    pub mpt_amount: Option<String>,
    #[serde(
        rename = "LockedAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub locked_amount: Option<String>,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    #[serde(
        rename = "ConfidentialBalanceInbox",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub confidential_balance_inbox: Option<String>,
    #[serde(
        rename = "ConfidentialBalanceSpending",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub confidential_balance_spending: Option<String>,
    #[serde(
        rename = "ConfidentialBalanceVersion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub confidential_balance_version: Option<u32>,
    #[serde(
        rename = "IssuerEncryptedBalance",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub issuer_encrypted_balance: Option<String>,
    #[serde(
        rename = "AuditorEncryptedBalance",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auditor_encrypted_balance: Option<String>,
    #[serde(
        rename = "HolderEncryptionKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub holder_encryption_key: Option<String>,
}

/// `MPTokenIssuance` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MPTokenIssuance {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Issuer")]
    pub issuer: AccountId,
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    #[serde(
        rename = "TransferFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transfer_fee: Option<u16>,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(
        rename = "AssetScale",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub asset_scale: Option<u8>,
    #[serde(
        rename = "MaximumAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub maximum_amount: Option<String>,
    #[serde(rename = "OutstandingAmount")]
    pub outstanding_amount: String,
    #[serde(
        rename = "LockedAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub locked_amount: Option<String>,
    #[serde(
        rename = "MPTokenMetadata",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mp_token_metadata: Option<String>,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    #[serde(rename = "DomainID", default, skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<Hash256>,
    #[serde(
        rename = "ImmutableFlags",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub immutable_flags: Option<u32>,
    #[serde(
        rename = "ReferenceHolding",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reference_holding: Option<Hash256>,
    #[serde(
        rename = "IssuerEncryptionKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub issuer_encryption_key: Option<String>,
    #[serde(
        rename = "AuditorEncryptionKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auditor_encryption_key: Option<String>,
    #[serde(
        rename = "ConfidentialOutstandingAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub confidential_outstanding_amount: Option<String>,
}

/// `NFTokenOffer` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NFTokenOffer {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Owner")]
    pub owner: AccountId,
    #[serde(rename = "NFTokenID")]
    pub nf_token_id: Hash256,
    #[serde(rename = "Amount")]
    pub amount: Amount,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "NFTokenOfferNode")]
    pub nf_token_offer_node: String,
    #[serde(
        rename = "Destination",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination: Option<AccountId>,
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `NFTokenPage` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NFTokenPage {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(
        rename = "PreviousPageMin",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_page_min: Option<Hash256>,
    #[serde(
        rename = "NextPageMin",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub next_page_min: Option<Hash256>,
    #[serde(rename = "NFTokens")]
    pub nf_tokens: Vec<Value>,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `NegativeUNL` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NegativeUNL {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(
        rename = "DisabledValidators",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub disabled_validators: Option<Vec<Value>>,
    #[serde(
        rename = "ValidatorToDisable",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub validator_to_disable: Option<String>,
    #[serde(
        rename = "ValidatorToReEnable",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub validator_to_re_enable: Option<String>,
    #[serde(
        rename = "PreviousTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_id: Option<Hash256>,
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
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    #[serde(rename = "TakerPays")]
    pub taker_pays: Amount,
    #[serde(rename = "TakerGets")]
    pub taker_gets: Amount,
    #[serde(rename = "BookDirectory")]
    pub book_directory: Hash256,
    #[serde(rename = "BookNode")]
    pub book_node: String,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
    #[serde(rename = "DomainID", default, skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<Hash256>,
    #[serde(
        rename = "AdditionalBooks",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_books: Option<Vec<Value>>,
}

/// `Oracle` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Oracle {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Owner")]
    pub owner: AccountId,
    #[serde(
        rename = "OracleDocumentID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub oracle_document_id: Option<u32>,
    #[serde(rename = "Provider")]
    pub provider: String,
    #[serde(rename = "PriceDataSeries")]
    pub price_data_series: Vec<Value>,
    #[serde(rename = "AssetClass")]
    pub asset_class: String,
    #[serde(rename = "LastUpdateTime")]
    pub last_update_time: u32,
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `PayChannel` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayChannel {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(rename = "Destination")]
    pub destination: AccountId,
    #[serde(rename = "Sequence", default, skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u32>,
    #[serde(rename = "Amount")]
    pub amount: Amount,
    #[serde(rename = "Balance")]
    pub balance: Amount,
    #[serde(rename = "PublicKey")]
    pub public_key: String,
    #[serde(rename = "SettleDelay")]
    pub settle_delay: u32,
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
    #[serde(
        rename = "CancelAfter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cancel_after: Option<u32>,
    #[serde(rename = "SourceTag", default, skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
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
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Owner")]
    pub owner: AccountId,
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    #[serde(rename = "AcceptedCredentials")]
    pub accepted_credentials: Vec<Value>,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `RippleState` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RippleState {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Balance")]
    pub balance: Amount,
    #[serde(rename = "LowLimit")]
    pub low_limit: Amount,
    #[serde(rename = "HighLimit")]
    pub high_limit: Amount,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    #[serde(rename = "LowNode", default, skip_serializing_if = "Option::is_none")]
    pub low_node: Option<String>,
    #[serde(
        rename = "LowQualityIn",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub low_quality_in: Option<u32>,
    #[serde(
        rename = "LowQualityOut",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub low_quality_out: Option<u32>,
    #[serde(rename = "HighNode", default, skip_serializing_if = "Option::is_none")]
    pub high_node: Option<String>,
    #[serde(
        rename = "HighQualityIn",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub high_quality_in: Option<u32>,
    #[serde(
        rename = "HighQualityOut",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub high_quality_out: Option<u32>,
    #[serde(
        rename = "HighSponsor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub high_sponsor: Option<AccountId>,
    #[serde(
        rename = "LowSponsor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub low_sponsor: Option<AccountId>,
}

/// `SignerList` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignerList {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Owner", default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<AccountId>,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "SignerQuorum")]
    pub signer_quorum: u32,
    #[serde(rename = "SignerEntries")]
    pub signer_entries: Vec<Value>,
    #[serde(rename = "SignerListID")]
    pub signer_list_id: u32,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `Sponsorship` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sponsorship {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    #[serde(rename = "Owner")]
    pub owner: AccountId,
    #[serde(rename = "Sponsee")]
    pub sponsee: AccountId,
    #[serde(rename = "FeeAmount", default, skip_serializing_if = "Option::is_none")]
    pub fee_amount: Option<Amount>,
    #[serde(rename = "MaxFee", default, skip_serializing_if = "Option::is_none")]
    pub max_fee: Option<Amount>,
    #[serde(
        rename = "RemainingOwnerCount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub remaining_owner_count: Option<u32>,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "SponseeNode")]
    pub sponsee_node: String,
}

/// `Ticket` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ticket {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "TicketSequence")]
    pub ticket_sequence: u32,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `Vault` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vault {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "Owner")]
    pub owner: AccountId,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(rename = "Data", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(rename = "Asset")]
    pub asset: Value,
    #[serde(
        rename = "AssetsTotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub assets_total: Option<String>,
    #[serde(
        rename = "AssetsAvailable",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub assets_available: Option<String>,
    #[serde(
        rename = "AssetsMaximum",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub assets_maximum: Option<String>,
    #[serde(
        rename = "LossUnrealized",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub loss_unrealized: Option<String>,
    #[serde(rename = "ShareMPTID")]
    pub share_mpt_id: Hash192,
    #[serde(rename = "WithdrawalPolicy")]
    pub withdrawal_policy: u8,
    #[serde(rename = "Scale", default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<u8>,
    #[serde(rename = "LEVersion", default, skip_serializing_if = "Option::is_none")]
    pub le_version: Option<u8>,
    #[serde(rename = "VaultKind", default, skip_serializing_if = "Option::is_none")]
    pub vault_kind: Option<u8>,
    #[serde(
        rename = "SubscriptionDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub subscription_date: Option<u32>,
    #[serde(
        rename = "RedemptionDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub redemption_date: Option<u32>,
}

/// `XChainOwnedClaimID` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XChainOwnedClaimID {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(rename = "XChainBridge")]
    pub x_chain_bridge: Value,
    #[serde(rename = "XChainClaimID")]
    pub x_chain_claim_id: String,
    #[serde(rename = "OtherChainSource")]
    pub other_chain_source: AccountId,
    #[serde(rename = "XChainClaimAttestations")]
    pub x_chain_claim_attestations: Vec<Value>,
    #[serde(rename = "SignatureReward")]
    pub signature_reward: Amount,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
    #[serde(rename = "PreviousTxnLgrSeq")]
    pub previous_txn_lgr_seq: u32,
}

/// `XChainOwnedCreateAccountClaimID` ledger entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XChainOwnedCreateAccountClaimID {
    /// The object's ID (rippled's `index`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<Hash256>,
    #[serde(rename = "Flags")]
    pub flags: u32,
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<AccountId>,
    #[serde(rename = "Account")]
    pub account: AccountId,
    #[serde(rename = "XChainBridge")]
    pub x_chain_bridge: Value,
    #[serde(rename = "XChainAccountCreateCount")]
    pub x_chain_account_create_count: String,
    #[serde(rename = "XChainCreateAccountAttestations")]
    pub x_chain_create_account_attestations: Vec<Value>,
    #[serde(rename = "OwnerNode")]
    pub owner_node: String,
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: Hash256,
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
