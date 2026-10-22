# Changelog

## v0.5.0 (2026-10-22)

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

## v0.4.0 (2026-10-22)

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

## v0.1.0 (2026-03-19)

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
