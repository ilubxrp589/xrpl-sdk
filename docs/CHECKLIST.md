# XRPL Rust SDK — Master Build Checklist

## Phase 1 · `xrpl-core` (Foundation)

### 1.1 Workspace Setup
- [ ] `Cargo.toml` workspace with `resolver = "2"`
- [ ] Members: `crates/xrpl-core`, `crates/xrpl-client`, `crates/xrpl-sdk`
- [ ] Shared `[workspace.dependencies]` for tokio, serde, hex, thiserror
- [ ] `rustfmt.toml` and `.clippy.toml` at root
- [ ] `docs/` directory with all spec `.md` files

### 1.2 Primitive Types (`xrpl-core/src/types/`)
- [ ] `AccountId` — newtype `[u8; 20]`, Display as base58check (XRPL alphabet)
- [ ] `Hash128` — newtype `[u8; 16]`
- [ ] `Hash160` — newtype `[u8; 20]`
- [ ] `Hash256` — newtype `[u8; 32]`, Display as uppercase hex
- [ ] `Blob` — newtype `Vec<u8>`
- [ ] `Amount` enum:
  - `Xrp(u64)` — drops, max 100_000_000_000_000_000
  - `Iou { value: IouAmount, currency: Currency, issuer: AccountId }`
- [ ] `Currency` — 3-char ASCII or 20-byte hex, Display handles both
- [ ] `IouAmount` — mantissa/exponent representation matching XRPL wire format
- [ ] `UInt8`, `UInt16`, `UInt32`, `UInt64` — plain newtype wrappers
- [ ] All types: `Debug`, `Clone`, `PartialEq`, `Eq`, `serde::Serialize/Deserialize`

### 1.3 Base58 / Address Encoding (`xrpl-core/src/address.rs`)
- [ ] XRPL base58 alphabet: `rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz`
- [ ] `encode_account_id(bytes: &[u8; 20]) -> String` — prefix `0x00`, 4-byte checksum
- [ ] `decode_account_id(addr: &str) -> Result<[u8; 20]>` — validates checksum
- [ ] `encode_seed(bytes: &[u8; 16], key_type: KeyType) -> String`
  - Ed25519 prefix: `[0x01, 0xE1, 0x4B]`
  - Secp256k1 prefix: `[0x21]`
- [ ] `decode_seed(seed: &str) -> Result<([u8; 16], KeyType)>`
- [ ] Tests: known address round-trips, invalid checksum rejection

### 1.4 Binary Codec (`xrpl-core/src/codec/`)
> **Read `docs/CODEC.md` before touching this module.**
- [ ] `FieldId` struct: `type_code: u8`, `field_code: u8`
- [ ] `FieldId::encode(&self) -> Vec<u8>` — 1/2/3 byte header per spec
- [ ] `FIELD_REGISTRY` — static map of field name → `(type_code, field_code, is_vl_encoded)`
- [ ] `encode_vl(len: usize) -> Vec<u8>` — XRPL variable-length prefix
- [ ] `decode_vl(buf: &[u8]) -> Result<(usize, usize)>` — returns (length, bytes_consumed)
- [ ] `encode_uint8/16/32/64(v) -> Vec<u8>`
- [ ] `encode_hash128/160/256(v: &[u8]) -> Vec<u8>` — raw bytes, no prefix
- [ ] `encode_amount_xrp(drops: u64) -> [u8; 8]` — bit 63=0, bit 62=1, bits 0-61=drops
- [ ] `encode_amount_iou(...) -> [u8; 48]` — 8B mantissa + 20B currency + 20B issuer
- [ ] `encode_account_id(id: &AccountId) -> Vec<u8>` — VL-prefixed 20 bytes
- [ ] `encode_blob(data: &[u8]) -> Vec<u8>` — VL-prefixed
- [ ] `encode_object_end() -> u8` — `0xE1`
- [ ] `encode_array_end() -> u8` — `0xF1`
- [ ] `sort_fields(fields: &mut [(FieldId, Vec<u8>)])` — ascending by (type_code, field_code)
- [ ] `encode_transaction(tx: &Transaction) -> Result<Vec<u8>>`
- [ ] `decode_field_id(buf: &[u8]) -> Result<(FieldId, usize)>`
- [ ] Tests against `tests/vectors/codec-fixtures.json` from xrpl-codec-fixtures repo

### 1.5 Cryptography (`xrpl-core/src/crypto/`)
> **Read `docs/CRYPTO.md` before touching this module.**
- [ ] `KeyType` enum: `Ed25519`, `Secp256k1`
- [ ] `Seed` struct: `[u8; 16]` + `KeyType`
- [ ] `Seed::from_base58(s: &str) -> Result<Seed>`
- [ ] `Seed::generate() -> Seed` — 16 random bytes via `rand::thread_rng`
- [ ] `Keypair` struct: `public_key: Vec<u8>`, `private_key: Vec<u8>`, `key_type: KeyType`
- [ ] `Keypair::from_seed(seed: &Seed) -> Result<Keypair>`
  - **Ed25519**: derive via SLIP-10 using `ed25519-dalek`
    - `private_key` = 32-byte scalar
    - `public_key` = `0xED` + 32-byte compressed point
  - **Secp256k1**: XRPL root keypair derivation using `k256` crate
    - Root key = HMAC-SHA512(key=b"Root Deterministic Key", data=seed)
    - `public_key` = 33-byte compressed point
- [ ] `Keypair::sign(message: &[u8]) -> Vec<u8>`
  - Ed25519: 64-byte raw signature
  - Secp256k1: DER-encoded signature (variable, typically 70-72 bytes)
- [ ] `Keypair::account_id() -> AccountId` — SHA256 then RIPEMD-160 of public key
- [ ] `Keypair::verify(message: &[u8], signature: &[u8]) -> bool`
- [ ] `sign_transaction(tx: &mut Transaction, keypair: &Keypair) -> Result<()>`
  - 1. Set `SigningPubKey` field
  - 2. Encode tx with `SIGN_PREFIX = [0x53, 0x54, 0x58, 0x00]`
  - 3. Hash with SHA512-half (first 32 bytes of SHA512)
  - 4. Sign the hash
  - 5. Set `TxnSignature` field
- [ ] Tests: known seed→keypair→address vectors, sign+verify round-trip

### 1.6 Transaction Types (`xrpl-core/src/transaction/`)
> **Read `docs/TRANSACTIONS.md` before adding transaction types.**
- [ ] `CommonFields` struct (fields present on all transactions):
  `Account`, `Fee`, `Flags`, `LastLedgerSequence`, `Sequence`,
  `SigningPubKey`, `TxnSignature`, `TransactionType`, `SourceTag` (opt),
  `AccountTxnID` (opt), `Memos` (opt), `Signers` (opt)
- [ ] `TransactionType` enum with u16 discriminants per spec
- [ ] `Transaction` enum — one variant per transaction type
- [ ] **Priority transaction types** (implement in order):
  - [ ] `Payment`
  - [ ] `OfferCreate`
  - [ ] `OfferCancel`
  - [ ] `TrustSet`
  - [ ] `AccountSet`
  - [ ] `EscrowCreate` / `EscrowFinish` / `EscrowCancel`
  - [ ] `NFTokenMint` / `NFTokenBurn` / `NFTokenCreateOffer` / `NFTokenAcceptOffer`
  - [ ] `AMMCreate` / `AMMDeposit` / `AMMWithdraw`
- [ ] `Memo` struct: `MemoData`, `MemoFormat`, `MemoType` (all hex blobs)
- [ ] `Signer` struct for multi-sign: `Account`, `TxnSignature`, `SigningPubKey`
- [ ] `PathStep` struct: `account` (opt), `currency` (opt), `issuer` (opt)
- [ ] `impl Transaction { fn fee(&self) -> &Amount; fn sequence(&mut self, seq: u32); }`

---

## Phase 2 · `xrpl-client`

### 2.1 HTTP JSON-RPC Client (`xrpl-client/src/http.rs`)
> **Read `docs/CLIENT_HTTP.md` before touching this module.**
- [ ] `XrplHttpClient` struct: `base_url: Url`, `inner: reqwest::Client`
- [ ] `XrplHttpClient::new(url: &str) -> Result<Self>`
- [ ] Generic `request<Req, Resp>(&self, method: &str, params: Req) -> Result<Resp>`
  - POST to base_url, body: `{"method": method, "params": [params]}`
  - Check `result.status == "success"`, else map to `XrplError::RpcError`
- [ ] **Account methods:**
  - [ ] `account_info(account: &AccountId, ledger: LedgerIndex) -> Result<AccountInfo>`
  - [ ] `account_offers(account: &AccountId) -> Result<Vec<Offer>>`
  - [ ] `account_lines(account: &AccountId) -> Result<Vec<TrustLine>>`
  - [ ] `account_nfts(account: &AccountId) -> Result<Vec<NFToken>>`
  - [ ] `account_tx(account: &AccountId, params: AccountTxParams) -> Result<AccountTxResult>`
- [ ] **Ledger methods:**
  - [ ] `ledger(index: LedgerIndex) -> Result<LedgerInfo>`
  - [ ] `ledger_current() -> Result<u32>` — returns current ledger index
  - [ ] `ledger_closed() -> Result<LedgerClosedResult>`
- [ ] **Transaction methods:**
  - [ ] `submit(tx_blob: &str) -> Result<SubmitResult>` — base64 encoded
  - [ ] `submit_and_wait(tx_blob: &str) -> Result<SubmitAndWaitResult>`
  - [ ] `tx(hash: &Hash256) -> Result<TxResult>`
- [ ] **Order book / DEX:**
  - [ ] `book_offers(taker_pays: &Currency, taker_gets: &Currency) -> Result<Vec<Offer>>`
  - [ ] `amm_info(asset: &Currency, asset2: &Currency) -> Result<AmmInfo>`
- [ ] **Fee:**
  - [ ] `fee() -> Result<FeeResult>` — returns current base fee, open ledger fee
- [ ] `LedgerIndex` enum: `Validated`, `Current`, `Closed`, `Index(u32)`
- [ ] Response structs for all above — derive `Deserialize`

### 2.2 WebSocket Client (`xrpl-client/src/ws.rs`)
> **Read `docs/CLIENT_WS.md` before touching this module.**
- [ ] `XrplWsClient` struct with `tokio-tungstenite` connection
- [ ] `XrplWsClient::connect(url: &str) -> Result<Self>`
- [ ] Request/response ID tracking: `Arc<AtomicU64>` counter, `HashMap<u64, oneshot::Sender>`
- [ ] Background read loop task: routes responses to waiting senders, events to subscription channel
- [ ] `request<Req, Resp>(&self, command: &str, params: Req) -> Result<Resp>` — async, awaits reply
- [ ] **Subscriptions:**
  - [ ] `subscribe_ledger(&self) -> Result<impl Stream<Item=LedgerEvent>>`
  - [ ] `subscribe_transactions(&self) -> Result<impl Stream<Item=TransactionEvent>>`
  - [ ] `subscribe_account(account: &AccountId) -> Result<impl Stream<Item=AccountEvent>>`
  - [ ] `subscribe_order_book(taker_pays, taker_gets) -> Result<impl Stream<Item=BookEvent>>`
- [ ] Reconnection: exponential backoff, re-subscribe on reconnect
- [ ] Ping/pong keepalive (30s interval)
- [ ] `close(&self) -> Result<()>`
- [ ] All HTTP methods mirrored on WS client

### 2.3 Shared Response Types (`xrpl-client/src/types/`)
- [ ] `AccountInfo` — account_data, ledger_current_index, validated
- [ ] `LedgerInfo` — ledger_hash, ledger_index, close_time, txn_count
- [ ] `SubmitResult` — engine_result, tx_blob, tx_json
- [ ] `TxResult` — transaction, metadata, validated
- [ ] `TransactionMeta` — AffectedNodes, TransactionResult, delivered_amount
- [ ] `FeeResult` — base_fee, median_fee, open_ledger_fee (all in drops as strings)
- [ ] `Offer`, `TrustLine`, `NFToken`, `AmmInfo`
- [ ] `LedgerEvent`, `TransactionEvent`, `AccountEvent`, `BookEvent`

---

## Phase 3 · `xrpl-sdk` (Facade)

### 3.1 Wallet (`xrpl-sdk/src/wallet.rs`)
- [ ] `Wallet` struct: `keypair: Keypair`, `address: String`, `classic_address: AccountId`
- [ ] `Wallet::generate() -> Wallet`
- [ ] `Wallet::from_seed(seed: &str) -> Result<Wallet>`
- [ ] `Wallet::from_secret(secret: &str) -> Result<Wallet>` — alias for from_seed
- [ ] `Wallet::sign_transaction(&self, tx: &mut Transaction) -> Result<String>` — returns hex blob

### 3.2 Client Facade (`xrpl-sdk/src/client.rs`)
- [ ] `XrplClient` enum: `Http(XrplHttpClient)`, `Ws(XrplWsClient)`
- [ ] `XrplClient::http(url: &str) -> Result<Self>`
- [ ] `XrplClient::ws(url: &str) -> Result<Self>`
- [ ] All HTTP/WS methods delegated through enum dispatch

### 3.3 Transaction Builder (`xrpl-sdk/src/builder.rs`)
- [ ] `PaymentBuilder` — fluent API: `.to()`, `.amount()`, `.destination_tag()`, `.memo()`
- [ ] `OfferCreateBuilder`
- [ ] `TrustSetBuilder`
- [ ] All builders: `.build(sequence, fee) -> Transaction`
- [ ] `autofill(tx: &mut Transaction, client: &XrplClient) -> Result<()>`
  - Fetches sequence, fee, last_ledger_sequence automatically

### 3.4 Public API (`xrpl-sdk/src/lib.rs`)
- [ ] Re-exports: `Wallet`, `XrplClient`, all builder types
- [ ] Re-exports from core: `Amount`, `AccountId`, `Hash256`, `Currency`, `Transaction`
- [ ] Feature flags: `http` (default), `ws`, `full` = http+ws

---

## Phase 4 · Examples & Docs

### 4.1 Examples (`examples/`)
- [ ] `send_payment.rs` — generate wallet, fund from faucet, send XRP payment, confirm
- [ ] `subscribe_ledger.rs` — WS stream, print each ledger close
- [ ] `place_offer.rs` — OfferCreate on DEX, poll for fill
- [ ] `mint_nft.rs` — NFTokenMint with metadata URI
- [ ] `amm_deposit.rs` — deposit into AMM pool

### 4.2 Documentation
- [ ] `README.md` — quick start, installation, 3 code examples
- [ ] Doc comments on all public types and methods (`///`)
- [ ] `cargo doc --no-deps --open` produces no warnings
- [ ] `CHANGELOG.md` initialized at `0.1.0-alpha`

### 4.3 CI (`/.github/workflows/ci.yml`)
- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] `cargo fmt --check`
- [ ] Test against testnet: `wss://s.altnet.rippletest.net:51233`

---

## Dependency Reference
```toml
tokio          = { version = "1", features = ["full"] }
reqwest        = { version = "0.12", features = ["json"] }
tokio-tungstenite = { version = "0.24", features = ["native-tls"] }
serde          = { version = "1", features = ["derive"] }
serde_json     = "1"
thiserror      = "1"
hex            = "0.4"
sha2           = "0.10"
ripemd         = "0.1"
hmac           = "0.12"
ed25519-dalek  = { version = "2", features = ["rand_core"] }
k256           = { version = "0.13", features = ["ecdsa"] }
rand           = "0.8"
bs58           = { version = "0.5", features = ["check"] }
futures        = "0.3"
```
