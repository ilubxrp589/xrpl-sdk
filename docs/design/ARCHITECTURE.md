# Architecture

## Workspace Layout
```
xrpl-sdk/
├── CLAUDE.md
├── Cargo.toml                  # workspace root
├── docs/
│   ├── CHECKLIST.md
│   ├── ARCHITECTURE.md         ← this file
│   ├── TYPES.md
│   ├── CODEC.md
│   ├── CRYPTO.md
│   ├── TRANSACTIONS.md
│   ├── CLIENT_HTTP.md
│   ├── CLIENT_WS.md
│   ├── ERRORS.md
│   └── TESTING.md
├── crates/
│   ├── xrpl-core/              # no_std-compatible, no network
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── address.rs      # base58check encode/decode
│   │       ├── types/
│   │       │   ├── mod.rs
│   │       │   ├── account_id.rs
│   │       │   ├── amount.rs
│   │       │   ├── currency.rs
│   │       │   ├── hash.rs
│   │       │   └── blob.rs
│   │       ├── codec/
│   │       │   ├── mod.rs
│   │       │   ├── field.rs    # FieldId, FIELD_REGISTRY
│   │       │   ├── encode.rs
│   │       │   ├── decode.rs
│   │       │   └── amount.rs   # XRP/IOU wire encoding
│   │       ├── crypto/
│   │       │   ├── mod.rs
│   │       │   ├── ed25519.rs
│   │       │   ├── secp256k1.rs
│   │       │   └── signing.rs
│   │       └── transaction/
│   │           ├── mod.rs
│   │           ├── common.rs
│   │           ├── types.rs    # TransactionType enum
│   │           └── variants/   # one file per tx type
│   ├── xrpl-client/            # requires tokio
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── http.rs
│   │       ├── ws.rs
│   │       └── types/          # response structs
│   └── xrpl-sdk/               # facade, re-exports
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── wallet.rs
│           ├── client.rs
│           └── builder/
├── examples/
│   ├── send_payment.rs
│   ├── subscribe_ledger.rs
│   └── place_offer.rs
└── tests/
    └── vectors/
        └── codec-fixtures.json  # from xrpl-codec-fixtures repo
```

## Dependency Rules
```
xrpl-core   →  (no internal deps) — crypto crates only
xrpl-client →  xrpl-core, reqwest, tokio-tungstenite
xrpl-sdk    →  xrpl-core, xrpl-client
examples    →  xrpl-sdk
```
**xrpl-core must never depend on xrpl-client.**

## Crate Responsibilities

| Crate | Owns | Does NOT own |
|---|---|---|
| `xrpl-core` | types, codec, crypto, tx structs | networking, async runtime |
| `xrpl-client` | HTTP + WS client, response types | business logic, wallet |
| `xrpl-sdk` | wallet, builders, autofill, facade | low-level encoding |

## Feature Flags (`xrpl-sdk`)
```toml
[features]
default = ["http"]
http    = ["xrpl-client/http"]
ws      = ["xrpl-client/ws"]
full    = ["http", "ws"]
```

## Error Strategy
- Each crate defines its own `Error` enum via `thiserror`
- `xrpl-sdk` wraps lower errors into a top-level `XrplSdkError`
- Never `unwrap()` in library code — only in tests and examples
- See `docs/ERRORS.md` for full taxonomy
