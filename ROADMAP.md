# Roadmap

The Halcyon XRPL SDK aims to be the most correct and most complete way to use the XRP Ledger
from Rust. Everything that has to agree with the ledger is tested against `rippled` itself, and
every release ships with documentation and examples.

Progress is tracked here and in [CHANGELOG.md](CHANGELOG.md). Suggestions and bug reports are
welcome as GitHub issues.

## 0.2: foundations and every request typed (released 2026-10-22, the first crates.io release)

- [x] Binary codec on `rippled` 3.4.1's definitions: all 83 transaction types, 357 fields,
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

## 0.3: every transaction, typed

- [ ] A typed model for each of `rippled`'s transaction types, generated from its own
      definitions, with required-field checks
- [ ] `Batch` (XLS-56): building, inner-transaction signing, `BatchSigners`
- [ ] Transaction flags as named constants for every type

## 0.4: developer experience

- [ ] A cookbook: send XRP, issue a token, trust lines, NFTs, AMM, escrow, checks, payment
      channels, multisign, batch, subscriptions, DEX orders
- [ ] Typed WebSocket stream events (transactions, validations, order books, path finding)
- [ ] A blocking (non-async) client
- [ ] A command-line tool (`cargo install`)
- [ ] An example of every public item on docs.rs

## Ongoing

- [ ] Codec benchmarks, published with each release
- [ ] Fuzzing in CI
- [ ] Following each `rippled` release: new amendments, fields and transaction types
