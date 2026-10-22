#![no_main]
//! Whatever decodes must encode again, and decoding that must give the same JSON.
use libfuzzer_sys::fuzz_target;
use xrpl_core::codec::{decode_transaction_binary, encode_transaction_json};

fuzz_target!(|data: &[u8]| {
    if let Ok(json) = decode_transaction_binary(data) {
        let bytes = encode_transaction_json(&json, false)
            .unwrap_or_else(|e| panic!("decoded but does not encode: {e}\n{json}"));
        let again = decode_transaction_binary(&bytes)
            .unwrap_or_else(|e| panic!("re-encoded bytes do not decode: {e}\n{json}"));
        assert_eq!(again, json, "decode → encode → decode changed the JSON");
    }
});
