//! Ledger hashes against a real ledger: mainnet ledger 107852951 (34
//! transactions), recorded from `rippled` 3.4.1's `ledger` method (API
//! version 2), both as binary with the transactions expanded and as JSON.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use serde_json::Value;
use xrpl_core::ledger::{transaction_tree_hash, verify_ledger_response, LedgerHeader};
use xrpl_core::types::Hash256;

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/mainnet_ledger_107852951.json")).unwrap()
}

#[test]
fn the_json_header_hashes_to_the_ledger_hash() {
    let json = &fixture()["json"];
    let header = LedgerHeader::from_json(&json["ledger"]).unwrap();
    assert_eq!(
        header.hash().to_string(),
        json["ledger_hash"].as_str().unwrap()
    );
    // The binary header is the same header.
    let binary = &fixture()["binary"]["ledger"]["ledger_data"];
    assert_eq!(
        hex::encode_upper(header.to_bytes()),
        binary.as_str().unwrap()
    );
}

#[test]
fn the_transactions_rebuild_the_transaction_hash() {
    let verified = verify_ledger_response(&fixture()["binary"]).unwrap();
    assert_eq!(verified.transactions.len(), 34);
    assert_eq!(verified.header.ledger_index, 107_852_951);
    assert!(verified.validated);
    let tx = verified.transactions[0].tx_json().unwrap();
    assert!(tx.get("TransactionType").is_some());
    assert!(verified.transactions[0]
        .meta_json()
        .unwrap()
        .get("TransactionResult")
        .is_some());
}

/// Metadata decoded from a real ledger's bytes is `rippled`'s JSON, so it
/// reads as `TransactionMetadata` (the result by name, as `rippled` writes it).
#[test]
fn decoded_metadata_is_rippleds_json() {
    let verified = verify_ledger_response(&fixture()["binary"]).unwrap();
    for tx in &verified.transactions {
        let meta = tx.meta_json().unwrap();
        assert!(meta["TransactionResult"]
            .as_str()
            .unwrap()
            .starts_with("te"));
        let typed: xrpl_core::transaction::TransactionMetadata =
            serde_json::from_value(meta).unwrap();
        assert!(!typed.affected_nodes.is_empty());
    }
}

#[test]
fn a_missing_or_changed_transaction_is_caught() {
    let mut answer = fixture()["binary"].clone();
    let txs = answer["ledger"]["transactions"].as_array_mut().unwrap();
    let removed = txs.pop().unwrap();
    let err = verify_ledger_response(&answer).unwrap_err();
    assert!(err.to_string().contains("transaction_hash"), "{err}");

    // Changed metadata changes the tree too.
    let mut answer = fixture()["binary"].clone();
    let meta = answer["ledger"]["transactions"][0]["meta_blob"]
        .as_str()
        .unwrap()
        .to_string();
    let last = if meta.ends_with('0') { "1" } else { "0" };
    answer["ledger"]["transactions"][0]["meta_blob"] =
        Value::from(format!("{}{last}", &meta[..meta.len() - 1]));
    assert!(verify_ledger_response(&answer).is_err());

    // A transaction that is not the one its hash names.
    let mut answer = fixture()["binary"].clone();
    answer["ledger"]["transactions"][0]["hash"] = Value::from("11".repeat(32));
    assert!(verify_ledger_response(&answer)
        .unwrap_err()
        .to_string()
        .contains("written beside it"));

    // A header that does not hash to the ledger_hash named.
    let mut answer = fixture()["binary"].clone();
    answer["ledger_hash"] = Value::from("00".repeat(32));
    assert!(verify_ledger_response(&answer)
        .unwrap_err()
        .to_string()
        .contains("ledger_hash"));
    drop(removed);
}

#[test]
fn order_does_not_matter() {
    let verified = verify_ledger_response(&fixture()["binary"]).unwrap();
    let mut pairs: Vec<(&[u8], &[u8])> = verified
        .transactions
        .iter()
        .map(|t| (t.tx_blob.as_slice(), t.meta.as_slice()))
        .collect();
    pairs.reverse();
    assert_eq!(
        transaction_tree_hash(pairs).unwrap(),
        verified.header.transaction_hash
    );
    assert_eq!(transaction_tree_hash([]).unwrap(), Hash256::ZERO);
}
