# Design notes (historical)

These are the plans the SDK was first built from, in March 2026. They are
kept for the record and are **not** a description of the SDK today: parts
were never built that way, and some of what they say is wrong (for example,
`CHECKLIST.md` and `CRYPTO.md` here describe deriving keys with SLIP-10 and
signing the SHA-512Half of the signing data for every key type; `rippled`
verifies Ed25519 over the data itself, and the SDK derives keys as
`rippled` does).

What the SDK does now is documented by:

- the API documentation on docs.rs (`halcyon-xrpl-core`, `halcyon-xrpl-client`,
  `halcyon-xrpl-sdk`), which is compiled and tested with the code;
- [`../CRYPTO.md`](../CRYPTO.md): keys, signing and verification;
- [`../PARITY.md`](../PARITY.md): what is checked against `rippled`, and the
  deliberate differences;
- [`../TESTING.md`](../TESTING.md) and [`../BENCHMARKS.md`](../BENCHMARKS.md).
