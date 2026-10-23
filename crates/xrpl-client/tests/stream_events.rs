//! Every typed stream event against messages recorded from `rippled` 3.4.1 (API version 2)
//! on mainnet (tests/fixtures/streams): each must become its own event
//! variant, with transactions typed, and keep every field the server sent.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;
use xrpl_client::stream::{OrderBookSnapshot, WsEvent};
use xrpl_core::transaction::models::Tx;

fn key_paths(v: &Value, prefix: &str, out: &mut BTreeSet<String>) {
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                let p = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                out.insert(p.clone());
                key_paths(x, &p, out);
            }
        }
        Value::Array(a) => a
            .iter()
            .for_each(|x| key_paths(x, &format!("{prefix}[]"), out)),
        _ => {}
    }
}

/// `typed` must keep every field of `message` but its `type`.
fn keeps_every_field(file: &str, message: &Value, typed: &impl Serialize) {
    let mut sent = BTreeSet::new();
    key_paths(message, "", &mut sent);
    sent.remove("type");
    let mut kept = BTreeSet::new();
    key_paths(&serde_json::to_value(typed).unwrap(), "", &mut kept);
    let dropped: Vec<_> = sent.difference(&kept).collect();
    assert!(
        dropped.is_empty(),
        "{file}: the typed event drops {dropped:?}"
    );
}

fn messages(file: &str) -> Vec<Value> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/streams")
        .join(file);
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn every_recorded_stream_message_is_typed_and_keeps_every_field() {
    let mut count = 0;
    for file in [
        "ledgerClosed.json",
        "transaction_validated.json",
        "transaction_proposed.json",
        "validationReceived.json",
        "serverStatus.json",
        "consensusPhase.json",
        "bookChanges.json",
        "peerStatusChange.json",
        "path_find.json",
    ] {
        for m in messages(file) {
            match WsEvent::from_message(m.clone()).unwrap() {
                WsEvent::Ledger(e) => keeps_every_field(file, &m, &e),
                WsEvent::Transaction(e) => {
                    assert!(
                        !matches!(e.tx_json, Tx::Other(_)),
                        "{file}: the transaction is not typed"
                    );
                    assert_eq!(e.validated, file == "transaction_validated.json");
                    keeps_every_field(file, &m, &e);
                }
                WsEvent::Validation(e) => keeps_every_field(file, &m, &e),
                WsEvent::Server(e) => keeps_every_field(file, &m, &e),
                WsEvent::Consensus(e) => keeps_every_field(file, &m, &e),
                WsEvent::BookChanges(e) => keeps_every_field(file, &m, &e),
                WsEvent::PeerStatus(e) => keeps_every_field(file, &m, &e),
                WsEvent::PathFind(e) => {
                    assert!(!e.alternatives.is_empty(), "{file}: no alternatives");
                    keeps_every_field(file, &m, &e);
                }
                other => panic!("{file}: not typed: {other:?}"),
            }
            count += 1;
        }
    }
    assert!(count >= 18, "{count}");
}

/// A `manifestReceived` message as `rippled` builds it (`NetworkOPs::pubManifest`:
/// master and signing keys, sequence, signature, master signature, domain and
/// the serialized manifest in hex), with a real validator's manifest (from
/// `rippled`'s `manifest` answer in tests/fixtures/rippled/manifest.json).
#[test]
fn a_manifest_message_is_typed_and_keeps_every_field() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rippled/manifest.json");
    let recorded: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let details = &recorded["result"]["details"];
    let manifest = base64_to_hex(recorded["result"]["manifest"].as_str().unwrap());
    let message = serde_json::json!({
        "type": "manifestReceived",
        "master_key": details["master_key"],
        "signing_key": details["ephemeral_key"],
        "seq": details["seq"],
        "signature": "304402200296DCF23518D44E47B28D3722C139399AB87C2504F19CCCF127B3733089569E022015116F6EA2B131A73557E49FB689F27E15823EB4F4E599925C0C6CCAF1AE4A56",
        "master_signature": "10B6C15F942827A98FAF41AB5EC02A358B7DF6D1288A719C68BEF11333FF9CEAC48E4FE6D659086C993DCC87B549C7330D4FECBA48AA66423B500DCB3011C601",
        "domain": details["domain"],
        "manifest": manifest,
    });
    match WsEvent::from_message(message.clone()).unwrap() {
        WsEvent::Manifest(e) => {
            // The signatures above are the ones inside the serialized manifest.
            let m = e.manifest.as_deref().unwrap();
            assert!(m.contains(&e.signature) && m.contains(&e.master_signature));
            keeps_every_field("manifestReceived", &message, &e);
        }
        other => panic!("not typed: {other:?}"),
    }
}

/// Standard base64 (the `manifest` answer) to upper-case hex (the stream's).
fn base64_to_hex(text: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut bits = 0u32;
    let mut count = 0;
    let mut out = String::new();
    for c in text.bytes().filter(|&c| c != b'=') {
        let value = u32::try_from(ALPHABET.iter().position(|&a| a == c).unwrap()).unwrap();
        bits = (bits << 6) | value;
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push_str(&format!("{:02X}", (bits >> count) & 0xFF));
        }
    }
    out
}

#[test]
fn an_order_book_snapshot_is_typed() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/streams/books_snapshot.json");
    let reply: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let snapshot: OrderBookSnapshot = serde_json::from_value(reply.clone()).unwrap();
    assert_eq!(snapshot.asks.as_ref().unwrap().len(), 2);
    assert_eq!(snapshot.bids.as_ref().unwrap().len(), 2);
    keeps_every_field("books_snapshot.json", &reply, &snapshot);
}

#[test]
fn unknown_messages_and_replies() {
    assert!(WsEvent::from_message(serde_json::json!({"id": 3, "result": {}})).is_none());
    assert!(matches!(
        WsEvent::from_message(serde_json::json!({"type": "somethingNew", "x": 1})),
        Some(WsEvent::Other(_))
    ));
}
