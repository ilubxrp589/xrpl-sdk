# Benchmarks

`halcyon-xrpl-core` against [xrpl-rust](https://crates.io/crates/xrpl-rust)
1.3.0, the other general-purpose Rust library for the XRP Ledger, doing the
same work on the same inputs.

## Results

Median time per operation (lower is better). The last column is how many times
faster halcyon-xrpl-core is, or how many times slower where it is slower.

| Operation | Input | halcyon-xrpl-core | xrpl-rust 1.3.0 | halcyon-xrpl-core is |
|---|---|---:|---:|---:|
| Decode binary → JSON | Payment (XRP) | 3.98 µs | 6.68 µs | 1.7× faster |
| | OfferCreate | 6.30 µs | 73.88 µs | 11.7× faster |
| | TrustSet | 4.60 µs | 38.75 µs | 8.4× faster |
| | NFTokenMint | 4.11 µs | 7.14 µs | 1.7× faster |
| | AMMDeposit | 6.22 µs | 28.66 µs | 4.6× faster |
| Encode JSON → binary | Payment (XRP) | 3.88 µs | 31.25 µs | 8.1× faster |
| | OfferCreate | 5.50 µs | 134 µs | 24.3× faster |
| | TrustSet | 4.18 µs | 75.38 µs | 18.0× faster |
| | NFTokenMint | 4.01 µs | 32.07 µs | 8.0× faster |
| | AMMDeposit | 5.94 µs | 70.45 µs | 11.9× faster |
| Derive keys from a seed | Ed25519 | 17.81 µs | 90.46 µs | 5.1× faster |
| | secp256k1 | 114 µs | 114 µs | the same |
| Sign signing data | Ed25519 | 36.05 µs | 36.03 µs | the same |
| | secp256k1 | 44.45 µs | 29.11 µs | 1.5× slower |
| Sign a transaction (JSON in, signed JSON out) | Ed25519 | 39.35 µs | 67.78 µs | 1.7× faster |
| | secp256k1 | 48.60 µs | 61.13 µs | 1.3× faster |
| Decode a classic address | `rHb9CJAW…` | 439 ns | 360 ns | 1.2× slower |

## What is measured

Each pair of rows does the same work, with each library's public functions:

| Operation | halcyon-xrpl-core | xrpl-rust 1.3.0 |
|---|---|---|
| Decode | `decode_transaction_binary` (from hex) | `core::binarycodec::decode` |
| Encode | `encode_transaction_json` (to hex) | `core::binarycodec::encode` |
| Derive keys | `Keypair::from_seed(Seed::from_base58(..))` | `core::keypairs::derive_keypair` |
| Sign | `Keypair::sign_signing_data` | `core::keypairs::sign` |
| Sign a transaction | `sign_transaction` | `encode_for_signing`, then `keypairs::sign` |
| Decode an address | `address::decode_account_id` | `core::addresscodec::decode_classic_address` |

**The inputs are real.** The transactions are the first of each type among
the mainnet transactions in the SDK's test fixtures. The keys come from the
public test seeds `sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r` (Ed25519) and
`sp5fghtJtpUorTwvof1NpDXAzNwf5` (secp256k1). The address is
`rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh`.

**Same answers first.** Before timing anything, the benchmark checks that both
libraries give the same answer:

- the same decoded JSON and encoded bytes for every transaction;
- the same public keys;
- the same signatures;
- the same signed Payment.

In the run above, every check agreed, so each row compares the same work.

**Signing is like for like.** Both libraries sign SHA-512Half of the signing
data for secp256k1 and the data itself for Ed25519, as `rippled` verifies.
Both use deterministic nonces (RFC 6979), so their signatures are identical.

## Where the differences come from

- **Encoding and decoding.** This is where halcyon-xrpl-core is fastest, and
  where an application spends most of its time: it encodes every transaction
  it signs and decodes every one it reads. halcyon-xrpl-core works directly
  on `serde_json::Value` and bytes, with the field definitions in a lookup
  table built once.
- **secp256k1.** xrpl-rust uses the `secp256k1` crate, Rust bindings to
  Bitcoin Core's C library libsecp256k1. halcyon-xrpl-core uses `k256`, a
  pure-Rust implementation. That choice keeps the SDK free of C code: it
  builds without a C toolchain, for WebAssembly (`wasm32-unknown-unknown`)
  and for `no_std` targets, with `unsafe` code forbidden in the SDK itself.
  The C library is faster at the raw signature; signing a whole transaction,
  which also encodes it, is still faster with halcyon-xrpl-core. Ed25519, the
  SDK's default key type, uses `ed25519-dalek` in both libraries.
- **Address decoding.** About 80 ns slower per address. Next to the
  microseconds spent encoding and signing each transaction, that is small,
  and it has not been optimized yet.

## Method

- **Machine.** AMD Ryzen 5 4500 (6 cores, 12 threads), Linux 6.17, Rust
  1.96.0, the `bench` profile (optimized), on a shared build machine (load
  average 1 to 2 during the run). Treat differences under about 5% as noise.
- **Harness.** [criterion](https://crates.io/crates/criterion) 0.5: 100
  samples per benchmark after a warm-up; the table shows the median.
- **Versions.** halcyon-xrpl-core at this commit; xrpl-rust 1.3.0 with
  `default-features = false, features = ["std", "core"]`.
- **Date.** 2026-10-22.

## Running them yourself

```bash
cd benchmarks
cargo bench                         # everything, about 10 minutes
cargo bench -- 'encode|decode'      # a subset, by regular expression
```

The benchmark crate (`benchmarks/`) is not part of the workspace and is never
published: it is the only place xrpl-rust is used, and the SDK does not depend
on it. Results print to the terminal; criterion's HTML reports are written to
`benchmarks/target/criterion/`.
