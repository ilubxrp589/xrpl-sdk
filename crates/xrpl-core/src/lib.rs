//! # halcyon-xrpl-core
//!
//! The XRP Ledger's data without the network: types, the binary codec, keys
//! and signatures, transaction hashes, ledger entry IDs and everyday
//! conversions. It is the foundation of `halcyon-xrpl-sdk`; use it on its own
//! for offline work such as signing services, hardware-wallet bridges,
//! decoding ledger data or indexing.
//!
//! ```toml
//! [dependencies]
//! halcyon-xrpl-core = "0.5"   # imported as `xrpl_core`
//! ```
//!
//! ## Modules
//!
//! | Module | Holds |
//! |---|---|
//! | [`address`] | classic addresses, X-addresses (XLS-5d), seeds: base58check with XRPL's alphabet |
//! | [`crypto`] | Ed25519 and secp256k1 keys from seeds; signing exactly as `rippled` verifies; payment channel claims |
//! | [`codec`] | JSON ↔ binary for transactions and ledger entries, driven by `rippled` 3.4.1's definitions |
//! | [`hashes`] | transaction IDs, and [`hashes::keylet`]: the ID of every kind of ledger entry |
//! | [`types`] | [`Amount`] (XRP, token, MPT), [`AccountId`], [`Currency`], hashes, and [`LedgerObject`]: all 31 ledger entry types, typed |
//! | [`transaction`] | [`transaction::models`]: a typed struct for each of `rippled`'s 82 transaction types, with its flags; [`transaction::batch`]: `Batch` (XLS-56); builders for common transactions |
//! | [`utils`] | XRP ↔ drops, ledger time, text ↔ hex, rates, NFTokenIDs, balance changes, reserves |
//! | [`error`] | [`CoreError`] |
//!
//! ## Decode a signed transaction, and encode it back
//!
//! The codec turns `rippled`'s binary form into the JSON `rippled` and
//! xrpl.js use, and back, byte for byte. This blob is a payment `rippled`
//! 3.4.1 signed:
//!
//! ```
//! use xrpl_core::codec::{decode_transaction_binary, encode_transaction_json};
//!
//! let blob = hex::decode(
//!     "120000220000000024000000016140000000000F424068400000000000000C7321030D58EB48B4420B1F7B9DF55087E0E29FEF0E8468F9A6825B01CA2C361042D43574473045022100BC95504DC72DDBDAA0F5EAA973EC07E057FD9A3E7500AC3CF07454C68219985E02200B0992C47857A867F5664BB6F74E62F086A59A0C31848E0FB3819127E8EC0A1281148049717CC948789F32F267ADC2582484E3DFA6988314B5F762798A53D543A014CAF8B297CFF8F2F937E8",
//! )?;
//! let tx = decode_transaction_binary(&blob)?;
//! assert_eq!(tx["TransactionType"], "Payment");
//! assert_eq!(tx["Amount"], "1000000");
//! assert_eq!(tx["Destination"], "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
//!
//! assert_eq!(encode_transaction_json(&tx, false)?, blob);
//!
//! // Its ID, as rippled reported it.
//! assert_eq!(
//!     xrpl_core::hashes::transaction_id(&blob).to_string(),
//!     "4A37F580F84306C73D3E38D9EB7C183A39154223CB173ACDD18BBB5C06BDD62C",
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Keys and addresses
//!
//! ```
//! use xrpl_core::address::{classic_to_x_address, x_address_to_classic, KeyType};
//! use xrpl_core::crypto::{Keypair, Seed};
//!
//! // A public test seed (never fund it on mainnet).
//! let seed = Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?;
//! assert_eq!(seed.key_type(), KeyType::Secp256k1);
//! let keys = Keypair::from_seed(&seed)?;
//! assert_eq!(keys.classic_address(), "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1");
//!
//! // X-addresses carry a destination tag inside the address.
//! let x = classic_to_x_address("rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1", Some(12345), false)?;
//! assert_eq!(x_address_to_classic(&x)?, ("rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1".to_string(), Some(12345), false));
//! # Ok::<(), xrpl_core::CoreError>(())
//! ```
//!
//! Signing: [`crypto::sign_transaction`] signs a JSON transaction, the way
//! `rippled` verifies it (secp256k1 over SHA-512Half of the signing data,
//! Ed25519 over the data itself).
//!
//! ## Typed ledger entries
//!
//! [`LedgerObject`] parses any entry `rippled` returns (from `ledger_entry`,
//! `account_objects` or `ledger_data`) into a struct for its type, and
//! serializes back to exactly `rippled`'s JSON. Types this crate does not
//! know stay available as [`LedgerObject::Other`]; an entry of a known type
//! that does not fit it (a field a newer `rippled` added) is
//! [`LedgerObject::Unparsed`], with the error, inside a response, and an
//! error from [`LedgerObject::from_value`].
//!
//! ```
//! use xrpl_core::types::{Amount, LedgerObject};
//!
//! let entry = LedgerObject::from_value(serde_json::json!({
//!     "LedgerEntryType": "AccountRoot",
//!     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
//!     "Balance": "148446663",
//!     "Flags": 0,
//!     "OwnerCount": 0,
//!     "PreviousTxnID": "0D5FB50FA65C9FE1538FD7E398FFFE9D1908DFA4576D8D7A020040686F93C77D",
//!     "PreviousTxnLgrSeq": 14091160,
//!     "Sequence": 336,
//! }))?;
//! match entry {
//!     LedgerObject::AccountRoot(a) => assert_eq!(a.balance, Amount::Xrp(148_446_663)),
//!     other => panic!("not an AccountRoot: {}", other.entry_type()),
//! }
//! # Ok::<(), serde_json::Error>(())
//! ```
//!
//! ## Typed transactions
//!
//! [`transaction::models`] has a struct for each of the 82 transaction types
//! `rippled` 3.4.1 defines, generated from its own definitions: every field
//! under `rippled`'s name, `new` with the required ones, and the flags as
//! constants. [`transaction::models::Tx`] parses any transaction strictly (a
//! field its type does not have is an error).
//!
//! ```
//! use xrpl_core::transaction::models::{OfferCreate, Tx, TxModel};
//! use xrpl_core::types::Amount;
//!
//! // Sell 25 XRP for 10 USD: the taker pays 10 USD and gets 25 XRP.
//! let mut offer = OfferCreate::new(
//!     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
//!     Amount::token("10", "USD", "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")?, // TakerPays
//!     Amount::Xrp(25_000_000),                                          // TakerGets
//! );
//! offer.common.flags = Some(OfferCreate::TF_IMMEDIATE_OR_CANCEL);
//! let json = offer.to_json()?;
//! assert_eq!(json["Flags"], 0x0002_0000);
//! assert!(matches!(Tx::from_value(json)?, Tx::OfferCreate(_)));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [`transaction::batch`] builds and signs `Batch` transactions: inner
//! transactions, `BatchSigners` from other accounts, and the fee.
//!
//! ## Without `std`
//!
//! With `default-features = false` the crate builds as `no_std` + `alloc`:
//! [`address`], [`crypto`] key derivation and signing, and the primitive
//! [`types`]. The codec, hashes, JSON support and most [`utils`] need the
//! `std` feature (on by default).
//!
//! ```toml
//! halcyon-xrpl-core = { version = "0.5", default-features = false }
//! ```
//!
//! ## How it is checked
//!
//! Signatures, multi-signatures, transaction hashes and channel claims match
//! `rippled` 3.4.1's `sign`, `sign_for` and `channel_authorize` byte for
//! byte. The codec passes all 34 transaction and 261 ledger-object fixtures
//! of the XRPL codec test suite. Every transaction type is checked against
//! `rippled`: 186 real transactions (51 types) round-trip through their typed
//! models to the same bytes and ID, and an example of each of the other types,
//! signed by `rippled` 3.4.1, encodes to `rippled`'s exact `tx_blob`. Batch
//! signatures verify against real batches `rippled` accepted. Ledger entry IDs match real mainnet entries
//! and a live Testnet run. The utilities pass xrpl.js's own fixtures.
//!
//! Where the format has edge cases (`Number`, amounts, token values, payment paths), each case
//! was put to `rippled` 3.4.1 itself, and the codec reads, writes and refuses what `rippled`
//! does; the few deliberate differences (each safer than `rippled`'s JSON reader) are listed in
//! the parity report. A field name `rippled` does not define, or an integer too large for its
//! field, is an error, never dropped or wrapped.
//! Eight fuzz targets run in CI on every change. Every check, with its numbers and how to repeat
//! it, is in the [parity report](https://github.com/ilubxrp589/xrpl-sdk/blob/main/docs/PARITY.md);
//! speed against xrpl-rust is in the
//! [benchmarks](https://github.com/ilubxrp589/xrpl-sdk/blob/main/docs/BENCHMARKS.md).
//!
//! No `unsafe` code; no `unwrap`, `expect` or unchecked indexing in library
//! code; private keys and seeds are zeroized on drop.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
// docs.rs labels each feature- or platform-gated item with what it needs.
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

pub mod address;
#[cfg(feature = "std")]
pub mod codec;
pub mod crypto;
pub mod error;
#[cfg(feature = "std")]
pub mod hashes;
#[cfg(feature = "std")]
pub mod ledger;
mod number_text;
pub mod transaction;
pub mod types;
pub mod utils;
#[cfg(feature = "wasm")]
pub mod wasm_bindgen_exports;

pub use error::CoreError;
pub use types::*;
