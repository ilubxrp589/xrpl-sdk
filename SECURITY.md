# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Active    |

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Please include:
- Description of the vulnerability
- Steps to reproduce
- Affected versions
- Potential impact

## Security Design

### Private Key Handling
- Private keys are stored as `Vec<u8>` and zeroed on drop via `zeroize::ZeroizeOnDrop`
- Seed bytes are zeroed on drop via `ZeroizeOnDrop`
- Intermediate key material (root private key, intermediate scalar) is explicitly zeroed after use
- Private keys never appear in tracing output — no tracing instrumentation in crypto code
- No private key material is serialized to JSON or included in error messages

### Cryptographic Primitives
- secp256k1 signing: `k256` crate (pure Rust, constant-time)
- Ed25519 signing: `ed25519-dalek` (pure Rust)
- SHA-512: `sha2` (pure Rust)
- RIPEMD-160: `ripemd` (pure Rust)
- All signatures are normalized to low-S form (BIP62 compliant)
- All primitives verified against published XRPL known-answer test vectors

### Amount Validation
- XRP amounts above the maximum supply (100,000,000,000,000,000 drops) are rejected at encode time
- All integer arithmetic on externally-provided values uses checked/saturating arithmetic

### Unsafe Code
- xrpl-core, xrpl-client, and xrpl-sdk contain zero unsafe code blocks
- Unsafe code exists only in third-party crypto dependencies

### Binary Codec
- The decoder is fuzz-tested against libFuzzer and never panics on any input
- Recursion depth limited to 16 levels to prevent stack overflow on malicious input
- Codec correctness verified against the official XRPLF fixture vectors
- All indexing uses bounds-checked `.get()` methods (no direct `slice[index]`)

### Dependency Security
- `cargo audit`: zero HIGH/CRITICAL advisories
- `cargo deny`: all dependencies use permissive licenses (MIT, Apache-2.0, BSD, ISC)
- OpenSSL banned — TLS via `rustls` (pure Rust)
- No wildcard dependency versions

## Known Advisories

None at this time.
