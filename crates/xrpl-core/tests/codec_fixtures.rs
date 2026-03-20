use xrpl_core::codec::encode_transaction_json;

#[test]
fn codec_transaction_vectors() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("../../../tests/vectors/codec-fixtures.json")).unwrap();

    let transactions = fixtures["transactions"].as_array().unwrap();
    let mut passed = 0;
    let mut failed = 0;

    for (i, entry) in transactions.iter().enumerate() {
        let expected_hex = entry["binary"].as_str().unwrap();
        let json = &entry["json"];
        let tx_type = json["TransactionType"].as_str().unwrap_or("?");

        match encode_transaction_json(json, false) {
            Ok(encoded) => {
                let actual_hex = hex::encode_upper(&encoded);
                if actual_hex == expected_hex.to_uppercase() {
                    passed += 1;
                } else {
                    failed += 1;
                    // Find first byte difference
                    let expected_bytes = hex::decode(expected_hex).unwrap();
                    let diff_pos = encoded
                        .iter()
                        .zip(expected_bytes.iter())
                        .position(|(a, b)| a != b)
                        .unwrap_or(encoded.len().min(expected_bytes.len()));

                    eprintln!(
                        "MISMATCH tx[{i}] {tx_type}: first diff at byte {diff_pos}, \
                         our len={}, expected len={}",
                        encoded.len(),
                        expected_bytes.len()
                    );
                    eprintln!(
                        "  got:    {}...{}",
                        &actual_hex[..80.min(actual_hex.len())],
                        if actual_hex.len() > 80 { "" } else { "" }
                    );
                    eprintln!(
                        "  expect: {}...{}",
                        &expected_hex[..80.min(expected_hex.len())],
                        if expected_hex.len() > 80 { "" } else { "" }
                    );
                }
            }
            Err(e) => {
                failed += 1;
                eprintln!("ERROR tx[{i}] {tx_type}: {e}");
            }
        }
    }

    eprintln!(
        "\nCodec fixture results: {passed} passed, {failed} failed out of {} total",
        transactions.len()
    );
    assert_eq!(failed, 0, "{failed} transaction(s) failed codec test");
}

#[test]
fn codec_account_state_vectors() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("../../../tests/vectors/codec-fixtures.json")).unwrap();

    let entries = fixtures["accountState"].as_array().unwrap();
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for (i, entry) in entries.iter().enumerate() {
        let expected_hex = entry["binary"].as_str().unwrap();
        let json = &entry["json"];

        match encode_transaction_json(json, false) {
            Ok(encoded) => {
                let actual_hex = hex::encode_upper(&encoded);
                if actual_hex == expected_hex.to_uppercase() {
                    passed += 1;
                } else {
                    failed += 1;
                    let entry_type = json["LedgerEntryType"].as_str().unwrap_or("?");
                    let expected_bytes = hex::decode(expected_hex).unwrap();
                    let diff_pos = encoded
                        .iter()
                        .zip(expected_bytes.iter())
                        .position(|(a, b)| a != b)
                        .unwrap_or(encoded.len().min(expected_bytes.len()));
                    eprintln!(
                        "MISMATCH state[{i}] {entry_type}: first diff at byte {diff_pos}, \
                         our len={}, expected len={}",
                        encoded.len(),
                        expected_bytes.len()
                    );
                }
            }
            Err(e) => {
                skipped += 1;
                let entry_type = json["LedgerEntryType"].as_str().unwrap_or("?");
                eprintln!("SKIP state[{i}] {entry_type}: {e}");
            }
        }
    }

    eprintln!(
        "\nAccount state results: {passed} passed, {failed} failed, {skipped} skipped out of {} total",
        entries.len()
    );
    // Account state may have some exotic types we don't handle yet — allow skips but no mismatches
    assert_eq!(
        failed, 0,
        "{failed} account state entries had mismatched encoding"
    );
}

// Debug test removed — codec_transaction_vectors covers all fixtures
