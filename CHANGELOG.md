# Changelog

Every change that matters to someone using the SDK, newest first. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the crates
follow [Semantic Versioning](https://semver.org/) (before 1.0, a minor release
may break the API, and says so under "Changed (breaking)").

## [Unreleased]

### Fixed
- `KeyType::Ed25519`'s documentation called it `rippled`'s default for new
  keys; `wallet_propose` defaults to secp256k1. (`Wallet::generate` and
  xrpl.js default to Ed25519.)
- docs/PARITY.md said the 6 recorded batches with `BatchSigners` were on
  Testnet; they are on Devnet.
- **`BlockingClient` has every `Client` method**, as its documentation
  said: the 18 it lacked (`account_tx`, `account_objects`, `book_offers`,
  `amm_info`, `ledger_entry`, the `_all` variants, …) are added, and a test
  keeps the two in step.
- **`autofill` prices each transaction type as `rippled` does.** It charged
  every type one base fee (the open-ledger fee), so an `AccountDelete` or
  `AMMCreate`, which cost one owner reserve (0.2 XRP on mainnet), was
  refused (`telCAN_NOT_QUEUE`), and `submit_and_wait` resubmitted it until
  it expired; an `EscrowFinish` with a fulfillment and the Confidential MPT
  types were refused the same way. Now the fee is the type's base cost at the
  open-ledger fee: one owner reserve (scaled when the open ledger escalates)
  for `AccountDelete`, `AMMCreate` and `LedgerStateFix`, 32 + *n*/16 extra
  base fees for an *n*-byte fulfillment, 9 extra for Confidential MPT, and for
  a `LoanSet` one per signature its counterparty will add (its signer list's
  size, or one). A `Batch` is priced from each inner transaction's own cost.
  The boundaries are checked with `rippled`'s `simulate`
  (`scripts/probe_cost.py`). The fee cap still applies.
- **wasm: waiting works outside a browser window.** The wait between
  `submit_and_wait`'s and `fund_and_wait`'s polls, and the browser WebSocket
  client's request timeout, set their timer on `window`, which a Web Worker,
  a Service Worker, Node.js and Cloudflare Workers do not have: there they
  waited forever. The timer is now the JavaScript global's `setTimeout`. A
  CI job runs the client's wasm tests in Node.js to keep it so.

### Added
- **Sign typed transactions.** `Wallet::sign(&tx)` takes a typed model
  (`models::Payment`, …), a `models::Tx`, a builder (its `validate` runs
  first) or JSON, and returns a `SignedTransaction`: the signed JSON, the
  blob and the transaction's ID (`hash`), as xrpl.js's `wallet.sign` does.
  `sign_and_submit(&client, &tx, &wallet)` autofills a copy, signs, submits
  and waits for the final outcome. The forms are one trait,
  `transaction::ToTxJson`. The README's quick start and the
  `testnet_quickstart` example now use a typed `Payment`.
- `verify_transaction(&tx)`: whether a signed transaction's signatures
  verify as `rippled` checks them: the single signature, or each
  multi-signer's (in account order), and a `Batch`'s `BatchSigners`. Every
  signed mainnet transaction in the test corpus (179 single, 2 multi-signed)
  and every one `rippled` signed offline (79) verifies; none does with a
  changed field.
- `Client::balances(account, ledger)`: an account's XRP, trust-line
  balances and MPTs as typed `Amount`s, all read from one ledger.
- **HTTP client settings and polite retries.** `XrplHttpClient::builder(url)`
  sets the timeout, connect timeout, headers (a provider's API key; `Debug`
  shows header names, never values), user agent, fee cap and `RetryPolicy`.
  A request is sent again only when the server says it is busy and did not
  take it (HTTP 429 or 503, `rippled`'s `slowDown` or `tooBusy`), waiting
  what `Retry-After` asks (at most 10 s) or 1 s, 2 s; never after a timeout
  or a dropped connection, so a transaction is never sent twice by a retry.
  `XrplHttpClient::new` uses these defaults (two retries). An HTTP error
  status is now `ClientError::HttpStatus { status, retry_after, excerpt }`
  (it was `UnexpectedResponse` text), from the faucet too.
- **WebSocket client settings.** `XrplWsClient::builder(url)` sets the
  request and connect timeouts, the ping interval, the silence limit after
  which a connection is replaced, the per-subscription event buffer and the
  fee cap (they were fixed). `XrplWsClient::connect` keeps the defaults.
- **Read any number of pages.** `Client::pages(request)` streams the
  pages of a paginated request one at a time with no page limit (an
  account's whole history, a list longer than 50 pages), each from the first
  page's ledger; `BlockingClient::pages` is the same as an iterator.
  `request_all` keeps its 50-page limit, and its error now points to
  `pages`.
- **Ledger entry flags by name.** Each typed ledger entry with flags has
  its `rippled` 3.4.1 flags as constants (`AccountRoot::LSF_DEFAULT_RIPPLE`,
  `RippleState::LSF_LOW_FREEZE`, `Offer::LSF_SELL`, … 51 in all, from
  `server_definitions`' `LEDGER_ENTRY_FLAGS`), `ALL_FLAGS`, and `has_flag`.
  Every flag on the mainnet entries in the test sample is a named one.
- **Standard traits.** `XrplHttpClient` is `Clone` (clones share one
  connection pool). The HTTP, WebSocket (native and wasm) and blocking
  clients are `Debug`, showing the server's host only, never a URL that may
  carry a token. `Subscription::into_stream` gives the events as a
  `futures::Stream`. `Hash128`/`160`/`192`/`256` are `FromStr` (hex),
  `Ord` and `AsRef<[u8]>`. `AccountId` is `Ord` (by its bytes, the order
  `rippled` sorts multi-signers in) and `AsRef<[u8]>`. `KeyType` is
  `Display`, `FromStr`, `Hash` and serde, written as `rippled` writes
  `key_type`. Every public type is `Debug` (the
  `missing_debug_implementations` lint is on), including the 28 transaction
  builders.
- `transaction::cost::transaction_cost`: a transaction's base cost (base
  fees and owner reserves) as `rippled` 3.4.1's `calculateBaseFee` computes
  it, for pricing a transaction offline.
- **Open a wallet from the backup another wallet gave you.**
  - `Wallet::from_mnemonic(phrase)`: a BIP-39 recovery phrase (12 to 24
    words, as Ledger devices and multi-coin wallets show) at
    `m/44'/144'/0'/0/0`, the first XRP account on Ledger and in xrpl.js.
    `Wallet::from_mnemonic_with(phrase, passphrase, &path)` takes a
    passphrase and another path (`DerivationPath::xrp_account(n)`).
  - `Wallet::from_secret_numbers(numbers)`: Xaman's eight blocks of six
    digits (XLS-12), secp256k1 as Xaman makes them;
    `from_secret_numbers_with_type` for Ed25519. A block above 65535 is
    refused (xrpl.js reads 99999 as 0x869F).
  - `Wallet::from_rfc1751(words, key_type)`: the twelve words `rippled`'s
    `wallet_propose` returns as `master_key`.
- In `xrpl_core::crypto`: `mnemonic::Mnemonic` (parse, generate, from
  entropy, `to_seed` with an NFKD passphrase), `hd::DerivationPath` (BIP-32
  secp256k1 derivation), `secret_numbers` and `rfc1751` encoders,
  `Seed::{from,to}_secret_numbers`, `Seed::{from,to}_rfc1751` and
  `Keypair::from_mnemonic`. Errors name a word's or block's position, never
  the word. New `CoreError::InvalidMnemonic` and
  `CoreError::InvalidDerivationPath`.
- Checked against the 24 BIP-39 reference vectors (with their BIP-32 master
  keys), the 17 private BIP-32 reference vectors, xrpl.js's wallet tests and
  `rippled` 3.4.1's `wallet_propose` for 23 seeds with both key types.
  `rippled` writes some RFC 1751 keys it cannot read back (the word `YOU`;
  about one key in 170); this crate reads them.

### Docs and project
- docs.rs labels each feature- or platform-gated item with what it needs
  (`doc_cfg`); CI builds the docs as docs.rs does (nightly, `--cfg docsrs`).
- CONTRIBUTING.md, issue forms (bug, feature; security reports go to the
  private channel) and a pull-request checklist.
- CI: tests on Linux, macOS and Windows; a SemVer check against the latest
  release (`cargo-semver-checks`); a `cargo package` dry run; the live
  Testnet job can fail again (it was `continue-on-error`).
- The request types' `account` fields, and `Signer`, `SignerEntry`,
  `BatchSigner` and `PathStep`'s, had one pasted description ("the account
  that sends the transaction or owns the entry"); each now says what it is.
- CHANGELOG.md follows Keep a Changelog, with the v0.3.0 and v0.2.0
  headings it had lost (their notes ran on under v0.4.0) and compare links.

## [v0.6.0] (2026-10-23)

Security. A full adversarial review of every crate found ways the SDK could
sign something other than what the caller wrote, report a payment that went
through as failed (inviting a second payment), let a server set any fee, and
leak secrets into logs. All are fixed here, each with a test; most are checked
against `rippled` 3.4.1's own answers. **Upgrade from 0.5 and earlier.** The
API changes this needs are listed under "Changed (breaking)"; most code that
compiles against 0.5 needs only a few edits.

### Security
- **The codec no longer changes the transaction you wrote.**
  - A field name `rippled` does not define is now an error
    (`CoreError::UnknownField`); it was dropped from the signed transaction,
    so `"destination_tag"` or `"DestinationTAg"` sent a payment to an
    exchange without its tag. This holds at the top level, inside inner
    objects and as an array element's name. The keys `rippled` adds to its
    responses (`hash`, `meta`, `ledger_index`, …) are still skipped.
  - A UInt8/16/32 value too large for its field is now an error; it was
    wrapped (`DestinationTag` 4294979641 was signed as 12345, `TransferFee`
    65536 as 0). Decimal strings are read as `rippled` reads them; `"0x…"`
    strings, which `rippled` refuses, are refused.
  - A token value with 19 or more digits could be read with the wrong sign
    and magnitude (`"9999999999999999999"` became negative), `""`, `"-"`,
    `"."` and `"e5"` read as zero, and digits past the sixteenth were cut
    instead of rounded. Token values now take `rippled`'s number syntax and
    round as it does (to nearest, ties to even), checked on 26 values and 16
    refusals against `rippled`'s `simulate`; any number of digits is read
    exactly.
  - The builders' common fields wrote `AccountTxnId` (serde PascalCase)
    instead of `AccountTxnID`, which the old codec dropped silently.
- **`autofill` caps the fee.** It signed whatever `open_ledger_fee` the
  server named, so a dishonest server (or a man in the middle on `http://`)
  could make a transaction burn the account's balance. Above the client's
  cap (2 XRP by default, as in xrpl.js; `with_max_fee_drops` on every
  client) it returns `ClientError::FeeTooHigh` and changes nothing.
- **`submit_and_wait` answers only with final outcomes.** It could report a
  transaction that went through as failed or expired, the usual cue to send
  it again: a preliminary `tec` was taken as final; `terQUEUED` was
  resubmitted and the resubmit's `tef` reported as "never reaches a ledger";
  expiry was decided from the open ledger before the transaction was looked
  up. And a validated `tec` came back as `Ok`. Now the transaction ID is
  computed locally; `Ok` means validated `tesSUCCESS`;
  `TransactionClaimed { result: Some(..) }` a validated `tec`;
  `TransactionExpired` only once the `LastLedgerSequence` is validated and
  the server has searched every ledger up to it; and the new
  `ClientError::OutcomeUnknown { hash, .. }` says "look it up before
  replacing it". A transaction the server dropped is resubmitted as the
  same blob.
- **`hxrpl` asks the server which network it is on** before it submits
  anything; anywhere but Testnet or Devnet needs `--yes`. It checked only
  `--network`, so `--url`/`XRPL_URL` pointing at a mainnet server sent real
  XRP without `--yes`, and `raw submit` had no guard.
- **secp256k1 `sign` / `verify` take exactly a 32-byte hash.** The ECDSA
  primitive used only the first 32 bytes of a longer input, so
  `Keypair::sign` / `verify` on a secp256k1 key accepted a signature over one
  message for any message with the same first 32 bytes. Transaction signing
  was not affected (it hashes first).
- **Secrets stay out of logs and errors.** `Seed` and `Keypair` derived
  `Debug`, so `{:?}` printed the secret bytes; seed decoding errors carried a
  seed character and a checksum of the secret. WebSocket and HTTP logs and
  transport errors named the URL, which can carry an API key; they name the
  host only.
- **The WebSocket client routes each subscription its own events.** Every
  receiver got every subscription's events, so a per-account deposit handler
  could credit the wrong account. `subscribe_account(a)` now hears only
  validated transactions naming `a`.

### Fixed
- **WebSocket client**: dropping it now closes the connection (the
  reconnect task kept the socket, keepalive and resubscriptions alive
  forever); a connection silent for 45 s is replaced (a laptop that slept or
  a NAT that dropped it left the client "connected" with streams quiet);
  resubscriptions after a reconnect are checked and a refusal reported
  (`WsEvent::ResubscribeFailed`); `WsEvent::Disconnected` marks the gap; a
  receiver that falls behind gets `WsEvent::Missed(n)` instead of an error
  that ended the usual `while let Ok(..)` loop; backoff no longer overflows
  after 64 attempts; 15 s connect timeout. In the browser: subscriptions
  deliver their events (they were dropped), requests time out after 30 s,
  a closed connection fails pending requests, dropping the client closes it.
- **Multi-signing**: `sign_for_multisigning` signs an empty `SigningPubKey`
  as `rippled`'s `sign_for` does (a missing one gave signatures that failed);
  `collect_signers` verifies every signature, refuses a repeated signer,
  more than 32, or a `TxnSignature`; `sign_transaction` refuses a
  transaction with `Signers`.
- **Batch**: `Flags`/`Sequence` given as decimal strings were read as 0
  (inner flags lost; every `BatchSigner` signature invalid);
  `verify_batch_signers` said yes with signers missing or empty; the fee
  ignored an outer transaction's own signers. New `required_batch_signers`
  (rippled's rule: inner initiators, counterparties and sponsors, minus the
  outer account) and `sign_batch_as` (sign through a regular key).
- **Transaction IDs**: `hash_signed_tx_blob` hashes `rippled`'s
  re-serialization, so a blob with reordered fields has the ID the ledger
  shows; `hash_signed_tx` of API v2 JSON (`DeliverMax`) hashed a Payment
  with no `Amount`.
- **Typed parsing never hides a failure**: a known transaction or ledger
  entry that does not parse is `Tx::Unparsed` / `LedgerObject::Unparsed`
  (with the error), not `Other`; `Tx::from_value` is strict for Batch inner
  transactions and refuses differing `Amount`/`DeliverMax`;
  `LedgerObject::from_value` refuses fields its type does not have.
- **Codec**: a two-key array element, a token in XRP's currency code, an
  `Issue` with fields its kind lacks, and an over-long VL length (its prefix
  was clamped while the payload was written in full) are refused; field
  headers are read in their shortest form only, as `rippled` reads them;
  inner objects are signed whole, as `STObject::add` writes them; currencies
  are canonical (`"USD"` given as 40 hex digits equals `"USD"`; `"XRP"` is
  the all-zero code; `"1"` is `rippled`'s noCurrency); hand-built amounts out
  of canonical form are refused, not bit-masked.
- **Paginated calls** (`*_all`) read one ledger: every page after the first
  asks for the first page's ledger (markers are only good there).
- **autofill**: `Sequence` 0 with a `TicketSequence`; `NetworkID` on networks
  above 1024; `LastLedgerSequence` current + 20 (was + 4); an X-address for
  the other kind of network (Testnet vs mainnet) is refused.
- **Builders**: AMMDeposit/AMMWithdraw set the mode flag `rippled` requires
  (inferred from the fields given) and `build()` checks what `rippled`'s
  preflight checks for AccountSet, SignerListSet, SetRegularKey,
  DelegateSet, Clawback (now with `Holder` for MPTs) and the AMM types.
- **Utilities**: reserves are computed in exact drops (`1.001` XRP was
  1,000,999 drops; NaN made the whole balance spendable); book liquidity is
  exact, counts funded amounts and documents its units; float helpers
  return `None` instead of inf/NaN.
- **Faucet**: a timeout; the status is checked before the body is parsed;
  `fund_and_wait` waits for the balance to grow (an existing account counted
  as funded); `amount_xrp` is `"100"`, not `"\"100\""`.
- **CLI**: a failed transaction prints its result and exits 1; `send-xrp`
  shows amount, destination, tag and fee first; `--seed-out FILE` (mode
  0600, never overwritten); a world-readable `--seed-file` is warned about;
  `watch --url https://…` is an error instead of a silent switch.
- **Other**: HTTP response bodies are capped (128 MiB); base58 strings over
  128 characters are refused before decoding; seeds come from the OS random
  source (not reseeded-on-fork thread RNG); `BlockingClient` inside an async
  runtime returns `ClientError::InsideAsyncRuntime` instead of panicking;
  `unix_to_ripple_time` does not overflow; an unknown engine-result prefix,
  or a WebSocket reply with neither result nor error, is no longer `Ok`.

### Added
- Keylets for every remaining entry type, each matching `rippled` 3.4.1's
  computed index: `bridge`, `xchain_owned_claim_id`,
  `xchain_owned_create_account_claim_id`, `deposit_preauth_credentials`,
  `sponsorship`, `book_base` / `book_directory`, `nftoken_page_min` / `max`.
- `utils::delivered_amount`, `TxResult::delivered_amount`,
  `TransactionEvent::delivered_amount`: what a payment actually delivered.
- `Client::autofill_for_multisigning(tx, signers)`,
  `Client::validated_ledger_index`, `Client::max_fee_drops`,
  `with_max_fee_drops` on every client; `DEFAULT_MAX_FEE_DROPS`,
  `LAST_LEDGER_OFFSET`.
- `Keypair::sign_message` / `verify_message` / `from_private_key`;
  `crypto::verify_signing_data`; `Seed::from_bytes`.
- `Currency::parse`, `Currency::from_code`, `Currency::no_currency`;
  `utils::reserve::reserve_xrp_to_drops`;
  `transaction::batch::{required_batch_signers, sign_batch_as}`;
  `Wallet::sign_batch_as`; `TxCommon::{network_id, delegate}`.
- `docs/CRYPTO.md` (keys, signing, verification, secret handling), a rewritten
  `SECURITY.md` with a private reporting channel, `scripts/probe_iou.py`.
- Clippy denies narrowing `as` casts in every crate.

### Changed (breaking)
- `Client::submit_and_wait(tx_blob)` takes no `last_ledger_sequence` (it is
  read from the blob, which must carry one) and has the outcomes above;
  `BlockingClient::submit_and_wait` likewise. `ClientError` has new variants
  (`OutcomeUnknown`, `FeeTooHigh`, `InsideAsyncRuntime`), `RpcError` a
  `response` field, `TransactionClaimed` a `result` field, and
  `ClientError` / `CoreError` are `#[non_exhaustive]`.
- `XrplWsClient::subscribe_*` and `events()` return a `Subscription`
  (`recv()` gives `Option<WsEvent>`: `while let Some(event) = sub.recv().await`);
  `WsEvent` has `Unparsed`, `Disconnected`, `ResubscribeFailed` and `Missed`.
- `Seed`, `Keypair` and `Wallet` fields are private: use `as_bytes()`,
  `key_type()`, `public_key()`, `private_key()`, `keypair()`, `account_id()`,
  `classic_address()`. `Seed::to_base58` and `Wallet::seed` return
  `Zeroizing<String>` (`.as_str()` to print); `ed25519::derive_keypair` /
  `secp256k1::derive_keypair` return the secret key as `Zeroizing<Vec<u8>>`.
- The codec refuses unknown fields, out-of-range integers, `"0x"` UInt32
  strings and the other inputs listed above. `encode_vl`, `encode_iou_value`
  and `encode_amount_iou` return `Result`.
- `IouAmount::from_decimal` / `new` round instead of truncating, and give
  zero below 10^-81.
- `Tx` and `LedgerObject` have an `Unparsed` variant.
- `utils::reserve` takes drops (`total_reserve_drops(base_drops, inc_drops,
  owners)`); `dex::liquidity_at_price(offers, max_quality: &str) -> String`;
  `offer_quality_xrp` and `spread_percent` return `Option<f64>`.
- `hxrpl`: `wallet faucet` takes `--seed-out`; `raw` takes `--yes`.

### Deprecated
- `Keypair::sign` / `Keypair::verify` (use `sign_message` / `verify_message`),
  `transaction::CommonFields` and `transaction::TransactionType` (use
  `transaction::models`).

### Docs
- `docs/PARITY.md`: the encoder's refusals, token values, the new keylets and
  two more deliberate differences. The design notes from March 2026 moved
  to `docs/design/` and are marked historical (they described signing that
  `rippled` rejects for Ed25519).

## [v0.5.0] (2026-10-22)

Trust: fuzzing in CI, benchmarks, a minimum Rust version, and a parity report.

### Fixed
- **`Number` (`STNumber`, the decimal type of vaults and loans) now follows `rippled` 3.4.1
  exactly**, checked value by value against `rippled`'s `simulate`:
  - Bytes not in normal form decode to the text `rippled` shows: mantissa 13830756765796644,
    exponent 1 reads `"13830756765796644e1"` (was `"138307567657966440"`), and exponents below
    -32,768 read `0`. Found by the new round-trip fuzz target.
  - Small decimals drop trailing zeros (`"0.0000000123"`), and `"999999999999999999"` no longer
    gains an `e0`.
  - Encoding refuses what `rippled` refuses instead of rounding it: more than 19 significant
    digits, digits beyond 64 bits, forms such as `"00012"`, `".5"` and `"5."`, exponents out of
    range, and JSON integers beyond 32 bits. 32-bit JSON integers are now accepted, as `rippled`
    accepts them.

- **Amounts follow `rippled`'s binary rules** (also checked against `simulate`):
  - A negative XRP amount decodes as `"-100"`; it decoded without its sign. Negative zero is
    refused. `decode_amount_xrp` refuses negative amounts (`Amount::Xrp` holds drops ≥ 0); the
    new `decode_xrp_signed` and `encode_xrp_signed` read and write the sign.
  - A token amount is decoded only in canonical form (mantissa 10^15 to 10^16 - 1, exponent -96
    to 80, zero as `0x8000000000000000`), and never with XRP's currency or issuer. Found by the
    round-trip fuzz target.
  - XRP drops in JSON are read with `rippled`'s number syntax: `"-100"`, `"+100"`, `"1e2"` and
    32-bit JSON integers are accepted; `"0100"` is refused.
  - XRP amounts are limited to 2^61 - 1 drops (bit 61 marks an MPT amount); 2^61 to 2^62 - 1
    were encoded as bytes `rippled` reads as an MPT.
  - MPT amounts: a magnitude above `i64::MAX` is refused when decoding (found by the round-trip
    fuzz target; `rippled` cannot write one as JSON); values are read with `rippled`'s number
    syntax (`"1e3"`, `"-0"`; not `"0100"` or `"1.5"`); `mpt_issuance_id` must be 24 bytes (a
    shorter or longer one produced malformed bytes).
- **`halcyon-xrpl-core` builds without default features again** (`no_std` + `alloc`):
  `crypto::sign_transaction` was re-exported without its `std` gate. CI now builds it.

- **Payment paths follow `rippled`'s binary rules** (checked against `simulate`): an empty path
  or PathSet and unknown path step bits are refused when decoding; an empty step (`{}`), which was
  encoded as the PathSet's end marker and corrupted the rest of the transaction, is refused when
  encoding. Found by the round-trip fuzz target.

- **Objects and arrays follow `rippled`'s reader**: a duplicate field, an end-of-array marker
  inside an object, an end-of-object marker inside an array, an array element that is not an
  object, and bytes after a transaction's end-of-object marker are refused. Each decoded into
  odd JSON (a duplicate silently replaced the first). Found by the round-trip fuzz target.
- `Issue` follows `rippled`'s reader: a token issue with an all-zero issuer is refused, and bytes
  marking an MPT issue without the 4-byte sequence are refused (they decoded as a token issue).
- `Number`: at the top exponent, a mantissa above `kMaxRepUp` (9,223,372,036,854,775,810) is
  refused, as in `rippled`; one at that limit is accepted when encoding.

### Changed (breaking)
- `PathStep` (in `transaction::models` and `xrpl_client::types`) has a new field,
  `mpt_issuance_id`: `rippled` 3.4.1 path steps can convert into an MPT (type 64), and the codec
  now reads and writes them.
- The codec refuses input that `rippled` refuses instead of rounding or guessing: `Number` values
  it cannot hold exactly, non-canonical amounts, empty paths and empty path steps (details
  above). `decode_amount_xrp` refuses a negative amount.

### Changed
- **secp256k1 signing is about twice as fast**: it signs from the secret scalar instead of first
  deriving the public key (which `k256`'s `SigningKey` does on every construction). Signatures
  are unchanged, byte for byte (RFC 6979; checked against `rippled`'s `sign`).

### Added
- **`rust-version`**: `halcyon-xrpl-core` 1.85, the other crates 1.88, checked in CI with the
  locked dependencies.
- **Fuzzing in CI** (`.github/workflows/fuzz.yml`): eight targets, four of them new
  (`fuzz_roundtrip`, `fuzz_models`, `fuzz_hashes`, `fuzz_utils`). They run a minute each on every
  pull request and 15 minutes each weekly, from a corpus seeded with real XRPL data
  (`fuzz/seed_corpus.py`).
- **Benchmarks against xrpl-rust** on the same inputs (`benchmarks/`, results in
  `docs/BENCHMARKS.md`).
- **`docs/PARITY.md`**: every check against `rippled` 3.4.1, with its numbers and how to repeat
  it. `docs/TESTING.md` rewritten to match the tests as they are.
- `crates/xrpl-core/scripts/probe_number.py`: asks a `rippled` node how it reads and writes
  `Number` values.
- The XRPLF codec fixtures are now checked in both directions (bytes → JSON as well as
  JSON → bytes).

## [v0.4.0] (2026-10-22)

Developer experience: a cookbook, the command line, a blocking client, typed streams.

### Added
- **`hxrpl`, the command line** (new crate `halcyon-xrpl-cli`): wallets (new, show, faucet), account
  queries, transactions (get, hash, decode, encode, sign, simulate, submit, send-xrp), ledger
  headers and entries, ledger entry IDs, fee, server, order books, live streams (`watch`),
  conversions, and any method (`raw`). Testnet by default; mainnet sends need `--yes`; seeds only
  from `XRPL_SEED` or `--seed-file`; long currency codes (`RLUSD`) are converted to hex.
- **A cookbook of 12 runnable recipes** (`crates/xrpl-sdk/examples`), each with its own faucet
  wallets, each run on Testnet: issuing a token, an NFT sale with royalty, a DEX order, an AMM,
  escrow, checks, a payment channel, multi-signing, Batch, MPTs, the blocking client, and live
  mainnet streams.
- **Typed WebSocket stream events**: `WsEvent::{Validation, Server, Consensus, BookChanges,
  Manifest, PeerStatus, PathFind, Other}` beside `Ledger` and `Transaction` (whose transaction is
  now a typed `Tx`), checked against messages recorded from mainnet. `XrplWsClient::subscribe` /
  `unsubscribe` for any parameters, and `subscribe_validations`, `subscribe_server`,
  `subscribe_consensus`, `subscribe_book_changes`, `subscribe_manifests`,
  `subscribe_transactions_proposed`, `subscribe_order_book` (with a typed `OrderBookSnapshot`).
- **`BlockingClient`**: every `Client` method as a plain call on its own runtime, plus `run` /
  `block_on` for anything else.
- `Amount::token("10", "USD", issuer)` (long codes become hex) and `Amount::mpt(value, id)`.

### Changed (breaking)
- `WsEvent` is `#[non_exhaustive]` and has new variants; `TransactionEvent::transaction` is a `Tx`.
- Flag constants split acronyms: `TF_MPT_CAN_TRANSFER` (was `TF_MPTCAN_TRANSFER`), `TF_LP_TOKEN`,
  `TF_MPT_LOCK`, …; `NFTOKEN` stays one word (`TF_SELL_NFTOKEN`).
- Transaction models: `Common::date` and `OfferCreate::owner_funds`, which rippled adds in streams
  and the codec ignores.
- The examples `send_payment`, `mint_nft` and `place_offer` (which needed manual funding) are
  replaced by the cookbook.

## [v0.3.0] (2026-10-22)

Every transaction type typed, and Batch.

### Added
- **`transaction::models`: a typed struct for each of the 82 transaction types in `rippled` 3.4.1**,
  generated from its `server_definitions` (`scripts/gen_transactions.py`): the common fields in
  `Common`, then each type's own fields under `rippled`'s names (334 in all), `new` with the
  required ones, and every flag as a constant (`Payment::TF_PARTIAL_PAYMENT`,
  `AccountSet::ASF_DEFAULT_RIPPLE`, `Batch::TF_ALL_OR_NOTHING`: 100 constants). Nested arrays are
  typed (`Memo`, `Signer`, `SignerEntry`, `PriceData`, `Permission`, `CredentialSpec`,
  `BatchSigner`, inner `Tx`s), as are `Asset`, `XChainBridge` and `PathStep`.
- **`Tx`** over all of them: `Tx::from_value` is strict (a field the type does not have is an
  error), deserializing inside a larger value is lenient (`Tx::Other`); `TxModel::to_json` gives
  `rippled`'s JSON. A Payment read from API v2 takes its `Amount` from `DeliverMax`.
- **`transaction::batch` (XLS-56)**: `inner_transaction`, `inner_transaction_ids`,
  `batch_signing_data`, `sign_batch` / `sign_batch_for` / `multisigned_batch_signer`,
  `add_batch_signers` (in the account-ID order `rippled` requires), `verify_batch_signers`,
  `batch_fee`; `Wallet::sign_batch` / `sign_batch_for`.
- `Client::autofill` handles a `Batch`: it marks each inner transaction, numbers those without a
  sequence (the outer account's continue after the outer `Sequence`), and prices the batch.
- Tests: 186 real transactions (51 types, mainnet and Testnet, including the SetFee of the
  December 2024 reserve change) round-trip to the same JSON, bytes and ID; an example of each of
  the other types, signed offline by `rippled` 3.4.1, encodes to its exact `tx_blob` — all 82
  types; batch signatures verify against 6 real batches; the live Testnet test (29 checks) runs a
  one-account and a two-account batch.

## [v0.2.0] (2026-10-22)

The first release on crates.io, as `halcyon-xrpl-core`, `halcyon-xrpl-client` and
`halcyon-xrpl-sdk`. v0.1.0 (below) was released on GitHub only.

### Typed requests

#### Added
- **A typed request and response for every public `rippled` method and the Clio-only ones**: 43
  requests (25 new), grouped as on xrpl.org. New: `noripple_check`, `ledger_closed`, `ledger_data`,
  `simulate`, `submit_multisigned`, `transaction_entry`, `book_changes`, `deposit_authorized`,
  `get_aggregate_price`, `nft_buy_offers`, `nft_sell_offers`, `ripple_path_find`, `channel_verify`,
  `feature`, `manifest`, `server_definitions`, `server_state`, `version`, `ping`, `random`,
  `vault_info`, and Clio's `nft_info`, `nft_history`, `nfts_by_issuer`, `mpt_holders`. Every request
  has a `new` with its required parameters and all of `rippled`'s optional ones as fields; the ones
  that page implement `Paginated`.
- Response types model every field `rippled` sends (`account_flags`, `queue_data`, AMM auction and
  vote slots, the whole of `server_info` / `server_state`, …) and serialize back to its JSON.
- `tests/rippled_responses.rs`: 44 recorded responses (mainnet `rippled` 3.4.1 and Clio, Testnet);
  each must parse, and its typed form must keep all of its fields (2,521 field paths).
- `Client::simulate` and `Client::submit_multisigned`.
- The live Testnet test sends a multi-signed transaction and checks the typed `account_channels`,
  `nft_sell_offers`, `get_aggregate_price` and (Clio) `mpt_holders` answers; with
  `XRPL_RECORD_DIR` it records them.

#### Fixed
- `ledger` failed against `rippled`: API version 1 writes the header's `ledger_index` as a string.

#### Changed
- Response types derive `Serialize` and `PartialEq`; several gained fields (e.g. `AccountInfo`,
  `ServerInfo`, `AmmInfo`). `LedgerEntryResult::node` is an `Option` (absent with `binary: true`);
  `AmmInfoRequest`'s assets are `Option`s (a pool can be named by `amm_account`);
  `AccountTxResult` gained `ledger_index_min` / `ledger_index_max`.
- `LedgerIndex` is `Copy` and defaults to `Validated`.

### Foundations

#### Changed (first crates.io release)
- **The crates are published as `halcyon-xrpl-core`, `halcyon-xrpl-client` and
  `halcyon-xrpl-sdk`** (`xrpl-sdk` is taken on crates.io). The library names stay `xrpl_core`,
  `xrpl_client` and `xrpl_sdk`, so `use` lines do not change; a path dependency needs
  `package = "halcyon-xrpl-core"` (etc.).
- `definitions.json` moved into the crate (`crates/xrpl-core/src/codec/`) so the published crate
  carries it; `codec::definitions::DEFINITIONS_JSON` is public. tests/vectors/definitions.json is a
  symlink to it.
- `submit` sorts `tel` results: they are neither applied nor relayed, so they used to wait for a
  validation that never came. `telINSUF_FEE_P` and `telCAN_NOT_QUEUE*` are retries (resubmitted by
  `submit_and_wait`); the other `tel`s are failures.

#### Added (utilities)
- `hashes`: `hash_signed_tx`, `hash_signed_tx_blob`, `transaction_id`, and `keylet::*`, the ID of
  every ledger entry type in rippled 3.4.1.
- `crypto::{sign_payment_channel_claim, verify_payment_channel_claim}` and
  `Wallet::sign_payment_channel_claim`.
- `utils`: `xrp_to_drops` / `drops_to_xrp`, ledger time ↔ Unix / ISO 8601 / `SystemTime`,
  `str_to_hex` / `hex_to_str`, TransferRate and quality conversions, `parse_nftoken_id`,
  `get_nftoken_id`, `get_balance_changes` — exact decimal arithmetic throughout.
- `faucet` (Testnet / Devnet) and `generate_faucet_wallet`; `Wallet::seed`.
- A live Testnet test (`--features live-tests`) that checks keylets, hashes, NFTokenIDs, balance
  changes and channel claims against what the network records.

#### Fixed (found by the live Testnet test)
- `fee` (and so `autofill` and `submit_transaction`) failed against real rippled servers: rippled
  sends `current_ledger_size`, `current_queue_size`, `expected_ledger_size` and `max_queue_size` as
  strings. Both forms are accepted now; `max_queue_size` and `levels` are added.
- The codec took a UInt64 only as exactly 16 hex digits. It now takes what rippled does: a JSON
  integer, or hex of any length that fits, and base 10 for `MaximumAmount`, `OutstandingAmount`,
  `MPTAmount` and `LockedAmount` (an MPTokenIssuanceCreate's `MaximumAmount` was refused).

#### Removed
- `codec::field::{FieldCode, FIELD_DEFS, lookup_field, lookup_field_by_id}`: an unused, stale copy of
  the field registry. `codec::definitions` (`lookup_field_def`, `lookup_field_def_by_id`) is the one
  the codec uses.

#### Added
- Typed ledger objects: `types::ledger_object` has a struct for each of rippled 3.4.1's 31 ledger entry
  types, generated from its `LEDGER_ENTRY_FORMATS` (`scripts/gen_ledger_objects.py`), and
  `LedgerObject` over them (unknown types → `Other`). 1,207 mainnet entries of 15 types parse typed
  and serialize back to exactly rippled's JSON; a 20-entry sample is a fixture test.
  `account_objects` / `account_objects_all` / `ledger_entry` return them (breaking).
- `Amount::Mpt` and `Hash192`; typed amounts in `AccountOffer`, `BookOffer` (+ `taker_*_funded`) and
  `AmmInfo` (breaking: they were `serde_json::Value`), checked against mainnet responses.
- Int32 fields (LoanScale, RemainingOwnerCountDelta): encode/decode as rippled's STInteger<int32_t>.
- PermissionValue decodes to rippled's names (granular permission, else TxType + 1).
- X-addresses (XLS-5d): `address::{encode_x_address, decode_x_address, classic_to_x_address,
  x_address_to_classic}`, checked against xrpl.js's 22 vectors; the codec takes X-addresses in
  Account/Destination (tag → SourceTag/DestinationTag), as xrpl.js does.
- `TypeCode::{Int32, Int64}` (a breaking change for exhaustive matches on `TypeCode`).

#### Changed
- **xrpl-client: one `Client` trait over HTTP, WebSocket and wasm WebSocket** (breaking: `use
  xrpl_client::Client;` to call the RPC methods). A transport implements `raw_request`; every
  method, `autofill`, `submit_and_wait` and the `_all` paginators are written once. New typed layer:
  `requests::*` (each `Request` names its method and response type), `Client::request`,
  `Client::request_all` for any `Paginated` request. The WS client's `request(command, params)` is
  now `raw_request`; `autofill_and_sign` / `submit_transaction` take any `Client`.
- WebSocket `submit` now sorts engine results like HTTP did (`tem`/`tef` failed, `ter` retry, `tec`
  claimed) and `submit_and_wait` resubmits on `ter`; the wasm client gains every method.
- Paginated requests omit `marker` on the first page instead of sending `"marker": null`.
- `LedgerIndex` serializes as rippled expects (it serialized unit variants as `null`).
- An HTTP error body is cut at 200 characters, not 200 bytes (which could split a UTF-8 character
  and panic).
- The wasm build compiles again (getrandom 0.3's `wasm_js` backend); CI lints it.
- tests/vectors/definitions.json is rippled 3.4.1's `server_definitions`: 44 fields, 7 transaction
  types, 1 ledger entry type and 8 results added; the Hooks-only fields dropped; UInt32 nth 53 is
  `ImmutableFlags` (was `MutableFlags`); type names Hash384/Hash512.
- Decoding a field id the definitions do not know is an error (`CoreError::UnknownField`), as in
  rippled. It used to be skipped, with a guessed length that was wrong for AccountID, PathSet, MPT
  amounts and nested objects, and the field vanished from the JSON.

#### Fixed
- Ed25519 signing: transactions and multisign Signer entries were signed over SHA512Half of the signing
  data; rippled verifies Ed25519 over the data itself, so every Ed25519 signature was rejected.
  New `Keypair::sign_signing_data` signs the way rippled verifies (secp256k1 signing is unchanged).
- `collect_signers` orders Signers by 20-byte AccountID (base58 text order differs, and rippled rejects
  an unsorted Signers array); it now returns `Result`.
- docs/TESTING.md: the two known-address vectors were wrong; replaced by xrpl.js's fixtures.

## [v0.1.0] (2026-03-19)

Initial release.

### xrpl-core
- Binary codec: encode/decode XRPL transactions (33/34 codec fixtures pass)
- Ed25519 and Secp256k1 key derivation and signing
- XRPL primitive types: AccountId, Amount, IouAmount, Currency, Hash128/160/256, Blob
- Base58check address encoding/decoding
- Transaction type enum (24 transaction types)
- Typed transaction builders with validation for all 24 transaction types
- Reserve calculation utilities (base_reserve, owner_reserve, available_balance)
- DEX utilities (amount_to_f64, offer_quality, midpoint_price, spread, liquidity)
- Multi-signing: encode_for_multisigning
- `no_std` support for types and crypto (codec requires std)

### xrpl-client
- HTTP JSON-RPC client with 18+ methods
- WebSocket client with subscriptions and automatic reconnection
- account_info, account_lines, account_offers, account_nfts, account_tx, account_objects, account_currencies, account_channels
- book_offers, amm_info, ledger, ledger_current, ledger_entry, server_info, gateway_balances
- fee, submit, tx
- Autofill (concurrent fetch of Sequence, Fee, LastLedgerSequence)
- submit_and_wait with polling until validated or expired
- Pagination helpers (_all variants) with 50-page safety cap
- TransactionExpired and PaginationLimitReached error variants
- tracing instrumentation on all RPC calls

### xrpl-sdk
- Wallet: generate, from_seed, sign_transaction, sign_and_encode
- Multi-signing: sign_for_multisigning, collect_signers
- autofill_and_sign and submit_transaction convenience functions
- Full re-exports from xrpl-core and xrpl-client
- 5 example programs: send_payment, subscribe_ledger, place_offer, check_balance, decode_blob, mint_nft

[Unreleased]: https://github.com/ilubxrp589/xrpl-sdk/compare/v0.6.0...HEAD
[v0.6.0]: https://github.com/ilubxrp589/xrpl-sdk/compare/v0.5.0...v0.6.0
[v0.5.0]: https://github.com/ilubxrp589/xrpl-sdk/compare/v0.4.0...v0.5.0
[v0.4.0]: https://github.com/ilubxrp589/xrpl-sdk/compare/v0.3.0...v0.4.0
[v0.3.0]: https://github.com/ilubxrp589/xrpl-sdk/compare/v0.2.0...v0.3.0
[v0.2.0]: https://github.com/ilubxrp589/xrpl-sdk/compare/v0.1.0...v0.2.0
[v0.1.0]: https://github.com/ilubxrp589/xrpl-sdk/releases/tag/v0.1.0
