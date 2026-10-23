#![no_main]
//! The string utilities and the exact-decimal arithmetic never panic.
use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use xrpl_core::address::decode_x_address;
use xrpl_core::types::Amount;
use xrpl_core::utils;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = utils::xrp_to_drops(s);
        let _ = utils::iso8601_to_ripple_time(s);
        let _ = utils::parse_nftoken_id(s);
        let _ = utils::hex_to_str(s);
        let _ = utils::rates::percent_to_transfer_rate(s);
        let _ = utils::rates::decimal_to_transfer_rate(s);
        let _ = utils::rates::percent_to_quality(s);
        let _ = utils::rates::decimal_to_quality(s);
        if let Ok(decoded) = decode_x_address(s) {
            let _ = decoded.classic_address();
            assert_eq!(decode_x_address(&decoded.to_string()).ok(), Some(decoded));
        }
        let mut parts = s.splitn(3, '|');
        if let (Some(v), Some(c), Some(i)) = (parts.next(), parts.next(), parts.next()) {
            if let Ok(a) = Amount::token(v, c, i) {
                let _ = a.to_string();
            }
        }
    }
    if let Ok(meta) = serde_json::from_slice::<Value>(data) {
        let _ = utils::get_balance_changes(&meta);
        let _ = utils::get_nftoken_id(&meta);
    }
    if data.len() >= 8 {
        let n = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let _ = utils::drops_to_xrp(n);
        let _ = utils::ripple_time_to_iso8601(n as u32);
        let _ = Amount::Xrp(n).to_string();
    }
});
