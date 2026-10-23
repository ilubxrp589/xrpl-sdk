//! # halcyon-xrpl-client
//!
//! Talk to `rippled` (and Clio) over HTTP JSON-RPC or WebSocket, natively or
//! in the browser, through one interface: the [`Client`] trait. Requests and
//! responses are typed; anything else is one [`Client::raw_request`] away.
//! Part of `halcyon-xrpl-sdk`, which re-exports all of it.
//!
//! ```toml
//! [dependencies]
//! halcyon-xrpl-client = "0.8"   # imported as `xrpl_client`
//! tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
//! ```
//!
//! ## Transports
//!
//! | Type | Transport | Targets | Feature |
//! |---|---|---|---|
//! | [`XrplHttpClient`] | JSON-RPC over HTTPS (`reqwest`, rustls) | native and `wasm32` | `http` (default) |
//! | [`XrplWsClient`] | WebSocket with subscriptions and automatic reconnection (`tokio-tungstenite`) | native | `ws` (default) |
//! | `XrplWsClient` (wasm) | JavaScript's `WebSocket` | `wasm32` | `ws` |
//!
//! A `wasm32` build also needs the `wasm` feature, which gives
//! `halcyon-xrpl-core` its random source and JavaScript bindings.
//!
//! All of them implement [`Transport`], and so [`Client`]: a transport
//! supplies [`Transport::send_raw`], and every method below is written once
//! on top of it, so they behave the same on every transport. With neither
//! feature (`default-features = false`) the crate is the [`Client`] trait,
//! the typed requests and responses, and nothing that touches the network:
//! bring your own [`Transport`].
//!
//! ## The `Client` trait
//!
//! | Kind | Methods |
//! |---|---|
//! | Accounts | `account_info`, `account_lines`, `account_offers`, `account_nfts`, `account_tx`, `account_objects`, `account_currencies`, `account_channels`, `gateway_balances` (and `_all` variants that follow `marker` through every page) |
//! | Ledger and server | `ledger`, `ledger_current`, `ledger_entry`, `fee`, `server_info` |
//! | DEX and AMM | `book_offers`, `amm_info` |
//! | Transactions | `tx`, `submit`, `submit_multisigned`, `submit_and_wait`, `simulate`, `autofill` |
//! | Generic | [`Client::request`] (any typed [`Request`]), [`Client::request_all`] (any [`Paginated`] request), [`Client::raw_request`] (any method, JSON in and out) |
//!
//! [`requests`] has a typed request and response for **every public
//! `rippled` method and the Clio-only ones**: 43 in all, from `account_info`
//! to `ledger_data`, `book_changes`, `get_aggregate_price`, `nft_sell_offers`,
//! `ripple_path_find`, `server_definitions`, `simulate`, `vault_info` and
//! `nfts_by_issuer`. Each has a `new` with its required parameters; optional
//! ones are fields:
//!
//! ```no_run
//! use xrpl_client::requests::{LedgerDataRequest, NftsByIssuerRequest};
//! use xrpl_client::{types::LedgerIndex, Client, XrplHttpClient};
//!
//! # async fn run() -> Result<(), xrpl_client::ClientError> {
//! let clio = XrplHttpClient::new("https://s2.ripple.com:51234")?; // Clio answers nfts_by_issuer
//! let nfts = clio.request_all(NftsByIssuerRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")).await?;
//!
//! let page = clio
//!     .request(&LedgerDataRequest::new(LedgerIndex::Validated).with_entry_type("amm").with_limit(10u32))
//!     .await?;
//! println!("{} NFTs; {} AMMs on the first page", nfts.len(), page.state.len());
//! # Ok(()) }
//! ```
//!
//! ```no_run
//! use xrpl_client::{Client, XrplHttpClient};
//! use xrpl_client::requests::BookOffersRequest;
//! use xrpl_client::types::LedgerIndex;
//!
//! # async fn run() -> Result<(), xrpl_client::ClientError> {
//! let client = XrplHttpClient::new("https://xrplcluster.com")?;
//!
//! let account = client.account_info("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", LedgerIndex::Validated).await?;
//! println!("balance: {}", account.account_data.balance); // "12.5 XRP"
//!
//! // The XRP/RLUSD order book, with amounts typed as `xrpl_core::types::Amount`.
//! let book = client
//!     .request(
//!         &BookOffersRequest::new(
//!             xrpl_core::types::Asset::xrp(),
//!             xrpl_core::types::Asset::token("RLUSD", "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De")?,
//!         )
//!         .with_limit(10u32),
//!     )
//!     .await?;
//! for offer in &book.offers {
//!     println!("{:?} for {:?}", offer.taker_gets, offer.taker_pays);
//! }
//! # Ok(()) }
//! ```
//!
//! ## Submitting
//!
//! [`Client::autofill`] fills `Sequence`, `Fee` (the open-ledger fee, refused
//! with [`ClientError::FeeTooHigh`] above the client's cap: 2 XRP unless set
//! with `with_max_fee_drops`), `LastLedgerSequence` (current +
//! [`client::LAST_LEDGER_OFFSET`]) and, on a network whose ID is above 1024,
//! `NetworkID`, where they are missing, fetching them concurrently.
//! [`Client::autofill_for_multisigning`] does the same with the fee for a
//! number of signers.
//!
//! [`Client::submit_and_wait`] submits a signed blob and answers only with
//! the final outcome: `Ok` for `tesSUCCESS` in a validated ledger;
//! [`ClientError::TransactionClaimed`] (holding the validated transaction)
//! for a validated `tec`; [`ClientError::TransactionFailed`] for a `tem`, or
//! a `tef`/`tel` no ledger holds; [`ClientError::TransactionExpired`] once
//! the transaction's `LastLedgerSequence` is validated and the server shows
//! it is in no ledger; [`ClientError::OutcomeUnknown`] when it was sent but
//! the outcome could not be learned. It looks the transaction up by the ID
//! it computes itself, and resubmits the same blob when the server dropped
//! it.
//!
//! [`Client::submit`] submits once and sorts the server's *preliminary*
//! result: `tes` is `Ok`; `tec` is [`ClientError::TransactionClaimed`];
//! `tem`, `tef` and most `tel` are [`ClientError::TransactionFailed`];
//! `ter`, `telINSUF_FEE_P` and `telCAN_NOT_QUEUE…` are
//! [`ClientError::TransactionRetry`]. Only a `tem` is final there.
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
//! while let Some(event) = events.recv().await {
//!     if let WsEvent::Transaction(tx) = event {
//!         println!("{tx:?}");
//!     }
//! }
//! # Ok(()) }
//! ```
//!
//! After a dropped connection the client reconnects with backoff, restores
//! every subscription, and sends [`stream::WsEvent::Reconnected`].
//!
//! ## Test networks
//!
//! [`faucet`] funds an address with test XRP on Testnet or Devnet, and knows
//! each network's public servers ([`faucet::FaucetNetwork`]). Only the address
//! is sent to the faucet.
//!
//! ## Typed responses
//!
//! Requests ask for `rippled`'s API version 2 ([`API_VERSION`]), and every
//! response type in [`types`] is checked against a version 2 response
//! recorded from a real server: 45 recordings of 42 methods (39 from
//! `rippled` 3.4.1 and Clio on mainnet, 6 from Testnet), 2,605 field paths. Each must parse, and
//! serializing it back must keep every field the server sent, so a field the
//! types do not model fails the test instead of silently vanishing
//! (`tests/rippled_responses.rs`). `account_objects`, `ledger_data` and
//! `ledger_entry` return typed [`xrpl_core::types::LedgerObject`]s.

#![warn(missing_docs)]
// docs.rs labels each feature- or platform-gated item with what it needs.
#![cfg_attr(docsrs, feature(doc_cfg))]

/// The host (and port) of a URL, without scheme, credentials, path or query:
/// what logs, errors and `Debug` show of a server (its URL may carry a token).
#[cfg(any(feature = "http", feature = "ws"))]
pub(crate) fn host_of(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    authority.rsplit('@').next().unwrap_or("").to_string()
}

#[cfg(all(feature = "http", not(target_arch = "wasm32")))]
pub mod blocking;
pub mod client;
pub mod error;
#[cfg(feature = "http")]
pub mod faucet;
#[cfg(feature = "http")]
pub mod http;
pub mod requests;
pub mod stream;
pub mod types;

#[cfg(all(feature = "ws", not(target_arch = "wasm32")))]
pub mod ws;

#[cfg(all(feature = "ws", target_arch = "wasm32"))]
pub mod ws_wasm;

pub use client::{Client, MaybeSend, MaybeSync, Transport, TransportFuture, API_VERSION};
pub use error::{ClientError, TransportErrorKind};
#[cfg(feature = "http")]
pub use http::XrplHttpClient;
pub use requests::{Paginated, Request};

#[cfg(all(feature = "ws", not(target_arch = "wasm32")))]
pub use ws::{Subscription, WsClientBuilder, XrplWsClient};

#[cfg(all(feature = "ws", target_arch = "wasm32"))]
pub use ws_wasm::{Subscription, XrplWsClient};
