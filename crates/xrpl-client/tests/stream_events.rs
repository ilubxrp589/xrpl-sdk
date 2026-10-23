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
