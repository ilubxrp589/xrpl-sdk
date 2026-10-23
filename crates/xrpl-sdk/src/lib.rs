//! # Halcyon XRPL SDK
//!
//! A Rust SDK for the [XRP Ledger](https://xrpl.org): create wallets, build,
//! sign and submit transactions, query `rippled` over HTTP or WebSocket, and
//! work with every XRPL data type. Signatures, hashes, the binary codec and
//! ledger entry IDs are checked byte for byte against `rippled` itself (see
//! [How it is checked](#how-it-is-checked)).
//!
//! This crate (`halcyon-xrpl-sdk` on crates.io, `xrpl_sdk` in code) is the one
//! most applications need. It is built from two smaller crates and re-exports
//! everything in them:
//!
//! | Crate (crates.io) | In code | What it holds |
//! |---|---|---|
//! | `halcyon-xrpl-core` | `xrpl_core` | Types, binary codec, keys and signing, hashes and ledger entry IDs, utilities. No networking. Types and crypto build without `std`. |
//! | `halcyon-xrpl-client` | `xrpl_client` | The [`Client`] trait over HTTP JSON-RPC, WebSocket and browser WebSocket (wasm); typed requests and responses; the Testnet/Devnet faucet. |
//! | `halcyon-xrpl-sdk` | `xrpl_sdk` | [`Wallet`], autofill + sign + submit in one call, and everything above. |
//!
//! ## Install
//!
//! ```toml
//! [dependencies]
//! halcyon-xrpl-sdk = "0.5"
//! tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
//! serde_json = "1"
//! ```
//!
//! ## Quick start: a funded Testnet wallet and a payment
//!
//! [`generate_faucet_wallet`] makes a new wallet on this machine and asks the
//! Testnet faucet to fund it. Only the address is sent; the seed never leaves.
//! [`submit_transaction`] fills in `Sequence`, `Fee` and `LastLedgerSequence`,
//! signs, submits, and waits until a validated ledger includes the
//! transaction.
//!
//! ```no_run
//! use xrpl_sdk::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let wallet = generate_faucet_wallet(FaucetNetwork::Testnet).await?;
//!     let client = XrplHttpClient::new(FaucetNetwork::Testnet.rpc_url())?;
//!
//!     let mut tx = serde_json::json!({
//!         "TransactionType": "Payment",
//!         "Account": wallet.classic_address(),
//!         "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
//!         "Amount": utils::xrp_to_drops("1.5")?.to_string(), // "1500000" drops
//!     });
//!     let result = submit_transaction(&client, &mut tx, &wallet).await?;
//!
//!     let meta = result.meta.as_ref().expect("a validated transaction has metadata");
//!     println!("result: {}", meta["TransactionResult"]);
//!     for account in utils::get_balance_changes(meta)? {
//!         println!("{}: {:?}", account.account, account.balances);
//!     }
//!     println!("keep this seed to reuse the wallet: {}", wallet.seed().unwrap().as_str());
//!     Ok(())
//! }
//! ```
//!
//! The same program is `examples/testnet_quickstart.rs`:
//! `cargo run --example testnet_quickstart`.
//!
//! ## Wallets and signing
//!
//! A [`Wallet`] is a key pair and its address. New wallets use Ed25519, as
//! `rippled` does by default; [`Wallet::generate_with_type`] makes secp256k1
//! ones. [`Wallet::from_seed`] restores a wallet from its seed (`s…`), and
//! [`Wallet::seed`] gives the seed of a wallet made from one. Private keys and
//! seeds are wiped from memory when dropped.
//!
//! Signing needs no network. This example signs a payment and checks the
//! result against `rippled` 3.4.1, which signed the same transaction with the
//! same (public, test-only) seed and reported this hash:
//!
//! ```
//! use xrpl_sdk::*;
//!
//! let wallet = Wallet::from_seed("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?;
//! assert_eq!(wallet.classic_address(), "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD");
//!
//! let tx = serde_json::json!({
//!     "TransactionType": "Payment",
//!     "Account": wallet.classic_address(),
//!     "Destination": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
//!     "Amount": "1000000",
//!     "Fee": "12",
//!     "Sequence": 1,
//!     "Flags": 0,
//! });
//! let blob = wallet.sign_and_encode(&tx)?; // hex, ready for `submit`
//! assert_eq!(
//!     hash_signed_tx_blob(&blob)?.to_string(),
//!     "05DD129B3CBA984E7DEB24FA2522C9DA9AAF4A518F1A01C78F2BD35B46298BF3",
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Multi-signing
//!
//! Each signer signs the same transaction with
//! [`Wallet::sign_for_multisigning`]; [`wallet::collect_signers`] puts the
//! signatures in the order `rippled` requires (by account ID, which is not the
//! order of the addresses as text) and clears `SigningPubKey`:
//!
//! ```
//! use xrpl_sdk::*;
//! use xrpl_sdk::wallet::collect_signers;
//!
//! let (alice, bob) = (Wallet::generate(), Wallet::generate());
//! let mut tx = serde_json::json!({
//!     "TransactionType": "Payment",
//!     "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", // has a SignerList of Alice and Bob
//!     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
//!     "Amount": "1000000",
//!     "Fee": "30", // (1 + number of signers) × the 10-drop base fee
//!     "Sequence": 7,
//!     "SigningPubKey": "",
//! });
//! let signatures = vec![alice.sign_for_multisigning(&tx)?, bob.sign_for_multisigning(&tx)?];
//! collect_signers(&mut tx, signatures)?;
//! let blob = hex::encode_upper(encode_transaction_json(&tx, false)?);
//! # assert_eq!(tx["Signers"].as_array().unwrap().len(), 2);
//! # let _ = blob;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Building transactions
//!
//! Transactions are `serde_json::Value`s in the same JSON `rippled` and
//! xrpl.js use, so every one of the 83 transaction types in `rippled` 3.4.1
//! works, including `Batch`, `DelegateSet`, `MPToken*`, `Credential*`,
//! `Vault*` and `Loan*`. The codec refuses fields it does not know, as
//! `rippled` does, instead of dropping them.
//!
//! Or use the typed models ([`models`]): a struct for each of the 82 types,
//! generated from `rippled`'s definitions, with every field and every flag
//! as a constant. `to_json` gives the JSON to autofill, sign and submit:
//!
//! ```
//! use xrpl_sdk::{models::{Payment, TrustSet}, Amount, TxModel};
//!
//! let mut pay = Payment::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", Amount::Xrp(2_000_000));
//! pay.destination_tag = Some(77);
//! assert_eq!(pay.to_json()?["DestinationTag"], 77);
//!
//! let mut line = TrustSet::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
//! line.limit_amount = Some(serde_json::from_value(serde_json::json!(
//!     {"currency": "USD", "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "value": "1000"}
//! ))?);
//! line.common.flags = Some(TrustSet::TF_SET_NO_RIPPLE);
//! assert_eq!(line.to_json()?["LimitAmount"]["value"], "1000");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! For common transactions there are also fluent builders that check
//! required fields ([`transactions`]):
//!
//! ```
//! use xrpl_sdk::transactions::{Payment, Transaction};
//!
//! let tx = Payment::builder("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
//!     .amount(serde_json::json!("1000000"))
//!     .destination("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")
//!     .build()?
//!     .to_json();
//! assert_eq!(tx["TransactionType"], "Payment");
//! # Ok::<(), xrpl_sdk::CoreError>(())
//! ```
//!
//! ### Batch: several transactions at once
//!
//! A `Batch` (XLS-56) applies up to eight transactions together, from one
//! or several accounts: all or nothing, only one, until one fails, or each
//! independently. [`Client::autofill`] numbers the inner transactions and
//! prices the batch; every other account involved approves it with
//! [`Wallet::sign_batch`]; the outer account signs and submits it as usual.
//!
//! ```no_run
//! use xrpl_sdk::*;
//!
//! # async fn run(client: &XrplHttpClient, alice: &Wallet, bob: &Wallet) -> Result<(), Box<dyn std::error::Error>> {
//! // Alice pays Bob 5 XRP and Bob pays Alice 2 XRP, all or nothing.
//! let pay_bob: Tx = models::Payment::new(alice.classic_address(), bob.classic_address(), Amount::Xrp(5_000_000)).into();
//! let pay_alice: Tx = models::Payment::new(bob.classic_address(), alice.classic_address(), Amount::Xrp(2_000_000)).into();
//! let mut batch_tx = models::Batch::new(alice.classic_address(), vec![pay_bob, pay_alice]);
//! batch_tx.common.flags = Some(models::Batch::TF_ALL_OR_NOTHING);
//!
//! let mut tx = batch_tx.to_json()?;
//! client.autofill(&mut tx).await?;                         // inner sequences and the batch fee
//! let approval = bob.sign_batch(&tx)?;                       // Bob approves
//! batch::add_batch_signers(&mut tx, vec![approval])?;
//! let blob = alice.sign_and_encode(&tx)?;                   // Alice signs the batch
//! client.submit_and_wait(&blob).await?;
//! for id in batch::inner_transaction_ids(&tx)? {
//!     println!("inner transaction {id}");                  // each is on the ledger under this ID
//! }
//! # Ok(()) }
//! ```
//!
//! ## Submitting, and what can go wrong
//!
//! | Function | What it does |
//! |---|---|
//! | [`submit_transaction`] | autofill, sign, submit, wait for the final outcome |
//! | [`autofill_and_sign`] | autofill and sign; returns the blob |
//! | [`Client::autofill`] | fill `Sequence`, `Fee` (capped), `LastLedgerSequence` and `NetworkID` where missing |
//! | [`Client::autofill_for_multisigning`] | the same, with the fee for a number of signers |
//! | [`Client::submit`] | submit a signed blob once; the answer is preliminary |
//! | [`Client::submit_and_wait`] | submit, then wait until it is validated or provably in no ledger |
//!
//! [`Client::submit_and_wait`], and so [`submit_transaction`], answers only
//! with a final outcome:
//!
//! | Outcome | Meaning | Sign a replacement? |
//! |---|---|---|
//! | `Ok(result)` | validated with `tesSUCCESS` | no need |
//! | [`ClientError::TransactionClaimed`] | validated with a `tec`: failed, fee charged (`result` holds the transaction) | yes |
//! | [`ClientError::TransactionFailed`] | malformed (`tem`), or refused (`tef`, `tel`) and in no ledger | yes |
//! | [`ClientError::TransactionExpired`] | its `LastLedgerSequence` is validated and no ledger holds it | yes |
//! | [`ClientError::OutcomeUnknown`] | sent, but the outcome could not be learned | **not yet**: look it up by `hash` first |
//! | [`ClientError::FeeTooHigh`] (from autofill) | the fee would pass the client's cap (2 XRP unless set with `with_max_fee_drops`) | nothing was sent |
//!
//! [`Client::submit`] sorts the server's *preliminary* result into a
//! [`ClientError`] (`tec` → [`ClientError::TransactionClaimed`], `tem`/`tef`/most
//! `tel` → [`ClientError::TransactionFailed`], `ter`/`telINSUF_FEE_P`/`telCAN_NOT_QUEUE…`
//! → [`ClientError::TransactionRetry`]). Only a `tem` is final there: a
//! transaction that got `tec` or `terQUEUED` can still succeed, so never sign
//! a replacement on a preliminary result alone.
//!
//! ## Querying the ledger
//!
//! [`XrplHttpClient`] and [`XrplWsClient`] implement the same [`Client`]
//! trait, so code written against one works with the other. Bring the trait
//! into scope (`use xrpl_sdk::*` does) to call its methods.
//!
//! ```no_run
//! use xrpl_sdk::*;
//!
//! # async fn run() -> Result<(), ClientError> {
//! let client = XrplHttpClient::new("https://xrplcluster.com")?;
//!
//! // A method with typed arguments and response.
//! let info = client.account_info("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", &LedgerIndex::Validated).await?;
//! println!("{} drops, sequence {}", info.account_data.balance, info.account_data.sequence);
//!
//! // Any typed request, by value.
//! let fee = client.request(&requests::FeeRequest {}).await?;
//! println!("open ledger fee: {} drops", fee.drops.open_ledger_fee);
//!
//! // Every page of a paginated request, following `marker`.
//! let lines = client
//!     .request_all(requests::AccountLinesRequest {
//!         limit: Some(400),
//!         ..requests::AccountLinesRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
//!     })
//!     .await?;
//! println!("{} trust lines", lines.len());
//!
//! // Every public rippled method has a typed request (see `requests`).
//! let book = client.request(&requests::BookChangesRequest::new(LedgerIndex::Validated)).await?;
//! for change in &book.changes {
//!     println!("{} / {}: close {}", change.currency_a, change.currency_b, change.close);
//! }
//!
//! // Or any method at all, as JSON.
//! let state = client.raw_request("server_state", serde_json::json!({})).await?;
//! println!("{}", state["state"]["server_state"]);
//! # Ok(()) }
//! ```
//!
//! ### Simulate before you submit
//!
//! [`Client::simulate`] runs a transaction against the open ledger and
//! returns the result and metadata it would have, without signing or
//! sending anything (rippled 2.4+):
//!
//! ```no_run
//! use xrpl_sdk::*;
//!
//! # async fn run() -> Result<(), ClientError> {
//! let client = XrplHttpClient::new("https://xrplcluster.com")?;
//! let outcome = client
//!     .simulate(&serde_json::json!({
//!         "TransactionType": "Payment",
//!         "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
//!         "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
//!         "Amount": "1000000",
//!     }))
//!     .await?;
//! println!("would be {}", outcome.engine_result); // e.g. tesSUCCESS or tecUNFUNDED_PAYMENT
//! # Ok(()) }
//! ```
//!
//! ## Streaming over WebSocket
//!
//! ```no_run
//! use xrpl_sdk::*;
//!
//! # async fn run() -> Result<(), ClientError> {
//! let ws = XrplWsClient::connect("wss://xrplcluster.com").await?;
//! let mut ledgers = ws.subscribe_ledger().await?;
//! while let Ok(event) = ledgers.recv().await {
//!     match event {
//!         WsEvent::Ledger(l) => println!("ledger {} closed with {:?} transactions", l.ledger_index, l.txn_count),
//!         WsEvent::Reconnected => println!("reconnected; subscriptions restored"),
//!         _ => {}
//!     }
//! }
//! # Ok(()) }
//! ```
//!
//! The WebSocket client reconnects on its own and resubscribes. Every
//! `rippled` stream arrives as a typed [`WsEvent`] (checked against messages
//! recorded from mainnet): subscribe with `subscribe_ledger`,
//! `subscribe_transactions` (each transaction a typed [`Tx`]),
//! `subscribe_transactions_proposed`, `subscribe_account`,
//! `subscribe_validations`, `subscribe_server`, `subscribe_consensus`,
//! `subscribe_book_changes`, `subscribe_manifests`, `subscribe_order_book`
//! (with the book's current offers), or `subscribe` with any parameters.
//!
//! ## Without async: the blocking client
//!
//! [`BlockingClient`] gives every method as a plain call, for scripts and
//! tools without an async runtime:
//!
//! ```no_run
//! use xrpl_sdk::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = BlockingClient::new("https://xrplcluster.com")?;
//!     let info = client.account_info("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", &LedgerIndex::Validated)?;
//!     println!("{} XRP", utils::drops_to_xrp(info.account_data.balance.parse()?));
//!     Ok(())
//! }
//! ```
//!
//! ## Cookbook
//!
//! Runnable recipes, each with its own funded Testnet wallets
//! (`cargo run --example <name>` in the repository), each run on Testnet
//! before release:
//!
//! | Recipe | Shows |
//! |---|---|
//! | `testnet_quickstart` | a funded wallet, a payment, its hash and balance changes |
//! | `issue_token` | issuing a token: DefaultRipple, trust lines, issuing, paying holder to holder |
//! | `nft_marketplace` | minting with a transfer fee, selling, reselling, the royalty |
//! | `dex_order` | an order, the order book, taking it, cancelling |
//! | `amm` | an AMM pool, a trade against it |
//! | `escrow` | XRP locked until a time, then released |
//! | `checks` | writing a check, cashing part of it |
//! | `payment_channel` | a channel paid with off-ledger signed claims |
//! | `multisign` | a signer list and a multi-signed transaction |
//! | `batch` | two accounts' payments in one all-or-nothing Batch |
//! | `mpt` | a Multi-Purpose Token: create, opt in, send |
//! | `blocking` | all of it without async |
//! | `watch_streams` | live mainnet ledgers, trading and validations |
//!
//! ## Command line
//!
//! `cargo install halcyon-xrpl-cli` installs `hxrpl`: wallets and the
//! faucet, account queries, signing and sending, transaction hashes and
//! ledger entry IDs, order books and live streams, from a terminal. It uses
//! Testnet unless told otherwise, needs `--yes` to send on mainnet, and reads
//! seeds only from `XRPL_SEED` or a file.
//!
//! ## Hashes and ledger entry IDs
//!
//! [`hash_signed_tx`] and [`hash_signed_tx_blob`] give a transaction's ID
//! before it is submitted. [`keylet`] gives the ID of any ledger entry, for
//! `ledger_entry` lookups or to recognise entries in metadata:
//!
//! ```
//! use xrpl_sdk::keylet;
//!
//! let account = keylet::account("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
//! assert_eq!(account.to_string(), "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8");
//!
//! // Order does not matter: a trust line has one ID for both accounts.
//! let a = keylet::trust_line("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "rB5TihdPbKgMrkFqrqUC3yLdE8hhv4BdeY", "USD")?;
//! let b = keylet::trust_line("rB5TihdPbKgMrkFqrqUC3yLdE8hhv4BdeY", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "USD")?;
//! assert_eq!(a, b);
//! # Ok::<(), xrpl_sdk::CoreError>(())
//! ```
//!
//! ## Utilities
//!
//! [`utils`] holds the conversions an application needs every day. Amounts
//! are computed in exact decimal, never in floating point.
//!
//! | Function | Does |
//! |---|---|
//! | [`utils::xrp_to_drops`], [`utils::drops_to_xrp`] | XRP ↔ drops, refusing more than 6 decimals |
//! | [`utils::ripple_time_to_iso8601`], [`utils::iso8601_to_ripple_time`], [`utils::ripple_time_to_unix`], [`utils::unix_to_ripple_time`] | ledger time (seconds since 2000-01-01) ↔ ISO 8601 / Unix |
//! | [`utils::str_to_hex`], [`utils::hex_to_str`] | text ↔ hex for `Domain`, memos, URIs |
//! | [`utils::percent_to_transfer_rate`], [`utils::decimal_to_quality`] and their inverses | `TransferRate`, `QualityIn` / `QualityOut` |
//! | [`utils::parse_nftoken_id`] | issuer, taxon, transfer fee, flags and sequence of an NFTokenID |
//! | [`utils::get_nftoken_id`] | the NFTokenID a mint created, from its metadata |
//! | [`utils::get_balance_changes`] | every account's balance changes, from metadata |
//!
//! ```
//! use xrpl_sdk::utils;
//!
//! assert_eq!(utils::xrp_to_drops("1.5")?, 1_500_000);
//! assert_eq!(utils::drops_to_xrp(12), "0.000012");
//! assert_eq!(utils::ripple_time_to_iso8601(0), "2000-01-01T00:00:00Z");
//! assert_eq!(utils::percent_to_transfer_rate("0.5%")?, 1_005_000_000);
//! assert_eq!(utils::hex_to_str(&utils::str_to_hex("example.com"))?, "example.com");
//!
//! let nft = utils::parse_nftoken_id("000B0539C35B55AA096BA6D87A6E6C965A6534150DC56E5E12C5D09E0000000C")?;
//! assert_eq!((nft.issuer.as_str(), nft.taxon, nft.transfer_fee), ("rJoxBSzpXhPtAuqFmqxQtGKjA13jUJWthE", 1337, 1337));
//! # Ok::<(), xrpl_sdk::CoreError>(())
//! ```
//!
//! ## Payment channels
//!
//! [`Wallet::sign_payment_channel_claim`] signs a claim off-ledger;
//! [`verify_payment_channel_claim`] checks one, as `rippled`'s
//! `channel_verify` does. The amount is the channel's running total, in drops.
//!
//! ```
//! use xrpl_sdk::*;
//!
//! let channel = "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3";
//! let source = Wallet::from_seed("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?;
//! let claim = source.sign_payment_channel_claim(channel, 1_000_000)?;
//! assert!(verify_payment_channel_claim(channel, 1_000_000, &claim, &source.public_key_hex())?);
//! assert!(!verify_payment_channel_claim(channel, 2_000_000, &claim, &source.public_key_hex())?);
//! # Ok::<(), xrpl_sdk::CoreError>(())
//! ```
//!
//! ## Feature flags
//!
//! | Crate | Feature | Default | Enables |
//! |---|---|---|---|
//! | `halcyon-xrpl-sdk`, `halcyon-xrpl-client` | `http`, `ws`, `full` | `http` | accepted for compatibility; on native targets both the HTTP and the WebSocket client are always built |
//! | | `live-tests` | no | the tests that talk to Testnet |
//! | `halcyon-xrpl-client` | `wasm` | no | the browser build: HTTP through `fetch`, WebSocket through the browser's `WebSocket` |
//! | `halcyon-xrpl-core` | `std` | yes | the codec, hashes, utilities and JSON support; without it, types and crypto only (`no_std` + `alloc`) |
//! | | `wasm` | no | `wasm32-unknown-unknown` builds and the JavaScript bindings |
//!
//! ## How it is checked
//!
//! Everything that has to agree with the XRP Ledger is tested against the
//! real thing, not against itself:
//!
//! - **Signatures, multi-signatures, transaction hashes and channel claims**:
//!   byte for byte against `rippled` 3.4.1's `sign`, `sign_for` and
//!   `channel_authorize`, for Ed25519 and secp256k1 keys.
//! - **Binary codec**: all 34 transaction and 261 ledger-object fixtures of the
//!   XRPL codec test suite, plus mainnet transactions and ledger entries.
//! - **Ledger entry IDs**: the `index` of real mainnet entries, xrpl.js's hash
//!   tests, and a live run that creates each kind of entry on Testnet.
//! - **Stream events**: messages recorded from `rippled` 3.4.1 on mainnet,
//!   one or more of each stream type, parse into their typed events and keep
//!   every field; streamed transactions parse into typed [`Tx`]s.
//! - **Cookbook**: every recipe runs end to end on Testnet before a release.
//! - **Transaction models**: 186 real transactions (51 types) round-trip
//!   through their typed models to the same JSON, bytes and ID; an example of
//!   every other type, signed by `rippled` 3.4.1, encodes to its exact
//!   `tx_blob`: all 82 types. Batch signatures verify against real batches.
//! - **Utilities**: xrpl.js's own fixtures and expected results.
//! - **Typed responses**: 44 responses recorded from `rippled` 3.4.1, Clio and
//!   Testnet; each must parse, and serializing it back must keep every one of
//!   their 2,521 field paths.
//! - **End to end** (`cargo test -p halcyon-xrpl-sdk --features live-tests`):
//!   two faucet wallets create a Ticket, Check, Escrow, payment channel (and
//!   redeem a claim), NFToken and offer, DID, Oracle, MPT issuance and holding,
//!   Credential, PermissionedDomain, DepositPreauth, Delegate and trust line on
//!   Testnet, push a multi-signed transaction through `submit_multisigned`,
//!   and submit two `Batch`es (one built from the typed models, approved by a
//!   second account) whose inner transactions must land under the IDs
//!   computed for them;
//!   every ID, hash and balance change must be what the network recorded, and
//!   the typed queries for what they made (channels, NFT offers, oracle prices,
//!   MPT holders through Clio) must parse the answers.
//! - **Edge cases of the format** (`Number`, amounts, payment paths): each was
//!   put to `rippled` 3.4.1 itself, and the codec reads, writes and refuses
//!   exactly what `rippled` does.
//! - **Robustness**: eight fuzz targets, seeded with all of the above, run in
//!   CI on every change: nothing panics, and decode → encode → decode is stable.
//!
//! Every check, with its numbers and how to repeat it, is in the
//! [parity report](https://github.com/ilubxrp589/xrpl-sdk/blob/main/docs/PARITY.md).
//! Speed against xrpl-rust, on the same inputs, is in the
//! [benchmarks](https://github.com/ilubxrp589/xrpl-sdk/blob/main/docs/BENCHMARKS.md).
//!
//! ## Minimum supported Rust version
//!
//! Rust 1.88 (1.85 for `halcyon-xrpl-core` alone), checked in CI.
//!
//! ## Safety
//!
//! No `unsafe` code, and no `unwrap`, `expect` or unchecked indexing in
//! library code (enforced by workspace lints). Private keys and seeds are
//! zeroized on drop.
//!
//! ## License
//!
//! MIT, with an attribution requirement: if you use, copy or modify this code,
//! credit James Turner with a link to <https://github.com/ilubxrp589/xrpl-sdk>.

#![warn(missing_docs)]

/// Wallets: keys and address, signing, multi-signing, batches and channel claims.
pub mod wallet;

/// The client crate (`halcyon-xrpl-client`), in full.
pub use xrpl_client;
/// The core crate (`halcyon-xrpl-core`), in full.
pub use xrpl_core;

// Re-exports from xrpl-core
pub use xrpl_core::address::KeyType;
pub use xrpl_core::codec::{decode_transaction_binary, encode_transaction_json};
pub use xrpl_core::crypto::{
    sign_payment_channel_claim, sign_transaction, verify_payment_channel_claim, Keypair, Seed,
};
pub use xrpl_core::hashes::{self, hash_signed_tx, hash_signed_tx_blob, keylet};
pub use xrpl_core::transaction::batch;
pub use xrpl_core::transaction::builders::{self as transactions, Transaction, TxCommon};
pub use xrpl_core::transaction::models::{self, Tx, TxModel};
pub use xrpl_core::types::*;
pub use xrpl_core::utils;
pub use xrpl_core::CoreError;

// Re-exports from xrpl-client
#[cfg(not(target_arch = "wasm32"))]
pub use xrpl_client::blocking::BlockingClient;
pub use xrpl_client::error::ClientError;
pub use xrpl_client::faucet::{self, FaucetNetwork};
pub use xrpl_client::http::XrplHttpClient;
pub use xrpl_client::requests::{self, Paginated, Request};
pub use xrpl_client::types::*;
pub use xrpl_client::Client;
pub use xrpl_client::XrplWsClient;

#[cfg(not(target_arch = "wasm32"))]
pub use xrpl_client::ws::{LedgerEvent, TransactionEvent, WsEvent};

pub use wallet::Wallet;

/// Generate a new wallet (Ed25519) and fund it with test XRP from `network`'s
/// faucet; returns once a validated ledger shows the account. Only the
/// address leaves this machine; keep the seed with [`Wallet::seed`].
pub async fn generate_faucet_wallet(network: FaucetNetwork) -> Result<Wallet, ClientError> {
    let wallet = Wallet::generate();
    faucet::fund_and_wait(network, wallet.classic_address()).await?;
    Ok(wallet)
}

/// Autofill a transaction, sign it, and return the hex-encoded blob.
pub async fn autofill_and_sign(
    client: &impl Client,
    tx: &mut serde_json::Value,
    wallet: &Wallet,
) -> Result<String, ClientError> {
    client.autofill(tx).await?;
    let blob = wallet.sign_and_encode(tx).map_err(ClientError::Core)?;
    Ok(blob)
}

/// Autofill, sign, submit, and wait for the final outcome: `Ok` only for a
/// transaction validated with `tesSUCCESS`. Every other ending is an error;
/// see [`Client::submit_and_wait`] for which ones are final and which
/// ([`ClientError::OutcomeUnknown`]) mean "look it up before replacing it".
pub async fn submit_transaction(
    client: &impl Client,
    tx: &mut serde_json::Value,
    wallet: &Wallet,
) -> Result<TxResult, ClientError> {
    client.autofill(tx).await?;
    let blob = wallet.sign_and_encode(tx).map_err(ClientError::Core)?;
    client.submit_and_wait(&blob).await
}
