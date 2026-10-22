//! # halcyon-xrpl-client
//!
//! Talk to `rippled` (and Clio) over HTTP JSON-RPC or WebSocket, natively or
//! in the browser, through one interface: the [`Client`] trait. Requests and
//! responses are typed; anything else is one [`Client::raw_request`] away.
//! Part of `halcyon-xrpl-sdk`, which re-exports all of it.
//!
//! ```toml
//! [dependencies]
//! halcyon-xrpl-client = "0.1"   # imported as `xrpl_client`
//! tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
//! ```
//!
//! ## Transports
//!
//! | Type | Transport | Targets |
//! |---|---|---|
//! | [`XrplHttpClient`] | JSON-RPC over HTTPS (`reqwest`, rustls) | native and `wasm32` |
//! | [`XrplWsClient`] | WebSocket with subscriptions and automatic reconnection (`tokio-tungstenite`) | native |
//! | `XrplWsClient` (wasm) | the browser's `WebSocket` | `wasm32` with the `wasm` feature |
//!
//! All of them implement [`Client`]: a transport supplies
//! [`Client::raw_request`], and every method below is written once on top of
//! it, so they behave the same on every transport.
//!
//! ## The `Client` trait
//!
//! | Kind | Methods |
//! |---|---|
//! | Accounts | `account_info`, `account_lines`, `account_offers`, `account_nfts`, `account_tx`, `account_objects`, `account_currencies`, `account_channels`, `gateway_balances` (and `_all` variants that follow `marker` through every page) |
//! | Ledger and server | `ledger`, `ledger_current`, `ledger_entry`, `fee`, `server_info` |
//! | DEX and AMM | `book_offers`, `amm_info` |
//! | Transactions | `tx`, `submit`, `submit_and_wait`, `autofill` |
//! | Generic | [`Client::request`] (any typed [`Request`]), [`Client::request_all`] (any [`Paginated`] request), [`Client::raw_request`] (any method, JSON in and out) |
//!
//! ```no_run
//! use xrpl_client::{Client, XrplHttpClient};
//! use xrpl_client::requests::BookOffersRequest;
//! use xrpl_client::types::LedgerIndex;
//!
//! # async fn run() -> Result<(), xrpl_client::ClientError> {
//! let client = XrplHttpClient::new("https://xrplcluster.com")?;
//!
//! let account = client.account_info("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", &LedgerIndex::Validated).await?;
//! println!("balance: {} drops", account.account_data.balance);
//!
//! // The XRP/RLUSD order book, with amounts typed as `xrpl_core::types::Amount`.
//! let book = client
//!     .request(&BookOffersRequest {
//!         taker_gets: serde_json::json!({"currency": "XRP"}),
//!         taker_pays: serde_json::json!({
//!             "currency": "524C555344000000000000000000000000000000",
//!             "issuer": "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De"
//!         }),
//!         limit: Some(10),
//!     })
//!     .await?;
//! for offer in &book.offers {
//!     println!("{:?} for {:?}", offer.taker_gets, offer.taker_pays);
//! }
//! # Ok(()) }
//! ```
//!
//! ## Submitting
//!
//! [`Client::autofill`] fills `Sequence`, `Fee` (the open-ledger fee) and
//! `LastLedgerSequence` (current + 4) where they are missing, fetching them
//! concurrently. [`Client::submit`] sorts the engine result: `tes` is `Ok`;
//! `tec` is [`ClientError::TransactionClaimed`]; `tem`, `tef` and most `tel`
//! are [`ClientError::TransactionFailed`]; `ter`, `telINSUF_FEE_P` and
//! `telCAN_NOT_QUEUE…` are [`ClientError::TransactionRetry`].
//! [`Client::submit_and_wait`] resubmits on a retry result and polls until
//! the transaction is in a validated ledger or its `LastLedgerSequence` has
//! passed ([`ClientError::TransactionExpired`]).
//!
//! ## WebSocket subscriptions
//!
//! ```no_run
//! use xrpl_client::XrplWsClient;
//! use xrpl_client::ws::WsEvent;
//!
//! # async fn run() -> Result<(), xrpl_client::ClientError> {
//! let ws = XrplWsClient::connect("wss://xrplcluster.com").await?;
//! let mut events = ws.subscribe_account("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").await?;
//! while let Ok(event) = events.recv().await {
//!     if let WsEvent::Transaction(tx) = event {
//!         println!("{tx:?}");
//!     }
//! }
//! # Ok(()) }
//! ```
//!
//! After a dropped connection the client reconnects with backoff, restores
//! every subscription, and sends [`ws::WsEvent::Reconnected`].
//!
//! ## Test networks
//!
//! [`faucet`] funds an address with test XRP on Testnet or Devnet, and knows
//! each network's public servers ([`faucet::FaucetNetwork`]). Only the address
//! is sent to the faucet.
//!
//! ## Typed responses
//!
//! Response types in [`types`] are checked against responses recorded from
//! `rippled` 3.4.1 (for example `fee`, `book_offers`, `amm_info`,
//! `account_offers`); `account_objects` and `ledger_entry` return typed
//! [`xrpl_core::types::LedgerObject`]s.

pub mod client;
pub mod error;
pub mod faucet;
pub mod http;
pub mod requests;
pub mod types;

#[cfg(not(target_arch = "wasm32"))]
pub mod ws;

#[cfg(target_arch = "wasm32")]
pub mod ws_wasm;

pub use client::Client;
pub use error::ClientError;
pub use http::XrplHttpClient;
pub use requests::{Paginated, Request};

#[cfg(not(target_arch = "wasm32"))]
pub use ws::XrplWsClient;

#[cfg(target_arch = "wasm32")]
pub use ws_wasm::XrplWsClient;
