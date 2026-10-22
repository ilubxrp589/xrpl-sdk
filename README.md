# Halcyon XRPL SDK

[![crates.io](https://img.shields.io/crates/v/halcyon-xrpl-sdk.svg)](https://crates.io/crates/halcyon-xrpl-sdk)
[![docs.rs](https://docs.rs/halcyon-xrpl-sdk/badge.svg)](https://docs.rs/halcyon-xrpl-sdk)
[![CI](https://github.com/ilubxrp589/xrpl-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/ilubxrp589/xrpl-sdk/actions/workflows/ci.yml)
[![downloads](https://img.shields.io/crates/d/halcyon-xrpl-sdk.svg)](https://crates.io/crates/halcyon-xrpl-sdk)
[![license](https://img.shields.io/badge/license-MIT%20%2B%20attribution-blue.svg)](LICENSE)

Community-built Rust SDK for the XRP Ledger — binary codec, async HTTP and WebSocket clients, typed transactions, automatic reconnection.

[Documentation](https://docs.rs/halcyon-xrpl-sdk) · [Changelog](CHANGELOG.md) · [Roadmap](ROADMAP.md) ·
[How it is checked](#how-it-is-checked)

## Crates

| Crate | Description | `no_std` |
|-------|-------------|----------|
| `halcyon-xrpl-core` | Types, binary codec, crypto (Ed25519 + Secp256k1), transaction builders | Yes (types + crypto) |
| `halcyon-xrpl-client` | HTTP JSON-RPC + WebSocket with subscriptions & auto-reconnect | No |
| `halcyon-xrpl-sdk` | Facade: wallet, autofill, re-exports from core + client | No |

The packages are named `halcyon-*` on crates.io; in code they are `xrpl_core`, `xrpl_client`
and `xrpl_sdk`.

## Install

```toml
# Full SDK (recommended)
[dependencies]
halcyon-xrpl-sdk = "0.2"

# Or individual crates
[dependencies]
halcyon-xrpl-core = "0.2"      # types + codec + crypto only
halcyon-xrpl-client = "0.2"    # HTTP + WebSocket clients
```

## Quick Start

### Send XRP Payment (on Testnet, with a faucet-funded wallet)

```rust,no_run
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A new wallet, funded with test XRP. Only its address leaves this machine.
    let wallet = generate_faucet_wallet(FaucetNetwork::Testnet).await?;
    let client = XrplHttpClient::new(FaucetNetwork::Testnet.rpc_url())?;

    let mut tx = serde_json::json!({
        "TransactionType": "Payment",
        "Account": wallet.classic_address(),
        "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        "Amount": utils::xrp_to_drops("1.5")?.to_string()
    });

    let result = submit_transaction(&client, &mut tx, &wallet).await?;
    for change in utils::get_balance_changes(result.meta.as_ref().unwrap())? {
        println!("{}: {:?}", change.account, change.balances);
    }
    println!("seed to keep: {}", wallet.seed().unwrap());
    Ok(())
}
```

Run it: `cargo run --example testnet_quickstart`.

### Hashes, ledger IDs and everyday helpers

```rust,ignore
use xrpl_sdk::*;

let tx_id = hash_signed_tx_blob(&blob)?;                      // the hash rippled will give it
let line = keylet::trust_line("rA…", "rB…", "USD")?;           // ID of any ledger entry
let when = utils::ripple_time_to_iso8601(814_406_400);         // ledger time → ISO 8601
let nft = utils::parse_nftoken_id("000B0539C35B55AA…")?;       // issuer, taxon, fee, flags
let changes = utils::get_balance_changes(&meta)?;              // exact, never floating point
let ok = verify_payment_channel_claim(&channel, 1_000_000, &signature, &public_key)?;
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

- Binary codec: `encode_transaction_json` / `decode_transaction_binary`. Its field registry is rippled 3.4.1's
  `server_definitions` (82 transaction types, 357 fields); all 34 transaction and 261 ledger-object
  XRPL codec fixtures round-trip byte for byte
- Ed25519 + Secp256k1 key derivation & signing, checked byte for byte against rippled's `sign` / `sign_for`
  and xrpl.js's keypair fixtures
- X-addresses (XLS-5d): encode/decode, and accepted by the codec in `Account` / `Destination`
- All XRPL primitive types with serde support; amounts typed for XRP, IOU and MPT
- Typed ledger objects for all 31 entry types, generated from rippled's own formats
- Typed transaction builders for 28 transaction types
- Transaction hashes (`hash_signed_tx`, `hash_signed_tx_blob`) and ledger entry IDs for every entry
  type (`keylet::account`, `trust_line`, `offer`, `amm`, `mptoken`, `credential`, `vault`, …)
- Payment channel claims: sign and verify
- Utilities: XRP ↔ drops, ledger time ↔ Unix / ISO 8601, text ↔ hex, TransferRate and quality,
  NFTokenID parsing, the NFTokenID a mint created, balance changes from metadata. Amounts are
  computed in exact decimal, never floating point
- Reserve calculation utilities
- DEX price/spread/liquidity utilities
- `no_std` compatible (types + crypto, no codec)
- No `unsafe`, and no `unwrap` / `expect` / unchecked indexing in library code (workspace lints)

### `xrpl-client`

- One `Client` trait over HTTP JSON-RPC, WebSocket and browser WebSocket (wasm)
- A typed request and response for every public `rippled` method and the Clio-only ones (43),
  each response checked against one recorded from a real server; `request_all` follows markers
- `simulate` (dry-run a transaction) and `submit_multisigned`
- WebSocket client with subscriptions & automatic reconnection
- Autofill (Sequence, Fee, LastLedgerSequence) with concurrent fetch
- `submit_and_wait` — reliable submission with polling until validated
- Pagination helpers (`_all` variants) for paginated endpoints
- `tracing` instrumentation on all RPC calls
- Testnet / Devnet faucet (`faucet::fund`, `fund_and_wait`)

### `halcyon-xrpl-sdk`

- `Wallet` — generate, from_seed, seed, sign, sign_and_encode, sign_payment_channel_claim
- `generate_faucet_wallet` — a funded Testnet / Devnet wallet in one call
- Multi-signing support (`sign_for_multisigning`, `collect_signers`)
- `autofill_and_sign` / `submit_transaction` convenience functions
- Full re-exports from core + client

## How it is checked

Everything that has to match the XRP Ledger is tested against the real thing, not against
itself:

| What | Checked against |
|------|-----------------|
| Signatures, multisign, transaction hashes, channel claims | rippled 3.4.1's own `sign`, `sign_for` and `channel_authorize`, byte for byte, Ed25519 and secp256k1 |
| Binary codec | all 34 transaction and 261 ledger-object XRPL codec fixtures; mainnet transactions and ledger entries |
| Ledger entry IDs | the `index` of real mainnet entries, xrpl.js's hash tests, and a live run that creates each entry type on Testnet |
| Balance changes, NFTokenIDs, rates, time, hex, X-addresses | xrpl.js's own test fixtures and expected results |
| Typed responses | 44 responses recorded from rippled 3.4.1, Clio and Testnet: each must parse, and serializing it back must keep all 2,521 field paths |
| End to end | `cargo test -p halcyon-xrpl-sdk --features live-tests`: two faucet wallets create a Ticket, Check, Escrow, PayChannel (and redeem a claim), NFToken and offer, DID, Oracle, MPT issuance and holding, Credential, PermissionedDomain, DepositPreauth, Delegate and trust line on Testnet, and send a multi-signed transaction; every ID, hash and balance change must be what the network recorded, and the typed queries for what they made must parse the answers |

## License

MIT, with an attribution requirement: if you use, copy or modify this code, credit James Turner with a link to https://github.com/ilubxrp589/xrpl-sdk. See [LICENSE](LICENSE).
