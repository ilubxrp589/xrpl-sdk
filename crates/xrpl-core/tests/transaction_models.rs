//! The typed transaction models against real transactions: 187 that
//! `rippled` applied on mainnet and Testnet (tests/fixtures/transactions.jsonl,
//! 50 transaction types). Each must parse strictly into its own type (no
//! field dropped), serialize back to exactly the same JSON, encode to the
//! same bytes, and hash to the ID the ledger gave it.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use serde_json::Value;
use std::collections::BTreeMap;
use xrpl_core::codec::encode_transaction_json;
use xrpl_core::hashes::hash_signed_tx;
use xrpl_core::transaction::models::{Tx, NAMES};

#[test]
fn real_transactions_round_trip_through_their_types() {
    let path = format!(
        "{}/tests/fixtures/transactions.jsonl",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut per_type: BTreeMap<String, usize> = BTreeMap::new();
    for line in std::fs::read_to_string(path).unwrap().lines() {
        let original: Value = serde_json::from_str(line).unwrap();
        let hash = original["hash"].as_str().unwrap().to_string();
        let typed = Tx::from_value(original.clone()).unwrap_or_else(|e| panic!("{hash}: {e}"));
        assert!(
            !matches!(typed, Tx::Other(_)),
            "{hash}: {} is not typed",
            typed.transaction_type()
        );
        let back = typed.to_value().unwrap();
        assert_eq!(back, original, "{hash}: the typed JSON differs");
        assert_eq!(
            encode_transaction_json(&back, false).unwrap(),
            encode_transaction_json(&original, false).unwrap(),
            "{hash}: the bytes differ"
        );
        assert_eq!(
            hash_signed_tx(&back)
                .unwrap_or_else(|e| panic!("{hash}: {e}"))
                .to_string(),
            hash,
            "{hash}: the ID differs"
        );
        *per_type
            .entry(typed.transaction_type().to_string())
            .or_default() += 1;
    }
    assert!(
        per_type.len() >= 50,
        "only {} types: {per_type:?}",
        per_type.len()
    );
    println!(
        "{} real transactions, {} types",
        per_type.values().sum::<usize>(),
        per_type.len()
    );
}

#[test]
fn every_type_rippled_defines_has_a_model() {
    assert_eq!(NAMES.len(), 82);
    for name in NAMES {
        let tx = Tx::from_value(
            serde_json::json!({"TransactionType": name, "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"}),
        );
        // Missing required fields are an error for the typed parse, never a silent `Other`.
        assert!(!matches!(tx, Ok(Tx::Other(_))), "{name}");
    }
}

#[test]
fn a_field_the_type_does_not_have_is_an_error() {
    let mut tx: Value = serde_json::from_str(
        std::fs::read_to_string(format!(
            "{}/tests/fixtures/transactions.jsonl",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap()
        .lines()
        .find(|l| l.contains("\"TransactionType\":\"Payment\""))
        .unwrap(),
    )
    .unwrap();
    tx["NFTokenTaxon"] = 1.into();
    let err = Tx::from_value(tx.clone()).unwrap_err().to_string();
    assert!(err.contains("NFTokenTaxon"), "{err}");
    // Leniently, inside a larger value, it is kept as `Other`.
    let lenient: Tx = serde_json::from_value(tx).unwrap();
    assert!(matches!(lenient, Tx::Other(_)));
}

/// One example of each transaction type, with every field filled in, that
/// `rippled` 3.4.1 signed offline (tests/fixtures/rippled_signed_transactions.jsonl):
/// rippled's own encoding of each of its transaction formats. The typed model
/// must read rippled's JSON, write it back unchanged, encode to rippled's
/// `tx_blob` byte for byte, and hash to rippled's `hash`.
#[test]
fn every_type_encodes_as_rippled_signs_it() {
    let path = format!(
        "{}/tests/fixtures/rippled_signed_transactions.jsonl",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut types = Vec::new();
    for line in std::fs::read_to_string(path).unwrap().lines() {
        let row: Value = serde_json::from_str(line).unwrap();
        let tx_json = row["tx_json"].clone();
        let name = tx_json["TransactionType"].as_str().unwrap().to_string();
        let typed = Tx::from_value(tx_json.clone()).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(!matches!(typed, Tx::Other(_)), "{name} is not typed");
        let back = typed.to_value().unwrap();
        assert_eq!(back, tx_json, "{name}: the typed JSON differs");
        assert_eq!(
            hex::encode_upper(encode_transaction_json(&back, false).unwrap()),
            row["tx_blob"].as_str().unwrap(),
            "{name}: our bytes differ from rippled's"
        );
        assert_eq!(
            hash_signed_tx(&back).unwrap().to_string(),
            tx_json["hash"].as_str().unwrap(),
            "{name}"
        );
        types.push(name);
    }
    assert_eq!(types.len(), 79, "every type but the 3 pseudo-transactions");
}

/// Between them, the two checks above cover every type `rippled` defines.
#[test]
fn every_type_is_checked_against_rippled() {
    let mut covered = std::collections::BTreeSet::new();
    for file in ["transactions.jsonl", "rippled_signed_transactions.jsonl"] {
        let path = format!("{}/tests/fixtures/{file}", env!("CARGO_MANIFEST_DIR"));
        for line in std::fs::read_to_string(path).unwrap().lines() {
            let v: Value = serde_json::from_str(line).unwrap();
            let tx = v.get("tx_json").unwrap_or(&v);
            covered.insert(tx["TransactionType"].as_str().unwrap().to_string());
        }
    }
    let missing: Vec<_> = NAMES.iter().filter(|n| !covered.contains(**n)).collect();
    assert!(
        missing.is_empty(),
        "not checked against rippled: {missing:?}"
    );
}
