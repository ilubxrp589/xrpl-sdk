#![no_main]
//! Transaction hashes and Batch signing data never panic, from binary or JSON.
use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use xrpl_core::hashes::{hash_signed_tx, hash_signed_tx_blob};
use xrpl_core::transaction::batch;

fuzz_target!(|data: &[u8]| {
    let _ = hash_signed_tx_blob(&hex::encode(data));
    if let Ok(json) = xrpl_core::codec::decode_transaction_binary(data) {
        let _ = hash_signed_tx(&json);
    }
    if let Ok(json) = serde_json::from_slice::<Value>(data) {
        let _ = hash_signed_tx(&json);
        let _ = batch::inner_transaction_ids(&json);
        let _ = batch::batch_signing_data(&json);
        let _ = batch::verify_batch_signers(&json);
        let _ = batch::inner_transaction(&json);
    }
});
