//! halcyon-xrpl-core against xrpl-rust 1.3.0, on the same inputs: real mainnet
//! transactions (from the SDK's own test fixtures) and the public test seeds.
//! Before timing anything, each pair of functions is checked to give the same
//! answer, so the comparison is between equivalent work.
//!
//! cargo bench (in this directory); results and method in ../docs/BENCHMARKS.md.
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::Value;
use xrpl_core::codec::{decode_transaction_binary, encode_transaction_json};
use xrpl_core::crypto::{sign_transaction, Keypair, Seed};

/// Real mainnet transactions of common types, as JSON and as hex.
fn inputs() -> Vec<(String, Value, String)> {
    let fixtures = include_str!("../../crates/xrpl-core/tests/fixtures/transactions.jsonl");
    let all: Vec<Value> = fixtures
        .lines()
        .map(|l| serde_json::from_str(l).expect("JSON"))
        .collect();
    let mut out = Vec::new();
    for want in [
        "Payment",
        "OfferCreate",
        "TrustSet",
        "NFTokenMint",
        "AMMDeposit",
    ] {
        // The first of each type (for Payment, the first XRP-to-XRP one, the most common kind).
        let mut tx = all
            .iter()
            .find(|tx| {
                tx["TransactionType"] == want && (want != "Payment" || tx["Amount"].is_string())
            })
            .expect("a fixture of each type")
            .clone();
        // rippled adds `hash`; it is not part of the transaction.
        tx.as_object_mut().expect("object").remove("hash");
        let blob = hex::encode_upper(encode_transaction_json(&tx, false).expect("encodes"));
        out.push((want.to_string(), tx, blob));
    }
    out
}

fn codec(c: &mut Criterion) {
    for (name, tx, blob) in inputs() {
        // Same answers first.
        let theirs = xrpl::core::binarycodec::decode(&blob).expect("xrpl-rust decodes it");
        let ours =
            decode_transaction_binary(&hex::decode(&blob).expect("hex")).expect("we decode it");
        let same_decode = theirs == ours;
        let their_hex = xrpl::core::binarycodec::encode(&tx).ok();
        let same_encode = their_hex.as_deref().map(|h| h.eq_ignore_ascii_case(&blob));
        eprintln!("{name}: decode agrees: {same_decode}; encode agrees: {same_encode:?}");

        let mut g = c.benchmark_group(format!("decode/{name}"));
        g.bench_function("halcyon-xrpl-core", |b| {
            b.iter(|| {
                decode_transaction_binary(&hex::decode(black_box(&blob)).expect("hex"))
                    .expect("decodes")
            })
        });
        g.bench_function("xrpl-rust", |b| {
            b.iter(|| xrpl::core::binarycodec::decode(black_box(&blob)).expect("decodes"))
        });
        g.finish();

        let mut g = c.benchmark_group(format!("encode/{name}"));
        g.bench_function("halcyon-xrpl-core", |b| {
            b.iter(|| {
                hex::encode_upper(encode_transaction_json(black_box(&tx), false).expect("encodes"))
            })
        });
        if their_hex.is_some() {
            g.bench_function("xrpl-rust", |b| {
                b.iter(|| xrpl::core::binarycodec::encode(black_box(&tx)).expect("encodes"))
            });
        }
        g.finish();
    }
}

/// xrpl-rust's equivalent of `sign_transaction` from its public functions: set
/// `SigningPubKey`, serialize for signing, sign, set `TxnSignature`.
fn their_sign_transaction(tx: &Value, public_key: &str, private_key: &str) -> Value {
    let mut tx = tx.clone();
    tx["SigningPubKey"] = Value::String(public_key.to_uppercase());
    tx.as_object_mut().expect("object").remove("TxnSignature");
    let data = hex::decode(xrpl::core::binarycodec::encode_for_signing(&tx).expect("encodes"))
        .expect("hex");
    tx["TxnSignature"] =
        Value::String(xrpl::core::keypairs::sign(&data, private_key).expect("signs"));
    tx
}

fn keys(c: &mut Criterion) {
    let payment = inputs()
        .into_iter()
        .find(|(name, ..)| name == "Payment")
        .expect("a payment")
        .1;
    for (name, seed) in [
        ("ed25519", "sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r"),
        ("secp256k1", "sp5fghtJtpUorTwvof1NpDXAzNwf5"),
    ] {
        let ours = Keypair::from_seed(&Seed::from_base58(seed).expect("seed")).expect("keys");
        let (their_pub, their_priv) =
            xrpl::core::keypairs::derive_keypair(seed, false).expect("keys");
        let message = b"STX\0 signing data of a typical payment, about 120 bytes long........................................";
        let same_sig = xrpl::core::keypairs::sign(message, &their_priv)
            .expect("signs")
            .eq_ignore_ascii_case(&hex::encode(
                ours.sign_signing_data(message).expect("signs"),
            ));
        eprintln!(
            "{name}: same public key: {}; same signature: {same_sig}",
            their_pub.eq_ignore_ascii_case(&hex::encode(&ours.public_key))
        );

        let mut g = c.benchmark_group(format!("derive_keypair/{name}"));
        g.bench_function("halcyon-xrpl-core", |b| {
            b.iter(|| {
                Keypair::from_seed(&Seed::from_base58(black_box(seed)).expect("seed"))
                    .expect("keys")
            })
        });
        g.bench_function("xrpl-rust", |b| {
            b.iter(|| xrpl::core::keypairs::derive_keypair(black_box(seed), false).expect("keys"))
        });
        g.finish();

        // Signing a transaction's signing data (both sign SHA-512Half of it for
        // secp256k1 and the data itself for Ed25519, as rippled verifies).
        let mut g = c.benchmark_group(format!("sign/{name}"));
        g.bench_function("halcyon-xrpl-core", |b| {
            b.iter(|| ours.sign_signing_data(black_box(message)).expect("signs"))
        });
        g.bench_function("xrpl-rust", |b| {
            b.iter(|| xrpl::core::keypairs::sign(black_box(message), &their_priv).expect("signs"))
        });
        g.finish();

        // A whole Payment, from JSON to signed JSON.
        let same_tx = sign_transaction(&payment, &ours).expect("signs")
            == their_sign_transaction(&payment, &their_pub, &their_priv);
        eprintln!("{name}: same signed Payment: {same_tx}");
        let mut g = c.benchmark_group(format!("sign_transaction/{name}"));
        g.bench_function("halcyon-xrpl-core", |b| {
            b.iter(|| sign_transaction(black_box(&payment), &ours).expect("signs"))
        });
        g.bench_function("xrpl-rust", |b| {
            b.iter(|| their_sign_transaction(black_box(&payment), &their_pub, &their_priv))
        });
        g.finish();
    }
}

fn addresses(c: &mut Criterion) {
    let address = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
    let mut g = c.benchmark_group("address/decode");
    g.bench_function("halcyon-xrpl-core", |b| {
        b.iter(|| xrpl_core::address::decode_account_id(black_box(address)).expect("decodes"))
    });
    g.bench_function("xrpl-rust", |b| {
        b.iter(|| {
            xrpl::core::addresscodec::decode_classic_address(black_box(address)).expect("decodes")
        })
    });
    g.finish();
}

criterion_group!(benches, codec, keys, addresses);
criterion_main!(benches);
