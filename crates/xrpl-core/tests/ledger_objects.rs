#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
use xrpl_core::types::LedgerObject;

#[test]
fn mainnet_ledger_objects_parse_typed_and_round_trip() {
    // Mainnet ledger entries from rippled 3.4.1 (.39, ledger #107827070): per type, the
    // samples with the fewest and the most fields. Each must parse into its typed variant
    // and serialize back to exactly rippled's JSON.
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/mainnet_ledger_objects.jsonl"
    );
    let mut bad = std::collections::BTreeMap::<String, (usize, String)>::new();
    let mut n = 0;
    for line in std::fs::read_to_string(path).unwrap().lines() {
        n += 1;
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        let ty = v["LedgerEntryType"].as_str().unwrap().to_string();
        let key = match LedgerObject::from_value(v.clone()) {
            Err(e) => Some(format!("{ty}: parse: {e}")),
            Ok(LedgerObject::Other(_)) => Some(format!("{ty}: Other")),
            Ok(o) => {
                let back = o.to_value().unwrap();
                if back == v {
                    None
                } else {
                    let diff: Vec<String> = v
                        .as_object()
                        .unwrap()
                        .iter()
                        .filter(|(k, x)| back.get(k.as_str()) != Some(x))
                        .map(|(k, x)| format!("{k}={x} -> {:?}", back.get(k.as_str())))
                        .chain(
                            back.as_object()
                                .unwrap()
                                .keys()
                                .filter(|k| v.get(k.as_str()).is_none())
                                .map(|k| format!("+{k}")),
                        )
                        .collect();
                    Some(format!("{ty}: roundtrip: {}", diff.join("; ")))
                }
            }
        };
        if let Some(k) = key {
            let short: String = k.chars().take(220).collect();
            let e = bad
                .entry(short.split(':').take(2).collect::<Vec<_>>().join(":"))
                .or_insert((0, short.clone()));
            e.0 += 1;
        }
    }
    assert!(n > 0);
    assert!(bad.is_empty(), "{n} objects: {bad:#?}");
}
