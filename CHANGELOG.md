# Changelog

## v0.1.0 (2026-03-19)

Initial release.

### xrpl-core
- Binary codec: encode/decode XRPL transactions (33/34 codec fixtures pass)
- Ed25519 and Secp256k1 key derivation and signing
- XRPL primitive types: AccountId, Amount, IouAmount, Currency, Hash128/160/256, Blob
- Base58check address encoding/decoding
- Transaction type enum (24 transaction types)
- Typed transaction builders with validation for all 24 transaction types
- Reserve calculation utilities (base_reserve, owner_reserve, available_balance)
- DEX utilities (amount_to_f64, offer_quality, midpoint_price, spread, liquidity)
- Multi-signing: encode_for_multisigning
- `no_std` support for types and crypto (codec requires std)

### xrpl-client
- HTTP JSON-RPC client with 18+ methods
- WebSocket client with subscriptions and automatic reconnection
- account_info, account_lines, account_offers, account_nfts, account_tx, account_objects, account_currencies, account_channels
- book_offers, amm_info, ledger, ledger_current, ledger_entry, server_info, gateway_balances
- fee, submit, tx
- Autofill (concurrent fetch of Sequence, Fee, LastLedgerSequence)
- submit_and_wait with polling until validated or expired
- Pagination helpers (_all variants) with 50-page safety cap
- TransactionExpired and PaginationLimitReached error variants
- tracing instrumentation on all RPC calls

### xrpl-sdk
- Wallet: generate, from_seed, sign_transaction, sign_and_encode
- Multi-signing: sign_for_multisigning, collect_signers
- autofill_and_sign and submit_transaction convenience functions
- Full re-exports from xrpl-core and xrpl-client
- 5 example programs: send_payment, subscribe_ledger, place_offer, check_balance, decode_blob, mint_nft
