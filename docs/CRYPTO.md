# Keys, signing and verification

How `halcyon-xrpl-core` derives keys, what it signs, how it checks
signatures, and what it does to keep secrets out of memory dumps, logs and
error messages. Every rule here is the one `rippled` 3.4.1 applies, and is
checked against `rippled` or published known answers (see the end).

## Seeds

A seed is 16 secret bytes and the key type they derive:

| Key type | Base58 form | Prefix bytes |
|---|---|---|
| Ed25519 (the default for new seeds) | `sEd…` (31 characters) | `0x01 0xE1 0x4B` |
| secp256k1 | `s…` (29 characters) | `0x21` |

```rust
use xrpl_core::crypto::{KeyType, Seed};

let seed = Seed::generate();                                // Ed25519
let secp = Seed::generate_with_type(KeyType::Secp256k1);
let again = Seed::from_base58(&seed.to_base58())?;          // to_base58 is a wiped string
assert_eq!(again.as_bytes(), seed.as_bytes());
# Ok::<(), xrpl_core::CoreError>(())
```

New seeds come from the operating system's random source (`OsRng`;
WebCrypto in the browser), never from a user-space generator, so a process
that forks cannot hand its children its own next seed.

## Key derivation

**Ed25519**: the secret key is SHA-512Half(seed); the public key is `0xED`
followed by the 32-byte Ed25519 point (33 bytes, as the ledger stores it).

**secp256k1** (`rippled`'s own scheme, not BIP32):

1. root secret = SHA-512Half(seed ‖ seq), for seq = 0, 1, … until the value
   is a valid secret key (non-zero, below the curve order);
2. root public key = the root secret's compressed point;
3. intermediate = SHA-512Half(root public ‖ 0 ‖ sub), for sub = 0, 1, …,
   the first valid one;
4. account secret = root secret + intermediate (mod the curve order); the
   account public key is its compressed point (33 bytes).

The account ID is RIPEMD-160(SHA-256(public key)); the classic address is
its base58check with prefix `0x00`.

`Keypair::from_private_key(key_type, secret)` derives the public key from
the secret key, so a key pair can never hold a public key that is not its
own.

## What is signed

Every signature covers **signing data**: a four-byte hash prefix and the
object's fields in canonical binary form, without the fields that are not
signed (`TxnSignature`, `Signers`, `BatchSigners`, …).

| What | Signing data |
|---|---|
| A transaction, single-signed | `STX\0` ‖ the transaction's signed fields (`SigningPubKey` set to the signer's key) |
| A transaction, multi-signed (each signer) | `SMT\0` ‖ the transaction's signed fields (`SigningPubKey` empty) ‖ the signer's AccountID |
| A Batch, by each `BatchSigner` | `BCH\0` ‖ outer Account ‖ outer Sequence (or TicketSequence) ‖ outer Flags ‖ number of inner transactions ‖ their IDs ‖ the signing account's AccountID (a multi-signed `BatchSigner` adds the signer's AccountID after that) |
| A payment channel claim | `CLM\0` ‖ channel ID ‖ amount in drops (8 bytes, big-endian) |

How the signing data is signed depends on the key type, as `rippled`
verifies it (`PublicKey.cpp`):

- **Ed25519** signs the signing data itself (Ed25519 hashes internally);
- **secp256k1** signs SHA-512Half(signing data), with an RFC 6979 nonce
  (deterministic: the same key and data always give the same signature),
  low-S, DER-encoded.

`Keypair::sign_message(data)` (and its alias `sign_signing_data`) does
exactly this for either key type, and `Keypair::verify_message` checks it.
The raw primitives are below them:

- `crypto::ed25519::sign(secret, message)` signs a message;
- `crypto::secp256k1::sign(secret, hash)` signs a **32-byte hash**, and
  refuses anything else. (The ECDSA primitive would silently use only the
  first 32 bytes of a longer input, so a signature over one long message
  would verify for every message starting with the same 32 bytes.)

(`Keypair::sign` / `Keypair::verify`, which took a 32-byte hash for
secp256k1 and a message for Ed25519, were removed in 0.8: too easy to get
wrong.)

## Keys held elsewhere: HSM, KMS, hardware wallet

A key does not have to be a `Keypair` in memory. Anything that signs the
signing data as above can sign, through one of two traits:

| Trait | For | Implement |
|---|---|---|
| `crypto::SigningKey` | a key reached synchronously: an HSM, a hardware wallet | `public_key()`, `sign_signing_data(data)` |
| `crypto::AsyncSigningKey` | a remote key: a cloud KMS, a signing service | `public_key()`, `async fn sign_signing_data(data)` |

`Keypair` and `Wallet` implement both (every `SigningKey` is an
`AsyncSigningKey`), and every signing function takes either:
`sign_transaction` / `sign_transaction_async`, `sign_for_multisigning` /
`_async`, the batch and channel-claim signers, and the SDK's `sign`,
`sign_async` and `sign_and_submit`.

What the functions do with a key's answer:

- **The public key must be one the ledger takes**: 33 bytes, `0xED` and the
  key (Ed25519), or compressed (`0x02`/`0x03` and X, secp256k1). A KMS
  usually gives the uncompressed point (65 bytes, `0x04`, X, Y; the last 65
  bytes of AWS KMS's `GetPublicKey` answer): `secp256k1::compressed_public_key`
  turns it into the form the ledger takes. `AccountId::from_public_key` gives
  the key's address.
- **A high-S secp256k1 signature is made low-S.** ECDSA signatures come in
  pairs (`S` and `n − S`), and `rippled` takes only the low one. A KMS that
  signs with a random nonce returns the high one about half the time; it is
  replaced by its low twin, which verifies the same.
- **A remote key's signature is checked** against its public key before it
  is used (`AsyncSigningKey` only: a check costs nothing next to a network
  call). The usual integration mistake, signing the data where the digest
  should be signed (secp256k1 signs SHA-512Half(data): a KMS `Sign` with
  message type `DIGEST`), fails here with `CoreError::SigningKeyFailed`, not
  as a refusal from the network. A `SigningKey` is not checked, so that
  signing with a `Keypair` costs no more than before; check one of your own
  once with `verify_signing_data`.

`examples/remote_signer.rs` (in the SDK crate) signs and submits on Testnet
through a stand-in signing service that holds the key.

## Verification

`crypto::verify_signing_data(public_key, data, signature)` checks any XRPL
signature the way `rippled` does: the key type is the key's first byte
(`0xED` for Ed25519), Ed25519 over the data, secp256k1 over its
SHA-512Half.

- secp256k1 keys must be 33 bytes, compressed (`0x02`/`0x03`), the only form
  `rippled` accepts; signatures must be strict DER and low-S.
- Ed25519 uses `verify_strict`, which also refuses small-order keys and
  `R` values. It can refuse a signature `rippled` would accept from such a
  key; it never accepts one `rippled` refuses.
- A malformed key or signature is an error; a well-formed signature that
  does not match is `Ok(false)`.

Higher-level checks built on it: `wallet::collect_signers` verifies every
multi-signature before it puts them on the transaction;
`transaction::batch::verify_batch_signers` checks that a Batch carries
exactly the signers `rippled` requires, each valid;
`crypto::verify_payment_channel_claim` checks a claim.

## Keeping secrets secret

- `Seed` and `Keypair` wipe their secret bytes when dropped
  (`ZeroizeOnDrop`); `Debug` shows only the key type (and public key), so
  `{:?}`, `dbg!` and `tracing` fields never print a secret.
- Derivation wipes its intermediates: the hashed seed, the SHA-512 digests,
  the root and intermediate secrets and their sum.
- `derive_keypair` returns the secret key in a `Zeroizing` buffer;
  `Seed::to_base58` and `Wallet::seed` return `Zeroizing<String>`.
- Seed decoding errors say what kind of mistake the string has, never a
  character of it or a checksum computed from it.
- No crypto code logs anything.

What this cannot promise: Rust may copy bytes it moves (a stack frame, a
reallocated buffer) where they are not wiped, and in WebAssembly the whole
linear memory is readable by the page's JavaScript. Treat the process as
able to see its keys.

## Checked against

- Keys and addresses: xrpl.js's ripple-keypairs fixtures and `rippled`'s
  `wallet_propose`, for both key types.
- Transaction signatures: `rippled` 3.4.1 `sign` (offline) produces the same
  blobs byte for byte (both schemes are deterministic).
- Multi-signatures: `rippled` 3.4.1 `sign_for`.
- Payment channel claims: `rippled` 3.4.1 `channel_authorize`, and xrpl.js's
  claim tests.
- Batch signatures: every Testnet and mainnet Batch with `BatchSigners` in
  the recorded sample verifies, and none does after one flag is changed.
