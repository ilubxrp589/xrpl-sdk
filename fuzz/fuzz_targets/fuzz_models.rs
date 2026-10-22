#![no_main]
//! Typed transactions and ledger entries never panic on any JSON, and a typed
//! transaction writes back what it read.
use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use xrpl_core::transaction::models::Tx;
use xrpl_core::types::LedgerObject;

fuzz_target!(|data: &[u8]| {
    let Ok(json) = serde_json::from_slice::<Value>(data) else {
        return;
    };
    if let Ok(tx) = Tx::from_value(json.clone()) {
        if !matches!(tx, Tx::Other(_)) {
            let back = tx.to_value().expect("a typed transaction serializes");
            let again = Tx::from_value(back.clone()).expect("what Tx wrote, Tx reads");
            assert_eq!(again.to_value().expect("serializes"), back);
        }
    }
    let _ = serde_json::from_value::<Tx>(json.clone());
    let _ = LedgerObject::from_value(json.clone());
    let _ = serde_json::from_value::<LedgerObject>(json);
});
