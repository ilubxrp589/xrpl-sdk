# Cryptography

## Key Types
XRPL supports two signature schemes:
- **Ed25519** — preferred, faster verification, smaller signatures
- **Secp256k1** — original scheme, ECDSA with DER encoding

---

## Seed
A seed is 16 random bytes + a key type tag.

**Base58 encoding:**
- Secp256k1: prefix `[0x21]` → encoded string starts with `s`
- Ed25519: prefix `[0x01, 0xE1, 0x4B]` → encoded string starts with `sEd`

**Derivation from family seed string (legacy):**
- Used when seed is given as a string like `sn3nxiW7v8KXzPzAqzyHXbSSKNuN9`
- Detect: starts with `s` but not `sEd` → secp256k1

---

## Ed25519 Key Derivation

```
seed_bytes: [u8; 16]
raw_private = SHA512(seed_bytes)   // 64 bytes
scalar = raw_private[0..32]        // clamp per Ed25519 spec (done by ed25519-dalek)
public_key_bytes = scalar * G      // 32 bytes compressed
wire_public_key = [0xED] + public_key_bytes  // 33 bytes total
```

**Signing:**
- Sign 32-byte SHA512-half hash
- Signature: 64 bytes (r || s)
- `SigningPubKey` field: 33 bytes with `0xED` prefix

**Crate:** `ed25519-dalek v2`

---

## Secp256k1 Key Derivation (XRPL root keypair)

XRPL uses a custom deterministic derivation — NOT BIP32/BIP44.

**Step 1: Root private key**
```
sequence = 0u32
loop:
    payload = seed_bytes + sequence.to_be_bytes()
    result = SHA512(payload)          // 64 bytes
    root_private = result[0..32]
    if root_private < secp256k1 order: break
    sequence += 1
```

**Step 2: Root public key**
```
root_public = root_private * G        // 33-byte compressed point
```

**Step 3: Account keypair (sequence 0)**
```
sequence = 0u32
sub_sequence = 0u32
loop:
    payload = root_public + sequence.to_be_bytes() + sub_sequence.to_be_bytes()
    result = SHA512(payload)
    intermediate = result[0..32]
    if intermediate < secp256k1 order:
        account_private = (root_private + intermediate) mod order
        break
    sub_sequence += 1
account_public = account_private * G  // 33-byte compressed
```

**Signing:**
- Sign 32-byte SHA512-half hash using ECDSA
- Signature: DER-encoded, typically 70–72 bytes
- Use low-S normalization (XRPL requires canonical low-S)
- `SigningPubKey` field: 33 bytes compressed point

**Crate:** `k256 v0.13` with `ecdsa` feature

---

## Account ID Derivation
From any public key (secp256k1 or ed25519):
```
account_id = RIPEMD160(SHA256(public_key_bytes))  // 20 bytes
```

---

## Transaction Signing Algorithm

```rust
fn sign_transaction(tx: &mut Transaction, keypair: &Keypair) -> Result<()> {
    // 1. Set SigningPubKey
    tx.signing_pub_key = keypair.public_key.clone();

    // 2. Remove TxnSignature if present
    tx.txn_signature = None;

    // 3. Serialize without signature
    let bytes = encode_transaction(tx)?;

    // 4. Prepend signing prefix
    let mut payload = vec![0x53, 0x54, 0x58, 0x00];
    payload.extend_from_slice(&bytes);

    // 5. SHA512-half
    let hash = sha512_half(&payload);  // first 32 bytes of SHA512

    // 6. Sign
    let sig = keypair.sign(&hash)?;

    // 7. Set TxnSignature
    tx.txn_signature = Some(sig);
    Ok(())
}
```

## Multi-Sign
Each signer produces their own `Signer` object:
- Set `SigningPubKey` = empty Blob (`0x00` VL prefix, 0 bytes)
- For each signer: sign the transaction prefixed with `[0x53, 0x4D, 0x54, 0x00]` (SMTX)
- Collect `Signers` array sorted by AccountId (ascending)
- Submit the assembled multi-signed transaction

---

## SHA512-Half
```rust
fn sha512_half(data: &[u8]) -> [u8; 32] {
    let digest = sha2::Sha512::digest(data);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest[..32]);
    out
}
```
