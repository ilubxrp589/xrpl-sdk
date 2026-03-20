#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The decoder MUST NEVER PANIC on any input.
    let _ = xrpl_core::codec::decode_transaction_binary(data);
});
