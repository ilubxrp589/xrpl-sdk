# Transaction Types

Reference: https://xrpl.org/transaction-formats.html

## Common Fields (ALL transactions)
| Field | Type | Required | Notes |
|---|---|---|---|
| TransactionType | UInt16 | ✓ | Discriminant value from table below |
| Account | AccountID | ✓ | Sending account |
| Fee | Amount (XRP) | ✓ | In drops; never IOU |
| Sequence | UInt32 | ✓ | Account sequence number |
| Flags | UInt32 | ✓ | Bitfield; use 0 if none |
| LastLedgerSequence | UInt32 | Recommended | current_ledger + 4 for safety |
| SigningPubKey | Blob | ✓ | 33 bytes; empty for multi-sign |
| TxnSignature | Blob | ✓ | Set after signing |
| SourceTag | UInt32 | Optional | Routing tag |
| AccountTxnID | Hash256 | Optional | Previous tx hash requirement |
| Memos | STArray | Optional | Array of Memo objects |
| Signers | STArray | Optional | Multi-sign array |

## TransactionType Codes
| Name | Code |
|---|---|
| Payment | 0 |
| EscrowCreate | 1 |
| EscrowFinish | 2 |
| AccountSet | 3 |
| EscrowCancel | 4 |
| SetRegularKey | 5 |
| OfferCreate | 7 |
| OfferCancel | 8 |
| TrustSet | 20 |
| AccountDelete | 21 |
| DepositPreauth | 19 |
| NFTokenMint | 25 |
| NFTokenBurn | 26 |
| NFTokenCreateOffer | 27 |
| NFTokenCancelOffer | 28 |
| NFTokenAcceptOffer | 29 |
| AMMCreate | 35 |
| AMMDeposit | 36 |
| AMMWithdraw | 37 |
| AMMVote | 38 |
| AMMBid | 39 |
| AMMDelete | 40 |

---

## Payment
```
TransactionType = 0
```
| Field | Type | Req | Notes |
|---|---|---|---|
| Amount | Amount | ✓ | Delivered amount; XRP or IOU |
| Destination | AccountID | ✓ | |
| DestinationTag | UInt32 | Opt | |
| InvoiceID | Hash256 | Opt | |
| Paths | PathSet | Opt | Required for cross-currency |
| SendMax | Amount | Opt | Max to debit; required for cross-currency |
| DeliverMin | Amount | Opt | Partial payments |

**Flags:**
- `0x00010000` = tfNoRippleDirect
- `0x00020000` = tfPartialPayment
- `0x00040000` = tfLimitQuality

---

## OfferCreate
```
TransactionType = 7
```
| Field | Type | Req | Notes |
|---|---|---|---|
| TakerPays | Amount | ✓ | What you want to receive |
| TakerGets | Amount | ✓ | What you offer to give |
| Expiration | UInt32 | Opt | Ripple epoch seconds |
| OfferSequence | UInt32 | Opt | Sequence of offer to replace |

**Flags:**
- `0x00010000` = tfPassive
- `0x00020000` = tfImmediateOrCancel
- `0x00040000` = tfFillOrKill
- `0x00080000` = tfSell

---

## OfferCancel
```
TransactionType = 8
```
| Field | Type | Req | Notes |
|---|---|---|---|
| OfferSequence | UInt32 | ✓ | Sequence of offer to cancel |

---

## TrustSet
```
TransactionType = 20
```
| Field | Type | Req | Notes |
|---|---|---|---|
| LimitAmount | Amount (IOU) | ✓ | currency+issuer+limit value |
| QualityIn | UInt32 | Opt | Inbound quality (0 = default) |
| QualityOut | UInt32 | Opt | Outbound quality (0 = default) |

**Flags:**
- `0x00020000` = tfSetfAuth
- `0x00040000` = tfSetNoRipple
- `0x00080000` = tfClearNoRipple
- `0x00100000` = tfSetFreeze
- `0x00200000` = tfClearFreeze

---

## AccountSet
```
TransactionType = 3
```
| Field | Type | Req | Notes |
|---|---|---|---|
| ClearFlag | UInt32 | Opt | AccountFlag to clear |
| SetFlag | UInt32 | Opt | AccountFlag to set |
| Domain | Blob | Opt | Lowercase hex of domain string |
| EmailHash | Hash128 | Opt | MD5 of email |
| MessageKey | Blob | Opt | Public key for encrypted messages |
| TransferRate | UInt32 | Opt | 10^9 = 0% fee |
| TickSize | UInt8 | Opt | 3–15 or 0 to disable |

**AccountFlags:**
- `asfRequireDest = 1`
- `asfRequireAuth = 2`
- `asfDisallowXRP = 3`
- `asfDisableMaster = 4`
- `asfDefaultRipple = 8`
- `asfDepositAuth = 9`

---

## EscrowCreate
```
TransactionType = 1
```
| Field | Type | Req | Notes |
|---|---|---|---|
| Amount | Amount (XRP) | ✓ | Locked amount |
| Destination | AccountID | ✓ | |
| DestinationTag | UInt32 | Opt | |
| CancelAfter | UInt32 | Opt | Ripple epoch; tx fails if ledger > this |
| FinishAfter | UInt32 | Opt | Ripple epoch; can finish after this |
| Condition | Blob | Opt | PREIMAGE-SHA-256 crypto-condition |

## EscrowFinish
```
TransactionType = 2
```
| Field | Type | Req | Notes |
|---|---|---|---|
| Owner | AccountID | ✓ | Account that created escrow |
| OfferSequence | UInt32 | ✓ | Sequence of EscrowCreate tx |
| Condition | Blob | Opt | Required if escrow has condition |
| Fulfillment | Blob | Opt | Preimage satisfying condition |

## EscrowCancel
```
TransactionType = 4
```
| Field | Type | Req | Notes |
|---|---|---|---|
| Owner | AccountID | ✓ | |
| OfferSequence | UInt32 | ✓ | |

---

## NFTokenMint
```
TransactionType = 25
```
| Field | Type | Req | Notes |
|---|---|---|---|
| NFTokenTaxon | UInt32 | ✓ | Arbitrary grouping integer |
| Issuer | AccountID | Opt | If minting on behalf of another |
| TransferFee | UInt16 | Opt | 0–50000 (basis points × 10) |
| URI | Blob | Opt | Max 512 bytes, hex-encoded URL |

**Flags:**
- `0x0001` = tfBurnable
- `0x0002` = tfOnlyXRP
- `0x0004` = tfTrustLine (deprecated)
- `0x0008` = tfTransferable

## NFTokenBurn
```
TransactionType = 26
```
| Field | Type | Req | Notes |
|---|---|---|---|
| NFTokenID | Hash256 | ✓ | 32-byte token ID |
| Owner | AccountID | Opt | If burning from another account |

## NFTokenCreateOffer
```
TransactionType = 27
```
| Field | Type | Req | Notes |
|---|---|---|---|
| NFTokenID | Hash256 | ✓ | |
| Amount | Amount | ✓ | 0 for transfer offers |
| Owner | AccountID | Opt | For buy offers |
| Destination | AccountID | Opt | Restricted offer |
| Expiration | UInt32 | Opt | |

**Flags:**
- `0x0001` = tfSellNFToken (sell offer; else buy offer)

## NFTokenAcceptOffer
```
TransactionType = 29
```
| Field | Type | Req | Notes |
|---|---|---|---|
| NFTokenBuyOffer | Hash256 | Opt | |
| NFTokenSellOffer | Hash256 | Opt | |
| NFTokenBrokerFee | Amount | Opt | Broker mode only |

---

## AMMCreate
```
TransactionType = 35
```
| Field | Type | Req | Notes |
|---|---|---|---|
| Amount | Amount | ✓ | First asset deposit |
| Amount2 | Amount | ✓ | Second asset deposit |
| TradingFee | UInt16 | ✓ | 0–1000 (basis points) |

## AMMDeposit
```
TransactionType = 36
```
| Field | Type | Req | Notes |
|---|---|---|---|
| Asset | STObject (Currency) | ✓ | First pool asset |
| Asset2 | STObject (Currency) | ✓ | Second pool asset |
| Amount | Amount | Opt | |
| Amount2 | Amount | Opt | |
| EPrice | Amount | Opt | |
| LPTokenOut | Amount | Opt | |

**Flags:** `0x000800` = tfSingleAsset, `0x001000` = tfTwoAsset, etc.

## AMMWithdraw
```
TransactionType = 37
```
Same `Asset`/`Asset2` as Deposit; amount fields control how much to withdraw.

---

## Memo Object (STObject inside Memos STArray)
```
Memo:
  MemoData:   Blob  (hex-encoded arbitrary data)
  MemoFormat: Blob  (hex-encoded MIME type, e.g., "text/plain")
  MemoType:   Blob  (hex-encoded URI describing purpose)
```
All three fields are optional individually, but at least one must be present.

---

## Ripple Epoch
XRPL timestamps use Ripple Epoch: seconds since **January 1, 2000 00:00 UTC**.
Unix timestamp → Ripple Epoch: `unix_ts - 946684800`
