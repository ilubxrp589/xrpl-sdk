# Testing Strategy

## Test Levels
1. **Unit tests** — in each `src/` file, `#[cfg(test)]` module
2. **Integration tests** — in `crates/*/tests/`
3. **Codec vector tests** — in `tests/vectors/` at workspace root
4. **Live network tests** — gated by `#[cfg(feature = "live-tests")]`

---

## Phase 1: Codec Vectors

**Source:** https://github.com/XRPLF/xrpl-codec-fixtures

Download `codec-fixtures.json` to `tests/vectors/codec-fixtures.json`.

Structure:
```json
{
  "accountState": [ { "blob": "HEX", "json": { ... } } ],
  "transactions": [ { "blob": "HEX", "json": { ... } } ]
}
```

Test: for each transaction entry, decode `json` → encode → assert hex output matches `blob`.
Also: decode `blob` → assert matches `json`.

```rust
#[test]
fn codec_transaction_vectors() {
    let fixtures: serde_json::Value = serde_json::from_str(
        include_str!("../../../tests/vectors/codec-fixtures.json")
    ).unwrap();
    for entry in fixtures["transactions"].as_array().unwrap() {
        let expected_hex = entry["blob"].as_str().unwrap();
        let json = &entry["json"];
        let tx = Transaction::from_json(json).unwrap();
        let encoded = encode_transaction(&tx).unwrap();
        assert_eq!(hex::encode_upper(&encoded), expected_hex,
            "transaction: {}", json["TransactionType"]);
    }
}
```

---

## Phase 1: Known Address Vectors

| Seed (base58) | Key Type | Classic Address |
|---|---|---|
| `sn3nxiW7v8KXzPzAqzyHXbSSKNuN9` | secp256k1 | `rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh` |
| `sEdTM1uX8pu2do5XvTnutH6HsouMaM2` | ed25519 | `rGWrZyax5eXbi5gs49MRZKkBENe7EI2q8L` |

```rust
#[test]
fn address_derivation_secp256k1() {
    let wallet = Wallet::from_seed("sn3nxiW7v8KXzPzAqzyHXbSSKNuN9").unwrap();
    assert_eq!(wallet.address, "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
}

#[test]
fn address_derivation_ed25519() {
    let wallet = Wallet::from_seed("sEdTM1uX8pu2do5XvTnutH6HsouMaM2").unwrap();
    assert_eq!(wallet.address, "rGWrZyax5eXbi5gs49MRZKkBENe7EI2q8L");
}
```

---

## Phase 1: Signing Vectors

XRPL test vectors for signing are in the codec-fixtures repo under `sign-fixtures.json`.
Each entry contains `seed`, `key_type`, `tx_json`, and `tx_blob` (signed blob).

Test: derive keypair from seed, sign tx_json, compare resulting blob to expected.

---

## Phase 1: Amount Encoding

```rust
#[test]
fn encode_xrp_1_xrp() {
    // 1 XRP = 1_000_000 drops
    let encoded = encode_amount_xrp(1_000_000);
    assert_eq!(encoded, [0x40, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x42, 0x40]);
}

#[test]
fn encode_xrp_zero() {
    let encoded = encode_amount_xrp(0);
    assert_eq!(encoded, [0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
}

#[test]
fn encode_iou_zero() {
    // Zero IOU has special encoding
    let encoded = encode_amount_iou_zero();
    assert_eq!(encoded[..8], [0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
}
```

---

## Phase 1: Base58 Vectors

```rust
#[test]
fn base58_encode_genesis_account() {
    // Account ID = all zeros = rHb9... (genesis account)
    let id = [0u8; 20];
    // actual genesis is derived from seed, use known pair
}

#[test]
fn base58_roundtrip() {
    let id: [u8; 20] = rand::random();
    let encoded = encode_account_id(&id);
    let decoded = decode_account_id(&encoded).unwrap();
    assert_eq!(id, decoded);
}

#[test]
fn base58_bad_checksum() {
    let addr = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTx"; // last char changed
    assert!(decode_account_id(addr).is_err());
}
```

---

## Phase 2: HTTP Client Tests

Use testnet — gate with `#[cfg(feature = "live-tests")]`:
```rust
#[tokio::test]
#[cfg(feature = "live-tests")]
async fn account_info_testnet() {
    let client = XrplHttpClient::new("https://s.altnet.rippletest.net:51234").unwrap();
    let account: AccountId = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".parse().unwrap();
    let info = client.account_info(&account, LedgerIndex::Validated).await.unwrap();
    assert!(info.account_data.sequence > 0);
}

#[tokio::test]
#[cfg(feature = "live-tests")]
async fn fee_testnet() {
    let client = XrplHttpClient::new("https://s.altnet.rippletest.net:51234").unwrap();
    let fee = client.fee().await.unwrap();
    assert!(fee.drops.base_fee >= 10);
}
```

---

## Phase 2: WebSocket Tests

```rust
#[tokio::test]
#[cfg(feature = "live-tests")]
async fn ws_subscribe_ledger() {
    let client = XrplWsClient::connect("wss://s.altnet.rippletest.net:51233").await.unwrap();
    let mut stream = client.subscribe_ledger().await.unwrap();
    // Wait for one ledger close (should arrive within ~4 seconds)
    let event = tokio::time::timeout(Duration::from_secs(10), stream.next())
        .await.expect("timeout").expect("stream ended");
    assert!(event.ledger_index > 0);
}
```

---

## Phase 3: End-to-End (Testnet)

```rust
#[tokio::test]
#[cfg(feature = "live-tests")]
async fn send_payment_e2e() {
    // Generate two wallets
    let sender = Wallet::generate();
    let receiver = Wallet::generate();

    // Fund sender via testnet faucet
    fund_from_faucet(&sender.address).await;
    tokio::time::sleep(Duration::from_secs(5)).await;

    let client = XrplClient::http("https://s.altnet.rippletest.net:51234").unwrap();

    // Build and send 1 XRP
    let mut tx = PaymentBuilder::new()
        .to(&receiver.classic_address)
        .amount(Amount::Xrp(1_000_000))
        .build_autofill(&client, &sender.classic_address).await.unwrap();

    let blob = sender.sign_transaction(&mut tx).unwrap();
    let result = client.submit(&blob).await.unwrap();
    assert_eq!(result.engine_result, "tesSUCCESS");
}
```

---

## Running Tests

```bash
# All unit and codec vector tests (no network)
cargo test --workspace

# Include live network tests
cargo test --workspace --features live-tests

# Single crate
cargo test -p xrpl-core

# With output
cargo test -- --nocapture

# Specific test
cargo test codec_transaction_vectors
```

---

## Clippy Config (`.clippy.toml`)
```toml
too-many-arguments-threshold = 7
cognitive-complexity-threshold = 30
```

Allowed lints in `lib.rs`:
```rust
#![deny(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
```
