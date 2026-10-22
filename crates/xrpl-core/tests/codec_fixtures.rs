//! The binary codec against XRPLF's shared codec fixtures
//! (tests/vectors/codec-fixtures.json, the vectors `ripple-binary-codec` in
//! xrpl.js tests against): 34 transactions and 261 ledger entries, each a JSON
//! object with the bytes it must encode to. Both directions must hold: the JSON
//! encodes to exactly those bytes, and the bytes decode to exactly that JSON.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use serde_json::Value;
use xrpl_core::codec::{decode_transaction_binary, encode_transaction_json};

/// Checks one section of the fixtures both ways; returns the failures.
fn check(section: &str) -> (usize, Vec<String>) {
    let fixtures: Value =
        serde_json::from_str(include_str!("../../../tests/vectors/codec-fixtures.json")).unwrap();
    let entries = fixtures[section].as_array().unwrap();
    let mut failures = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        let binary = entry["binary"].as_str().unwrap().to_uppercase();
        let json = &entry["json"];
        let kind = json["TransactionType"]
            .as_str()
            .or(json["LedgerEntryType"].as_str())
            .unwrap_or("?");
        match encode_transaction_json(json, false) {
            Ok(bytes) if hex::encode_upper(&bytes) == binary => {}
            Ok(bytes) => {
                let expected = hex::decode(&binary).unwrap();
                let at = bytes
                    .iter()
                    .zip(&expected)
                    .position(|(a, b)| a != b)
                    .unwrap_or(bytes.len().min(expected.len()));
                failures.push(format!(
                    "{section}[{i}] {kind}: encodes differently from byte {at}"
                ));
            }
            Err(e) => failures.push(format!("{section}[{i}] {kind}: does not encode: {e}")),
        }
        match decode_transaction_binary(&hex::decode(&binary).unwrap()) {
            Ok(decoded) if &decoded == json => {}
            Ok(decoded) => {
                let differ: Vec<&String> = json
                    .as_object()
                    .unwrap()
                    .iter()
                    .filter(|(k, v)| decoded.get(k.as_str()) != Some(*v))
                    .map(|(k, _)| k)
                    .collect();
                failures.push(format!(
                    "{section}[{i}] {kind}: decodes differently: {differ:?}"
                ));
            }
            Err(e) => failures.push(format!("{section}[{i}] {kind}: does not decode: {e}")),
        }
    }
    eprintln!(
        "{section}: {} entries, {} failures",
        entries.len(),
        failures.len()
    );
    (entries.len(), failures)
}

#[test]
fn codec_transaction_vectors() {
    let (n, failures) = check("transactions");
    assert_eq!(n, 34);
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn codec_account_state_vectors() {
    let (n, failures) = check("accountState");
    assert_eq!(n, 261);
    assert!(failures.is_empty(), "{failures:#?}");
}
