# XRPL Type System

## Primitive Types

### AccountId
- **Wire**: 20 raw bytes (no prefix in object context; VL-prefixed when in Blob context)
- **Display**: base58check with payload prefix `0x00`, XRPL alphabet
- **Rust**: `pub struct AccountId([u8; 20]);`
- Classic addresses start with `r`; never store the string, always store the 20 bytes

### Hash128 / Hash160 / Hash256
- **Wire**: raw bytes, fixed-length, no prefix
- **Hash256 Display**: uppercase hex, 64 chars
- **Hash160 Display**: uppercase hex, 40 chars
- **Rust**: `pub struct Hash256([u8; 32]);` etc.

### Amount
Two variants — never mix them up:

**XRP (drops)**
- **Wire**: 8 bytes big-endian
  - Bit 63 (MSB): `0` = XRP (not IOU)
  - Bit 62: `1` = positive (always 1 for valid amounts)
  - Bits 0-61: unsigned drop count
- **Valid range**: 0 to 100,000,000,000,000,000 (10^17)
- **Never store XRP in decimal — only drops (u64)**

**IOU**
- **Wire**: 48 bytes total
  - Bytes 0-7: mantissa + exponent (see CODEC.md § IOU Amount)
  - Bytes 8-27: 20-byte currency code
  - Bytes 28-47: 20-byte issuer AccountId (raw, not VL-prefixed)
- **Rust**:
  ```rust
  pub enum Amount {
      Xrp(u64),
      Iou { value: IouAmount, currency: Currency, issuer: AccountId },
  }
  ```

### IouAmount
XRPL uses a custom floating-point:
- Mantissa: 54-bit unsigned integer (value 10^15 to 10^16-1, normalized)
- Exponent: 8-bit signed integer (range -96 to +80)
- Positive bit: bit 62 of the 8-byte encoding
- Zero: all 64 bits = 0 except bit 63=1, bit 62=0 (special encoding)
- **Rust**:
  ```rust
  pub struct IouAmount { pub mantissa: u64, pub exponent: i8 }
  ```

### Currency
- **Standard** (3-char ISO): e.g., `USD`, `EUR`, `BTC`
  - Wire: 20 bytes, first 12 bytes = 0x00, bytes 12-14 = ASCII chars, last 5 bytes = 0x00
- **Non-standard** (20-byte hex): used for NFT/AMM pools
  - Wire: raw 20 bytes, must NOT have bytes 12-14 all-ASCII (to avoid collision)
- **XRP special**: 20 zero bytes — only used in PathStep, never in Amount directly
- **Rust**:
  ```rust
  pub enum Currency {
      Standard([u8; 3]),   // stored as ASCII bytes
      NonStandard([u8; 20]),
  }
  impl Currency { pub fn xrp() -> Self { ... } }
  ```

### Blob
- **Wire**: VL-prefixed raw bytes (see CODEC.md § Variable-Length Encoding)
- **Rust**: `pub struct Blob(pub Vec<u8>);`
- Used for: `SigningPubKey`, `TxnSignature`, `MemoData`, `MemoFormat`, `MemoType`

### UInt8 / UInt16 / UInt32 / UInt64
- Wire: big-endian, fixed-width (1/2/4/8 bytes)
- Used as-is, no special encoding beyond byte order

---

## XRPL Base58 Alphabet
```
rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz
```
**This is NOT Bitcoin's base58 alphabet.** Position of each char matters.

## Seed / Address Version Bytes
| Purpose | Prefix bytes | Result starts with |
|---|---|---|
| Classic address | `[0x00]` | `r` |
| Secp256k1 seed | `[0x21]` | `s` |
| Ed25519 seed | `[0x01, 0xE1, 0x4B]` | `sEd` |

## Checksum
All base58check values: append `SHA256(SHA256(payload))[0..4]` to payload before encoding.

## Serde Representations (JSON API)
| Type | JSON format |
|---|---|
| `AccountId` | classic address string `"rHb9..."` |
| `Hash256` | uppercase hex string `"ABC123..."` |
| `Amount::Xrp` | decimal string of drops `"1000000"` |
| `Amount::Iou` | object `{"value":"1.5","currency":"USD","issuer":"r..."}` |
| `Blob` | uppercase hex string |
| `UInt32` | JSON number |
| `UInt64` | decimal string (JS can't handle 64-bit ints) |
