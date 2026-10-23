# Parity with `rippled` 3.4.1

An XRP Ledger library is only useful if the server agrees with it: a
transaction whose bytes, hash or signature differ in one bit from what
`rippled` computes is rejected or, worse, lands with a different ID than the
one you are waiting for. This report lists every way this SDK is checked
against `rippled`, with the numbers, the test that holds each check in place,
and how to repeat it.

**Reference server:** `rippled` (xrpld) 3.4.1, the release running on mainnet
in October 2026. The SDK embeds that release's `server_definitions` (82
transaction types, 31 ledger entry types, 357 fields, 195 result codes), so
the codec knows exactly the fields `rippled` 3.4.1 knows: an unknown field is
an error here, as it is there, both when reading bytes and when writing JSON
(a misspelt `DestinationTag` is refused, never dropped from what is signed),
and an integer too large for its field is refused, never wrapped.

**Last full run:** 2026-10-23, halcyon-xrpl-* 0.6.0, Rust 1.96.0, Linux x86-64.
`cargo test --workspace` runs everything below except the live Testnet test
and the fuzzers, offline, in well under a minute.

## Summary

| Area | Checked against | Count | Where |
|---|---|---|---|
| Transaction bytes and hash, one of every type | `rippled` 3.4.1 `sign` (offline) | 79 types, every field filled | `crates/xrpl-core/tests/transaction_models.rs` |
| Real transactions: typed, re-encoded, re-hashed | Mainnet and Testnet ledgers | 186 transactions, 51 types | `crates/xrpl-core/tests/transaction_models.rs` |
| Every transaction type `rippled` defines | The two sets above together | 82 of 82 | `every_type_is_checked_against_rippled` |
| Codec both ways (JSON → bytes → JSON) | XRPLF codec fixtures (also used by xrpl.js) | 34 transactions + 261 ledger entries | `crates/xrpl-core/tests/codec_fixtures.rs` |
| `Number` (`STNumber`) reading and writing | `rippled` 3.4.1 `simulate` | 37 byte forms, 27 written values, 25 refusals | `crates/xrpl-core/src/codec/number.rs` |
| Amounts: XRP, tokens, MPTs | `rippled` 3.4.1 `simulate` | 27 byte forms, 18 JSON values | `codec/amount.rs`, `codec/encode.rs`, `codec/decode.rs` |
| Token values from JSON: syntax, rounding, range | `rippled` 3.4.1 `simulate` | 26 values, 16 refusals | `types/amount.rs` (`reads_token_values_as_rippled_does`) |
| Payment paths, including MPT steps | `rippled` 3.4.1 `simulate` | 13 byte forms, 10 JSON values | `codec/decode.rs`, `codec/encode.rs` |
| Objects and arrays: markers, duplicates, element types | `rippled` 3.4.1 `simulate` | 11 transactions | `codec/decode.rs` |
| `Issue` (vault and AMM assets): XRP, token, MPT | `rippled` 3.4.1 `simulate` | 6 byte forms | `codec/decode.rs` |
| Signatures (secp256k1 and Ed25519) | `rippled` 3.4.1 `sign` and `sign_for` | Byte for byte, both key types | `crypto/signing.rs`, `xrpl-sdk/src/wallet.rs` |
| Payment channel claims | `rippled` 3.4.1 `channel_authorize` | Byte for byte | `crypto/channel.rs` |
| Batch signing data | Batches `rippled` accepted | 6 of 6 verify; tampering fails | `transaction/batch.rs` |
| Ledger entry IDs (keylets) | Mainnet entries' `index`, and `rippled` 3.4.1 `ledger_entry` | 14 entries, 12 types; 9 more IDs of 6 more kinds | `hashes.rs` |
| Ledger entries: typed and back | Mainnet ledger #107827070 | 20 entries, 15 types, exact JSON | `crates/xrpl-core/tests/ledger_objects.rs` |
| Every typed response | Recorded `rippled` 3.4.1 and Clio responses | 44 methods, no field lost | `crates/xrpl-client/tests/rippled_responses.rs` |
| Every stream event | Recorded `rippled` 3.4.1 stream messages | 9 messages, one of each kind | `crates/xrpl-client/tests/stream_events.rs` |
| End to end | The XRPL Testnet | 15 kinds of entry ID, 2 batches, multi-signing, every hash | `crates/xrpl-sdk/tests/live_testnet.rs` |
| Never panics, round trips hold | libFuzzer + AddressSanitizer | 8 targets, about 86 million inputs in the final run | `fuzz/` |

## 1. The binary codec

**One of every transaction type, signed by `rippled`.** For each of the 79
transaction types a user can submit, an example with every field filled was
signed by `rippled` 3.4.1 (`sign` in offline mode, which needs no network).
For each one the test requires that:

- the typed model reads `rippled`'s `tx_json` with no field left over,
- writes it back as exactly the same JSON,
- encodes it to exactly `rippled`'s `tx_blob`, and
- hashes it to exactly `rippled`'s `hash`.

`rippled`'s own `tx_blob` is the ground truth here. Since every field of
every transaction format is filled, `rippled` itself encodes each of those
fields at least once, including the unusual types: `Number`, `Issue`,
`XChainBridge`, `PathSet` and nested `STArray`s.

**Real transactions.** 186 transactions that mainnet or Testnet applied, 51
types between them, including the three pseudo-transactions (`EnableAmendment`,
`SetFee`, `UNLModify`) only validators produce. Each must parse strictly into
its own typed model, serialize back to the same JSON, encode to the same bytes
and hash to the ID the ledger gave it. 146 of them are signed with Ed25519, 2
are multi-signed, 9 are batches.

**Every type.** Between the two sets, all 82 transaction types in `rippled`
3.4.1's definitions are covered; a test fails if a future definitions update
adds a type no fixture exercises.

**What the encoder refuses**, as `rippled`'s JSON parser refuses it (tests in
`codec/encode.rs`): a field name it does not define (at the top level, inside
inner objects, and as an array element's name; the keys `rippled` adds to its
responses, such as `hash` and `meta`, are skipped); an integer too large for
its UInt8, UInt16 or UInt32 field; an array element that is not exactly one
inner object; a token amount in XRP's currency code; an `Issue` with a field
its kind does not have; and a Payment whose `DeliverMax` differs from its
`Amount` (alone, `DeliverMax` is the `Amount`, as in `rippled`). Inner
objects are signed whole, as `STObject::add` writes them; only the top level
leaves out the fields that are not signed.

**The XRPLF codec fixtures.** `tests/vectors/codec-fixtures.json`, the shared
vectors the xrpl.js codec is tested against: 34 transactions and 261 ledger
entries. Both directions are required for every entry: the JSON encodes to the
fixture's bytes, and the bytes decode to the fixture's JSON.

**`Number`.** The decimal type of vaults and loans (`STNumber`) has the most
intricate rules in the format, so it was put to `rippled` 3.4.1 directly
(`simulate`, which applies nothing; script:
`crates/xrpl-core/scripts/probe_number.py`):

- 37 byte forms, including non-normalized mantissas, `i64::MIN`, and
  exponents at, beyond and below ±32,768. The SDK shows each exactly as
  `rippled` does, and refuses the three that `rippled` refuses.
- 23 JSON strings (`"1.5"`, `"1E5"`, `"+5"`, `"0.0000000123"`, `"1e32768"`,
  `"9223372036854775810"`, …) and 4 JSON integers. The SDK writes `rippled`'s
  bytes and shows `rippled`'s text.
- 25 values `rippled` refuses rather than round: more than 19 significant
  digits, digits beyond 64 bits, `"00012"`, `".5"`, `"5."`, exponent overflow
  and underflow, and JSON integers beyond 32 bits. The SDK refuses each.
  It never rounds a value silently.

**Amounts.** Also put to `rippled` 3.4.1 through `simulate`, as bytes and as
JSON:

- XRP's sign. A negative amount (bit 62 clear) reads as `"-100"`, and
  negative zero is refused.
- A token amount is accepted only in `rippled`'s canonical form: a mantissa of
  10^15 to 10^16 - 1 and an exponent of -96 to 80, or zero written as exactly
  `0x8000000000000000`. Its currency and its issuer must not be XRP's (all
  zeros).
- XRP in JSON. `"-100"`, `"+100"`, `"1e2"` and the integer `100` are written
  as `rippled` writes them. `"1.5"`, `"00100"` and `" 100"` are refused, as
  `rippled` refuses them.
- MPT amounts. Whole units up to `i64::MAX`, in the same number syntax
  (`"1e3"` is 1000, `"-0"` is 0). A negative zero reads as `"0"`, and lead-byte
  bits other than the MPT and sign bits are ignored, as in `rippled`. A
  magnitude above `i64::MAX`, which `rippled` cannot write as JSON, is
  refused.

**Payment paths.** Also put to `rippled` 3.4.1 through `simulate`:

- Path steps convert into a currency or, new in 3.4.1, an MPT (type 64,
  `mpt_issuance_id`). Both are read and written as `rippled` does.
- An empty path, an empty PathSet, unknown type bits, and a step with both a
  currency and an MPT are refused, as `rippled` refuses them.

**Issues.** The asset type of vaults and AMMs reads as `rippled` reads it
(`STIssue`): XRP (20 zero bytes), a token (currency and issuer; an all-zero
issuer is refused), or an MPT (issuer, then `NO_ACCOUNT`, then the sequence;
refused without the sequence).

**Objects and arrays.** As `rippled` reads them (`STObject::set`,
`STArray`):

- A field that appears twice in an object is refused, at any depth.
- An end-of-array marker inside an object, an end-of-object marker inside an
  array, and an array element that is not an object are refused.
- Fields out of canonical order are read (and re-encoded in order), as
  `rippled` reads them.

**Other `rippled`-specific rules** checked by unit tests in `codec/`:

- `UInt64` fields take hex, except the MPT amount fields, which take base 10.
- `Int32`.
- `PermissionValue` names, both granular permissions and transaction types.
- An `AccountID` field that was never set is `""`.
- The currency code `"1"` (`noCurrency`) is accepted.
- A mainnet `DelegateSet` decodes exactly as `rippled`'s binary `tx` shows it.

## 2. Keys and signatures

- **Keys and addresses.** Derived from a seed, they are the ones `rippled`'s
  `wallet_propose` gives, for both secp256k1 and Ed25519. Checked with
  xrpl.js's keypair fixtures, including their signatures.
- **Single signatures.** `sign_transaction` produces `rippled` 3.4.1's
  `sign` output byte for byte, for both key types. Both schemes are
  deterministic (RFC 6979 for secp256k1; Ed25519 by design), so equal bytes
  mean equal keys, equal signing data and equal encoding.
- **Hash before signing.** As `rippled` verifies (`PublicKey.cpp`),
  secp256k1 signs the SHA-512Half of the signing data and Ed25519 signs the
  data itself.
- **Multi-signatures.** `sign_for` signatures match `rippled` 3.4.1's
  `sign_for` byte for byte.
- **Payment channel claims.** They match `rippled`'s `channel_authorize`
  byte for byte, and a claim made by the SDK was accepted on Testnet.
- **Batch signatures.** All 6 Testnet batches with `BatchSigners` in the
  recorded set, which `rippled` accepted, verify against the SDK's batch
  signing data (`rippled` 3.4.1's layout, which includes the outer account
  and sequence). A single changed byte makes all of them fail.

## 3. Hashes and IDs

- **Transaction IDs.** All 265 transactions above hash to their `rippled` IDs.
- **Ledger entry IDs.** Of the mainnet sample, the 14 entries whose ID
  follows from their own fields hash to the `index` `rippled` gave them:
  AccountRoot, AMM, Amendments, Check, Delegate, DirectoryNode (owner root),
  FeeSettings, LedgerHashes, Offer, RippleState, SignerList and Ticket. On
  Testnet, every entry the live test creates is checked the same way (below).
- **The other kinds.** For entries no sample holds, `rippled` 3.4.1's
  `ledger_entry` reports the index it computes even when the entry does not
  exist; the SDK's IDs match it for `Bridge` (from either door, XRP and token
  bridges), `XChainOwnedClaimID`, `XChainOwnedCreateAccountClaimID`,
  `DepositPreauth` by credentials (one and two credentials, in either order)
  and `Sponsorship`. An order-book directory's ID matches a mainnet
  `DirectoryNode`'s `RootIndex`.
- **A blob's ID** is the hash of `rippled`'s own re-serialization: a signed
  blob whose fields were reordered on the way still has the ID the ledger
  shows.
- **NFT and MPT IDs.** `NFTokenID` matches `rippled`'s on a mainnet mint and
  on Testnet; `MPTokenIssuanceID` on Testnet.
- **Transaction cost.** `transaction::cost` follows `rippled` 3.4.1's
  `calculateBaseFee` for every type that overrides it (owner reserve for
  `AccountDelete`, `AMMCreate` and `LedgerStateFix`; 32 + *n*/16 extra base
  fees for an `EscrowFinish` fulfillment of *n* bytes; 9 extra for
  Confidential MPT; counterparty signatures for `LoanSet`; every part of a
  `Batch`). `rippled`'s `simulate` on mainnet refuses one drop less than the
  computed cost and accepts the cost itself, for `AccountDelete` (200,000
  drops) and a 100-byte fulfillment (390 drops)
  (`crates/xrpl-core/scripts/probe_cost.py`).

## 4. Typed ledger entries and responses

- **Ledger entries.** All 31 entry types have typed models generated from the
  definitions. The 20 mainnet entries sampled from ledger #107827070 (the
  fewest-field and most-field sample of each of 15 types) parse into their
  typed variant and serialize back to exactly `rippled`'s JSON.
- **Responses.** Every typed response is checked against a response recorded
  from a real server: 44 methods from `rippled` 3.4.1 on mainnet, and Clio
  (`s2.ripple.com`) for the Clio-only methods. Each must parse into its
  request's response type and serialize back with every field the server
  sent. A field the types do not model fails the test instead of vanishing.
- **Streams.** 9 recorded stream messages (ledger, transactions, proposed
  transactions, validations, book changes, order books, server, peer status,
  consensus) each become their own typed event and keep every field.

## 5. Live, on the XRPL Testnet

`cargo test -p halcyon-xrpl-sdk --features live-tests` funds two wallets from
the Testnet faucet and submits one transaction of each kind that creates a
ledger entry. The following are checked against what the network did:

- **Hashes.** Every submitted transaction's hash, as `hash_signed_tx_blob`
  computes it before submission, against the hash `rippled` reports.
- **Entry IDs.** The ID of every created entry, as the matching `keylet`
  computes it: ticket, owner directory, check, escrow, payment channel, NFT
  offer, DID, oracle, MPT issuance, MPToken, credential, permissioned domain,
  deposit preauthorization, delegate and trust line (15 kinds).
- **IDs read from metadata.** `get_nftoken_id` (from the NFT page diff) and
  the MPT issuance ID.
- **Balances.** `get_balance_changes` against the validated metadata.
- **Channel claims.** A payment channel claim signed by the SDK, which the
  network accepted.
- **Batches.** A one-account batch, and a two-account batch with a
  `BatchSigner`. Every inner transaction lands under the ID
  `inner_transaction_ids` computed.
- **Multi-signing.** A signer list, then a multi-signed transaction through
  `submit_multisigned`.
- **Requests.** The typed requests for what was created (`account_channels`,
  `nft_sell_offers`, `get_aggregate_price`, and Clio's `mpt_holders`) parse
  what the servers answer.

The last full run checked 29 of these. With `XRPL_RECORD_DIR=<dir>`, it also writes the
responses in the format of the recorded fixtures, so the response fixtures can
be refreshed from a live network.

## 6. Fuzzing

Eight libFuzzer targets (`fuzz/`) run with AddressSanitizer, from a corpus
seeded with all the real data above (`fuzz/seed_corpus.py`). They require:

- the decoder, encoder, amount codec, address and seed parsers, hashes, Batch
  helpers, typed models and utilities never panic on any input;
- a typed transaction writes back what it read;
- whatever decodes encodes again, and decoding that gives the same JSON.

| Target | Inputs in the final run (2026-10-22, 2 minutes each) |
|---|---|
| `fuzz_decode` | 2,554,663 |
| `fuzz_roundtrip` | 1,984,477, after 13,976,235 in a 15-minute run |
| `fuzz_encode` | 4,559,482 |
| `fuzz_models` | 743,248 |
| `fuzz_hashes` | 1,088,519 |
| `fuzz_utils` | 1,681,292 |
| `fuzz_amount` | 58,048,086 |
| `fuzz_address` | 1,245,231 |

None of them found anything in that run, with AddressSanitizer on. Earlier
runs found the six issues below, each fixed before the next run.

CI fuzzes every target for a minute on every pull request and for 15 minutes
every Monday (`.github/workflows/fuzz.yml`).

**What fuzzing found.** Six issues, all on bytes `rippled` would not have
written itself, and all now fixed with `rippled`'s answers as the tests.

The first: the round-trip target found that `Number` bytes not in
normal form decoded to a different text than `rippled` shows. For example, a
mantissa of 13830756765796644 with exponent 1 decoded to
`"138307567657966440"`, where `rippled` shows `"13830756765796644e1"`. Probing
`rippled` 3.4.1 then showed two more differences, both on input `rippled`
refuses:

- the encoder rounded values beyond 19 significant digits instead of refusing
  them;
- it accepted forms such as `"00012"` and `".5"`.

All three now follow `rippled` exactly (section 1).

The second, from the same target after that fix: a token amount whose
mantissa was not normalized (`274877906944e-97`) decoded, but could not be
encoded again. `rippled` refuses such bytes outright. Probing its amount rules
then showed that a negative XRP amount decoded without its sign. The decoder
now refuses what `rippled` refuses and keeps XRP's sign (section 1).

The third: an MPT amount with a magnitude above `i64::MAX` decoded but could
not be encoded. `rippled` cannot write such an amount as JSON, so no ledger
holds one; the decoder now refuses it. The encoder also now checks that an
`mpt_issuance_id` is 24 bytes, and reads MPT values with `rippled`'s number
syntax.

The fourth: a PathSet with empty paths and a step with unknown type bits
decoded, and re-encoding it wrote the empty step as the PathSet's end marker,
so the bytes that followed no longer decoded. `rippled` refuses such bytes.
Probing its path rules then showed the MPT path step that 3.4.1 added, which
the codec did not know. Paths now follow `rippled` (section 1), MPT steps
included.

The fifth: a `Number` whose mantissa, once normalized, exceeded `rippled`'s
limit at the top exponent (`kMaxRepUp`, 9,223,372,036,854,775,810). `rippled`
refuses such bytes; the decoder accepted them.

The sixth: end-of-object and end-of-array markers in the wrong places, and an
array element that was not an object, decoded as if they were ordinary fields.
`rippled` refuses them, and also refuses a field that appears twice, which the
decoder had silently overwritten. Objects and arrays now follow `rippled`
(section 1).

## 7. Deliberate differences, and what is not checked

Six places where the SDK deliberately does not do what `rippled` does are
marked below; each refuses or keeps more than `rippled` would, never less.
The rest are places this report makes no claim.

- **Transaction rules beyond the format.** The SDK encodes any well-formed
  transaction. Rules `rippled` applies when it checks a transaction
  (preflight and preclaim) are the server's, and are not reproduced. Examples:
  an XRP amount above the total supply, or a `Number` rounded to an asset's
  precision.
- **Server-side computation.** Path finding, fee escalation and consensus are
  answered by the server, not computed by the SDK.
- **API version 2.** Responses are typed for `rippled`'s API version 1, the
  default when a request names none.
- **XRP above 10^17 drops (deliberate difference).** `rippled`'s JSON parser refuses XRP amounts
  above the total supply. The codec writes them, up to 2^61 - 1 drops,
  because mainnet ledger entries hold such amounts (an offer asking ten times
  the supply exists at ledger #106455034) and the codec must reproduce them.
- **Empty payment paths (deliberate difference).** `rippled`'s JSON parser
  accepts an empty path (`[[]]`), but writes bytes its own binary reader then
  refuses, so the transaction can be signed but never submitted. The SDK
  refuses an empty path when encoding.
- **Bytes after the end (deliberate difference).** Reading a transaction,
  `rippled` stops at an end-of-object marker and ignores any bytes after it,
  so fields after the marker are silently dropped. The SDK accepts the marker
  as the last byte and refuses anything after it.
- **Inner object templates.** Which fields an inner object may hold (a
  `Memo` only its three memo fields, for example) is checked by `rippled`
  later, when it applies the transaction; neither `rippled`'s reader nor the
  SDK's decoder checks it.
- **Path steps' `type` field.** `rippled` adds a derived `type` to each path
  step it shows; the SDK's decoder leaves it out, as the XRPL codec fixtures
  (and xrpl.js) do, and the encoder ignores it, as `rippled` does.
- **Long token values (deliberate difference).** `rippled`'s JSON reader
  takes a token value's digits as one 64-bit integer, so it refuses a value
  with more digits than that. The SDK reads any number of digits exactly and
  rounds past the sixteenth as `rippled` rounds (to nearest, ties to even);
  for every value `rippled` reads, the bytes are the same.
- **Ed25519 verification (deliberate difference).** The SDK verifies strictly
  (`verify_strict`), which also refuses small-order public keys and `R`
  values; it can refuse a signature `rippled` would accept from such a key,
  never accept one `rippled` refuses.
- **Other JSON amount forms (deliberate difference).** `rippled` also reads amounts written as
  `"10/USD/rIssuer"` strings or as arrays. The SDK reads the standard forms:
  a drops string for XRP, and an object for tokens and MPTs.
- **Cross-chain bridge transactions.** They are encoded byte for byte against
  `rippled`'s `sign`, but no public network runs them, so they have not been
  submitted live.

## Repeating the checks

```bash
cargo test --workspace                                    # all offline checks
cargo test -p halcyon-xrpl-core --test codec_fixtures -- --nocapture   # with counts
cargo test -p halcyon-xrpl-sdk --features live-tests -- --nocapture    # Testnet, ~2 minutes

python3 fuzz/seed_corpus.py                               # fuzzing (nightly Rust)
cargo +nightly fuzz run fuzz_roundtrip -- -max_total_time=120

# Against your own rippled (admin RPC; simulate applies nothing):
RIPPLED_RPC=http://127.0.0.1:5005 python3 crates/xrpl-core/scripts/probe_number.py \
  encode 1.5 138307567657966440 && \
  python3 crates/xrpl-core/scripts/probe_number.py decode 13830756765796644,1
```

A difference from `rippled` is a bug. Please report it with the input, what
the SDK did and what `rippled` did:
<https://github.com/ilubxrp589/xrpl-sdk/issues>.
