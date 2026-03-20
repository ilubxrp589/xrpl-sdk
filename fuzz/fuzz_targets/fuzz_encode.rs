#![no_main]
use libfuzzer_sys::fuzz_target;
use serde_json::Value;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(json) = serde_json::from_str::<Value>(s) {
            let _ = xrpl_core::codec::encode_transaction_json(&json, false);
            let _ = xrpl_core::codec::encode_transaction_json(&json, true);
        }
    }
});
