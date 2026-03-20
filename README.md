# xrpl-sdk

Production-grade Rust SDK for the XRP Ledger — binary codec, async HTTP and WebSocket clients, typed transactions, automatic reconnection.

## Crates

| Crate | Description | `no_std` |
|-------|-------------|----------|
| `xrpl-core` | Types, binary codec, crypto (Ed25519 + Secp256k1), transaction builders | Yes (types + crypto) |
| `xrpl-client` | HTTP JSON-RPC + WebSocket with subscriptions & auto-reconnect | No |
| `xrpl-sdk` | Facade: wallet, autofill, re-exports from core + client | No |

## Install

```toml
# Full SDK (recommended)
[dependencies]
xrpl-sdk = "0.1"

# Or individual crates
[dependencies]
xrpl-core = "0.1"      # types + codec + crypto only
xrpl-client = "0.1"    # HTTP + WebSocket clients
```

## Quick Start

### Send XRP Payment

```rust,no_run
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = XrplHttpClient::new("https://s.altnet.rippletest.net:51234")?;
    let wallet = Wallet::from_seed("sEdTM1uX8pu2do5XvTnutH6HsouMaM2")?;

    let mut tx = serde_json::json!({
        "TransactionType": "Payment",
        "Account": wallet.classic_address(),
        "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        "Amount": "1000000"
    });

    let result = submit_transaction(&client, &mut tx, &wallet).await?;
    println!("Validated: {:?}", result.validated);
    Ok(())
}
```

### Subscribe to Ledger Closes (WebSocket)

```rust,no_run
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = XrplWsClient::connect("wss://s.altnet.rippletest.net:51233").await?;
    let mut rx = client.subscribe_ledger().await?;

    for _ in 0..5 {
        if let Ok(event) = rx.recv().await {
            if let WsEvent::Ledger(le) = event {
                println!("Ledger #{}: {} txns", le.ledger_index, le.txn_count.unwrap_or(0));
            }
        }
    }
    Ok(())
}
```

### Read All Trust Lines (Pagination)

```rust,no_run
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = XrplHttpClient::new("https://s.altnet.rippletest.net:51234")?;
    let lines = client.account_lines_all("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", &LedgerIndex::Validated).await?;
    println!("{} trust lines", lines.len());
    for line in &lines {
        println!("  {} {} (issuer: {})", line.balance, line.currency, line.account);
    }
    Ok(())
}
```

### Typed Transaction Builders

```rust
use xrpl_core::transaction::builders::{Payment, Transaction};
use serde_json::json;

let tx = Payment::builder("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
    .amount(json!("1000000"))
    .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
    .fee("12")
    .sequence(1)
    .build()
    .unwrap();

let json = tx.to_json();
assert_eq!(json["TransactionType"], "Payment");
```

## Features

### `xrpl-core`

- Binary codec: `encode_transaction_json` / `decode_transaction_binary` (33/34 XRPL codec fixtures pass)
- Ed25519 + Secp256k1 key derivation & signing
- All XRPL primitive types with serde support
- Typed transaction builders for 24 transaction types
- Reserve calculation utilities
- DEX price/spread/liquidity utilities
- `no_std` compatible (types + crypto, no codec)

### `xrpl-client`

- HTTP JSON-RPC client with 18+ methods
- WebSocket client with subscriptions & automatic reconnection
- Autofill (Sequence, Fee, LastLedgerSequence) with concurrent fetch
- `submit_and_wait` — reliable submission with polling until validated
- Pagination helpers (`_all` variants) for paginated endpoints
- `tracing` instrumentation on all RPC calls

### `xrpl-sdk`

- `Wallet` — generate, from_seed, sign, sign_and_encode
- Multi-signing support (`sign_for_multisigning`, `collect_signers`)
- `autofill_and_sign` / `submit_transaction` convenience functions
- Full re-exports from core + client

## Comparison

| Feature | xrpl-sdk (this) | sephynox/xrpl-rust | gmosx/xrpl-sdk-rust |
|---------|-----------------|-------------------|-------------------|
| Binary codec | Full (33/34 fixtures) | Full | Partial |
| Ed25519 + Secp256k1 | Both | Both | Ed25519 only |
| HTTP client | Yes | Yes | Yes |
| WebSocket + auto-reconnect | Yes | No | Partial |
| Typed transaction builders | 24 types | All types | Some types |
| Autofill | Yes | Yes | No |
| submit_and_wait | Yes | No | No |
| Multi-signing | Yes | Yes | No |
| no_std core | Yes | No | No |
| tracing | Yes | No | No |
| Pagination helpers | Yes | No | No |

## License

MIT
