#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = xrpl_core::address::decode_account_id(s);
        let _ = xrpl_core::address::decode_seed(s);
    }
});
