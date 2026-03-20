#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() == 8 {
        let arr: [u8; 8] = data.try_into().unwrap();
        let _ = xrpl_core::codec::amount::decode_amount_xrp(&arr);
        let _ = xrpl_core::codec::amount::decode_iou_value(&arr);
    }
    let _ = xrpl_core::codec::amount::decode_amount(data);
});
