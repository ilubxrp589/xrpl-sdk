//! [`Batch`](https://xrpl.org/docs/references/protocol/transactions/types/batch)
//! (XLS-56): up to eight transactions, from one or several accounts, applied
//! together — all or nothing (`TF_ALL_OR_NOTHING`), only the first that
//! succeeds (`TF_ONLY_ONE`), until one fails (`TF_UNTIL_FAILURE`), or each on
//! its own (`TF_INDEPENDENT`).
//!
//! How a batch is put together, as `rippled` 3.4.1 checks it:
//!
//! 1. Each inner transaction is marked with `tfInnerBatchTxn`, has a `Fee` of
//!    `"0"`, an empty `SigningPubKey`, no signature, and either a `Sequence`
//!    or a `TicketSequence` ([`inner_transaction`] does the marking).
//! 2. The outer `Batch` carries them in `RawTransactions` (2 to 8) and pays
//!    for all of them ([`batch_fee`]).
//! 3. Every account other than the outer one that has an inner transaction
//!    signs the batch ([`sign_batch`], or [`sign_batch_for`] for a
//!    multi-signed account); [`add_batch_signers`] puts the signatures in the
//!    order `rippled` requires.
//! 4. The outer account signs the whole `Batch` like any transaction, and
//!    submits it.
//!
//! ```
//! use serde_json::json;
//! use xrpl_core::crypto::{Keypair, Seed};
//! use xrpl_core::transaction::batch::{self, Batch};
//! use xrpl_core::transaction::models::BatchFlags;
//!
//! let alice = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
//! let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
//! let (a, b) = (alice.classic_address(), bob.classic_address());
//!
//! // Alice pays Bob 5 XRP and Bob pays Alice 1 XRP, all or nothing.
//! let mut tx = json!({
//!     "TransactionType": "Batch",
//!     "Account": a,
//!     "Sequence": 10,
//!     "Flags": u32::from(BatchFlags::ALL_OR_NOTHING),
//!     "Fee": batch::batch_fee(10, 2, 1).to_string(),
//!     "RawTransactions": [
//!         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "5000000", "Sequence": 11}))?},
//!         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": b, "Destination": a, "Amount": "1000000", "Sequence": 3}))?},
//!     ],
//! });
//! let bob_signs = batch::sign_batch(&tx, &bob)?;
//! batch::add_batch_signers(&mut tx, vec![bob_signs])?;
//! assert!(batch::verify_batch_signers(&tx)?);
//! // Alice then signs `tx` (crypto::sign_transaction) and submits it.
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub use super::models::Batch;
use super::models::{BatchSigner, Signer};

/// `tfInnerBatchTxn` (`UniversalFlags::INNER_BATCH_TXN`): marks a transaction
/// inside a `Batch`.
const TF_INNER_BATCH_TXN: u32 = 0x4000_0000;
use crate::address::decode_account_id;
use crate::codec::encode::read_uint32;
use crate::crypto::signing::public_key_to_account_id;
use crate::crypto::{verify_signing_data, AsyncSigningKey, SigningKey};
use crate::hashes::hash_signed_tx;
use crate::types::{AccountId, Hash256};
use crate::CoreError;
use serde_json::Value;

/// `HashPrefix::Batch` ("BCH\0").
const PREFIX_BATCH: [u8; 4] = *b"BCH\0";

/// The most inner transactions a batch may hold (`kMaxBatchTxCount`).
///
/// ```
/// use xrpl_core::transaction::batch::{MAX_INNER_TRANSACTIONS, MIN_INNER_TRANSACTIONS};
///
/// let inner = 3; // the transactions you want to batch
/// assert!((MIN_INNER_TRANSACTIONS..=MAX_INNER_TRANSACTIONS).contains(&inner));
/// assert_eq!(MAX_INNER_TRANSACTIONS, 8);
/// ```
pub const MAX_INNER_TRANSACTIONS: usize = 8;

/// The fewest inner transactions a batch may hold.
///
/// ```
/// use xrpl_core::transaction::batch::{MAX_INNER_TRANSACTIONS, MIN_INNER_TRANSACTIONS};
///
/// let inner = 1; // one transaction is not a batch
/// assert!(!(MIN_INNER_TRANSACTIONS..=MAX_INNER_TRANSACTIONS).contains(&inner));
/// assert_eq!(MIN_INNER_TRANSACTIONS, 2);
/// ```
pub const MIN_INNER_TRANSACTIONS: usize = 2;

/// The most signers a multi-signed `BatchSigner` may hold (`kMaxMultiSigners`).
///
/// ```
/// use xrpl_core::transaction::batch::{multisigned_batch_signer, MAX_NESTED_SIGNERS};
///
/// assert_eq!(MAX_NESTED_SIGNERS, 32);
/// // From 1 to MAX_NESTED_SIGNERS signers: none is refused.
/// assert!(multisigned_batch_signer("rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1", vec![]).is_err());
/// ```
pub const MAX_NESTED_SIGNERS: usize = 32;

/// Make `tx` an inner transaction: set `tfInnerBatchTxn`, `Fee` to `"0"` and
/// `SigningPubKey` to `""`, and drop any signature. Its `Sequence` (or
/// `TicketSequence`, with `Sequence` 0) must be set by the caller: the outer
/// `Batch` uses its account's next sequence, so an inner transaction from the
/// same account uses the one after.
///
/// ```
/// use serde_json::json;
/// use xrpl_core::transaction::batch::inner_transaction;
///
/// let payment = json!({"TransactionType": "Payment", "Account": "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD",
///     "Destination": "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1", "Amount": "5000000", "Sequence": 11, "Fee": "12"});
/// let inner = inner_transaction(&payment)?;
/// assert_eq!(inner["Flags"], 0x4000_0000); // tfInnerBatchTxn
/// assert_eq!(inner["Fee"], "0");
/// assert_eq!(inner["SigningPubKey"], "");
/// assert_eq!(inner["Sequence"], 11); // kept
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::ValidationError`] if `tx` is not a JSON object, or its `Flags`
/// is not a 32-bit unsigned integer.
pub fn inner_transaction(tx: &Value) -> Result<Value, CoreError> {
    let mut tx = tx.clone();
    let obj = tx
        .as_object_mut()
        .ok_or_else(|| CoreError::ValidationError("a transaction is a JSON object".into()))?;
    let flags = match obj.get("Flags") {
        None | Some(Value::Null) => 0,
        Some(v) => read_uint32(v).ok_or_else(|| {
            CoreError::ValidationError(format!("Flags must be a 32-bit unsigned integer, got {v}"))
        })?,
    };
    obj.insert("Flags".into(), Value::from(flags | TF_INNER_BATCH_TXN));
    obj.insert("Fee".into(), Value::String("0".into()));
    obj.insert("SigningPubKey".into(), Value::String(String::new()));
    obj.remove("TxnSignature");
    obj.remove("Signers");
    Ok(tx)
}

/// The inner transactions of a batch, in order.
///
/// ```
/// use xrpl_core::transaction::batch;
///
/// # use serde_json::json;
/// # use xrpl_core::crypto::{Keypair, Seed};
/// # use xrpl_core::transaction::models::BatchFlags;
/// # let alice = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// # let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// # let (a, b) = (alice.classic_address(), bob.classic_address());
/// # let mut tx = json!({
/// #     "TransactionType": "Batch", "Account": a, "Sequence": 10,
/// #     "Flags": u32::from(BatchFlags::ALL_OR_NOTHING), "Fee": batch::batch_fee(10, 2, 1).to_string(),
/// #     "RawTransactions": [
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "5000000", "Sequence": 11}))?},
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": b, "Destination": a, "Amount": "1000000", "Sequence": 3}))?},
/// #     ],
/// # });
/// // `tx`: the Batch of the module example: Alice (`a`) pays Bob (`b`, keys
/// // `bob`) and Bob pays Alice, all or nothing.
/// let inner = batch::inner_transactions(&tx)?;
/// assert_eq!(inner.len(), 2);
/// assert_eq!(inner[1]["Account"], b);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::ValidationError`] if `batch` has no `RawTransactions` array, or
/// an entry is not `{"RawTransaction": …}`.
pub fn inner_transactions(batch: &Value) -> Result<Vec<&Value>, CoreError> {
    let raw = batch
        .get("RawTransactions")
        .and_then(Value::as_array)
        .ok_or_else(|| CoreError::ValidationError("a Batch needs RawTransactions".into()))?;
    raw.iter()
        .map(|r| {
            r.get("RawTransaction").ok_or_else(|| {
                CoreError::ValidationError("expected {\"RawTransaction\": …}".into())
            })
        })
        .collect()
}

/// The IDs of the inner transactions, in order: what the batch signatures
/// cover, and the hashes the inner transactions get on the ledger.
///
/// ```
/// use xrpl_core::hashes::hash_signed_tx;
/// use xrpl_core::transaction::batch;
///
/// # use serde_json::json;
/// # use xrpl_core::crypto::{Keypair, Seed};
/// # use xrpl_core::transaction::models::BatchFlags;
/// # let alice = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// # let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// # let (a, b) = (alice.classic_address(), bob.classic_address());
/// # let mut tx = json!({
/// #     "TransactionType": "Batch", "Account": a, "Sequence": 10,
/// #     "Flags": u32::from(BatchFlags::ALL_OR_NOTHING), "Fee": batch::batch_fee(10, 2, 1).to_string(),
/// #     "RawTransactions": [
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "5000000", "Sequence": 11}))?},
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": b, "Destination": a, "Amount": "1000000", "Sequence": 3}))?},
/// #     ],
/// # });
/// // `tx`: the Batch of the module example: Alice (`a`) pays Bob (`b`, keys
/// // `bob`) and Bob pays Alice, all or nothing.
/// let ids = batch::inner_transaction_ids(&tx)?;
/// let inner = batch::inner_transactions(&tx)?;
/// assert_eq!(ids.len(), 2);
/// assert_eq!(ids[0], hash_signed_tx(inner[0])?);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// The errors of [`inner_transactions`]; [`CoreError::ValidationError`] for an
/// inner transaction that is neither signed nor marked `tfInnerBatchTxn`, and a
/// codec error ([`CoreError::UnknownField`], [`CoreError::CodecError`], …) for
/// one that does not encode.
pub fn inner_transaction_ids(batch: &Value) -> Result<Vec<Hash256>, CoreError> {
    inner_transactions(batch)?
        .into_iter()
        .map(hash_signed_tx)
        .collect()
}

/// What every batch signature covers (`serializeBatch` in `rippled`):
/// "BCH\0", the outer account, its sequence (or ticket), the outer flags, the
/// number of inner transactions and their IDs. Each signer then appends its
/// account ID.
///
/// ```
/// use xrpl_core::transaction::batch;
///
/// # use serde_json::json;
/// # use xrpl_core::crypto::{Keypair, Seed};
/// # use xrpl_core::transaction::models::BatchFlags;
/// # let alice = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// # let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// # let (a, b) = (alice.classic_address(), bob.classic_address());
/// # let mut tx = json!({
/// #     "TransactionType": "Batch", "Account": a, "Sequence": 10,
/// #     "Flags": u32::from(BatchFlags::ALL_OR_NOTHING), "Fee": batch::batch_fee(10, 2, 1).to_string(),
/// #     "RawTransactions": [
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "5000000", "Sequence": 11}))?},
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": b, "Destination": a, "Amount": "1000000", "Sequence": 3}))?},
/// #     ],
/// # });
/// // `tx`: the Batch of the module example: Alice (`a`) pays Bob (`b`, keys
/// // `bob`) and Bob pays Alice, all or nothing.
/// let data = batch::batch_signing_data(&tx)?;
/// assert_eq!(&data[..4], b"BCH\0");
/// // The prefix, the account, its sequence, the flags, the count, two IDs.
/// assert_eq!(data.len(), 4 + 20 + 4 + 4 + 4 + 2 * 32);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::ValidationError`] if `batch` has no `Account`, has neither a
/// `Sequence` nor a `TicketSequence`, or has a `Sequence`, `TicketSequence` or
/// `Flags` that is not a 32-bit unsigned integer; an address error
/// ([`CoreError::InvalidAddress`], `InvalidBase58` or `InvalidChecksum`) for an
/// `Account` that is not a classic address; and the errors of
/// [`inner_transaction_ids`].
pub fn batch_signing_data(batch: &Value) -> Result<Vec<u8>, CoreError> {
    let account = batch
        .get("Account")
        .and_then(Value::as_str)
        .ok_or_else(|| CoreError::ValidationError("a Batch needs an Account".into()))?;
    let field = |name: &str| -> Result<Option<u32>, CoreError> {
        match batch.get(name) {
            None | Some(Value::Null) => Ok(None),
            Some(v) => read_uint32(v).map(Some).ok_or_else(|| {
                CoreError::ValidationError(format!("{name} must be a 32-bit unsigned integer"))
            }),
        }
    };
    let sequence = match (field("Sequence")?, field("TicketSequence")?) {
        (Some(s), _) if s != 0 => s,
        (_, Some(t)) => t,
        (Some(s), None) => s,
        (None, None) => {
            return Err(CoreError::ValidationError(
                "a Batch needs a Sequence or TicketSequence".into(),
            ))
        }
    };
    let ids = inner_transaction_ids(batch)?;
    let mut data = Vec::with_capacity(4 + 20 + 4 + 4 + 4 + 32 * ids.len());
    data.extend_from_slice(&PREFIX_BATCH);
    data.extend_from_slice(&decode_account_id(account)?);
    data.extend_from_slice(&sequence.to_be_bytes());
    data.extend_from_slice(&field("Flags")?.unwrap_or(0).to_be_bytes());
    let count = u32::try_from(ids.len())
        .map_err(|_| CoreError::ValidationError("too many inner transactions".into()))?;
    data.extend_from_slice(&count.to_be_bytes());
    for id in &ids {
        data.extend_from_slice(&id.0);
    }
    Ok(data)
}

/// Sign `batch` as one of its participants with a single key (a
/// [`Keypair`](crate::crypto::Keypair) or any [`SigningKey`]): the
/// `BatchSigner` to add with [`add_batch_signers`]. The key is the account's
/// master key; for an account that signs with a regular key, use
/// [`sign_batch_as`]. For a remote key, [`sign_batch_async`].
///
/// ```
/// use xrpl_core::transaction::batch;
///
/// # use serde_json::json;
/// # use xrpl_core::crypto::{Keypair, Seed};
/// # use xrpl_core::transaction::models::BatchFlags;
/// # let alice = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// # let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// # let (a, b) = (alice.classic_address(), bob.classic_address());
/// # let mut tx = json!({
/// #     "TransactionType": "Batch", "Account": a, "Sequence": 10,
/// #     "Flags": u32::from(BatchFlags::ALL_OR_NOTHING), "Fee": batch::batch_fee(10, 2, 1).to_string(),
/// #     "RawTransactions": [
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "5000000", "Sequence": 11}))?},
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": b, "Destination": a, "Amount": "1000000", "Sequence": 3}))?},
/// #     ],
/// # });
/// // `tx`: the Batch of the module example: Alice (`a`) pays Bob (`b`, keys
/// // `bob`) and Bob pays Alice, all or nothing.
/// let signer = batch::sign_batch(&tx, &bob)?;
/// assert_eq!(signer.account, b);
/// batch::add_batch_signers(&mut tx, vec![signer])?;
/// assert!(batch::verify_batch_signers(&tx)?);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// The errors of [`batch_signing_data`], and the key's own
/// ([`CoreError::SigningKeyFailed`] for a public key the XRP Ledger does not
/// take).
pub fn sign_batch<K: SigningKey + ?Sized>(
    batch: &Value,
    key: &K,
) -> Result<BatchSigner, CoreError> {
    sign_batch_as(batch, &master_account(SigningKey::public_key(key))?, key)
}

/// [`sign_batch`] with an [`AsyncSigningKey`] (a remote key), whose
/// signature is checked against its public key.
///
/// ```
/// use xrpl_core::transaction::batch;
///
/// # async fn run() -> Result<(), xrpl_core::CoreError> {
/// # use serde_json::json;
/// # use xrpl_core::crypto::{Keypair, Seed};
/// # use xrpl_core::transaction::models::BatchFlags;
/// # let alice = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// # let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// # let (a, b) = (alice.classic_address(), bob.classic_address());
/// # let mut tx = json!({
/// #     "TransactionType": "Batch", "Account": a, "Sequence": 10,
/// #     "Flags": u32::from(BatchFlags::ALL_OR_NOTHING), "Fee": batch::batch_fee(10, 2, 1).to_string(),
/// #     "RawTransactions": [
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "5000000", "Sequence": 11}))?},
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": b, "Destination": a, "Amount": "1000000", "Sequence": 3}))?},
/// #     ],
/// # });
/// // `tx`: the Batch of the module example: Alice (`a`) pays Bob (`b`, keys
/// // `bob`) and Bob pays Alice, all or nothing.
/// // `bob` stands in for a remote key: every Keypair is an AsyncSigningKey too.
/// let signer = batch::sign_batch_async(&tx, &bob).await?;
/// batch::add_batch_signers(&mut tx, vec![signer])?;
/// assert!(batch::verify_batch_signers(&tx)?);
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
/// As [`sign_batch`], and [`CoreError::SigningKeyFailed`] for a signature
/// that does not verify.
pub async fn sign_batch_async<K: AsyncSigningKey + ?Sized>(
    batch: &Value,
    key: &K,
) -> Result<BatchSigner, CoreError> {
    let account = master_account(AsyncSigningKey::public_key(key))?;
    sign_batch_as_async(batch, &account, key).await
}

/// The address of the account a public key is the master key of.
fn master_account(public_key: &[u8]) -> Result<String, CoreError> {
    let public_key = crate::crypto::signing_key::checked_public_key(public_key)?;
    Ok(AccountId::from_bytes(public_key_to_account_id(public_key)).to_address())
}

/// Sign `batch` for `account` with `key`, the account's master key or its
/// regular key: the `BatchSigner` to add with [`add_batch_signers`].
/// (`rippled` appends the signing account, not the key's address, to the
/// data each batch signature covers.)
///
/// ```
/// use xrpl_core::transaction::batch;
///
/// # use serde_json::json;
/// # use xrpl_core::crypto::{Keypair, Seed};
/// # use xrpl_core::transaction::models::BatchFlags;
/// # let alice = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// # let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// # let (a, b) = (alice.classic_address(), bob.classic_address());
/// # let mut tx = json!({
/// #     "TransactionType": "Batch", "Account": a, "Sequence": 10,
/// #     "Flags": u32::from(BatchFlags::ALL_OR_NOTHING), "Fee": batch::batch_fee(10, 2, 1).to_string(),
/// #     "RawTransactions": [
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "5000000", "Sequence": 11}))?},
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": b, "Destination": a, "Amount": "1000000", "Sequence": 3}))?},
/// #     ],
/// # });
/// // `tx`: the Batch of the module example: Alice (`a`) pays Bob (`b`, keys
/// // `bob`) and Bob pays Alice, all or nothing.
/// // Bob signs with his regular key (the key he set with SetRegularKey).
/// let regular = Keypair::from_seed(&Seed::generate())?;
/// let signer = batch::sign_batch_as(&tx, &b, &regular)?;
/// assert_eq!(signer.account, b);
/// batch::add_batch_signers(&mut tx, vec![signer])?;
/// assert!(batch::verify_batch_signers(&tx)?);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// The errors of [`batch_signing_data`], an address error
/// ([`CoreError::InvalidAddress`], `InvalidBase58` or `InvalidChecksum`) for an
/// `account` that is not a classic address, and the key's own errors.
pub fn sign_batch_as<K: SigningKey + ?Sized>(
    batch: &Value,
    account: &str,
    key: &K,
) -> Result<BatchSigner, CoreError> {
    let data = batch_signer_data(batch, account)?;
    let signature = crate::crypto::signing_key::sign_with(key, &data)?;
    Ok(single_batch_signer(
        account,
        SigningKey::public_key(key),
        &signature,
    ))
}

/// [`sign_batch_as`] with an [`AsyncSigningKey`] (a remote key), whose
/// signature is checked against its public key.
///
/// ```
/// use xrpl_core::transaction::batch;
///
/// # async fn run() -> Result<(), xrpl_core::CoreError> {
/// # use serde_json::json;
/// # use xrpl_core::crypto::{Keypair, Seed};
/// # use xrpl_core::transaction::models::BatchFlags;
/// # let alice = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// # let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// # let (a, b) = (alice.classic_address(), bob.classic_address());
/// # let mut tx = json!({
/// #     "TransactionType": "Batch", "Account": a, "Sequence": 10,
/// #     "Flags": u32::from(BatchFlags::ALL_OR_NOTHING), "Fee": batch::batch_fee(10, 2, 1).to_string(),
/// #     "RawTransactions": [
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "5000000", "Sequence": 11}))?},
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": b, "Destination": a, "Amount": "1000000", "Sequence": 3}))?},
/// #     ],
/// # });
/// // `tx`: the Batch of the module example: Alice (`a`) pays Bob (`b`, keys
/// // `bob`) and Bob pays Alice, all or nothing.
/// // Bob's regular key; a Keypair stands in for a remote key here.
/// let regular = Keypair::from_seed(&Seed::generate())?;
/// let signer = batch::sign_batch_as_async(&tx, &b, &regular).await?;
/// batch::add_batch_signers(&mut tx, vec![signer])?;
/// assert!(batch::verify_batch_signers(&tx)?);
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
/// As [`sign_batch_as`], and [`CoreError::SigningKeyFailed`] for a signature
/// that does not verify.
pub async fn sign_batch_as_async<K: AsyncSigningKey + ?Sized>(
    batch: &Value,
    account: &str,
    key: &K,
) -> Result<BatchSigner, CoreError> {
    let data = batch_signer_data(batch, account)?;
    let signature = crate::crypto::signing_key::sign_with_async(key, &data).await?;
    Ok(single_batch_signer(
        account,
        AsyncSigningKey::public_key(key),
        &signature,
    ))
}

/// What `account`'s batch signature covers.
fn batch_signer_data(batch: &Value, account: &str) -> Result<Vec<u8>, CoreError> {
    let mut data = batch_signing_data(batch)?;
    data.extend_from_slice(&decode_account_id(account)?);
    Ok(data)
}

fn single_batch_signer(account: &str, public_key: &[u8], signature: &[u8]) -> BatchSigner {
    BatchSigner {
        account: account.to_string(),
        signing_pub_key: hex::encode_upper(public_key),
        txn_signature: Some(hex::encode_upper(signature)),
        signers: None,
    }
}

/// Sign `batch` for `batch_signer`, an account that approves it through its
/// signer list, as `signer`, a member of that list, with `key` (`signer`'s
/// master key or regular key): one of the `Signers` of `batch_signer`'s
/// `BatchSigner` (combine them with [`multisigned_batch_signer`]).
///
/// ```
/// use xrpl_core::transaction::batch;
///
/// # use serde_json::json;
/// # use xrpl_core::crypto::{Keypair, Seed};
/// # use xrpl_core::transaction::models::BatchFlags;
/// # let alice = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// # let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// # let (a, b) = (alice.classic_address(), bob.classic_address());
/// # let mut tx = json!({
/// #     "TransactionType": "Batch", "Account": a, "Sequence": 10,
/// #     "Flags": u32::from(BatchFlags::ALL_OR_NOTHING), "Fee": batch::batch_fee(10, 2, 1).to_string(),
/// #     "RawTransactions": [
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "5000000", "Sequence": 11}))?},
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": b, "Destination": a, "Amount": "1000000", "Sequence": 3}))?},
/// #     ],
/// # });
/// // `tx`: the Batch of the module example: Alice (`a`) pays Bob (`b`, keys
/// // `bob`) and Bob pays Alice, all or nothing.
/// // Bob approves through his signer list, which has Carol on it.
/// let carol = Keypair::from_seed(&Seed::generate())?;
/// let carol_signs = batch::sign_batch_for(&tx, &b, &carol.classic_address(), &carol)?;
/// assert_eq!(carol_signs.account, carol.classic_address());
/// let for_bob = batch::multisigned_batch_signer(&b, vec![carol_signs])?;
/// batch::add_batch_signers(&mut tx, vec![for_bob])?;
/// assert!(batch::verify_batch_signers(&tx)?);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// The errors of [`batch_signing_data`], an address error
/// ([`CoreError::InvalidAddress`], `InvalidBase58` or `InvalidChecksum`) for a
/// `batch_signer` or `signer` that is not a classic address, and the key's
/// own errors.
pub fn sign_batch_for<K: SigningKey + ?Sized>(
    batch: &Value,
    batch_signer: &str,
    signer: &str,
    key: &K,
) -> Result<Signer, CoreError> {
    let data = nested_signer_data(batch, batch_signer, signer)?;
    let signature = crate::crypto::signing_key::sign_with(key, &data)?;
    Ok(Signer::new(
        signer,
        hex::encode_upper(SigningKey::public_key(key)),
        hex::encode_upper(signature),
    ))
}

/// [`sign_batch_for`] with an [`AsyncSigningKey`] (a remote key), whose
/// signature is checked against its public key.
///
/// ```
/// use xrpl_core::transaction::batch;
///
/// # async fn run() -> Result<(), xrpl_core::CoreError> {
/// # use serde_json::json;
/// # use xrpl_core::crypto::{Keypair, Seed};
/// # use xrpl_core::transaction::models::BatchFlags;
/// # let alice = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// # let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// # let (a, b) = (alice.classic_address(), bob.classic_address());
/// # let mut tx = json!({
/// #     "TransactionType": "Batch", "Account": a, "Sequence": 10,
/// #     "Flags": u32::from(BatchFlags::ALL_OR_NOTHING), "Fee": batch::batch_fee(10, 2, 1).to_string(),
/// #     "RawTransactions": [
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "5000000", "Sequence": 11}))?},
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": b, "Destination": a, "Amount": "1000000", "Sequence": 3}))?},
/// #     ],
/// # });
/// // `tx`: the Batch of the module example: Alice (`a`) pays Bob (`b`, keys
/// // `bob`) and Bob pays Alice, all or nothing.
/// // Carol, on Bob's signer list; a Keypair stands in for a remote key here.
/// let carol = Keypair::from_seed(&Seed::generate())?;
/// let carol_signs = batch::sign_batch_for_async(&tx, &b, &carol.classic_address(), &carol).await?;
/// let for_bob = batch::multisigned_batch_signer(&b, vec![carol_signs])?;
/// batch::add_batch_signers(&mut tx, vec![for_bob])?;
/// assert!(batch::verify_batch_signers(&tx)?);
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
/// As [`sign_batch_for`], and [`CoreError::SigningKeyFailed`] for a
/// signature that does not verify.
pub async fn sign_batch_for_async<K: AsyncSigningKey + ?Sized>(
    batch: &Value,
    batch_signer: &str,
    signer: &str,
    key: &K,
) -> Result<Signer, CoreError> {
    let data = nested_signer_data(batch, batch_signer, signer)?;
    let signature = crate::crypto::signing_key::sign_with_async(key, &data).await?;
    Ok(Signer::new(
        signer,
        hex::encode_upper(AsyncSigningKey::public_key(key)),
        hex::encode_upper(signature),
    ))
}

/// What `signer`'s signature for `batch_signer` covers: the batch data, the
/// approving account, then the signer.
fn nested_signer_data(
    batch: &Value,
    batch_signer: &str,
    signer: &str,
) -> Result<Vec<u8>, CoreError> {
    let mut data = batch_signer_data(batch, batch_signer)?;
    data.extend_from_slice(&decode_account_id(signer)?);
    Ok(data)
}

/// A multi-signed `BatchSigner` for `batch_signer` from its signers'
/// signatures ([`sign_batch_for`]), sorted by account ID as `rippled` requires;
/// 1 to 32 signers, each once.
///
/// ```
/// use xrpl_core::transaction::batch;
/// use xrpl_core::types::AccountId;
///
/// # use serde_json::json;
/// # use xrpl_core::crypto::{Keypair, Seed};
/// # use xrpl_core::transaction::models::BatchFlags;
/// # let alice = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// # let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// # let (a, b) = (alice.classic_address(), bob.classic_address());
/// # let mut tx = json!({
/// #     "TransactionType": "Batch", "Account": a, "Sequence": 10,
/// #     "Flags": u32::from(BatchFlags::ALL_OR_NOTHING), "Fee": batch::batch_fee(10, 2, 1).to_string(),
/// #     "RawTransactions": [
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "5000000", "Sequence": 11}))?},
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": b, "Destination": a, "Amount": "1000000", "Sequence": 3}))?},
/// #     ],
/// # });
/// // `tx`: the Batch of the module example: Alice (`a`) pays Bob (`b`, keys
/// // `bob`) and Bob pays Alice, all or nothing.
/// // Carol and Dave, both on Bob's signer list, sign for him.
/// let (carol, dave) = (Keypair::from_seed(&Seed::generate())?, Keypair::from_seed(&Seed::generate())?);
/// let signers = vec![
///     batch::sign_batch_for(&tx, &b, &carol.classic_address(), &carol)?,
///     batch::sign_batch_for(&tx, &b, &dave.classic_address(), &dave)?,
/// ];
/// let for_bob = batch::multisigned_batch_signer(&b, signers)?;
/// let nested = for_bob.signers.as_ref().unwrap();
/// let (first, second): (AccountId, AccountId) = (nested[0].account.parse()?, nested[1].account.parse()?);
/// assert!(first < second); // in account-ID order
/// batch::add_batch_signers(&mut tx, vec![for_bob])?;
/// assert!(batch::verify_batch_signers(&tx)?);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::ValidationError`] for no signers, more than 32, or an account
/// that signs twice; an address error ([`CoreError::InvalidAddress`],
/// `InvalidBase58` or `InvalidChecksum`) for a signer's account or
/// `batch_signer` that is not a classic address.
pub fn multisigned_batch_signer(
    batch_signer: &str,
    signers: Vec<Signer>,
) -> Result<BatchSigner, CoreError> {
    if signers.is_empty() || signers.len() > MAX_NESTED_SIGNERS {
        return Err(CoreError::ValidationError(format!(
            "a multi-signed BatchSigner has 1 to {MAX_NESTED_SIGNERS} signers, got {}",
            signers.len()
        )));
    }
    let signers = sorted_by_account_id(signers, |s| &s.account)?;
    if signers
        .windows(2)
        .any(|w| matches!(w, [x, y] if x.account == y.account))
    {
        return Err(CoreError::ValidationError(
            "an account signs a BatchSigner once".into(),
        ));
    }
    decode_account_id(batch_signer)?;
    Ok(BatchSigner {
        account: batch_signer.to_string(),
        signing_pub_key: String::new(),
        txn_signature: None,
        signers: Some(signers),
    })
}

/// Put `signers` into the batch's `BatchSigners`, in ascending account-ID
/// order (as `rippled` requires; base58 text order differs). Signers already
/// there are kept; an account may appear only once.
///
/// ```
/// use xrpl_core::transaction::batch;
///
/// # use serde_json::json;
/// # use xrpl_core::crypto::{Keypair, Seed};
/// # use xrpl_core::transaction::models::BatchFlags;
/// # let alice = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// # let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// # let (a, b) = (alice.classic_address(), bob.classic_address());
/// # let mut tx = json!({
/// #     "TransactionType": "Batch", "Account": a, "Sequence": 10,
/// #     "Flags": u32::from(BatchFlags::ALL_OR_NOTHING), "Fee": batch::batch_fee(10, 2, 1).to_string(),
/// #     "RawTransactions": [
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "5000000", "Sequence": 11}))?},
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": b, "Destination": a, "Amount": "1000000", "Sequence": 3}))?},
/// #     ],
/// # });
/// // `tx`: the Batch of the module example: Alice (`a`) pays Bob (`b`, keys
/// // `bob`) and Bob pays Alice, all or nothing.
/// let signer = batch::sign_batch(&tx, &bob)?;
/// batch::add_batch_signers(&mut tx, vec![signer.clone()])?;
/// assert_eq!(tx["BatchSigners"][0]["BatchSigner"]["Account"], b);
/// assert!(batch::add_batch_signers(&mut tx, vec![signer]).is_err()); // Bob twice
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::ValidationError`] if `batch` is not a JSON object, a
/// `BatchSigners` entry already there does not parse, or an account appears
/// twice; an address error ([`CoreError::InvalidAddress`], `InvalidBase58` or
/// `InvalidChecksum`) for a signer's account that is not a classic address.
pub fn add_batch_signers(batch: &mut Value, signers: Vec<BatchSigner>) -> Result<(), CoreError> {
    let mut all: Vec<BatchSigner> = match batch.get("BatchSigners") {
        Some(existing) => existing
            .as_array()
            .into_iter()
            .flatten()
            .map(|w| {
                serde_json::from_value(w.get("BatchSigner").cloned().unwrap_or(Value::Null))
                    .map_err(|e| CoreError::ValidationError(format!("BatchSigners: {e}")))
            })
            .collect::<Result<_, _>>()?,
        None => Vec::new(),
    };
    all.extend(signers);
    let all = sorted_by_account_id(all, |s| &s.account)?;
    if all
        .windows(2)
        .any(|w| matches!(w, [x, y] if x.account == y.account))
    {
        return Err(CoreError::ValidationError(
            "an account may sign a batch only once".into(),
        ));
    }
    let wrapped: Vec<Value> = all
        .iter()
        .map(|s| serde_json::to_value(s).map(|v| serde_json::json!({ "BatchSigner": v })))
        .collect::<Result<_, _>>()
        .map_err(|e| CoreError::CodecError(e.to_string()))?;
    batch
        .as_object_mut()
        .ok_or_else(|| CoreError::ValidationError("a transaction is a JSON object".into()))?
        .insert("BatchSigners".into(), Value::Array(wrapped));
    Ok(())
}

/// The accounts that must sign `batch` (each a `BatchSigner`), as `rippled`
/// works them out (`Batch::preflightSigValidated`): every inner
/// transaction's initiator (its `Delegate`, or else its `Account`), its
/// `Counterparty`, and its `Sponsor` when it carries a `SponsorSignature`,
/// leaving out the batch's own account; in ascending account-ID order, each
/// once.
///
/// ```
/// use xrpl_core::transaction::batch;
///
/// # use serde_json::json;
/// # use xrpl_core::crypto::{Keypair, Seed};
/// # use xrpl_core::transaction::models::BatchFlags;
/// # let alice = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// # let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// # let (a, b) = (alice.classic_address(), bob.classic_address());
/// # let mut tx = json!({
/// #     "TransactionType": "Batch", "Account": a, "Sequence": 10,
/// #     "Flags": u32::from(BatchFlags::ALL_OR_NOTHING), "Fee": batch::batch_fee(10, 2, 1).to_string(),
/// #     "RawTransactions": [
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "5000000", "Sequence": 11}))?},
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": b, "Destination": a, "Amount": "1000000", "Sequence": 3}))?},
/// #     ],
/// # });
/// // `tx`: the Batch of the module example: Alice (`a`) pays Bob (`b`, keys
/// // `bob`) and Bob pays Alice, all or nothing.
/// // Alice sends the batch; only Bob signs it as a BatchSigner.
/// assert_eq!(batch::required_batch_signers(&tx)?, vec![b]);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// [`CoreError::ValidationError`] if `batch` has no `Account`, its
/// `RawTransactions` are missing or malformed (see [`inner_transactions`]), or
/// an inner transaction has no `Account`; an address error
/// ([`CoreError::InvalidAddress`], `InvalidBase58` or `InvalidChecksum`) for
/// any of these accounts that is not a classic address.
pub fn required_batch_signers(batch: &Value) -> Result<Vec<String>, CoreError> {
    let outer = batch
        .get("Account")
        .and_then(Value::as_str)
        .ok_or_else(|| CoreError::ValidationError("a Batch needs an Account".into()))?;
    let outer_id = decode_account_id(outer)?;
    let mut required: Vec<([u8; 20], String)> = Vec::new();
    let mut add = |account: &str| -> Result<(), CoreError> {
        let id = decode_account_id(account)?;
        if id != outer_id && !required.iter().any(|(k, _)| *k == id) {
            required.push((id, account.to_string()));
        }
        Ok(())
    };
    for inner in inner_transactions(batch)? {
        let text = |name: &str| inner.get(name).and_then(Value::as_str);
        let initiator = text("Delegate")
            .or_else(|| text("Account"))
            .ok_or_else(|| {
                CoreError::ValidationError("an inner transaction has no Account".into())
            })?;
        add(initiator)?;
        if let Some(counterparty) = text("Counterparty") {
            add(counterparty)?;
        }
        if let (Some(sponsor), true) = (text("Sponsor"), inner.get("SponsorSignature").is_some()) {
            add(sponsor)?;
        }
    }
    required.sort_by_key(|(k, _)| *k);
    Ok(required.into_iter().map(|(_, a)| a).collect())
}

/// Whether `batch`'s `BatchSigners` are what `rippled` accepts: exactly the
/// [`required_batch_signers`], in strictly ascending account-ID order, each
/// with a valid signature (a single one, or 1-32 distinct nested `Signers`,
/// never both). A batch that needs no signers must have none.
///
/// `rippled` also checks each key against the account (master key, regular
/// key or signer list), which needs the ledger; this does not.
///
/// ```
/// use xrpl_core::transaction::batch;
///
/// # use serde_json::json;
/// # use xrpl_core::crypto::{Keypair, Seed};
/// # use xrpl_core::transaction::models::BatchFlags;
/// # let alice = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
/// # let bob = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// # let (a, b) = (alice.classic_address(), bob.classic_address());
/// # let mut tx = json!({
/// #     "TransactionType": "Batch", "Account": a, "Sequence": 10,
/// #     "Flags": u32::from(BatchFlags::ALL_OR_NOTHING), "Fee": batch::batch_fee(10, 2, 1).to_string(),
/// #     "RawTransactions": [
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "5000000", "Sequence": 11}))?},
/// #         {"RawTransaction": batch::inner_transaction(&json!({"TransactionType": "Payment", "Account": b, "Destination": a, "Amount": "1000000", "Sequence": 3}))?},
/// #     ],
/// # });
/// // `tx`: the Batch of the module example: Alice (`a`) pays Bob (`b`, keys
/// // `bob`) and Bob pays Alice, all or nothing.
/// assert!(!batch::verify_batch_signers(&tx)?); // Bob has not signed yet
/// let signer = batch::sign_batch(&tx, &bob)?;
/// batch::add_batch_signers(&mut tx, vec![signer])?;
/// assert!(batch::verify_batch_signers(&tx)?);
/// tx["Flags"] = u32::from(BatchFlags::ONLY_ONE).into(); // changed after Bob signed
/// assert!(!batch::verify_batch_signers(&tx)?);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
///
/// # Errors
///
/// The errors of [`batch_signing_data`] and [`required_batch_signers`];
/// [`CoreError::ValidationError`] if `BatchSigners` is not an array or an entry
/// does not parse; an address error for a nested signer's account;
/// [`CoreError::InvalidHex`] for a key or signature that is not hex;
/// [`CoreError::InvalidKeyLength`] or [`CoreError::SigningFailed`] for a
/// malformed key (or secp256k1 signature). A signature that does not match is
/// `Ok(false)`, not an error.
pub fn verify_batch_signers(batch: &Value) -> Result<bool, CoreError> {
    let data = batch_signing_data(batch)?;
    let required = required_batch_signers(batch)?;
    let signers: Vec<BatchSigner> = match batch.get("BatchSigners") {
        None | Some(Value::Null) => Vec::new(),
        Some(v) => v
            .as_array()
            .ok_or_else(|| CoreError::ValidationError("BatchSigners must be an array".into()))?
            .iter()
            .map(|w| {
                serde_json::from_value(w.get("BatchSigner").cloned().unwrap_or(Value::Null))
                    .map_err(|e| CoreError::ValidationError(format!("BatchSigners: {e}")))
            })
            .collect::<Result<_, _>>()?,
    };
    if signers.iter().map(|s| &s.account).ne(required.iter()) {
        return Ok(false);
    }
    for signer in signers {
        let mut own = data.clone();
        own.extend_from_slice(&decode_account_id(&signer.account)?);
        let ok = match (&signer.txn_signature, &signer.signers) {
            (Some(sig), None) => verify(&signer.signing_pub_key, &own, sig)?,
            (None, Some(nested)) if !nested.is_empty() && nested.len() <= MAX_NESTED_SIGNERS => {
                let ids = nested
                    .iter()
                    .map(|s| decode_account_id(&s.account))
                    .collect::<Result<Vec<_>, _>>()?;
                if ids.windows(2).any(|w| matches!(w, [a, b] if a >= b)) {
                    return Ok(false);
                }
                let mut all = true;
                for (s, id) in nested.iter().zip(&ids) {
                    let mut d = own.clone();
                    d.extend_from_slice(id);
                    all &= verify(&s.signing_pub_key, &d, &s.txn_signature)?;
                }
                all
            }
            _ => false,
        };
        if !ok {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `items` in ascending account-ID order (the order `rippled` checks; the
/// order of addresses as text is different).
fn sorted_by_account_id<T>(
    items: Vec<T>,
    account: impl Fn(&T) -> &String,
) -> Result<Vec<T>, CoreError> {
    let mut keyed = items
        .into_iter()
        .map(|i| Ok((decode_account_id(account(&i))?, i)))
        .collect::<Result<Vec<_>, CoreError>>()?;
    keyed.sort_by_key(|(k, _)| *k);
    Ok(keyed.into_iter().map(|(_, i)| i).collect())
}

/// A signature over XRPL signing data, as `rippled` verifies it.
fn verify(public_key_hex: &str, data: &[u8], signature_hex: &str) -> Result<bool, CoreError> {
    let key = hex::decode(public_key_hex).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
    let sig = hex::decode(signature_hex).map_err(|e| CoreError::InvalidHex(e.to_string()))?;
    verify_signing_data(&key, data, &sig)
}

/// The `Fee` (drops) a batch needs when each inner transaction costs the
/// base fee (`Batch::calculateBaseFee`): the base fee twice (the batch
/// itself), once per inner transaction, and once per signature:
/// `batch_signatures` counts every BatchSigner signature (each signer of a
/// multi-signed `BatchSigner`) **and** each of the outer transaction's own
/// `Signers` when it is multi-signed. Inner transactions that cost more on
/// their own (AccountDelete, AMMCreate, EscrowFinish with a fulfillment, …)
/// add their extra: [`cost::transaction_cost`](super::cost::transaction_cost)
/// prices a batch with every rule. Saturates instead of overflowing.
///
/// ```
/// use xrpl_core::transaction::batch::batch_fee;
///
/// // Two inner transactions and one BatchSigner at 10 drops: 10 × (2 + 2 + 1).
/// assert_eq!(batch_fee(10, 2, 1), 50);
/// assert_eq!(batch_fee(u64::MAX, 2, 1), u64::MAX); // saturates
/// ```
pub fn batch_fee(base_fee_drops: u64, inner_transactions: usize, batch_signatures: usize) -> u64 {
    let units = 2u64
        .saturating_add(u64::try_from(inner_transactions).unwrap_or(u64::MAX))
        .saturating_add(u64::try_from(batch_signatures).unwrap_or(u64::MAX));
    base_fee_drops.saturating_mul(units)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::crypto::Keypair;
    use crate::transaction::models::BatchFlags;

    /// Every `Batch` with `BatchSigners` in the recorded sample (mainnet and
    /// Testnet transactions `rippled` accepted): our signing data must make
    /// their signatures verify. A wrong byte anywhere would fail them all.
    #[test]
    fn signatures_rippled_accepted_verify() {
        let path = format!(
            "{}/tests/fixtures/transactions.jsonl",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut checked = 0;
        for line in std::fs::read_to_string(path).unwrap().lines() {
            let tx: Value = serde_json::from_str(line).unwrap();
            if tx["TransactionType"] == "Batch" && tx.get("BatchSigners").is_some() {
                assert!(verify_batch_signers(&tx).unwrap(), "{}", tx["hash"]);
                let mut tampered = tx.clone();
                tampered["Flags"] = Value::from(tx["Flags"].as_u64().unwrap() ^ 0x0001_0000);
                assert!(!verify_batch_signers(&tampered).unwrap());
                checked += 1;
            }
        }
        assert!(checked >= 1, "no batch with signers in the sample");
    }

    #[test]
    fn signing_round_trips_and_sorts() {
        let alice = Keypair::from_seed(
            &crate::crypto::Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r").unwrap(),
        )
        .unwrap();
        let bob = Keypair::from_seed(
            &crate::crypto::Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5").unwrap(),
        )
        .unwrap();
        let carol = Keypair::from_seed(
            &crate::crypto::Seed::from_base58("sn3nxiW7v8KXzPzAqzyHXbSSKNuN9").unwrap(),
        )
        .unwrap();
        let (a, b, c) = (
            alice.classic_address(),
            bob.classic_address(),
            carol.classic_address(),
        );
        let pay = |from: &str, to: &str, seq: u32| {
            inner_transaction(&serde_json::json!({"TransactionType": "Payment", "Account": from, "Destination": to, "Amount": "1", "Sequence": seq})).unwrap()
        };
        let mut tx = serde_json::json!({
            "TransactionType": "Batch", "Account": a, "Sequence": 5, "Flags": u32::from(BatchFlags::INDEPENDENT),
            "Fee": batch_fee(10, 3, 2).to_string(),
            "RawTransactions": [{"RawTransaction": pay(&a, &b, 6)}, {"RawTransaction": pay(&b, &c, 1)}, {"RawTransaction": pay(&c, &a, 1)}],
        });
        assert_eq!(
            tx["RawTransactions"][0]["RawTransaction"]["Flags"],
            TF_INNER_BATCH_TXN
        );
        let signers = vec![
            sign_batch(&tx, &carol).unwrap(),
            sign_batch(&tx, &bob).unwrap(),
        ];
        add_batch_signers(&mut tx, signers).unwrap();
        assert!(verify_batch_signers(&tx).unwrap());
        let order: Vec<[u8; 20]> = tx["BatchSigners"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| decode_account_id(s["BatchSigner"]["Account"].as_str().unwrap()).unwrap())
            .collect();
        assert!(
            order.windows(2).all(|w| w[0] < w[1]),
            "BatchSigners not in account-ID order"
        );
        assert!(add_batch_signers(&mut tx.clone(), vec![sign_batch(&tx, &bob).unwrap()]).is_err());
        assert_eq!(batch_fee(10, 3, 2), 70);
        // A multi-signed BatchSigner: Bob's signer list signs for him.
        let mut multi = tx.clone();
        multi.as_object_mut().unwrap().remove("BatchSigners");
        let for_bob = multisigned_batch_signer(
            &b,
            vec![
                sign_batch_for(&multi, &b, &carol.classic_address(), &carol).unwrap(),
                sign_batch_for(&multi, &b, &alice.classic_address(), &alice).unwrap(),
            ],
        )
        .unwrap();
        let carol_signs = sign_batch(&multi, &carol).unwrap();
        add_batch_signers(&mut multi, vec![for_bob, carol_signs]).unwrap();
        assert!(verify_batch_signers(&multi).unwrap());
    }

    fn keys(seed: &str) -> Keypair {
        Keypair::from_seed(&crate::crypto::Seed::from_base58(seed).unwrap()).unwrap()
    }

    /// A batch from `a` with one inner payment from `a` and one from `b`.
    fn two_party_batch(a: &str, b: &str) -> Value {
        let pay = |from: &str, to: &str, seq: u32| {
            serde_json::json!({"RawTransaction": inner_transaction(&serde_json::json!({
                "TransactionType": "Payment", "Account": from, "Destination": to,
                "Amount": "1", "Sequence": seq})).unwrap()})
        };
        serde_json::json!({"TransactionType": "Batch", "Account": a, "Sequence": 5,
            "Flags": u32::from(BatchFlags::ALL_OR_NOTHING), "RawTransactions": [pay(a, b, 6), pay(b, a, 1)]})
    }

    /// rippled accepts exactly the required signers: none missing, none
    /// extra, never the outer account, never both kinds of signature.
    #[test]
    fn verification_wants_exactly_the_required_signers() {
        let (alice, bob, carol) = (
            keys("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r"),
            keys("sp5fghtJtpUorTwvof1NpDXAzNwf5"),
            keys("sn3nxiW7v8KXzPzAqzyHXbSSKNuN9"),
        );
        let (a, b) = (alice.classic_address(), bob.classic_address());
        let tx = two_party_batch(&a, &b);
        assert_eq!(required_batch_signers(&tx).unwrap(), vec![b.clone()]);
        // Missing.
        assert!(!verify_batch_signers(&tx).unwrap());
        let mut ok = tx.clone();
        let signer = sign_batch(&ok, &bob).unwrap();
        add_batch_signers(&mut ok, vec![signer]).unwrap();
        assert!(verify_batch_signers(&ok).unwrap());
        // Extra (carol has no inner transaction), and the outer account.
        for extra in [&carol, &alice] {
            let mut t = tx.clone();
            let signers = vec![
                sign_batch(&t, &bob).unwrap(),
                sign_batch(&t, extra).unwrap(),
            ];
            add_batch_signers(&mut t, signers).unwrap();
            assert!(!verify_batch_signers(&t).unwrap());
        }
        // Both a signature and nested signers; empty nested signers.
        let mut both = ok.clone();
        both["BatchSigners"][0]["BatchSigner"]["Signers"] = serde_json::json!([]);
        assert!(!verify_batch_signers(&both).unwrap());
        assert!(multisigned_batch_signer(&b, vec![]).is_err());
        // A Delegate signs for the account it acts for.
        let mut delegated = tx.clone();
        delegated["RawTransactions"][1]["RawTransaction"]["Delegate"] =
            serde_json::json!(carol.classic_address());
        assert_eq!(
            required_batch_signers(&delegated).unwrap(),
            vec![carol.classic_address()]
        );
    }

    /// Flags written as a decimal string (which the codec encodes) are read
    /// the same way here: they are kept on inner transactions and signed.
    #[test]
    fn string_flags_are_read_as_the_codec_reads_them() {
        let inner = inner_transaction(&serde_json::json!({"TransactionType": "OfferCreate",
            "Flags": "131072"}))
        .unwrap();
        assert_eq!(inner["Flags"], 131_072 | TF_INNER_BATCH_TXN);
        assert!(inner_transaction(&serde_json::json!({"Flags": "0x20000"})).is_err());
        let a = keys("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r").classic_address();
        let b = keys("sp5fghtJtpUorTwvof1NpDXAzNwf5").classic_address();
        let numeric = two_party_batch(&a, &b);
        let mut text = numeric.clone();
        text["Flags"] = serde_json::json!(u32::from(BatchFlags::ALL_OR_NOTHING).to_string());
        text["Sequence"] = serde_json::json!("5");
        assert_eq!(
            batch_signing_data(&text).unwrap(),
            batch_signing_data(&numeric).unwrap()
        );
    }

    /// An account that signs with its regular key signs as the account.
    #[test]
    fn a_regular_key_signs_for_its_account() {
        let (alice, bob, regular) = (
            keys("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r"),
            keys("sp5fghtJtpUorTwvof1NpDXAzNwf5"),
            keys("sn3nxiW7v8KXzPzAqzyHXbSSKNuN9"),
        );
        let b = bob.classic_address();
        let mut tx = two_party_batch(&alice.classic_address(), &b);
        let signer = sign_batch_as(&tx, &b, &regular).unwrap();
        assert_eq!(signer.account, b);
        assert_eq!(
            signer.signing_pub_key,
            hex::encode_upper(regular.public_key())
        );
        add_batch_signers(&mut tx, vec![signer]).unwrap();
        assert!(verify_batch_signers(&tx).unwrap());
    }

    #[test]
    fn the_fee_counts_every_signature_and_saturates() {
        // 10 × (2 + 2 inner + 1 batch signer + 2 outer signers)
        assert_eq!(batch_fee(10, 2, 1 + 2), 70);
        assert_eq!(batch_fee(u64::MAX, 2, 1), u64::MAX);
    }
}
