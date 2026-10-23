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

/// A field the type does not have is an error from `from_value`; inside a
/// response the entry is kept as `Unparsed`, with the error, never `Other`.
#[test]
fn an_entry_with_an_unknown_field_is_unparsed_not_other() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/mainnet_ledger_objects.jsonl"
    );
    let line = std::fs::read_to_string(path).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(line.lines().next().unwrap()).unwrap();
    v["NewAmendmentField"] = 1.into();
    let err = LedgerObject::from_value(v.clone()).unwrap_err().to_string();
    assert!(err.contains("NewAmendmentField"), "{err}");
    let lenient: LedgerObject = serde_json::from_value(v.clone()).unwrap();
    assert!(
        matches!(&lenient, LedgerObject::Unparsed { error, .. } if error.contains("NewAmendmentField")),
        "{lenient:?}"
    );
    assert_eq!(lenient.entry_type(), v["LedgerEntryType"].as_str().unwrap());
    assert_eq!(lenient.to_value().unwrap(), v);
}

/// Every flag set on a mainnet entry in the sample is one its type's flags
/// type names, and `contains` reads it.
#[test]
fn mainnet_flags_are_named() {
    use xrpl_core::flags::Flags;
    use xrpl_core::types::ledger_object::{AccountRootFlags, RippleStateFlags};
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/mainnet_ledger_objects.jsonl"
    );
    let mut checked = 0;
    for line in std::fs::read_to_string(path).unwrap().lines() {
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        let unknown = match LedgerObject::from_value(v).unwrap() {
            LedgerObject::AccountRoot(a) => a.flags.unknown_bits(),
            LedgerObject::RippleState(r) => {
                assert_eq!(
                    r.flags.contains(RippleStateFlags::LOW_RESERVE),
                    r.flags.bits() & 0x0001_0000 != 0
                );
                r.flags.unknown_bits()
            }
            LedgerObject::Offer(o) => o.flags.unknown_bits(),
            _ => continue,
        };
        assert_eq!(unknown, 0, "unknown flag bits {unknown:#010X}");
        checked += 1;
    }
    assert!(checked > 3, "{checked}");
    assert_eq!(AccountRootFlags::DEFAULT_RIPPLE.bits(), 0x0080_0000);
    // Written as the number rippled writes.
    assert_eq!(
        serde_json::to_value(AccountRootFlags::DEFAULT_RIPPLE).unwrap(),
        0x0080_0000
    );
}
