#![no_main]
//! The key-backup readers, the ledger checks and the signature checks never
//! panic, on any input: recovery phrases, secret numbers, RFC 1751 keys,
//! derivation paths, ledger headers and answers, signed transactions and
//! transaction costs.
use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use xrpl_core::address::KeyType;
use xrpl_core::crypto::hd::DerivationPath;
use xrpl_core::crypto::mnemonic::Mnemonic;
use xrpl_core::crypto::{rfc1751, secret_numbers, verify_transaction, Seed};
use xrpl_core::ledger::{state_tree_hash, transaction_tree_hash, verify_ledger_response, LedgerHeader};
use xrpl_core::transaction::cost::transaction_cost;
use xrpl_core::types::Hash256;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // The phrase is not turned into a seed: 2,048 PBKDF2 rounds per input
        // would slow the fuzzer without reaching new code.
        if let Ok(m) = Mnemonic::parse(s) {
            assert_eq!(Mnemonic::parse(m.phrase()).unwrap(), m);
        }
        if let Ok(seed) = secret_numbers::decode(s) {
            assert_eq!(*secret_numbers::decode(&secret_numbers::encode(&seed)).unwrap(), *seed);
        }
        if let Ok(seed) = rfc1751::decode(s) {
            assert_eq!(*rfc1751::decode(&rfc1751::encode(&seed)).unwrap(), *seed);
        }
        if let Ok(path) = s.parse::<DerivationPath>() {
            assert_eq!(path.to_string().parse::<DerivationPath>().unwrap(), path);
            // Deriving costs elliptic-curve work: only for short paths.
            if path.indices().len() <= 3 {
                let _ = path.derive(&[7u8; 64]);
            }
        }
        let _ = s.parse::<KeyType>();
        let _ = s.parse::<Hash256>();
        let _ = Seed::from_secret_numbers(s, KeyType::Secp256k1);
    }
    let _ = LedgerHeader::from_bytes(data);
    // A seed from the input, now and then (deriving is slow).
    if data.len() >= 16 && data.first() == Some(&0x5E) {
        let _ = DerivationPath::default().derive(&data[..data.len().min(64)]);
    }
    // Pairs of chunks as (transaction, metadata) and (ID, entry).
    let chunks: Vec<&[u8]> = data.chunks(24).collect();
    let pairs: Vec<(&[u8], &[u8])> = chunks.chunks(2).filter_map(|c| Some((*c.first()?, *c.get(1)?))).collect();
    let _ = transaction_tree_hash(pairs.iter().copied());
    let _ = state_tree_hash(pairs.iter().map(|(k, v)| {
        let mut id = [0u8; 32];
        id[..k.len().min(32)].copy_from_slice(&k[..k.len().min(32)]);
        (Hash256(id), *v)
    }));
    if let Ok(json) = serde_json::from_slice::<Value>(data) {
        let _ = LedgerHeader::from_json(&json);
        let _ = verify_ledger_response(&json);
        let _ = verify_transaction(&json);
        let _ = transaction_cost(&json, 2);
    }
});
