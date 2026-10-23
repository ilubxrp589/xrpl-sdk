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
//!     "Flags": Batch::TF_ALL_OR_NOTHING,
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
use super::models::{BatchSigner, Signer, TF_INNER_BATCH_TXN};
use crate::address::decode_account_id;
use crate::crypto::{ed25519, secp256k1, sha512_half, Keypair};
use crate::hashes::hash_signed_tx;
use crate::types::Hash256;
use crate::CoreError;
use serde_json::Value;

/// `HashPrefix::Batch` ("BCH\0").
const PREFIX_BATCH: [u8; 4] = *b"BCH\0";

/// The most inner transactions a batch may hold (`kMaxBatchTxCount`).
pub const MAX_INNER_TRANSACTIONS: usize = 8;

/// The fewest inner transactions a batch may hold.
pub const MIN_INNER_TRANSACTIONS: usize = 2;

/// Make `tx` an inner transaction: set `tfInnerBatchTxn`, `Fee` to `"0"` and
/// `SigningPubKey` to `""`, and drop any signature. Its `Sequence` (or
/// `TicketSequence`, with `Sequence` 0) must be set by the caller: the outer
/// `Batch` uses its account's next sequence, so an inner transaction from the
/// same account uses the one after.
pub fn inner_transaction(tx: &Value) -> Result<Value, CoreError> {
    let mut tx = tx.clone();
    let obj = tx
        .as_object_mut()
        .ok_or_else(|| CoreError::ValidationError("a transaction is a JSON object".into()))?;
    let flags = obj.get("Flags").and_then(Value::as_u64).unwrap_or(0);
    obj.insert(
        "Flags".into(),
        Value::from(flags | u64::from(TF_INNER_BATCH_TXN)),
    );
    obj.insert("Fee".into(), Value::String("0".into()));
    obj.insert("SigningPubKey".into(), Value::String(String::new()));
    obj.remove("TxnSignature");
    obj.remove("Signers");
    Ok(tx)
}

/// The inner transactions of a batch, in order.
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
pub fn batch_signing_data(batch: &Value) -> Result<Vec<u8>, CoreError> {
    let account = batch
        .get("Account")
        .and_then(Value::as_str)
        .ok_or_else(|| CoreError::ValidationError("a Batch needs an Account".into()))?;
    let field = |name: &str| batch.get(name).and_then(Value::as_u64).map(|v| v as u32);
    let sequence = match (field("Sequence"), field("TicketSequence")) {
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
    data.extend_from_slice(&field("Flags").unwrap_or(0).to_be_bytes());
    data.extend_from_slice(&(ids.len() as u32).to_be_bytes());
    for id in &ids {
        data.extend_from_slice(&id.0);
    }
    Ok(data)
}

/// Sign `batch` as one of its participants with a single key: the
/// `BatchSigner` to add with [`add_batch_signers`].
pub fn sign_batch(batch: &Value, keypair: &Keypair) -> Result<BatchSigner, CoreError> {
    let mut data = batch_signing_data(batch)?;
    data.extend_from_slice(&keypair.account_id().0);
    Ok(BatchSigner {
        account: keypair.classic_address(),
        signing_pub_key: hex::encode_upper(keypair.public_key()),
        txn_signature: Some(hex::encode_upper(keypair.sign_signing_data(&data)?)),
        signers: None,
    })
}

/// Sign `batch` for `batch_signer`, an account that signs through its signer
/// list: one of the `Signers` of its `BatchSigner` (combine them with
/// [`multisigned_batch_signer`]).
pub fn sign_batch_for(
    batch: &Value,
    batch_signer: &str,
    keypair: &Keypair,
) -> Result<Signer, CoreError> {
    let mut data = batch_signing_data(batch)?;
    data.extend_from_slice(&decode_account_id(batch_signer)?);
    data.extend_from_slice(&keypair.account_id().0);
    Ok(Signer {
        account: keypair.classic_address(),
        signing_pub_key: hex::encode_upper(keypair.public_key()),
        txn_signature: hex::encode_upper(keypair.sign_signing_data(&data)?),
    })
}

/// A multi-signed `BatchSigner` for `batch_signer` from its signers'
/// signatures ([`sign_batch_for`]), sorted by account ID as `rippled` requires.
pub fn multisigned_batch_signer(
    batch_signer: &str,
    signers: Vec<Signer>,
) -> Result<BatchSigner, CoreError> {
    let signers = sorted_by_account_id(signers, |s| &s.account)?;
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

/// Whether every `BatchSigners` signature in `batch` is valid for it (as
/// `rippled`'s `checkBatchSign` checks them; it also checks the signers are
/// the right accounts, and multi-signers against their signer lists).
pub fn verify_batch_signers(batch: &Value) -> Result<bool, CoreError> {
    let data = batch_signing_data(batch)?;
    let signers = batch
        .get("BatchSigners")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for wrapped in signers {
        let signer: BatchSigner =
            serde_json::from_value(wrapped.get("BatchSigner").cloned().unwrap_or(Value::Null))
                .map_err(|e| CoreError::ValidationError(format!("BatchSigners: {e}")))?;
        let mut own = data.clone();
        own.extend_from_slice(&decode_account_id(&signer.account)?);
        let ok = match (&signer.txn_signature, &signer.signers) {
            (Some(sig), _) => verify(&signer.signing_pub_key, &own, sig)?,
            (None, Some(nested)) => {
                let mut all = true;
                for s in nested {
                    let mut d = own.clone();
                    d.extend_from_slice(&decode_account_id(&s.account)?);
                    all &= verify(&s.signing_pub_key, &d, &s.txn_signature)?;
                }
                all
            }
            (None, None) => false,
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
    if key.first() == Some(&0xED) {
        ed25519::verify(&key, data, &sig)
    } else {
        secp256k1::verify(&key, &sha512_half(data), &sig)
    }
}

/// The `Fee` (drops) a batch needs when each inner transaction costs the
/// base fee: the base fee twice (the batch itself), once per inner
/// transaction, and once per batch signature (each signer of a multi-signed
/// `BatchSigner` counts). Inner transactions that cost more on their own
/// (AccountDelete, AMMCreate) add their extra.
pub fn batch_fee(base_fee_drops: u64, inner_transactions: usize, batch_signatures: usize) -> u64 {
    base_fee_drops * (2 + inner_transactions as u64 + batch_signatures as u64)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

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
            "TransactionType": "Batch", "Account": a, "Sequence": 5, "Flags": Batch::TF_INDEPENDENT,
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
                sign_batch_for(&multi, &b, &carol).unwrap(),
                sign_batch_for(&multi, &b, &alice).unwrap(),
            ],
        )
        .unwrap();
        let carol_signs = sign_batch(&multi, &carol).unwrap();
        add_batch_signers(&mut multi, vec![for_bob, carol_signs]).unwrap();
        assert!(verify_batch_signers(&multi).unwrap());
    }
}
