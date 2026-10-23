# Testing

How the SDK is tested, where each test's data comes from, and how to run or
refresh it. [PARITY.md](PARITY.md) lists what the tests establish against
`rippled` 3.4.1, with the numbers.

## Running the tests

```bash
cargo test --workspace                  # everything offline: unit, fixtures, doc examples, CLI
cargo test -p halcyon-xrpl-core         # one crate
cargo test codec_ -- --nocapture        # by name, with the tests' own counts printed
RUSTFLAGS="--cfg xrpl_live_tests" cargo test -p halcyon-xrpl-sdk --test live_testnet -- --nocapture   # the live Testnet test
```

The offline suite needs no network and takes well under a minute on a laptop.
The live test needs the XRPL Testnet faucet and takes about two minutes.

## How the tests are organized

| Kind | Where | What it holds |
|---|---|---|
| Unit tests | `#[cfg(test)]` module of each source file | The rules of one module, often with `rippled`'s own answers as the expected values |
| Fixture tests | `crates/*/tests/*.rs` | Whole-crate behaviour against recorded real-network data |
| Doc examples | `///` and `//!` comments | Every example in the API docs compiles and runs |
| Transport tests | `#[cfg(test)]` of `client.rs`, `ws.rs`, `http.rs` | `submit_and_wait` and `autofill` against a mock server that answers each call as `rippled` would (every outcome path, on a paused clock); the WebSocket client against a local WebSocket server (per-subscription events, drop closes, reconnect and resubscribe); HTTP body caps and URL-free errors against a local server |
| CLI tests | `crates/xrpl-cli/tests/cli.rs` | The offline `hxrpl` commands, run as a user would; the `--yes` guard against a local server that says which network it is on |
| Live test | `crates/xrpl-sdk/tests/live_testnet.rs` | End to end on Testnet (`--cfg xrpl_live_tests`) |
| Fuzz targets | `fuzz/fuzz_targets/` | Never panic, and round trips hold, on any input |
| Benchmarks | `benchmarks/` | Speed against xrpl-rust on the same inputs ([BENCHMARKS.md](BENCHMARKS.md)) |

## The fixtures

Expected values come from real servers and networks, not from this SDK. Each
fixture records where it came from.

| File | Contents | Source |
|---|---|---|
| `crates/xrpl-core/tests/fixtures/rippled_signed_transactions.jsonl` | One of each of the 79 user transaction types, every field filled, with `rippled`'s `tx_blob` and `hash` | `rippled` 3.4.1 `sign`, offline |
| `crates/xrpl-core/tests/fixtures/transactions.jsonl` | 186 transactions, 51 types, as validated | Mainnet and Testnet (`tx`) |
| `crates/xrpl-core/tests/fixtures/mainnet_ledger_objects.jsonl` | 20 ledger entries, 15 types | Mainnet ledger #107827070 (`ledger_data`) |
| `crates/xrpl-core/tests/fixtures/xrpljs/` | Transaction metadata and expected balance changes | xrpl.js's own test fixtures |
| `tests/vectors/codec-fixtures.json` | 34 transactions and 261 ledger entries with their bytes | XRPLF codec fixtures (shared with xrpl.js) |
| `crates/xrpl-core/src/codec/definitions.json` | Every type, field and result code | `rippled` 3.4.1 `server_definitions` |
| `crates/xrpl-client/tests/fixtures/rippled/` | 45 recorded responses: every public `rippled` and Clio method, some in two forms (binary, with options); API version 2 | `rippled` 3.4.1 on mainnet; Clio (`s2.ripple.com`) for Clio-only methods; Testnet for what the live test creates |
| `crates/xrpl-client/tests/fixtures/streams/` | Messages of every stream kind, and `path_find` updates, API version 2 | `rippled` 3.4.1 on mainnet (WebSocket) |

Unit tests that compare against `rippled` carry a comment saying which call
produced the expected value (for example `// rippled 3.4.1 offline sign of
the Payment ...`), so any expected value can be re-derived.

### Refreshing fixtures

- **Responses.** `crates/xrpl-client/scripts/record_fixtures.py responses
  --rippled <url>` sends each recorded request again to a mainnet `rippled`
  (the Clio-only ones to a mainnet Clio), in API version 2, and writes the
  answers in place; `record_fixtures.py streams --ws <url>` records the
  stream messages, an order book snapshot and a `path_find` search from a
  `rippled`'s admin WebSocket. The Testnet recordings come from the live
  test run with `XRPL_RECORD_DIR=<dir>`: it writes the responses to the
  requests it makes in the fixture format; copy them into
  `crates/xrpl-client/tests/fixtures/rippled/`.
- **`Number` rules.** `crates/xrpl-core/scripts/probe_number.py` asks a
  `rippled` node (through `simulate`, which applies nothing) how it writes and
  reads `Number` values. The tables in `src/codec/number.rs` are its answers.
- **Token values.** `crates/xrpl-core/scripts/probe_iou.py` asks the same
  way how `rippled` reads a token value from JSON (syntax, rounding, range);
  the table in `src/types/amount.rs` (`reads_token_values_as_rippled_does`)
  holds its answers.
- **Ledger entry IDs without a sample.** `rippled`'s `ledger_entry` reports
  the index it computes even for an entry that does not exist; the expected
  IDs in `hashes.rs` (`new_keylets_match_rippled`) came from it.
- **Definitions and typed models.** When `rippled` releases a new version
  (the weekly definitions check opens an issue), record its
  `server_definitions` result over
  `crates/xrpl-client/tests/fixtures/rippled/server_definitions.json`, copy
  its types, fields, transaction and ledger entry types and result codes into
  `crates/xrpl-core/src/codec/definitions.json` (CI checks the two agree),
  and run `crates/xrpl-core/scripts/gen_transactions.py` and
  `gen_ledger_objects.py`.
  These regenerate the typed transactions, ledger entries and flags. The
  test `every_type_is_checked_against_rippled` then names any new transaction
  type that no fixture covers yet.

## The live Testnet test

`--cfg xrpl_live_tests` (in `RUSTFLAGS`) enables `crates/xrpl-sdk/tests/live_testnet.rs`. It
funds two wallets from the Testnet faucet, submits one transaction of each
kind that creates a ledger entry, two batches and a multi-signed transaction,
and checks every hash, entry ID and parsed response against what the network
did. A transaction whose amendment Testnet has not enabled (`temDISABLED`) is
skipped, not failed. CI runs it after every push to `main`.

## Fuzzing

```bash
rustup toolchain install nightly && cargo install cargo-fuzz
python3 fuzz/seed_corpus.py                       # seed every target from the fixtures
cargo +nightly fuzz run fuzz_roundtrip            # until stopped
cargo +nightly fuzz run fuzz_roundtrip -- -max_total_time=120
cargo +nightly fuzz run fuzz_roundtrip fuzz/artifacts/fuzz_roundtrip/crash-…   # replay a crash
```

| Target | Requires |
|---|---|
| `fuzz_decode` | Decoding any bytes never panics |
| `fuzz_roundtrip` | Whatever decodes encodes again, and decodes to the same JSON |
| `fuzz_encode` | Encoding any JSON, for submission or for signing, never panics |
| `fuzz_models` | Typed transactions and ledger entries never panic on any JSON, and a typed transaction writes back what it read |
| `fuzz_hashes` | Transaction hashes and Batch signing data never panic, from bytes or JSON |
| `fuzz_utils` | The string and metadata utilities never panic |
| `fuzz_amount` | The amount codec never panics |
| `fuzz_address` | Address and seed decoding never panic |

CI fuzzes each target for a minute on every pull request and for 15 minutes
every Monday, and uploads any crashing input.

## Minimum supported Rust version

`halcyon-xrpl-core` needs Rust 1.85; `halcyon-xrpl-client`,
`halcyon-xrpl-sdk` and `halcyon-xrpl-cli` need 1.88 (for the URL handling in
`reqwest`). Each crate declares it as `rust-version`. CI checks both, with the
locked dependencies, so a dependency update that raises either fails there
before it reaches a release. Raising the MSRV is a minor-version change and
goes in the CHANGELOG.

## Continuous integration

`.github/workflows/ci.yml` runs on every push and pull request:

| Job | Runs |
|---|---|
| Test | `cargo test --workspace` on Linux, macOS and Windows |
| SemVer | `cargo-semver-checks` against the release on crates.io: no breaking change unless the version says so |
| Package | `cargo package --workspace`: each crate builds from its packaged files alone |
| Clippy | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| Format | `cargo fmt --check` |
| MSRV | `cargo check --locked` on Rust 1.85 (core) and 1.88 (everything) |
| WASM build | Clippy for `wasm32-unknown-unknown` (core, client, SDK, with `wasm`), and the client's unit tests in Node.js |
| Features | Clippy for the client and the SDK with no features, `http` only and `ws` only |
| Definitions | The codec's definitions are the recorded `server_definitions` answer (`check_definitions.py --offline`) |
| Benchmarks and examples | The benchmarks and the Cloudflare Worker example (outside the workspace) build |
| Docs | `cargo doc` with warnings as errors, on stable and as docs.rs builds it (nightly, `--cfg docsrs`) |
| Licences and sources | `cargo-deny`: every dependency's licence is allowed, crates come from crates.io |
| Security Audit | `cargo audit` against the RustSec database |
| Live Testnet Tests | The live test (`--cfg xrpl_live_tests`), after pushes to `main` |

`.github/workflows/fuzz.yml` runs the fuzz targets as described above.

`.github/workflows/definitions.yml` runs every Monday (and by hand): it
compares `rippled`'s definitions on Devnet, Testnet and mainnet with the
SDK's (`crates/xrpl-core/scripts/check_definitions.py`), and when a network
knows something new (a transaction type, field, ledger entry, result code
or flag) opens an issue that lists exactly what changed and how to catch up.
Devnet gets a new `rippled` first, so the issue comes before mainnet
changes.

## The rules the code is held to

The workspace denies, in library code:

- `unsafe`;
- `unwrap` and `expect`;
- unchecked indexing.

A test module opts out explicitly with
`#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]`.
Every public item is documented (`#![warn(missing_docs)]`, with docs built
with warnings as errors).
