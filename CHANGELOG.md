# Changelog

## v0.3.0 (2026-10-22)

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
