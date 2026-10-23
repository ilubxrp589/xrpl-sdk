# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.8.x   | Yes: security fixes are released as 0.8.x |
| 0.7.x   | No: upgrade to 0.8 (the CHANGELOG says how to update for each change) |
| 0.6.x and older | No: upgrade to 0.8 (0.6.0 fixed the issues listed in its CHANGELOG under "Security") |

## Reporting a vulnerability

**Do not open a public issue.** Report it privately through GitHub: on the
repository's **Security** tab, choose **Report a vulnerability**. Please
include what is affected (crate, version, function), how to reproduce it,
and what an attacker could do with it.

The report is acknowledged and you are kept informed. A confirmed issue is
fixed in a release, with an advisory that credits you unless you ask
otherwise.

## What the SDK does to stay safe

This section states what the code does; each claim is covered by tests.

### Keys and secrets ([docs/CRYPTO.md](docs/CRYPTO.md))

- `Seed` and `Keypair` wipe their secret bytes when dropped, and their
  `Debug` output never shows them. Key derivation wipes its intermediate
  secrets; secret keys and seed strings are returned in `Zeroizing` buffers.
- Seeds come from the operating system's random source, never a user-space
  generator.
- Errors never carry a secret: seed decoding errors name the kind of
  mistake, not a character of the seed or a checksum computed from it. No
  crypto code logs anything; clients log the server's host, never a URL
  (URLs can hold API keys), and transport errors leave the URL out.
- secp256k1 signs only a 32-byte hash and verifies only 33-byte compressed
  keys and low-S signatures; Ed25519 verification is strict.
- Rust may leave copies of moved bytes that are not wiped, and in
  WebAssembly the page's JavaScript can read all of memory: treat the
  process as able to see its keys.

### What gets signed

- The codec refuses what `rippled` refuses instead of changing the
  transaction: a field name it does not define (a misspelt `DestinationTag`
  is an error, not dropped), an integer too large for its field (it is not
  wrapped), malformed amounts, currencies and arrays. Where the SDK
  deliberately differs from `rippled`, [docs/PARITY.md](docs/PARITY.md) says
  so.
- `Client::autofill` refuses a fee above the client's cap (2 XRP unless set
  otherwise), so a busy network or a dishonest server cannot make a
  transaction spend more.
- `Client::submit_and_wait` reports only final outcomes, looks the
  transaction up by the ID it computes itself, and says "outcome unknown"
  (with the hash) rather than guess, so a payment that went through is never
  reported as failed and resent.
- `hxrpl` asks the server which network it is on before it submits anything,
  and needs `--yes` anywhere but Testnet or Devnet.

### Inputs from servers

- Response bodies are capped (128 MiB); base58 strings over 128 characters
  are refused before decoding; the binary decoder limits nesting to 16
  levels and reads only canonical field headers.
- The binary codec is fuzzed in CI (round trip, typed models, hashes,
  utilities) and checked against `rippled` 3.4.1's own answers.

### Code

- No `unsafe` code (`unsafe_code = "forbid"` in every crate). `unwrap`,
  `expect`, unchecked indexing and narrowing `as` casts are denied by
  Clippy in library code; each remaining exception states why it cannot
  fail.
- TLS is `rustls`; OpenSSL is not used. `cargo audit` runs in CI;
  `deny.toml` holds the license and source policy for `cargo deny`.

## Known advisories

None.
