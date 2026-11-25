//! Co-signing a [`LoanSet`](https://xrpl.org/docs/references/protocol/transactions/types/loanset)
//! (XLS-66, the `LendingProtocol` amendment): the second signature every new
//! loan needs.
//!
//! A loan is agreed between two accounts: the **borrower** and the **owner of
//! the loan broker** that lends from its vault. One of them sends the
//! `LoanSet` (its `Account`), and the other, the *counterparty*, approves it
//! with a `CounterpartySignature` inside the same transaction. The
//! counterparty is the transaction's `Counterparty` field when it is set, and
//! otherwise the owner of the broker named by `LoanBrokerID`. `rippled`
//! refuses a `LoanSet` without a `CounterpartySignature` (`temBAD_SIGNER`),
//! except inside a `Batch`, where the counterparty approves the whole batch
//! with a `BatchSigner` instead (see [`batch`](super::batch)): there the
//! `LoanSet` names its `Counterparty` and carries no `CounterpartySignature`
//! (`rippled` refuses one with a signature, `temINVALID_FLAG`).
//!
//! How a loan is put together, as `rippled` 3.4.1 checks it:
//!
//! 1. The sender builds the `LoanSet` and fills in **everything** —
//!    `Sequence` (or `TicketSequence`), `Fee`, `LastLedgerSequence` and its
//!    own `SigningPubKey` (`""` if it will multi-sign). The counterparty signs
//!    all of those fields, so none may change afterwards. The `Fee` must
//!    include one base fee per counterparty signature (`autofill` in
//!    `xrpl-client` adds them; [`cost`](super::cost) prices it offline).
//! 2. The counterparty signs it: [`sign_as_counterparty`] with a single key
//!    (its master key or regular key), or, for an account that signs through
//!    its signer list, each member signs with [`sign_for_counterparty`] and
//!    [`add_counterparty_signers`] combines their signatures.
//! 3. The sender signs the transaction as usual
//!    ([`crypto::sign_transaction`](crate::crypto::sign_transaction)) and
//!    submits it. Its own signature does not cover the
//!    `CounterpartySignature`, and the counterparty's does not cover the
//!    sender's `TxnSignature`, so steps 2 and 3 may come in either order.
//!
//! # Which bytes the counterparty signs
//!
//! The counterparty signs the same fields as the sender: the transaction
//! without its signature fields (`TxnSignature`, `Signers`,
//! `CounterpartySignature`, …). What differs is the four-byte prefix in front
//! of them, which depends on the
//! [`fixCleanup3_4_0`](https://xrpl.org/resources/known-amendments) amendment
//! ([`SigningRules`]):
//!
//! | | single signature | each multi-signature |
//! |---|---|---|
//! | with `fixCleanup3_4_0` ([`SigningRules::Current`]) | `"CPT\0"` | `"CPM\0"`, then the signer's account ID |
//! | before it ([`SigningRules::BeforeFixCleanup3_4_0`]) | `"STX\0"` | `"SMT\0"`, then the signer's account ID |
//!
//! Before the fix, a signature could be moved from one role to another; with
//! it, a counterparty signature is valid only as one. `fixCleanup3_4_0` is
//! enabled on Mainnet, Testnet and Devnet, so [`SigningRules::Current`] (the
//! default) is right for every public network. `Client::counterparty_signing_rules`
//! in `xrpl-client` asks a server which rules its network follows.
//!
//! ```
//! use serde_json::json;
//! use xrpl_core::crypto::{sign_transaction, verify_transaction, Keypair, Seed};
//! use xrpl_core::transaction::loan::{self, SigningRules};
//!
//! // Alice borrows from the broker Bob owns.
//! let alice = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
//! let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
//! let tx = json!({
//!     "TransactionType": "LoanSet",
//!     "Account": alice.classic_address(),
//!     "LoanBrokerID": "92230C14371FE613C3080DE0C78FDC76EF39078722D592DBFF89E6B59115F3B6",
//!     "PrincipalRequested": "1000000",
//!     "PaymentTotal": 12,
//!     "PaymentInterval": 2592000,
//!     "InterestRate": 500,
//!     "Sequence": 7,
//!     "Fee": "20", // 10 drops for the transaction, 10 for Bob's signature
//!     "LastLedgerSequence": 7155889,
//!     "SigningPubKey": hex::encode_upper(alice.public_key()),
//! });
//! // Bob approves the loan, then Alice signs and would submit it.
//! let approved = loan::sign_as_counterparty(&tx, &bob, SigningRules::Current)?;
//! assert!(loan::verify_counterparty_signature(&approved, SigningRules::Current)?);
//! let signed = sign_transaction(&approved, &alice)?;
//! assert!(verify_transaction(&signed)?); // Alice's signature and Bob's
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use super::models::Signer;
use crate::address::decode_account_id;
use crate::codec::encode::read_uint32;
use crate::codec::encode_for_signing;
use crate::crypto::signing_key::{checked_public_key, sign_with, sign_with_async};
use crate::crypto::{verify_signing_data, AsyncSigningKey, SigningKey, MAX_MULTISIGNERS};
use crate::types::Hash256;
use crate::CoreError;
use serde_json::{json, Map, Value};

/// `tfInnerBatchTxn`: marks a transaction inside a `Batch`.
const TF_INNER_BATCH_TXN: u32 = 0x4000_0000;

/// The name of the amendment that gave counterparty signatures their own
/// prefixes. Its ID ([`amendment_id`](crate::hashes::amendment_id)) is what
/// the ledger's `Amendments` entry lists once it is enabled.
///
/// ```
/// use xrpl_core::hashes::amendment_id;
/// use xrpl_core::transaction::loan::FIX_CLEANUP_3_4_0;
///
/// assert_eq!(FIX_CLEANUP_3_4_0, "fixCleanup3_4_0");
/// assert_eq!(amendment_id(FIX_CLEANUP_3_4_0).to_string(),
///     "98433DD001A5737F773D74F8CA2A25A065089C73B2E611C760BAF369E4FECA76");
/// ```
pub const FIX_CLEANUP_3_4_0: &str = "fixCleanup3_4_0";

/// Which prefix a counterparty signature covers: it depends on whether the
/// network has enabled the `fixCleanup3_4_0` amendment (`signingPrefix` in
/// `rippled` 3.4.1). See the [module documentation](self) for the bytes.
///
/// [`SigningRules::Current`], the default, is right for Mainnet, Testnet and
/// Devnet, which all have the amendment. A signature made under the wrong
/// rules does not verify, and `submit` refuses the transaction with
/// `invalidTransaction` ("fails local checks: Counterparty: Invalid
/// signature.").
///
/// ```
/// use xrpl_core::transaction::loan::SigningRules;
///
/// assert_eq!(SigningRules::default(), SigningRules::Current);
/// assert_eq!(SigningRules::from_fix_cleanup_3_4_0(true), SigningRules::Current);
/// assert_eq!(
///     SigningRules::from_fix_cleanup_3_4_0(false),
///     SigningRules::BeforeFixCleanup3_4_0,
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum SigningRules {
    /// `fixCleanup3_4_0` is enabled: the counterparty signs with its own
    /// prefixes, `"CPT\0"` (single) and `"CPM\0"` (multi).
    #[default]
    Current,
    /// `fixCleanup3_4_0` is not enabled: the counterparty signs the same
    /// bytes as the sender, `"STX\0"` (single) and `"SMT\0"` (multi).
    BeforeFixCleanup3_4_0,
}

impl SigningRules {
    /// The rules of a network on which `fixCleanup3_4_0` is (`true`) or is
    /// not (`false`) enabled.
    ///
    /// ```
    /// use xrpl_core::transaction::loan::SigningRules;
    ///
    /// // From a `feature` lookup of "fixCleanup3_4_0": {"enabled": true, …}
    /// let enabled = true;
    /// assert_eq!(SigningRules::from_fix_cleanup_3_4_0(enabled), SigningRules::Current);
    /// ```
    pub const fn from_fix_cleanup_3_4_0(enabled: bool) -> Self {
        if enabled {
            Self::Current
        } else {
            Self::BeforeFixCleanup3_4_0
        }
    }

    /// The rules of a network whose enabled amendments are `enabled`: the
    /// `Amendments` list of its ledger's `Amendments` entry
    /// ([`keylet::amendments`](crate::hashes::keylet::amendments)).
    ///
    /// ```
    /// use xrpl_core::hashes::amendment_id;
    /// use xrpl_core::transaction::loan::{SigningRules, FIX_CLEANUP_3_4_0};
    ///
    /// let enabled = [amendment_id("MultiSign"), amendment_id(FIX_CLEANUP_3_4_0)];
    /// assert_eq!(SigningRules::from_enabled_amendments(&enabled), SigningRules::Current);
    /// assert_eq!(
    ///     SigningRules::from_enabled_amendments(&enabled[..1]),
    ///     SigningRules::BeforeFixCleanup3_4_0,
    /// );
    /// ```
    pub fn from_enabled_amendments<'a>(enabled: impl IntoIterator<Item = &'a Hash256>) -> Self {
        let fix = crate::hashes::amendment_id(FIX_CLEANUP_3_4_0);
        Self::from_fix_cleanup_3_4_0(enabled.into_iter().any(|id| *id == fix))
    }

    /// The prefix in front of a single counterparty signature's data
    /// (`HashPrefix::CounterpartyTxSign`, or `TxSign` before the fix).
    ///
    /// ```
    /// use xrpl_core::transaction::loan::SigningRules;
    ///
    /// assert_eq!(SigningRules::Current.single_signing_prefix(), *b"CPT\0");
    /// assert_eq!(SigningRules::BeforeFixCleanup3_4_0.single_signing_prefix(), *b"STX\0");
    /// ```
    pub const fn single_signing_prefix(self) -> [u8; 4] {
        match self {
            Self::Current => *b"CPT\0",
            Self::BeforeFixCleanup3_4_0 => *b"STX\0",
        }
    }

    /// The prefix in front of each multi-signature's data
    /// (`HashPrefix::CounterpartyTxMultiSign`, or `TxMultiSign` before the
    /// fix).
    ///
    /// ```
    /// use xrpl_core::transaction::loan::SigningRules;
    ///
    /// assert_eq!(SigningRules::Current.multi_signing_prefix(), *b"CPM\0");
    /// assert_eq!(SigningRules::BeforeFixCleanup3_4_0.multi_signing_prefix(), *b"SMT\0");
    /// ```
    pub const fn multi_signing_prefix(self) -> [u8; 4] {
        match self {
            Self::Current => *b"CPM\0",
            Self::BeforeFixCleanup3_4_0 => *b"SMT\0",
        }
    }
}

/// The `LoanSet`'s signed fields (the transaction without its signature
/// fields), with no prefix and no checks beyond its type.
fn signed_fields(tx: &Value) -> Result<Vec<u8>, CoreError> {
    let kind = tx.get("TransactionType").and_then(Value::as_str);
    if kind != Some("LoanSet") {
        return Err(CoreError::ValidationError(format!(
            "only a LoanSet carries a CounterpartySignature, got {}",
            kind.unwrap_or("no TransactionType")
        )));
    }
    let data = encode_for_signing(tx)?;
    // encode_for_signing writes "STX\0" and then the fields.
    Ok(data.get(4..).unwrap_or_default().to_vec())
}

/// Refuse a `LoanSet` that is not ready to be co-signed: a field the
/// counterparty signs could still change, or it is a `Batch` inner
/// transaction.
fn check_ready(tx: &Value) -> Result<(), CoreError> {
    let invalid = |m: &str| Err(CoreError::ValidationError(format!("LoanSet: {m}")));
    let Some(obj) = tx.as_object() else {
        return invalid("a transaction is a JSON object");
    };
    let flags = match obj.get("Flags") {
        None | Some(Value::Null) => 0,
        Some(v) => read_uint32(v).ok_or_else(|| {
            CoreError::ValidationError(format!("Flags must be a 32-bit unsigned integer, got {v}"))
        })?,
    };
    if flags & TF_INNER_BATCH_TXN != 0 {
        return invalid(
            "inside a Batch the counterparty approves the whole batch with a BatchSigner \
             (batch::sign_batch), not a CounterpartySignature",
        );
    }
    if !obj.get("SigningPubKey").is_some_and(Value::is_string) {
        return invalid(
            "set SigningPubKey (the sender's public key, or \"\" if it will multi-sign) before \
             the counterparty signs: it is one of the fields the counterparty's signature covers",
        );
    }
    if obj.get("Fee").is_none_or(Value::is_null) {
        return invalid(
            "set the Fee (autofill) before the counterparty signs: it is one of the fields the \
             counterparty's signature covers",
        );
    }
    // Every transaction has a Sequence; one that uses a Ticket has Sequence 0
    // and its TicketSequence. Both are signed.
    if obj.get("Sequence").is_none_or(Value::is_null) {
        return invalid(
            "set the Sequence (autofill; 0 with a TicketSequence) before the counterparty \
             signs: it is one of the fields the counterparty's signature covers",
        );
    }
    Ok(())
}

/// What a single counterparty signature covers: the prefix of `rules`
/// (`"CPT\0"` under [`SigningRules::Current`]) and the transaction without
/// its signature fields. This is what to hand a remote signer that signs raw
/// XRPL signing data; [`sign_as_counterparty`] does it all with a local key.
///
/// The transaction must be ready: a `LoanSet`, not inside a `Batch`, with its
/// `SigningPubKey`, `Fee` and `Sequence` set (`Sequence` 0 with a
/// `TicketSequence` for a ticketed one), since the signature covers them.
///
/// ```
/// use serde_json::json;
/// use xrpl_core::codec::encode_for_signing;
/// use xrpl_core::transaction::loan::{counterparty_signing_data, SigningRules};
///
/// let tx = json!({"TransactionType": "LoanSet", "Account": "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD",
///     "LoanBrokerID": "92230C14371FE613C3080DE0C78FDC76EF39078722D592DBFF89E6B59115F3B6",
///     "PrincipalRequested": "1000000", "Sequence": 7, "Fee": "20",
///     "SigningPubKey": "ED5F5AC8B98974A3CA843326D9B88CEBD0560177B973EE0B149F782CFAA06DC66A"});
/// let data = counterparty_signing_data(&tx, SigningRules::Current)?;
/// assert_eq!(&data[..4], b"CPT\0");
/// // The same fields the sender signs; only the prefix differs.
/// assert_eq!(data[4..], encode_for_signing(&tx)?[4..]);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::ValidationError`] if `tx` is not a `LoanSet`, is a `Batch`
/// inner transaction, has `Flags` that are not a 32-bit unsigned integer, or
/// lacks its `SigningPubKey`, `Fee` or `Sequence`;
/// the codec's errors ([`CoreError::UnknownField`], [`CoreError::CodecError`],
/// …) for a transaction that does not encode.
pub fn counterparty_signing_data(tx: &Value, rules: SigningRules) -> Result<Vec<u8>, CoreError> {
    check_ready(tx)?;
    let fields = signed_fields(tx)?;
    let mut data = Vec::with_capacity(4 + fields.len());
    data.extend_from_slice(&rules.single_signing_prefix());
    data.extend_from_slice(&fields);
    Ok(data)
}

/// What one member of the counterparty's signer list signs: the multi-signing
/// prefix of `rules` (`"CPM\0"` under [`SigningRules::Current`]), the
/// transaction without its signature fields, then the member's (`signer`'s)
/// 20-byte account ID. For a remote signer; [`sign_for_counterparty`] does it
/// all with a local key.
///
/// ```
/// use serde_json::json;
/// use xrpl_core::transaction::loan::{counterparty_multisigning_data, SigningRules};
/// use xrpl_core::types::AccountId;
///
/// let tx = json!({"TransactionType": "LoanSet", "Account": "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD",
///     "LoanBrokerID": "92230C14371FE613C3080DE0C78FDC76EF39078722D592DBFF89E6B59115F3B6",
///     "PrincipalRequested": "1000000", "Sequence": 7, "Fee": "30",
///     "SigningPubKey": "ED5F5AC8B98974A3CA843326D9B88CEBD0560177B973EE0B149F782CFAA06DC66A"});
/// let member = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
/// let data = counterparty_multisigning_data(&tx, member, SigningRules::Current)?;
/// assert!(data.starts_with(b"CPM\0"));
/// assert!(data.ends_with(AccountId::from_address(member)?.as_bytes()));
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// As [`counterparty_signing_data`], and an address error
/// ([`CoreError::InvalidAddress`], `InvalidBase58` or `InvalidChecksum`) for a
/// `signer` that is not a classic address.
pub fn counterparty_multisigning_data(
    tx: &Value,
    signer: &str,
    rules: SigningRules,
) -> Result<Vec<u8>, CoreError> {
    check_ready(tx)?;
    let id = decode_account_id(signer)?;
    let fields = signed_fields(tx)?;
    let mut data = Vec::with_capacity(4 + fields.len() + 20);
    data.extend_from_slice(&rules.multi_signing_prefix());
    data.extend_from_slice(&fields);
    data.extend_from_slice(&id);
    Ok(data)
}

/// Approve a `LoanSet` as its counterparty with a single key (a
/// [`Keypair`](crate::crypto::Keypair) or any [`SigningKey`]: the
/// counterparty's master key, or its regular key): returns the transaction
/// with its `CounterpartySignature` set to `{SigningPubKey, TxnSignature}`;
/// `tx` is left as it was. A single signature already there is replaced.
///
/// The sender then signs the result as usual. For a remote key,
/// [`sign_as_counterparty_async`]; for a counterparty that signs through its
/// signer list, [`sign_for_counterparty`].
///
/// ```
/// use serde_json::json;
/// use xrpl_core::crypto::{Keypair, Seed};
/// use xrpl_core::transaction::loan::{self, SigningRules};
///
/// // Bob owns the broker Alice (rLUEXYu…) borrows from.
/// let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// let tx = json!({"TransactionType": "LoanSet", "Account": "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD",
///     "LoanBrokerID": "92230C14371FE613C3080DE0C78FDC76EF39078722D592DBFF89E6B59115F3B6",
///     "PrincipalRequested": "1000000", "Sequence": 7, "Fee": "20",
///     "SigningPubKey": "ED5F5AC8B98974A3CA843326D9B88CEBD0560177B973EE0B149F782CFAA06DC66A"});
/// let approved = loan::sign_as_counterparty(&tx, &bob, SigningRules::Current)?;
/// assert_eq!(approved["CounterpartySignature"]["SigningPubKey"], hex::encode_upper(bob.public_key()));
/// assert!(loan::verify_counterparty_signature(&approved, SigningRules::Current)?);
/// // Signed under one set of rules, it does not verify under the other.
/// assert!(!loan::verify_counterparty_signature(&approved, SigningRules::BeforeFixCleanup3_4_0)?);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// The errors of [`counterparty_signing_data`];
/// [`CoreError::ValidationError`] if the `LoanSet` already has a multi-signed
/// `CounterpartySignature`; and the key's own errors
/// ([`CoreError::SigningKeyFailed`] for a public key the XRP Ledger does not
/// take).
pub fn sign_as_counterparty<K: SigningKey + ?Sized>(
    tx: &Value,
    key: &K,
    rules: SigningRules,
) -> Result<Value, CoreError> {
    let data = counterparty_signing_data(tx, rules)?;
    let public_key = checked_public_key(SigningKey::public_key(key))?;
    let signature = sign_with(key, &data)?;
    with_single_signature(tx, public_key, &signature)
}

/// [`sign_as_counterparty`] with an [`AsyncSigningKey`] (a remote key: a
/// KMS, a signing service), whose signature is checked against its public
/// key before it is set.
///
/// ```
/// use serde_json::json;
/// use xrpl_core::transaction::loan::{self, SigningRules};
///
/// # async fn run() -> Result<(), xrpl_core::CoreError> {
/// # use xrpl_core::crypto::{Keypair, Seed};
/// // `bob` stands in for a remote key: every Keypair is an AsyncSigningKey too.
/// let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// let tx = json!({"TransactionType": "LoanSet", "Account": "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD",
///     "LoanBrokerID": "92230C14371FE613C3080DE0C78FDC76EF39078722D592DBFF89E6B59115F3B6",
///     "PrincipalRequested": "1000000", "Sequence": 7, "Fee": "20",
///     "SigningPubKey": "ED5F5AC8B98974A3CA843326D9B88CEBD0560177B973EE0B149F782CFAA06DC66A"});
/// let approved = loan::sign_as_counterparty_async(&tx, &bob, SigningRules::Current).await?;
/// assert!(loan::verify_counterparty_signature(&approved, SigningRules::Current)?);
/// # Ok(())
/// # }
/// # fn block_on<F: core::future::Future>(f: F) -> F::Output {
/// #     use core::task::{Context, Poll, Waker};
/// #     let mut f = core::pin::pin!(f);
/// #     loop {
/// #         if let Poll::Ready(v) = f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
/// #             return v;
/// #         }
/// #     }
/// # }
/// # block_on(run()).unwrap();
/// ```
///
/// # Errors
///
/// As [`sign_as_counterparty`], and [`CoreError::SigningKeyFailed`] for a
/// signature that does not verify.
pub async fn sign_as_counterparty_async<K: AsyncSigningKey + ?Sized>(
    tx: &Value,
    key: &K,
    rules: SigningRules,
) -> Result<Value, CoreError> {
    let data = counterparty_signing_data(tx, rules)?;
    let public_key = checked_public_key(AsyncSigningKey::public_key(key))?;
    let signature = sign_with_async(key, &data).await?;
    with_single_signature(tx, public_key, &signature)
}

/// `tx` with `CounterpartySignature` set to one key's signature.
fn with_single_signature(
    tx: &Value,
    public_key: &[u8],
    signature: &[u8],
) -> Result<Value, CoreError> {
    if counterparty_signature(tx).is_some_and(|cs| cs.contains_key("Signers")) {
        return Err(CoreError::ValidationError(
            "LoanSet: its CounterpartySignature is already multi-signed (Signers); a \
             counterparty signs once, either way"
                .into(),
        ));
    }
    let mut tx = tx.clone();
    let obj = tx
        .as_object_mut()
        .ok_or_else(|| CoreError::ValidationError("a transaction is a JSON object".into()))?;
    obj.insert(
        "CounterpartySignature".into(),
        json!({
            "SigningPubKey": hex::encode_upper(public_key),
            "TxnSignature": hex::encode_upper(signature),
        }),
    );
    Ok(tx)
}

fn counterparty_signature(tx: &Value) -> Option<&Map<String, Value>> {
    tx.get("CounterpartySignature").and_then(Value::as_object)
}

/// Sign a `LoanSet` as `signer`, a member of the counterparty's signer list,
/// with `key` (`signer`'s master key or regular key): one of the `Signers` of
/// the counterparty's multi-signature. Collect enough of them to meet the
/// list's quorum, then combine them with [`add_counterparty_signers`].
///
/// ```
/// use serde_json::json;
/// use xrpl_core::crypto::{Keypair, Seed};
/// use xrpl_core::transaction::loan::{self, SigningRules};
///
/// // The counterparty signs through its signer list; Carol and Dave are on it.
/// let carol = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// let dave = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// let mut tx = json!({"TransactionType": "LoanSet", "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "LoanBrokerID": "92230C14371FE613C3080DE0C78FDC76EF39078722D592DBFF89E6B59115F3B6",
///     "PrincipalRequested": "1000000", "Sequence": 7, "Fee": "30", // two counterparty signatures
///     "SigningPubKey": "ED5F5AC8B98974A3CA843326D9B88CEBD0560177B973EE0B149F782CFAA06DC66A"});
/// let rules = SigningRules::Current;
/// let signatures = vec![
///     loan::sign_for_counterparty(&tx, &carol.classic_address(), &carol, rules)?,
///     loan::sign_for_counterparty(&tx, &dave.classic_address(), &dave, rules)?,
/// ];
/// loan::add_counterparty_signers(&mut tx, signatures)?;
/// assert_eq!(tx["CounterpartySignature"]["Signers"].as_array().unwrap().len(), 2);
/// assert!(loan::verify_counterparty_signature(&tx, rules)?);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// The errors of [`counterparty_multisigning_data`], and the key's own
/// ([`CoreError::SigningKeyFailed`] for a public key the XRP Ledger does not
/// take).
pub fn sign_for_counterparty<K: SigningKey + ?Sized>(
    tx: &Value,
    signer: &str,
    key: &K,
    rules: SigningRules,
) -> Result<Signer, CoreError> {
    let data = counterparty_multisigning_data(tx, signer, rules)?;
    let public_key = checked_public_key(SigningKey::public_key(key))?;
    let signature = sign_with(key, &data)?;
    Ok(Signer::new(
        signer,
        hex::encode_upper(public_key),
        hex::encode_upper(signature),
    ))
}

/// [`sign_for_counterparty`] with an [`AsyncSigningKey`] (a remote key),
/// whose signature is checked against its public key.
///
/// ```
/// use serde_json::json;
/// use xrpl_core::transaction::loan::{self, SigningRules};
///
/// # async fn run() -> Result<(), xrpl_core::CoreError> {
/// # use xrpl_core::crypto::{Keypair, Seed};
/// // Carol, on the counterparty's signer list; a Keypair stands in for a remote key.
/// let carol = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// let mut tx = json!({"TransactionType": "LoanSet", "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "LoanBrokerID": "92230C14371FE613C3080DE0C78FDC76EF39078722D592DBFF89E6B59115F3B6",
///     "PrincipalRequested": "1000000", "Sequence": 7, "Fee": "20",
///     "SigningPubKey": "ED5F5AC8B98974A3CA843326D9B88CEBD0560177B973EE0B149F782CFAA06DC66A"});
/// let rules = SigningRules::Current;
/// let signature = loan::sign_for_counterparty_async(&tx, &carol.classic_address(), &carol, rules).await?;
/// loan::add_counterparty_signers(&mut tx, vec![signature])?;
/// assert!(loan::verify_counterparty_signature(&tx, rules)?);
/// # Ok(())
/// # }
/// # fn block_on<F: core::future::Future>(f: F) -> F::Output {
/// #     use core::task::{Context, Poll, Waker};
/// #     let mut f = core::pin::pin!(f);
/// #     loop {
/// #         if let Poll::Ready(v) = f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
/// #             return v;
/// #         }
/// #     }
/// # }
/// # block_on(run()).unwrap();
/// ```
///
/// # Errors
///
/// As [`sign_for_counterparty`], and [`CoreError::SigningKeyFailed`] for a
/// signature that does not verify.
pub async fn sign_for_counterparty_async<K: AsyncSigningKey + ?Sized>(
    tx: &Value,
    signer: &str,
    key: &K,
    rules: SigningRules,
) -> Result<Signer, CoreError> {
    let data = counterparty_multisigning_data(tx, signer, rules)?;
    let public_key = checked_public_key(AsyncSigningKey::public_key(key))?;
    let signature = sign_with_async(key, &data).await?;
    Ok(Signer::new(
        signer,
        hex::encode_upper(public_key),
        hex::encode_upper(signature),
    ))
}

/// Put `signers` ([`sign_for_counterparty`]) into the `LoanSet`'s
/// multi-signed `CounterpartySignature`, in ascending account-ID order (as
/// `rippled` requires; the order of addresses as text differs). Signers
/// already there are kept, so signatures can be gathered in several rounds;
/// each account may sign once, and there may be 1 to 32 in all.
///
/// The result has the shape `rippled` accepted on Devnet:
/// `{"Signers": [{"Signer": {Account, SigningPubKey, TxnSignature}}, …]}`,
/// with no `SigningPubKey` or `TxnSignature` of its own.
///
/// ```
/// use serde_json::json;
/// use xrpl_core::crypto::{Keypair, Seed};
/// use xrpl_core::transaction::loan::{self, SigningRules};
///
/// let carol = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// let mut tx = json!({"TransactionType": "LoanSet", "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
///     "LoanBrokerID": "92230C14371FE613C3080DE0C78FDC76EF39078722D592DBFF89E6B59115F3B6",
///     "PrincipalRequested": "1000000", "Sequence": 7, "Fee": "20",
///     "SigningPubKey": "ED5F5AC8B98974A3CA843326D9B88CEBD0560177B973EE0B149F782CFAA06DC66A"});
/// let signature = loan::sign_for_counterparty(&tx, &carol.classic_address(), &carol, SigningRules::Current)?;
/// loan::add_counterparty_signers(&mut tx, vec![signature.clone()])?;
/// assert_eq!(tx["CounterpartySignature"]["Signers"][0]["Signer"]["Account"], carol.classic_address());
/// assert!(loan::add_counterparty_signers(&mut tx, vec![signature]).is_err()); // Carol twice
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::ValidationError`] if `tx` is not a `LoanSet`, its
/// `CounterpartySignature` is single-signed or its `Signers` do not parse, an
/// account appears twice, or there would be no signers or more than 32; an
/// address error ([`CoreError::InvalidAddress`], `InvalidBase58` or
/// `InvalidChecksum`) for a signer's account that is not a classic address.
pub fn add_counterparty_signers(tx: &mut Value, signers: Vec<Signer>) -> Result<(), CoreError> {
    let invalid = |m: String| CoreError::ValidationError(format!("LoanSet: {m}"));
    if tx.get("TransactionType").and_then(Value::as_str) != Some("LoanSet") {
        return Err(CoreError::ValidationError(
            "only a LoanSet carries a CounterpartySignature".into(),
        ));
    }
    let mut all: Vec<Signer> = Vec::new();
    if let Some(existing) = counterparty_signature(tx) {
        if existing.contains_key("TxnSignature")
            || existing
                .get("SigningPubKey")
                .and_then(Value::as_str)
                .is_some_and(|k| !k.is_empty())
        {
            return Err(invalid(
                "its CounterpartySignature is single-signed; a counterparty signs once, either way"
                    .into(),
            ));
        }
        for entry in existing
            .get("Signers")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let signer = entry.get("Signer").cloned().unwrap_or(Value::Null);
            all.push(
                serde_json::from_value(signer)
                    .map_err(|e| invalid(format!("CounterpartySignature.Signers: {e}")))?,
            );
        }
    }
    all.extend(signers);
    if all.is_empty() || all.len() > MAX_MULTISIGNERS {
        return Err(invalid(format!(
            "a multi-signed CounterpartySignature has 1 to {MAX_MULTISIGNERS} signers, got {}",
            all.len()
        )));
    }
    let mut keyed = all
        .into_iter()
        .map(|s| Ok((decode_account_id(&s.account)?, s)))
        .collect::<Result<Vec<_>, CoreError>>()?;
    keyed.sort_by_key(|(id, _)| *id);
    if keyed
        .windows(2)
        .any(|w| matches!(w, [(x, _), (y, _)] if x == y))
    {
        return Err(invalid(
            "an account signs a CounterpartySignature once".into(),
        ));
    }
    let wrapped = keyed
        .into_iter()
        .map(|(_, s)| serde_json::to_value(s).map(|v| json!({ "Signer": v })))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CoreError::CodecError(e.to_string()))?;
    tx.as_object_mut()
        .ok_or_else(|| CoreError::ValidationError("a transaction is a JSON object".into()))?
        .insert(
            "CounterpartySignature".into(),
            json!({ "Signers": wrapped }),
        );
    Ok(())
}

/// Whether the `LoanSet`'s `CounterpartySignature` is one `rippled` accepts
/// under `rules`: a single signature (`SigningPubKey` and `TxnSignature`, no
/// `Signers`) over [`counterparty_signing_data`], or 1 to 32 multi-signatures
/// (no `TxnSignature`, an empty or absent `SigningPubKey`) in strictly
/// ascending account-ID order, each over [`counterparty_multisigning_data`].
/// A `LoanSet` with no `CounterpartySignature` is `Ok(false)`.
///
/// `rippled` also checks the key against the counterparty account (its
/// master key, regular key or signer list, and the list's quorum), which
/// needs the ledger, and refuses any `CounterpartySignature` on a `Batch`
/// inner `LoanSet`; this checks the signature only.
///
/// ```
/// use serde_json::json;
/// use xrpl_core::crypto::{Keypair, Seed};
/// use xrpl_core::transaction::loan::{self, SigningRules};
///
/// let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// let tx = json!({"TransactionType": "LoanSet", "Account": "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD",
///     "LoanBrokerID": "92230C14371FE613C3080DE0C78FDC76EF39078722D592DBFF89E6B59115F3B6",
///     "PrincipalRequested": "1000000", "Sequence": 7, "Fee": "20",
///     "SigningPubKey": "ED5F5AC8B98974A3CA843326D9B88CEBD0560177B973EE0B149F782CFAA06DC66A"});
/// let rules = SigningRules::Current;
/// assert!(!loan::verify_counterparty_signature(&tx, rules)?); // not signed yet
/// let mut approved = loan::sign_as_counterparty(&tx, &bob, rules)?;
/// assert!(loan::verify_counterparty_signature(&approved, rules)?);
/// approved["PrincipalRequested"] = json!("2000000"); // changed after Bob signed
/// assert!(!loan::verify_counterparty_signature(&approved, rules)?);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::ValidationError`] if `tx` is not a `LoanSet`, or its
/// `CounterpartySignature` is not an object or has fields of the wrong type;
/// the codec's errors for a transaction that does not encode; an address
/// error for a signer's account; [`CoreError::InvalidHex`] for a key or
/// signature that is not hex; [`CoreError::InvalidKeyLength`] or
/// [`CoreError::SigningFailed`] for a malformed key (or secp256k1
/// signature). A signature that does not match is `Ok(false)`, not an error.
pub fn verify_counterparty_signature(tx: &Value, rules: SigningRules) -> Result<bool, CoreError> {
    let fields = signed_fields(tx)?;
    let invalid = |m: &str| CoreError::ValidationError(format!("CounterpartySignature: {m}"));
    let cs = match tx.get("CounterpartySignature") {
        None | Some(Value::Null) => return Ok(false),
        Some(v) => v.as_object().ok_or_else(|| invalid("must be an object"))?,
    };
    let text = |v: Option<&Value>, name: &str| -> Result<Option<String>, CoreError> {
        match v {
            None => Ok(None),
            Some(Value::String(s)) => Ok(Some(s.clone())),
            Some(_) => Err(invalid(&format!("{name} must be a hex string"))),
        }
    };
    let public_key = text(cs.get("SigningPubKey"), "SigningPubKey")?.unwrap_or_default();
    let signature = text(cs.get("TxnSignature"), "TxnSignature")?;
    let signers = match cs.get("Signers") {
        None => None,
        Some(v) => Some(
            v.as_array()
                .ok_or_else(|| invalid("Signers must be an array"))?,
        ),
    };
    if !public_key.is_empty() {
        // Single-signed: a signature and no Signers.
        let (Some(signature), None) = (signature, signers) else {
            return Ok(false);
        };
        let mut data = rules.single_signing_prefix().to_vec();
        data.extend_from_slice(&fields);
        return verify(&public_key, &data, &signature);
    }
    // Multi-signed: Signers and no signature of its own.
    let Some(signers) = signers.filter(|s| signature.is_none() && !s.is_empty()) else {
        return Ok(false);
    };
    if signers.len() > MAX_MULTISIGNERS {
        return Ok(false);
    }
    let mut start = rules.multi_signing_prefix().to_vec();
    start.extend_from_slice(&fields);
    let mut previous: Option<[u8; 20]> = None;
    let mut all = true;
    for entry in signers {
        let signer = entry.get("Signer").unwrap_or(entry);
        let field = |name: &str| {
            signer
                .get(name)
                .and_then(Value::as_str)
                .ok_or_else(|| invalid(&format!("a signer's {name} is missing or not a string")))
        };
        let id = decode_account_id(field("Account")?)?;
        if previous.is_some_and(|p| p >= id) {
            return Ok(false);
        }
        previous = Some(id);
        let mut data = start.clone();
        data.extend_from_slice(&id);
        all &= verify(field("SigningPubKey")?, &data, field("TxnSignature")?)?;
    }
    Ok(all)
}

/// A signature over XRPL signing data, as `rippled` verifies it.
fn verify(public_key_hex: &str, data: &[u8], signature_hex: &str) -> Result<bool, CoreError> {
    let key = hex::decode(public_key_hex).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
    let sig = hex::decode(signature_hex).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
    if key.is_empty() {
        return Ok(false);
    }
    verify_signing_data(&key, data, &sig)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::crypto::{sign_for_multisigning, sign_transaction, verify_transaction, Keypair};

    fn keys(seed: &str) -> Keypair {
        Keypair::from_seed(&crate::crypto::Seed::from_base58(seed).unwrap()).unwrap()
    }

    fn fixture() -> Vec<Value> {
        let path = format!(
            "{}/tests/fixtures/loan_set_devnet.jsonl",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::read_to_string(path)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    /// Every `LoanSet` in the Devnet sample is one the network applied
    /// (`tesSUCCESS` or a `tec` result, both past the signature check), on a
    /// network with `fixCleanup3_4_0`: each counterparty signature must verify
    /// under `Current` and fail under the old rules, and any change to a
    /// signed field must break it. A wrong byte or prefix fails them all.
    #[test]
    fn counterparty_signatures_rippled_accepted_verify() {
        let (mut single, mut multi) = (0, 0);
        for record in fixture() {
            let tx = &record["tx"];
            let hash = &record["hash"];
            assert!(
                verify_counterparty_signature(tx, SigningRules::Current).unwrap(),
                "{hash}"
            );
            assert!(
                !verify_counterparty_signature(tx, SigningRules::BeforeFixCleanup3_4_0).unwrap(),
                "{hash}"
            );
            // The sender's own signature, and both together.
            assert!(verify_transaction(tx).unwrap(), "{hash}");
            let mut tampered = tx.clone();
            let seq = tx["Sequence"].as_u64().unwrap();
            tampered["Sequence"] = Value::from(seq + 1);
            assert!(!verify_counterparty_signature(&tampered, SigningRules::Current).unwrap());
            // The transaction ID is the one the ledger recorded.
            assert_eq!(
                crate::hashes::hash_signed_tx(tx).unwrap().to_string(),
                hash.as_str().unwrap(),
            );
            if tx["CounterpartySignature"].get("Signers").is_some() {
                multi += 1;
            } else {
                single += 1;
            }
        }
        assert!(single >= 50, "{single} single-signed in the sample");
        assert!(multi >= 1, "no multi-signed counterparty in the sample");
    }

    fn loan_from(account: &str, signing_pub_key: &str) -> Value {
        serde_json::json!({
            "TransactionType": "LoanSet", "Account": account,
            "LoanBrokerID": "92230C14371FE613C3080DE0C78FDC76EF39078722D592DBFF89E6B59115F3B6",
            "PrincipalRequested": "1000000", "PaymentTotal": 12, "PaymentInterval": 2592000,
            "InterestRate": 500, "Sequence": 7, "Fee": "30", "LastLedgerSequence": 7155889,
            "SigningPubKey": signing_pub_key,
        })
    }

    #[test]
    fn single_round_trip_under_both_rules_and_either_order() {
        let (alice, bob) = (
            keys("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r"),
            keys("sp5fghtJtpUorTwvof1NpDXAzNwf5"),
        );
        let tx = loan_from(
            &alice.classic_address(),
            &hex::encode_upper(alice.public_key()),
        );
        for rules in [SigningRules::Current, SigningRules::BeforeFixCleanup3_4_0] {
            let approved = sign_as_counterparty(&tx, &bob, rules).unwrap();
            assert!(verify_counterparty_signature(&approved, rules).unwrap());
            // The sender signing after the counterparty leaves it valid ...
            let signed = sign_transaction(&approved, &alice).unwrap();
            assert!(verify_counterparty_signature(&signed, rules).unwrap());
            // ... and so does the counterparty signing after the sender.
            let signed_first = sign_transaction(&tx, &alice).unwrap();
            let approved_after = sign_as_counterparty(&signed_first, &bob, rules).unwrap();
            assert_eq!(
                approved_after["CounterpartySignature"],
                approved["CounterpartySignature"]
            );
            assert!(
                verify_transaction(&approved_after).unwrap() == (rules == SigningRules::Current)
            );
        }
        // Under the old rules the counterparty signs exactly the bytes the
        // sender signs (a signature could move between roles: what the fix
        // ended); under the current ones only the prefix differs.
        assert_eq!(
            counterparty_signing_data(&tx, SigningRules::BeforeFixCleanup3_4_0).unwrap(),
            encode_for_signing(&tx).unwrap()
        );
        let current = counterparty_signing_data(&tx, SigningRules::Current).unwrap();
        assert_eq!(&current[..4], b"CPT\0");
        assert_eq!(current[4..], encode_for_signing(&tx).unwrap()[4..]);
    }

    #[test]
    fn multi_signatures_sort_merge_and_verify() {
        let (alice, bob, carol, dave) = (
            keys("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r"),
            keys("sp5fghtJtpUorTwvof1NpDXAzNwf5"),
            keys("sn3nxiW7v8KXzPzAqzyHXbSSKNuN9"),
            keys("snoPBrXtMeMyMHUVTgbuqAfg1SUTb"),
        );
        let tx = loan_from(
            &alice.classic_address(),
            &hex::encode_upper(alice.public_key()),
        );
        let rules = SigningRules::Current;
        let sig = |k: &Keypair, t: &Value| {
            sign_for_counterparty(t, &k.classic_address(), k, rules).unwrap()
        };
        // Two rounds: Dave first, then Bob and Carol.
        let mut multi = tx.clone();
        add_counterparty_signers(&mut multi, vec![sig(&dave, &tx)]).unwrap();
        add_counterparty_signers(&mut multi, vec![sig(&carol, &tx), sig(&bob, &tx)]).unwrap();
        let ids: Vec<[u8; 20]> = multi["CounterpartySignature"]["Signers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| decode_account_id(s["Signer"]["Account"].as_str().unwrap()).unwrap())
            .collect();
        assert_eq!(ids.len(), 3);
        assert!(
            ids.windows(2).all(|w| w[0] < w[1]),
            "not in account-ID order"
        );
        assert!(multi["CounterpartySignature"]
            .get("SigningPubKey")
            .is_none());
        assert!(verify_counterparty_signature(&multi, rules).unwrap());
        assert!(
            !verify_counterparty_signature(&multi, SigningRules::BeforeFixCleanup3_4_0).unwrap()
        );
        // The sender signs last; everything still verifies.
        let signed = sign_transaction(&multi, &alice).unwrap();
        assert!(verify_transaction(&signed).unwrap());
        // A counterparty multi-signature is not an ordinary one: the
        // ordinary multi-signing data (SMT\0) differs under Current rules.
        let ordinary = sign_for_multisigning(
            &{
                let mut t = tx.clone();
                t["SigningPubKey"] = Value::from("");
                t
            },
            &bob.classic_address(),
            &bob,
        )
        .unwrap();
        assert_ne!(ordinary.txn_signature, sig(&bob, &tx).txn_signature);
        // Out of order, a duplicate, both kinds, or none: refused.
        let mut swapped = multi.clone();
        swapped["CounterpartySignature"]["Signers"]
            .as_array_mut()
            .unwrap()
            .swap(0, 1);
        assert!(!verify_counterparty_signature(&swapped, rules).unwrap());
        assert!(add_counterparty_signers(&mut multi.clone(), vec![sig(&bob, &tx)]).is_err());
        let mut both = multi.clone();
        both["CounterpartySignature"]["TxnSignature"] = Value::from("AB");
        assert!(!verify_counterparty_signature(&both, rules).unwrap());
        assert!(add_counterparty_signers(&mut tx.clone(), vec![]).is_err());
        // A single signature cannot be turned into a multi-signature, nor back.
        let single = sign_as_counterparty(&tx, &bob, rules).unwrap();
        assert!(add_counterparty_signers(&mut single.clone(), vec![sig(&carol, &tx)]).is_err());
        assert!(sign_as_counterparty(&multi, &bob, rules).is_err());
        // 33 signers is one too many.
        let many: Vec<Signer> = (0..=MAX_MULTISIGNERS)
            .map(|_| {
                let k = Keypair::from_seed(&crate::crypto::Seed::generate()).unwrap();
                sign_for_counterparty(&tx, &k.classic_address(), &k, rules).unwrap()
            })
            .collect();
        assert!(add_counterparty_signers(&mut tx.clone(), many).is_err());
    }

    /// The counterparty signs the sender's SigningPubKey, Fee and Sequence:
    /// signing before they are final is refused, as is a Batch inner LoanSet
    /// or another transaction type.
    #[test]
    fn refuses_what_could_still_change() {
        let bob = keys("sp5fghtJtpUorTwvof1NpDXAzNwf5");
        let tx = loan_from("rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD", "");
        assert!(sign_as_counterparty(&tx, &bob, SigningRules::Current).is_ok()); // "" is set
        for missing in ["SigningPubKey", "Fee", "Sequence"] {
            let mut t = tx.clone();
            t.as_object_mut().unwrap().remove(missing);
            let e = sign_as_counterparty(&t, &bob, SigningRules::Current).unwrap_err();
            assert!(e.to_string().contains(missing), "{e}");
        }
        let mut ticketed = tx.clone();
        ticketed["Sequence"] = Value::from(0);
        ticketed["TicketSequence"] = Value::from(9);
        assert!(sign_as_counterparty(&ticketed, &bob, SigningRules::Current).is_ok());
        // rippled requires Sequence (0) beside a TicketSequence, and adding it
        // later would break the signature.
        ticketed.as_object_mut().unwrap().remove("Sequence");
        assert!(sign_as_counterparty(&ticketed, &bob, SigningRules::Current).is_err());
        let mut null_fee = tx.clone();
        null_fee["Fee"] = Value::Null;
        assert!(sign_as_counterparty(&null_fee, &bob, SigningRules::Current).is_err());
        let mut inner = tx.clone();
        inner["Flags"] = Value::from(TF_INNER_BATCH_TXN);
        assert!(sign_as_counterparty(&inner, &bob, SigningRules::Current)
            .unwrap_err()
            .to_string()
            .contains("BatchSigner"));
        let mut payment = tx.clone();
        payment["TransactionType"] = Value::from("Payment");
        assert!(sign_as_counterparty(&payment, &bob, SigningRules::Current).is_err());
        assert!(verify_counterparty_signature(&payment, SigningRules::Current).is_err());
    }

    /// Re-signing replaces a single signature; a malformed object is an error
    /// and a missing one is simply not signed.
    #[test]
    fn verification_reads_shapes_as_rippled_does() {
        let (bob, carol) = (
            keys("sp5fghtJtpUorTwvof1NpDXAzNwf5"),
            keys("sn3nxiW7v8KXzPzAqzyHXbSSKNuN9"),
        );
        let tx = loan_from("rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD", "");
        let rules = SigningRules::Current;
        let by_bob = sign_as_counterparty(&tx, &bob, rules).unwrap();
        let by_carol = sign_as_counterparty(&by_bob, &carol, rules).unwrap();
        assert_eq!(
            by_carol["CounterpartySignature"]["SigningPubKey"],
            hex::encode_upper(carol.public_key())
        );
        assert!(verify_counterparty_signature(&by_carol, rules).unwrap());
        let mut not_object = tx.clone();
        not_object["CounterpartySignature"] = Value::from("AB");
        assert!(verify_counterparty_signature(&not_object, rules).is_err());
        // A single signature with Signers beside it: rippled's "Cannot both
        // single- and multi-sign".
        let mut both = by_bob.clone();
        both["CounterpartySignature"]["Signers"] = serde_json::json!([]);
        assert!(!verify_counterparty_signature(&both, rules).unwrap());
        // A multi-signature whose SigningPubKey is written as "" is the same.
        let mut multi = tx.clone();
        add_counterparty_signers(
            &mut multi,
            vec![sign_for_counterparty(&tx, &bob.classic_address(), &bob, rules).unwrap()],
        )
        .unwrap();
        multi["CounterpartySignature"]["SigningPubKey"] = Value::from("");
        assert!(verify_counterparty_signature(&multi, rules).unwrap());
    }
}
