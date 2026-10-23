# Roadmap

The Halcyon XRPL SDK aims to be the most correct and most complete way to use the XRP Ledger
from Rust. Everything that has to agree with the ledger is tested against `rippled` itself, and
every release ships with documentation and examples.

Progress is tracked here and in [CHANGELOG.md](CHANGELOG.md). Suggestions and bug reports are
welcome as GitHub issues.

## 0.2: foundations and every request typed (released 2026-10-22, the first crates.io release)

- [x] Binary codec on `rippled` 3.4.1's definitions: all 82 transaction types, 357 fields,
      byte-exact on the XRPL codec fixtures and mainnet data
- [x] Ed25519 and secp256k1 signing and multi-signing, byte for byte with `rippled`
- [x] One `Client` trait over HTTP, WebSocket (subscriptions, auto-reconnect) and browser
      WebSocket; typed requests and responses; autofill; `submit_and_wait`
- [x] Typed ledger objects for all 31 entry types; typed amounts (XRP, tokens, MPT)
- [x] X-addresses, PermissionValue names, Int32 fields
- [x] Transaction hashes and the ID of every ledger entry type (`keylet`)
- [x] Utilities: XRP ↔ drops, ledger time, text ↔ hex, rates, NFTokenIDs, balance changes
- [x] Payment channel claims: sign and verify
- [x] A typed request and response for every public `rippled` and Clio method (43), each response
      checked field by field against one recorded from a real server
- [x] Testnet / Devnet faucet and `generate_faucet_wallet`
- [x] A live Testnet test that checks IDs, hashes and helpers against what the network records

## 0.3: every transaction, typed (released 2026-10-22)

- [x] A typed model for each of `rippled`'s 82 transaction types, generated from its own
      definitions, each checked against `rippled` (real transactions or `rippled`'s own signing)
- [x] `Batch` (XLS-56): inner transactions, `BatchSigners`, the fee, `autofill`; run on Testnet
- [x] Transaction flags as named constants for every type

## 0.4: developer experience (released 2026-10-22)

- [x] A cookbook: 12 runnable recipes (token issuing, NFTs, DEX, AMM, escrow, checks, payment
      channels, multisign, batch, MPT, blocking, live streams), each run on Testnet
- [x] Typed WebSocket stream events for every rippled stream, checked against mainnet messages
- [x] A blocking (non-async) client
- [x] A command-line tool: `cargo install halcyon-xrpl-cli` (`hxrpl`)
- [ ] An example of every public item on docs.rs

## 0.5: trust (released 2026-10-22)

- [x] Fuzzing in CI: eight targets (nine since 0.7), seeded with real XRPL data, on every pull request and weekly
- [x] Benchmarks against xrpl-rust on the same inputs ([docs/BENCHMARKS.md](docs/BENCHMARKS.md))
- [x] A minimum supported Rust version, checked in CI (1.85 for core, 1.88 for the rest)
- [x] A parity report: every check against `rippled` 3.4.1 ([docs/PARITY.md](docs/PARITY.md))
- [x] The codec reads and writes exactly what `rippled` does at the edges: `Number`, amounts,
      payment paths (including MPT path steps), each probed against `rippled` itself

## 0.6: security (released 2026-10-23)

- [x] A full adversarial review of every crate, with each finding fixed and tested (see the
      [changelog](CHANGELOG.md)): the codec refuses what `rippled` refuses instead of changing the
      transaction, fees are capped, `submit_and_wait` answers only with final outcomes, secrets
      never reach logs or errors, each WebSocket subscription hears only its own events
- [x] Keylets for every ledger entry type, checked against the index `rippled` computes
- [x] `delivered_amount` helpers; exact reserves and book liquidity
- [x] Narrowing `as` casts denied by Clippy in every crate
- [x] [docs/CRYPTO.md](docs/CRYPTO.md) and a security policy with a private reporting channel

## 0.7: complete and professional (released 2026-10-23, no breaking changes)

- [x] Open a wallet from another wallet's backup: a BIP-39 recovery phrase (BIP-32 paths, as
      Ledger and xrpl.js), Xaman's secret numbers, `rippled`'s RFC 1751 `master_key`
- [x] Sign typed transactions: `Wallet::sign` returns the blob and the hash; `sign_and_submit`
- [x] `autofill` prices every transaction type as `rippled` does (owner reserve for
      `AccountDelete` and `AMMCreate`, fulfillments, Confidential MPT, `LoanSet`, `Batch`)
- [x] `verify_transaction`, `Client::balances`, ledger entry flags by name
- [x] Check a ledger against its hash (header, transaction and state trees)
- [x] Client settings: HTTP and WebSocket builders, retries only when the server says it is busy,
      a typed HTTP status error; `Client::pages` with no page limit
- [x] Standard traits on every public type; `BlockingClient` has every method
- [x] wasm: waiting works in Web Workers and Node.js, tested in CI
- [x] CI on Linux, macOS and Windows; SemVer check; docs built as docs.rs builds them;
      contributor guide and issue forms
- [x] `# Errors` in the documentation of every function that can fail

## 0.8: one consistent API (the 1.0 candidate)

One deliberate set of breaking changes, so that 1.0 can then stay stable:

- [x] One way to build a transaction (the typed models, with setters and checks); the builders
      retired
- [x] Room to grow without breaking: `#[non_exhaustive]` on protocol types, setters instead of
      struct literals
- [x] One type per idea: assets, path steps, signers and balances the same everywhere; typed
      flags per transaction and ledger entry type (addresses stay strings: X-addresses and
      pseudo-accounts must round-trip)
- [x] A `Client` trait whose futures are `Send`, usable as `dyn` (`Transport`); errors that
      separate the caller's mistakes from the server's, with no `reqwest` types
- [x] An external signer (hardware wallet, HSM, cloud key service): `SigningKey` and
      `AsyncSigningKey`
- [x] `rippled` API version 2 by default, with typed transaction metadata; every recorded
      response re-recorded in version 2
- [x] An order book helper with typed assets; typed `path_find`
- [x] Features that gate code (`http`, `ws`, `wasm`), trimmed dependencies
- [x] Apache-2.0 licence, with a NOTICE

## 1.0

The 0.8 API, settled, with a stability promise: no breaking change without a major version.

## Ongoing

- [ ] Benchmarks re-run and published with each release
- [ ] Following each `rippled` release: new amendments, fields and transaction types (a weekly
      check compares Devnet, Testnet and mainnet with the SDK's definitions and opens an issue
      when they differ)
