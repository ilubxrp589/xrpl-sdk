//! The key backups other wallets show, checked against `rippled` 3.4.1:
//! `wallet_propose` answers for 23 seeds (both key types), recorded by
//! scratch probe and kept in `tests/vectors/rfc1751_rippled.json`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use xrpl_core::crypto::{KeyType, Keypair, Seed};

fn vectors() -> serde_json::Value {
    serde_json::from_str(include_str!("vectors/rfc1751_rippled.json")).unwrap()
}

#[test]
fn rfc1751_matches_wallet_propose() {
    let file = vectors();
    let cases = file["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 23);
    for case in cases {
        let seed_hex = case["seed_hex"].as_str().unwrap();
        for (name, key_type) in [
            ("secp256k1", KeyType::Secp256k1),
            ("ed25519", KeyType::Ed25519),
        ] {
            let answer = &case[name];
            let words = answer["master_key"].as_str().unwrap();
            let seed = Seed::from_rfc1751(words, key_type).unwrap();
            assert_eq!(hex::encode_upper(seed.as_bytes()), seed_hex);
            assert_eq!(seed.to_rfc1751().as_str(), words);
            let keypair = Keypair::from_seed(&seed).unwrap();
            assert_eq!(
                hex::encode_upper(keypair.public_key()),
                answer["public_key_hex"].as_str().unwrap()
            );
            assert_eq!(
                keypair.classic_address(),
                answer["account_id"].as_str().unwrap()
            );
            // Secret numbers hold the same 16 bytes.
            let numbers = seed.to_secret_numbers();
            assert_eq!(
                Seed::from_secret_numbers(&numbers, key_type)
                    .unwrap()
                    .as_bytes(),
                seed.as_bytes()
            );
        }
    }
}

/// `wallet_propose` wrote these words for this seed, but reading them back
/// (as a `passphrase`) gives a different seed: `rippled`'s dictionary search
/// misses `YOU`. This crate reads them as written.
#[test]
fn rfc1751_reads_the_word_rippled_misses() {
    let file = vectors();
    let case = &file["you_case"];
    let words = case["master_key"].as_str().unwrap();
    assert!(words.split(' ').any(|w| w == "YOU"));
    assert_eq!(case["rippled_reads_back"], false);
    let seed = Seed::from_rfc1751(words, KeyType::Secp256k1).unwrap();
    assert_eq!(
        hex::encode_upper(seed.as_bytes()),
        case["seed_hex"].as_str().unwrap()
    );
    assert_ne!(
        case["seed_hex"].as_str().unwrap(),
        case["rippled_answer"]["master_seed_hex"].as_str().unwrap()
    );
}
