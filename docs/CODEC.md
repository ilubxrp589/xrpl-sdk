# Binary Codec

Reference: https://xrpl.org/serialization.html

## Overview
XRPL transactions are serialized to a canonical binary format for signing and submission.
The encoder produces a byte stream: sorted field headers + encoded field data, concatenated.

---

## Field Header Encoding
Every field has a `type_code` (what kind of data) and `field_code` (which field of that type).

**1-byte header**: both codes fit in 4 bits each (1–15)
```
byte = (type_code << 4) | field_code
```

**2-byte header**: type_code > 15, field_code fits in 4 bits
```
byte[0] = field_code          (low nibble only, high nibble = 0)
byte[1] = type_code
```

**2-byte header**: type_code fits in 4 bits, field_code > 15
```
byte[0] = (type_code << 4)    (high nibble only, low nibble = 0)
byte[1] = field_code
```

**3-byte header**: both > 15
```
byte[0] = 0x00
byte[1] = type_code
byte[2] = field_code
```

---

## Type Codes
| Type | Code | VL-prefixed? |
|---|---|---|
| UInt16 | 1 | No |
| UInt32 | 2 | No |
| UInt64 | 3 | No |
| Hash128 | 4 | No |
| Hash256 | 5 | No |
| Amount | 6 | No |
| Blob | 7 | Yes |
| AccountID | 8 | Yes (in field context) |
| STObject | 14 | No (terminated by 0xE1) |
| STArray | 15 | No (terminated by 0xF1) |
| UInt8 | 16 | No |
| Hash160 | 17 | No |
| PathSet | 18 | Special (see below) |
| Vector256 | 19 | Yes |

---

## Variable-Length (VL) Encoding
Used for Blob, AccountID (when in field), Vector256.

| Length | Bytes | Encoding |
|---|---|---|
| 0–192 | 1 byte | `length` |
| 193–12480 | 2 bytes | `((length - 193) / 256) + 193`, `(length - 193) % 256` |
| 12481–918744 | 3 bytes | `byte[0] = 241 + (n >> 16)`, `byte[1] = (n >> 8) & 0xFF`, `byte[2] = n & 0xFF` where `n = length - 12481` |

In practice, keys and signatures are always < 192 bytes, so 1-byte VL prefix is the common case.

---

## Amount Encoding

### XRP Amount (8 bytes)
```
bit 63 = 0         (marks as XRP, not IOU)
bit 62 = 1         (marks as positive)
bits 0-61 = drops  (unsigned 62-bit integer)
```
Example: 1 XRP = 1,000,000 drops
```
0x40_00_00_00_00_0F_42_40
```
Zero XRP:
```
0x40_00_00_00_00_00_00_00
```

### IOU Amount (48 bytes = 8B mantissa/exp + 20B currency + 20B issuer)

**Mantissa/exponent 8 bytes:**
- Bit 63 = 1 (marks as IOU)
- Bit 62 = 1 (positive) / 0 (negative)
- Bits 54-61 = exponent + 97 (8-bit unsigned biased)
- Bits 0-53 = mantissa (54-bit unsigned, normalized to 10^15 ≤ m < 10^16)

**Zero IOU:** all 8 bytes = `0x80 00 00 00 00 00 00 00` (bit 63 set, rest 0)

**Encoding algorithm:**
1. Parse decimal string to (mantissa, exponent)
2. Normalize: multiply/divide mantissa until `10^15 ≤ mantissa < 10^16`
3. Adjust exponent accordingly
4. Pack into 8 bytes as described above

---

## Canonical Field Order
Fields MUST be sorted before encoding. Sort ascending by:
1. `type_code` (primary)
2. `field_code` (secondary)

**Never encode fields in source-code definition order** — always sort first.

---

## Field Registry (Key Fields)
| Field Name | Type Code | Field Code | Notes |
|---|---|---|---|
| TransactionType | UInt16 | 2 | u16 discriminant |
| Flags | UInt32 | 2 | |
| SourceTag | UInt32 | 3 | optional |
| Sequence | UInt32 | 4 | |
| DestinationTag | UInt32 | 14 | optional |
| LastLedgerSequence | UInt32 | 27 | |
| Amount | Amount | 1 | |
| Fee | Amount | 8 | XRP only |
| SendMax | Amount | 9 | |
| DeliverMin | Amount | 10 | |
| TakerPays | Amount | 4 | |
| TakerGets | Amount | 5 | |
| LimitAmount | Amount | 3 | TrustSet |
| SigningPubKey | Blob | 3 | 33 or 33+1 bytes |
| TxnSignature | Blob | 4 | 64 or 71-73 bytes |
| MemoData | Blob | 13 | |
| MemoFormat | Blob | 14 | |
| MemoType | Blob | 12 | |
| Account | AccountID | 1 | |
| Destination | AccountID | 3 | |
| Issuer | AccountID | 4 | |
| Memos | STArray | 9 | array of Memo objects |
| Signers | STArray | 3 | array of Signer objects |
| Memo | STObject | 10 | inner object in Memos array |
| Signer | STObject | 16 | inner object in Signers array |
| SignerEntry | STObject | 17 | |
| TransactionHash | Hash256 | 7 | |

Full registry: https://github.com/XRPLF/xrpl.js/blob/main/packages/ripple-binary-codec/src/enums/definitions.json

---

## STObject Encoding
```
[field_header][field_data]  (repeated, sorted)
0xE1                         (object end marker)
```

## STArray Encoding
```
[object_header][fields...][0xE1]  (one encoded object, ends with 0xE1)
(repeated for each array element)
0xF1                               (array end marker)
```

---

## PathSet Encoding
Used for Payment `Paths` field:
- Multiple paths separated by `0xFF`
- Array of paths terminated by `0x00`
- Each step in a path: 1 type byte + optional account (20B) + optional currency (20B) + optional issuer (20B)
  - Bit 0 of type byte: has account
  - Bit 4 of type byte: has currency
  - Bit 5 of type byte: has issuer

---

## Signing
When signing, prefix the serialized transaction bytes with:
```
[0x53, 0x54, 0x58, 0x00]   // "STX\0"
```
Then hash with SHA512 and take the first 32 bytes (SHA512-half).
**Do NOT include `TxnSignature` field when computing the signing hash** — omit it even if present.
