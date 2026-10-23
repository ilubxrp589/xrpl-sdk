//! Typed transactions: a struct for each of the 82 transaction types in
//! `rippled` 3.4.1, generated from its own definitions (`formats.rs`), and
//! [`Tx`] over all of them.
//!
//! Each struct holds the fields every transaction has in [`Common`], then
//! its own fields under `rippled`'s names. `new` takes the required ones;
//! `with_…` sets an optional one (each type's own fields, and through
//! [`TxModel`] the common ones: `with_fee`, `with_sequence`, `with_memo`, …),
//! so a transaction reads as one expression. Flags are constants on the type
//! (`Payment::TF_PARTIAL_PAYMENT`, `AccountSet::ASF_DEFAULT_RIPPLE`). The
//! fields stay public to read and change; the types are `#[non_exhaustive]`,
//! so a field `rippled` adds later is not a breaking change.
//!
//! ```
//! use xrpl_core::transaction::models::{Memo, Payment, Tx, TxModel};
//! use xrpl_core::types::Amount;
//!
//! let pay = Payment::new(
//!     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
//!     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
//!     Amount::Xrp(1_000_000),
//! )
//! .with_destination_tag(42u32)
//! .with_flags(Payment::TF_PARTIAL_PAYMENT)
//! .with_memo(Memo::text("invoice 17"));
//!
//! let json = pay.to_json()?; // rippled's JSON, `TransactionType` included
//! assert_eq!(json["TransactionType"], "Payment");
//! assert_eq!(json["Amount"], "1000000");
//!
//! // And back: a transaction from rippled, typed.
//! match Tx::from_value(json)? {
//!     Tx::Payment(p) => assert_eq!(p.destination_tag, Some(42)),
//!     other => panic!("{}", other.transaction_type()),
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [`Tx::from_value`] is strict: a field the type does not have is an error,
//! not silently dropped. Deserializing a [`Tx`] (inside a larger response) is
//! lenient: what does not fit stays available as [`Tx::Other`].

mod formats;
mod validate;

pub use formats::*;

use crate::types::{Amount, Hash128, Hash192, Hash256};
use crate::CoreError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::BTreeSet;

/// What every typed transaction offers, whatever its type.
pub trait TxModel: Serialize + Into<Tx> + Clone {
    /// The `TransactionType` (`"Payment"`, `"OfferCreate"`, …).
    const TRANSACTION_TYPE: &'static str;
    /// The fields every transaction has.
    fn common(&self) -> &Common;
    /// The common fields, to change.
    fn common_mut(&mut self) -> &mut Common;

    /// `rippled`'s JSON for the transaction, `TransactionType` included: what
    /// `autofill`, the signers and `encode_transaction_json` take.
    ///
    /// # Errors
    ///
    /// [`CoreError::CodecError`] if a field does not serialize to JSON (for
    /// example a hand-built `Currency::Standard` code that is not UTF-8).
    fn to_json(&self) -> Result<Value, CoreError> {
        let mut v = serde_json::to_value(self).map_err(|e| CoreError::CodecError(e.to_string()))?;
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "TransactionType".into(),
                Value::String(Self::TRANSACTION_TYPE.into()),
            );
        }
        Ok(v)
    }

    /// The checks `rippled` makes before applying the transaction that need
    /// nothing from the ledger (its `preflight`): an AMM deposit's amounts
    /// fit its mode flag, a signer list's weights reach its quorum, a
    /// `TickSize` is in range, and so on. [`ToTxJson`](crate::transaction::ToTxJson)
    /// and so `Wallet::sign` run it, so a transaction that `rippled` would
    /// refuse (`tem…`) is refused here, saying why, before it is signed.
    ///
    /// ```
    /// use xrpl_core::transaction::models::{AMMVote, Asset, TxModel};
    ///
    /// let vote = AMMVote::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", Asset::xrp(),
    ///     Asset::token("USD", "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"), 1500);
    /// assert!(vote.validate().is_err()); // at most 1000 (1%)
    /// ```
    ///
    /// # Errors
    ///
    /// [`CoreError::ValidationError`] naming the type and the rule broken,
    /// and the address errors of a field that holds an account.
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }

    /// With `Fee` set, in drops.
    #[must_use]
    fn with_fee(mut self, drops: u64) -> Self {
        self.common_mut().fee = Some(drops.to_string());
        self
    }

    /// With `Sequence` set.
    #[must_use]
    fn with_sequence(mut self, sequence: u32) -> Self {
        self.common_mut().sequence = Some(sequence);
        self
    }

    /// With `TicketSequence` set, and `Sequence` 0 as a ticket's transaction
    /// has.
    #[must_use]
    fn with_ticket_sequence(mut self, ticket: u32) -> Self {
        let common = self.common_mut();
        common.ticket_sequence = Some(ticket);
        common.sequence = Some(0);
        self
    }

    /// With `LastLedgerSequence` set: the last ledger the transaction may be
    /// in.
    #[must_use]
    fn with_last_ledger_sequence(mut self, ledger: u32) -> Self {
        self.common_mut().last_ledger_sequence = Some(ledger);
        self
    }

    /// With `Flags` set (the type's `TF_` constants, combined with `|`).
    #[must_use]
    fn with_flags(mut self, flags: u32) -> Self {
        self.common_mut().flags = Some(flags);
        self
    }

    /// With `SourceTag` set.
    #[must_use]
    fn with_source_tag(mut self, tag: u32) -> Self {
        self.common_mut().source_tag = Some(tag);
        self
    }

    /// With `NetworkID` set (required on networks whose ID is above 1024;
    /// `autofill` sets it).
    #[must_use]
    fn with_network_id(mut self, id: u32) -> Self {
        self.common_mut().network_id = Some(id);
        self
    }

    /// With `memo` added to `Memos`.
    #[must_use]
    fn with_memo(mut self, memo: Memo) -> Self {
        self.common_mut()
            .memos
            .get_or_insert_with(Vec::new)
            .push(memo);
        self
    }

    /// With `Delegate` set: the account this one acts for (XLS-75).
    #[must_use]
    fn with_delegate(mut self, account: impl Into<String>) -> Self {
        self.common_mut().delegate = Some(account.into());
        self
    }
}

/// The fields every transaction has (`rippled`'s common transaction fields).
/// `Fee`, `Sequence` and `SigningPubKey` are required on the ledger but
/// optional here, so a transaction can be built before `autofill` and
/// signing fill them in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Common {
    /// The sending account (a classic address; `""` on pseudo-transactions
    /// whose account is never set).
    #[serde(rename = "Account")]
    pub account: String,
    /// Transaction cost in drops.
    #[serde(rename = "Fee", default, skip_serializing_if = "Option::is_none")]
    pub fee: Option<String>,
    /// The sequence number (`Sequence`).
    #[serde(rename = "Sequence", default, skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u32>,
    /// Flag bits (`Flags`).
    #[serde(rename = "Flags", default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,
    /// The last ledger the transaction may appear in.
    #[serde(
        rename = "LastLedgerSequence",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub last_ledger_sequence: Option<u32>,
    /// A tag the sender uses to tell its own payments apart (`SourceTag`).
    #[serde(rename = "SourceTag", default, skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<u32>,
    /// A ticket to use instead of a sequence number (`Sequence` is then 0).
    #[serde(
        rename = "TicketSequence",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub ticket_sequence: Option<u32>,
    /// Valid only if the account's last transaction had this hash (`AccountTxnID`).
    #[serde(
        rename = "AccountTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub account_txn_id: Option<Hash256>,
    /// Notes attached to the transaction (`Memos`).
    #[serde(
        rename = "Memos",
        default,
        skip_serializing_if = "Option::is_none",
        with = "memos_opt"
    )]
    pub memos: Option<Vec<Memo>>,
    /// Required on networks with ID above 1024.
    #[serde(rename = "NetworkID", default, skip_serializing_if = "Option::is_none")]
    pub network_id: Option<u32>,
    /// The account sending on `Account`'s behalf (DelegateSet permissions).
    #[serde(rename = "Delegate", default, skip_serializing_if = "Option::is_none")]
    pub delegate: Option<String>,
    /// Hex; empty for a multi-signed transaction.
    #[serde(
        rename = "SigningPubKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub signing_pub_key: Option<String>,
    /// The signature, hex (`TxnSignature`).
    #[serde(
        rename = "TxnSignature",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub txn_signature: Option<String>,
    /// The signatures of a multi-signed transaction (`Signers`).
    #[serde(
        rename = "Signers",
        default,
        skip_serializing_if = "Option::is_none",
        with = "signers_opt"
    )]
    pub signers: Option<Vec<Signer>>,
    /// The last transaction that changed this entry (`PreviousTxnID`).
    #[serde(
        rename = "PreviousTxnID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_txn_id: Option<Hash256>,
    /// A limit on the operations the transaction may perform (`OperationLimit`).
    #[serde(
        rename = "OperationLimit",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub operation_limit: Option<u32>,
    /// The account sponsoring the fee or reserve (`Sponsor`).
    #[serde(rename = "Sponsor", default, skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<String>,
    /// How the sponsor pays (`SponsorFlags`).
    #[serde(
        rename = "SponsorFlags",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sponsor_flags: Option<u32>,
    /// The sponsor's signature (`SponsorSignature`).
    #[serde(
        rename = "SponsorSignature",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sponsor_signature: Option<Value>,
    /// The transaction's ID: not a transaction field, but `rippled` writes it
    /// into the JSON it returns. The codec ignores it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<Hash256>,
    /// The close time of the ledger it is in (ledger time): not a
    /// transaction field, but `rippled` adds it in streams and `tx`. The
    /// codec ignores it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<u64>,
}

impl Common {
    /// Just `Account`; everything else unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            fee: None,
            sequence: None,
            flags: None,
            last_ledger_sequence: None,
            source_tag: None,
            ticket_sequence: None,
            account_txn_id: None,
            memos: None,
            network_id: None,
            delegate: None,
            signing_pub_key: None,
            txn_signature: None,
            signers: None,
            previous_txn_id: None,
            operation_limit: None,
            sponsor: None,
            sponsor_flags: None,
            sponsor_signature: None,
            hash: None,
            date: None,
        }
    }
}

/// One of a transaction's `Memos`. Each part is hex (see `utils::str_to_hex`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Memo {
    /// What kind of memo, hex (for example a MIME type) (`MemoType`).
    #[serde(rename = "MemoType", default, skip_serializing_if = "Option::is_none")]
    pub memo_type: Option<String>,
    /// The memo's content, hex (`MemoData`).
    #[serde(rename = "MemoData", default, skip_serializing_if = "Option::is_none")]
    pub memo_data: Option<String>,
    /// How the content is encoded, hex (`MemoFormat`).
    #[serde(
        rename = "MemoFormat",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub memo_format: Option<String>,
}

/// One signature of a multi-signed transaction (`Signers`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Signer {
    /// The signer's account (`Account`).
    #[serde(rename = "Account")]
    pub account: String,
    /// The signing public key, hex; empty when multi-signed (`SigningPubKey`).
    #[serde(rename = "SigningPubKey")]
    pub signing_pub_key: String,
    /// The signature, hex (`TxnSignature`).
    #[serde(rename = "TxnSignature")]
    pub txn_signature: String,
}

/// A member of a signer list (SignerListSet's `SignerEntries`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SignerEntry {
    /// An account on the signer list (`Account`).
    #[serde(rename = "Account")]
    pub account: String,
    /// The signer's weight toward the quorum (`SignerWeight`).
    #[serde(rename = "SignerWeight")]
    pub signer_weight: u16,
    /// Data about the signer, of the list owner's choosing (`WalletLocator`).
    #[serde(
        rename = "WalletLocator",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub wallet_locator: Option<Hash256>,
}

/// One price of an oracle (OracleSet's `PriceDataSeries`): `AssetPrice` ×
/// 10^-`Scale` of `QuoteAsset` per `BaseAsset`. Omitting `AssetPrice`
/// deletes the pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PriceData {
    /// The asset priced (for example `XRP`) (`BaseAsset`).
    #[serde(rename = "BaseAsset")]
    pub base_asset: String,
    /// The asset the price is in (for example `USD`) (`QuoteAsset`).
    #[serde(rename = "QuoteAsset")]
    pub quote_asset: String,
    /// UInt64: a number in JSON, or hex as `rippled` writes it.
    #[serde(
        rename = "AssetPrice",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub asset_price: Option<Value>,
    /// The price's decimal places (`Scale`).
    #[serde(rename = "Scale", default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<u8>,
}

/// An account in AMMBid's `AuthAccounts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AuthAccount {
    /// The account.
    #[serde(rename = "Account")]
    pub account: String,
}

/// One permission DelegateSet grants: a transaction type (`"Payment"`) or a
/// granular permission (`"TrustlineAuthorize"`, …).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Permission {
    /// A transaction type or a granular permission name (`PermissionValue`).
    #[serde(rename = "PermissionValue")]
    pub permission_value: String,
}

/// A credential, by issuer and type (DepositPreauth's
/// `AuthorizeCredentials`, PermissionedDomainSet's `AcceptedCredentials`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CredentialSpec {
    /// The issuer (`Issuer`).
    #[serde(rename = "Issuer")]
    pub issuer: String,
    /// Hex.
    #[serde(rename = "CredentialType")]
    pub credential_type: String,
}

/// An account's approval of a `Batch` it has inner transactions in: a
/// signature (`TxnSignature`) or a multi-signature (`Signers`). See
/// `transaction::batch`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BatchSigner {
    /// The account signing the batch (`Account`): one with an inner transaction in it.
    #[serde(rename = "Account")]
    pub account: String,
    /// Hex; empty when `signers` is used.
    #[serde(rename = "SigningPubKey")]
    pub signing_pub_key: String,
    /// The signature, hex (`TxnSignature`).
    #[serde(
        rename = "TxnSignature",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub txn_signature: Option<String>,
    /// The signatures of a multi-signed transaction (`Signers`).
    #[serde(
        rename = "Signers",
        default,
        skip_serializing_if = "Option::is_none",
        with = "signers_opt"
    )]
    pub signers: Option<Vec<Signer>>,
}

/// An asset: XRP, a token (currency + issuer) or an MPT. `rippled` writes
/// `{"currency": "XRP"}`, `{"currency", "issuer"}` or `{"mpt_issuance_id"}`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Asset {
    /// The currency code (`currency`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// The issuer (`Issuer`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    /// The MPT issuance's ID (`mpt_issuance_id`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mpt_issuance_id: Option<String>,
}

impl Asset {
    /// XRP.
    pub fn xrp() -> Self {
        Self {
            currency: Some("XRP".into()),
            ..Default::default()
        }
    }
    /// A token: currency code and issuer.
    pub fn token(currency: impl Into<String>, issuer: impl Into<String>) -> Self {
        Self {
            currency: Some(currency.into()),
            issuer: Some(issuer.into()),
            ..Default::default()
        }
    }
    /// An MPT, by MPTokenIssuanceID.
    pub fn mpt(mpt_issuance_id: impl Into<String>) -> Self {
        Self {
            mpt_issuance_id: Some(mpt_issuance_id.into()),
            ..Default::default()
        }
    }
}

/// A cross-chain bridge: its door accounts and assets on each chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct XChainBridge {
    /// The bridge's door account on the locking chain (`LockingChainDoor`).
    #[serde(rename = "LockingChainDoor")]
    pub locking_chain_door: String,
    /// The bridged asset on the locking chain (`LockingChainIssue`).
    #[serde(rename = "LockingChainIssue")]
    pub locking_chain_issue: Asset,
    /// The bridge's door account on the issuing chain (`IssuingChainDoor`).
    #[serde(rename = "IssuingChainDoor")]
    pub issuing_chain_door: String,
    /// The bridged asset on the issuing chain (`IssuingChainIssue`).
    #[serde(rename = "IssuingChainIssue")]
    pub issuing_chain_issue: Asset,
}

/// One step of a payment path: an account to ripple through, or a currency
/// (and issuer) to convert into. `type` / `type_hex` are what `rippled`
/// writes back; the codec derives them from the other fields.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PathStep {
    /// An account to ripple through (`account`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// The currency code (`currency`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// An MPT to convert into, instead of a currency (`mpt_issuance_id`; path
    /// steps of type 64, new in `rippled` 3.4.1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mpt_issuance_id: Option<String>,
    /// The issuer (`Issuer`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    /// The step's type bits (1 account, 16 currency, 32 issuer, 64 MPT) (`type`).
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub step_type: Option<u32>,
    /// The step's type, hex.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_hex: Option<String>,
}

/// `rippled` wraps each element of an object array in an object named for
/// it (`[{"Memo": {…}}, …]`); these modules read and write that form as a
/// plain `Vec`.
macro_rules! wrapped_array {
    ($module:ident, $opt:ident, $ty:ty, $key:literal) => {
        // Each array is required in some transactions and optional in others,
        // so one of the pair may go unused.
        #[allow(dead_code)]
        mod $module {
            use super::*;
            use serde::de::Error;
            use std::collections::BTreeMap;

            pub(super) fn serialize<S: Serializer>(items: &[$ty], s: S) -> Result<S::Ok, S::Error> {
                let wrapped: Vec<BTreeMap<&str, &$ty>> =
                    items.iter().map(|i| BTreeMap::from([($key, i)])).collect();
                wrapped.serialize(s)
            }

            pub(super) fn deserialize<'de, D: Deserializer<'de>>(
                d: D,
            ) -> Result<Vec<$ty>, D::Error> {
                let wrapped: Vec<BTreeMap<String, $ty>> = Vec::deserialize(d)?;
                wrapped
                    .into_iter()
                    .map(|mut m| {
                        let item = m.remove($key).ok_or_else(|| {
                            D::Error::custom(concat!("expected {\"", $key, "\": …}"))
                        })?;
                        if m.is_empty() {
                            Ok(item)
                        } else {
                            Err(D::Error::custom(concat!(
                                "unexpected keys next to \"",
                                $key,
                                "\""
                            )))
                        }
                    })
                    .collect()
            }
        }

        #[allow(dead_code)]
        mod $opt {
            use super::*;

            pub(super) fn serialize<S: Serializer>(
                items: &Option<Vec<$ty>>,
                s: S,
            ) -> Result<S::Ok, S::Error> {
                match items {
                    Some(items) => super::$module::serialize(items, s),
                    None => s.serialize_none(),
                }
            }

            pub(super) fn deserialize<'de, D: Deserializer<'de>>(
                d: D,
            ) -> Result<Option<Vec<$ty>>, D::Error> {
                super::$module::deserialize(d).map(Some)
            }
        }
    };
}

wrapped_array!(memos, memos_opt, Memo, "Memo");
wrapped_array!(signers, signers_opt, Signer, "Signer");
wrapped_array!(
    signer_entries,
    signer_entries_opt,
    SignerEntry,
    "SignerEntry"
);
wrapped_array!(
    price_data_series,
    price_data_series_opt,
    PriceData,
    "PriceData"
);
wrapped_array!(auth_accounts, auth_accounts_opt, AuthAccount, "AuthAccount");
wrapped_array!(permissions, permissions_opt, Permission, "Permission");
wrapped_array!(credentials, credentials_opt, CredentialSpec, "Credential");
wrapped_array!(raw_transactions, raw_transactions_opt, Tx, "RawTransaction");
wrapped_array!(batch_signers, batch_signers_opt, BatchSigner, "BatchSigner");

impl Memo {
    /// An empty memo; set its parts with the `with_` methods.
    pub fn new() -> Self {
        Self::default()
    }

    /// A text memo: `text` as UTF-8 in `MemoData`, and `MemoFormat`
    /// `text/plain`, both hex-encoded as the ledger stores them.
    pub fn text(text: &str) -> Self {
        Self {
            memo_type: None,
            memo_data: Some(hex::encode_upper(text.as_bytes())),
            memo_format: Some(hex::encode_upper(b"text/plain")),
        }
    }

    /// With `MemoType` set, as hex.
    #[must_use]
    pub fn with_memo_type(mut self, hex: impl Into<String>) -> Self {
        self.memo_type = Some(hex.into());
        self
    }

    /// With `MemoData` set, as hex.
    #[must_use]
    pub fn with_memo_data(mut self, hex: impl Into<String>) -> Self {
        self.memo_data = Some(hex.into());
        self
    }

    /// With `MemoFormat` set, as hex.
    #[must_use]
    pub fn with_memo_format(mut self, hex: impl Into<String>) -> Self {
        self.memo_format = Some(hex.into());
        self
    }
}

impl Signer {
    /// A signer's entry: its account, public key and signature (hex). The
    /// signing functions make these; this is for one made elsewhere.
    pub fn new(
        account: impl Into<String>,
        signing_pub_key: impl Into<String>,
        txn_signature: impl Into<String>,
    ) -> Self {
        Self {
            account: account.into(),
            signing_pub_key: signing_pub_key.into(),
            txn_signature: txn_signature.into(),
        }
    }
}

impl SignerEntry {
    /// A member of a signer list with its weight.
    pub fn new(account: impl Into<String>, signer_weight: u16) -> Self {
        Self {
            account: account.into(),
            signer_weight,
            wallet_locator: None,
        }
    }

    /// With `WalletLocator` set (any 256-bit value the owner chooses).
    #[must_use]
    pub fn with_wallet_locator(mut self, locator: Hash256) -> Self {
        self.wallet_locator = Some(locator);
        self
    }
}

impl PriceData {
    /// A price oracle entry for the `base_asset`/`quote_asset` pair, with no
    /// price (which removes the pair from the oracle).
    pub fn new(base_asset: impl Into<String>, quote_asset: impl Into<String>) -> Self {
        Self {
            base_asset: base_asset.into(),
            quote_asset: quote_asset.into(),
            asset_price: None,
            scale: None,
        }
    }

    /// With `AssetPrice` (as `rippled` writes a UInt64: a hex string) and
    /// `Scale` set: the price is `asset_price × 10^-scale`.
    #[must_use]
    pub fn with_price(mut self, asset_price: impl Into<Value>, scale: u8) -> Self {
        self.asset_price = Some(asset_price.into());
        self.scale = Some(scale);
        self
    }
}

impl AuthAccount {
    /// An account allowed to trade at the AMM auction slot's discount.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            account: account.into(),
        }
    }
}

impl Permission {
    /// A permission to delegate (`DelegateSet`), by its name
    /// (`"Payment"`, `"TrustlineAuthorize"`, …).
    pub fn new(permission_value: impl Into<String>) -> Self {
        Self {
            permission_value: permission_value.into(),
        }
    }
}

impl CredentialSpec {
    /// A credential by its issuer and type (hex).
    pub fn new(issuer: impl Into<String>, credential_type: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            credential_type: credential_type.into(),
        }
    }
}

impl BatchSigner {
    /// A `BatchSigner` for `account` with its public key and no signature
    /// yet. The batch signing functions make complete ones.
    pub fn new(account: impl Into<String>, signing_pub_key: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            signing_pub_key: signing_pub_key.into(),
            txn_signature: None,
            signers: None,
        }
    }
}

impl XChainBridge {
    /// A cross-chain bridge: each chain's door account and the asset it
    /// holds.
    pub fn new(
        locking_chain_door: impl Into<String>,
        locking_chain_issue: Asset,
        issuing_chain_door: impl Into<String>,
        issuing_chain_issue: Asset,
    ) -> Self {
        Self {
            locking_chain_door: locking_chain_door.into(),
            locking_chain_issue,
            issuing_chain_door: issuing_chain_door.into(),
            issuing_chain_issue,
        }
    }
}

impl PathStep {
    /// A step through an account (rippling through its trust lines).
    pub fn account(account: impl Into<String>) -> Self {
        Self {
            account: Some(account.into()),
            ..Self::default()
        }
    }

    /// A step that converts into `currency` (issued by `issuer`, or XRP when
    /// `issuer` is `None`).
    pub fn currency(currency: impl Into<String>, issuer: Option<String>) -> Self {
        Self {
            currency: Some(currency.into()),
            issuer,
            ..Self::default()
        }
    }

    /// A step that converts into an MPT.
    pub fn mpt(mpt_issuance_id: impl Into<String>) -> Self {
        Self {
            mpt_issuance_id: Some(mpt_issuance_id.into()),
            ..Self::default()
        }
    }
}

macro_rules! tx_enum {
    ($($name:ident),*) => {
        /// A transaction, typed by its `TransactionType`.
        #[allow(clippy::upper_case_acronyms, clippy::large_enum_variant)]
        #[derive(Debug, Clone, PartialEq)]
        #[non_exhaustive]
        pub enum Tx {
            $(
                #[doc = concat!("`", stringify!($name), "`")]
                $name($name),
            )*
            /// A `TransactionType` this crate does not know: as JSON.
            Other(Value),
            /// A transaction of a type this crate knows that did not parse as
            /// that type (a field a newer `rippled` added, or a value of the
            /// wrong shape). Only lenient parsing (inside a larger value, such
            /// as a stream event or a response) makes one; [`Tx::from_value`]
            /// returns the error instead.
            Unparsed {
                /// The `TransactionType`.
                transaction_type: String,
                /// Why it did not parse.
                error: String,
                /// The transaction, as JSON.
                value: Value,
            },
        }

        impl Tx {
            /// [`TxModel::validate`] for the transaction's type (`Other` and
            /// `Unparsed` pass: there is nothing typed to check).
            ///
            /// # Errors
            ///
            /// As [`TxModel::validate`].
            pub fn validate(&self) -> Result<(), CoreError> {
                match self {
                    $(Tx::$name(t) => t.validate(),)*
                    _ => Ok(()),
                }
            }

            /// The `TransactionType`.
            pub fn transaction_type(&self) -> &str {
                match self {
                    $(Tx::$name(_) => stringify!($name),)*
                    Tx::Other(v) => v.get("TransactionType").and_then(Value::as_str).unwrap_or_default(),
                    Tx::Unparsed { transaction_type, .. } => transaction_type,
                }
            }

            /// The common fields, unless this is [`Tx::Other`] or [`Tx::Unparsed`].
            pub fn common(&self) -> Option<&Common> {
                match self {
                    $(Tx::$name(t) => Some(&t.common),)*
                    Tx::Other(_) | Tx::Unparsed { .. } => None,
                }
            }

            /// The common fields, to change.
            pub fn common_mut(&mut self) -> Option<&mut Common> {
                match self {
                    $(Tx::$name(t) => Some(&mut t.common),)*
                    Tx::Other(_) | Tx::Unparsed { .. } => None,
                }
            }

            /// Parse `rippled`'s JSON for a transaction. Strict: a known type
            /// with a field its struct does not have, or a field of the wrong
            /// shape, is an error, and so is a Batch whose inner transaction
            /// is one. An unknown type is [`Tx::Other`].
            ///
            /// # Errors
            ///
            /// [`CoreError::UnknownField`] for a field the type does not have;
            /// [`CoreError::CodecError`] for a field of the wrong shape or a
            /// required one missing, a Payment whose `Amount` and `DeliverMax`
            /// differ, or a Batch with an inner transaction that does not
            /// parse.
            pub fn from_value(mut v: Value) -> Result<Self, CoreError> {
                let ty = v.get("TransactionType").and_then(Value::as_str).map(str::to_owned);
                // API v2 writes a Payment's Amount only as DeliverMax; both,
                // differing, is refused (as rippled refuses it).
                if ty.as_deref() == Some("Payment") {
                    match (v.get("Amount"), v.get("DeliverMax").cloned()) {
                        (Some(amount), Some(dm)) if *amount != dm => {
                            return Err(CoreError::CodecError(
                                "Payment: Amount and DeliverMax differ".into(),
                            ))
                        }
                        (None, Some(dm)) => {
                            if let Some(obj) = v.as_object_mut() {
                                obj.insert("Amount".into(), dm);
                            }
                        }
                        _ => {}
                    }
                }
                let tx = match ty.as_deref() {
                    $(Some(stringify!($name)) => Tx::$name(
                        serde_json::from_value(v.clone()).map_err(|e| CoreError::CodecError(format!("{}: {e}", stringify!($name))))?,
                    ),)*
                    _ => return Ok(Tx::Other(v)),
                };
                if let Tx::Batch(batch) = &tx {
                    for inner in &batch.raw_transactions {
                        if let Tx::Unparsed { transaction_type, error, .. } = inner {
                            return Err(CoreError::CodecError(format!(
                                "Batch: inner {transaction_type}: {error}"
                            )));
                        }
                    }
                }
                let back = tx.to_value()?;
                let dropped = dropped_paths(&v, &back);
                if dropped.is_empty() {
                    Ok(tx)
                } else {
                    Err(CoreError::UnknownField(format!("{}: {}", tx.transaction_type(), dropped.join(", "))))
                }
            }

            /// `rippled`'s JSON for the transaction, `TransactionType` included.
            ///
            /// # Errors
            ///
            /// [`CoreError::CodecError`] if a field of a typed transaction does
            /// not serialize (see [`TxModel::to_json`]); `Other` and `Unparsed`
            /// never fail.
            pub fn to_value(&self) -> Result<Value, CoreError> {
                match self {
                    $(Tx::$name(t) => t.to_json(),)*
                    Tx::Other(v) | Tx::Unparsed { value: v, .. } => Ok(v.clone()),
                }
            }
        }
    };
}
each_transaction_type!(tx_enum);

/// Key paths of `original` that `back` does not have.
pub(crate) fn dropped_paths(original: &Value, back: &Value) -> Vec<String> {
    fn paths(v: &Value, prefix: &str, out: &mut BTreeSet<String>) {
        match v {
            Value::Object(m) => {
                for (k, x) in m {
                    let p = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{prefix}.{k}")
                    };
                    out.insert(p.clone());
                    paths(x, &p, out);
                }
            }
            Value::Array(a) => a.iter().for_each(|x| paths(x, &format!("{prefix}[]"), out)),
            _ => {}
        }
    }
    let (mut sent, mut kept) = (BTreeSet::new(), BTreeSet::new());
    paths(original, "", &mut sent);
    paths(back, "", &mut kept);
    sent.difference(&kept).cloned().collect()
}

/// Lenient: a transaction that does not fit its type becomes
/// [`Tx::Unparsed`] (carrying the error), so one odd transaction never fails
/// a whole response, and is never mistaken for a transaction of an unknown
/// type. [`Tx::from_value`] is strict.
impl<'de> Deserialize<'de> for Tx {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(deserializer)?;
        Ok(Tx::from_value(v.clone()).unwrap_or_else(|e| Tx::Unparsed {
            transaction_type: v
                .get("TransactionType")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_string(),
            error: e.to_string(),
            value: v,
        }))
    }
}

impl Serialize for Tx {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_value()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}
