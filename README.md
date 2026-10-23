# Halcyon XRPL SDK

[![crates.io](https://img.shields.io/crates/v/halcyon-xrpl-sdk.svg)](https://crates.io/crates/halcyon-xrpl-sdk)
[![docs.rs](https://docs.rs/halcyon-xrpl-sdk/badge.svg)](https://docs.rs/halcyon-xrpl-sdk)
[![CI](https://github.com/ilubxrp589/xrpl-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/ilubxrp589/xrpl-sdk/actions/workflows/ci.yml)
[![downloads](https://img.shields.io/crates/d/halcyon-xrpl-sdk.svg)](https://crates.io/crates/halcyon-xrpl-sdk)
[![Fuzz](https://github.com/ilubxrp589/xrpl-sdk/actions/workflows/fuzz.yml/badge.svg)](https://github.com/ilubxrp589/xrpl-sdk/actions/workflows/fuzz.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.85%20core%20%C2%B7%201.88-orange.svg)](#minimum-supported-rust-version)
[![license](https://img.shields.io/badge/license-MIT%20%2B%20attribution-blue.svg)](LICENSE)

Community-built Rust SDK for the XRP Ledger — binary codec, async HTTP and WebSocket clients, typed transactions, automatic reconnection.

[Documentation](https://docs.rs/halcyon-xrpl-sdk) · [Changelog](CHANGELOG.md) · [Roadmap](ROADMAP.md) ·
[How it is checked](#how-it-is-checked) · [Parity with rippled](docs/PARITY.md) · [Benchmarks](docs/BENCHMARKS.md)

## Crates

| Crate | Description | `no_std` |
|-------|-------------|----------|
| `halcyon-xrpl-core` | Types, binary codec, crypto (Ed25519 + Secp256k1), transaction builders | Yes (types + crypto) |
| `halcyon-xrpl-client` | HTTP JSON-RPC + WebSocket with subscriptions & auto-reconnect | No |
| `halcyon-xrpl-sdk` | Facade: wallet, autofill, re-exports from core + client | No |
| `halcyon-xrpl-cli` | `hxrpl`, the command line: wallets, faucet, queries, signing, sending, live streams | — |

The packages are named `halcyon-*` on crates.io; in code they are `xrpl_core`, `xrpl_client`
and `xrpl_sdk`.

## Install

```toml
# Full SDK (recommended)
[dependencies]
halcyon-xrpl-sdk = "0.5"

# Or individual crates
[dependencies]
halcyon-xrpl-core = "0.5"      # types + codec + crypto only
halcyon-xrpl-client = "0.5"    # HTTP + WebSocket clients
```

### Minimum supported Rust version

`halcyon-xrpl-core` builds on Rust 1.85 or later; `halcyon-xrpl-client`, `halcyon-xrpl-sdk` and
`halcyon-xrpl-cli` on 1.88 or later. CI checks both with the locked dependencies, and raising
either is a minor-version change, noted in the [changelog](CHANGELOG.md).

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
    println!("seed to keep: {}", wallet.seed().unwrap().as_str());
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

## Cookbook

Runnable recipes in [`crates/xrpl-sdk/examples`](crates/xrpl-sdk/examples). Each one makes its own
funded Testnet wallets, so `cargo run --example <name>` works with nothing to set up. Every recipe
below was run against Testnet (and `watch_streams` against mainnet) before release.

| Recipe | What it does |
|--------|--------------|
| `testnet_quickstart` | a funded wallet, a payment, its hash and balance changes |
| `issue_token` | issue a token: DefaultRipple, trust lines, issuing, paying holder to holder |
| `nft_marketplace` | mint an NFT with a 5% transfer fee, sell it, resell it, and the artist's royalty |
| `dex_order` | place an order, read the order book, take part of it, cancel the rest |
| `amm` | create an XRP/USD AMM pool, trade against it, watch the pool move |
| `escrow` | lock XRP until a time, then release it |
| `checks` | write a check and cash part of it |
| `payment_channel` | open a channel, pay with off-ledger signed claims, redeem one |
| `multisign` | a signer list of two, a transaction signed by both, `submit_multisigned` |
| `batch` | two accounts' payments in one all-or-nothing Batch, co-signed |
| `mpt` | create a Multi-Purpose Token, opt in, send it |
| `blocking` | the same, with no async: `BlockingClient` |
| `watch_streams` | live mainnet ledgers, order-book trading and validations over WebSocket |
| `check_balance`, `decode_blob`, `subscribe_ledger` | an account's spendable balance; decoding a blob; the ledger stream |

## Command line

```sh
cargo install halcyon-xrpl-cli
hxrpl wallet faucet                                       # a funded Testnet wallet
hxrpl --network mainnet account info rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh
XRPL_SEED=s… hxrpl tx send-xrp rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe 1.5
hxrpl --network mainnet book XRP RLUSD.rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De
hxrpl --network mainnet watch ledger --count 5
hxrpl tx hash 120000220000000024…                         # a signed blob's ID, offline
```

Testnet unless told otherwise; sending on mainnet needs `--yes`; seeds only from `XRPL_SEED` or
`--seed-file`. See [`crates/xrpl-cli`](crates/xrpl-cli).

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
- **Typed transactions for all 82 types**, generated from rippled's definitions: every field,
  `new` with the required ones, and every flag as a constant (`Payment::TF_PARTIAL_PAYMENT`)
- **Batch (XLS-56)**: inner transactions, `BatchSigners` from other accounts (single or multi-signed),
  the batch fee, and `autofill` that numbers and prices a batch
- Fluent builders for 28 common transaction types
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
- WebSocket client with subscriptions & automatic reconnection; typed events for every rippled
  stream (ledger, transactions, validations, server, consensus, book changes, manifests, peer status)
  and order books with a snapshot
- `BlockingClient`: every method as a plain call, no async runtime needed
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
itself. [docs/PARITY.md](docs/PARITY.md) has every check, with its numbers and how to repeat it.

| What | Checked against |
|------|-----------------|
| Signatures, multisign, transaction hashes, channel claims | rippled 3.4.1's own `sign`, `sign_for` and `channel_authorize`, byte for byte, Ed25519 and secp256k1 |
| Binary codec | all 34 transaction and 261 ledger-object XRPL codec fixtures; mainnet transactions and ledger entries |
| Ledger entry IDs | the `index` of real mainnet entries, xrpl.js's hash tests, and a live run that creates each entry type on Testnet |
| Balance changes, NFTokenIDs, rates, time, hex, X-addresses | xrpl.js's own test fixtures and expected results |
| Transaction models | 186 real transactions (51 types) round-trip to the same JSON, bytes and ID; an example of every other type, signed by rippled 3.4.1, encodes to its exact `tx_blob`: all 82 types |
| Edge cases of the format: `Number`, amounts, paths, issues, object structure | 94 binary forms and 80 JSON values put to rippled 3.4.1's `simulate`: the SDK writes and shows each as rippled does, and refuses what rippled refuses instead of rounding or guessing |
| Batch | signatures verify against real batches rippled accepted; two batches (one from the typed models, approved by a second account) run on Testnet |
| Typed responses | 44 responses recorded from rippled 3.4.1, Clio and Testnet: each must parse, and serializing it back must keep all 2,521 field paths |
| End to end | `cargo test -p halcyon-xrpl-sdk --features live-tests`: two faucet wallets create a Ticket, Check, Escrow, PayChannel (and redeem a claim), NFToken and offer, DID, Oracle, MPT issuance and holding, Credential, PermissionedDomain, DepositPreauth, Delegate and trust line on Testnet, and send a multi-signed transaction; every ID, hash and balance change must be what the network recorded, and the typed queries for what they made must parse the answers |
| Robustness | 8 fuzz targets with AddressSanitizer, seeded with all of the above (about 86 million inputs in the final run before 0.5.0; a minute each on every pull request, 15 minutes each weekly): nothing panics, and decode → encode → decode is stable |

## Performance

Measured against xrpl-rust 1.3.0 on the same real mainnet transactions, after checking that both
give identical results ([docs/BENCHMARKS.md](docs/BENCHMARKS.md) has the method and every number):

- encoding a transaction is 8 to 24 times faster, decoding 1.7 to 12 times faster;
- deriving Ed25519 keys is 5 times faster, and signing a whole transaction 1.3 to 1.7 times faster;
- a bare secp256k1 signature is 1.5 times slower (pure-Rust `k256`, against the C library
  libsecp256k1), and decoding an address 1.2 times slower.

## License

MIT, with an attribution requirement: if you use, copy or modify this code, credit James Turner with a link to https://github.com/ilubxrp589/xrpl-sdk. See [LICENSE](LICENSE).
