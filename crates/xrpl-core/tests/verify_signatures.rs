//! `verify_transaction` against signatures the network and `rippled` made:
//! every signed mainnet transaction in the corpus (single and multi-signed)
//! and every transaction `rippled` 3.4.1 signed offline verifies, and none
//! verifies once a signed field changes.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use serde_json::Value;
use xrpl_core::crypto::verify_transaction;

fn corpus() -> Vec<Value> {
    let mut txs = Vec::new();
    for line in include_str!("fixtures/transactions.jsonl").lines() {
        txs.push(serde_json::from_str(line).unwrap());
    }
    for line in include_str!("fixtures/rippled_signed_transactions.jsonl").lines() {
        let record: Value = serde_json::from_str(line).unwrap();
        txs.push(record["tx_json"].clone());
    }
    txs
}

fn is_signed(tx: &Value) -> bool {
    tx.get("TxnSignature").is_some() || tx.get("Signers").is_some()
}

#[test]
fn every_real_signature_verifies() {
    let (mut single, mut multi) = (0, 0);
    for tx in corpus().iter().filter(|t| is_signed(t)) {
        assert!(verify_transaction(tx).unwrap(), "{}", tx["TransactionType"]);
        if tx.get("Signers").is_some() {
            multi += 1;
        } else {
            single += 1;
        }
    }
    assert_eq!((single, multi), (179 + 79, 2));
}

#[test]
fn a_changed_field_breaks_the_signature() {
    for tx in corpus().iter().filter(|t| is_signed(t)) {
        let mut changed = tx.clone();
        let sequence = changed["Sequence"].as_u64().unwrap();
        changed["Sequence"] = Value::from(sequence ^ 1);
        assert!(
            !verify_transaction(&changed).unwrap(),
            "{}",
            tx["TransactionType"]
        );
    }
}

#[test]
fn multi_signers_must_be_in_account_order() {
    let tx = corpus()
        .into_iter()
        .find(|t| {
            t.get("Signers")
                .and_then(Value::as_array)
                .is_some_and(|s| s.len() > 1)
        })
        .expect("a multi-signed transaction with two or more signers");
    let mut swapped = tx.clone();
    swapped["Signers"].as_array_mut().unwrap().swap(0, 1);
    assert!(!verify_transaction(&swapped).unwrap());
}

#[test]
fn unsigned_is_an_error_not_false() {
    let tx = serde_json::json!({"TransactionType": "Payment", "Account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"});
    assert!(verify_transaction(&tx).is_err());
}
