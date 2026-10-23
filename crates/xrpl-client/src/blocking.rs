//! A blocking client: every [`Client`] method as a plain function call, for
//! scripts, command-line tools and programs without an async runtime.
//!
//! [`BlockingClient`] owns a small single-threaded Tokio runtime and runs each
//! call to completion on it. Called from inside an async runtime (where
//! blocking would stall it) its methods return
//! [`ClientError::InsideAsyncRuntime`]; use the async client there.
//!
//! ```no_run
//! use xrpl_client::blocking::BlockingClient;
//! use xrpl_client::requests::BookChangesRequest;
//! use xrpl_client::Client; // for `run` with the async methods
//! use xrpl_client::types::LedgerIndex;
//!
//! fn main() -> Result<(), xrpl_client::ClientError> {
//!     let client = BlockingClient::new("https://xrplcluster.com")?;
//!
//!     let fee = client.fee()?;
//!     println!("open ledger fee: {} drops", fee.drops.open_ledger_fee);
//!
//!     let account = client.account_info("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", LedgerIndex::Validated)?;
//!     println!("balance: {}", account.account_data.balance); // "12.5 XRP"
//!
//!     // Any typed request, or any async method with `run`.
//!     let changes = client.request(&BookChangesRequest::new(LedgerIndex::Validated))?;
//!     let lines = client.run(|c| c.account_lines_all("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", LedgerIndex::Validated))?;
//!     println!("{} books traded, {} trust lines", changes.changes.len(), lines.len());
//!     Ok(())
//! }
//! ```

use crate::client::Client;
use crate::error::ClientError;
use crate::http::XrplHttpClient;
use crate::requests::{Paginated, Request};
use crate::types::*;
use serde_json::Value;
use std::future::Future;

/// A [`Client`] that blocks: each method waits for the server's answer.
pub struct BlockingClient<C: Client = XrplHttpClient> {
    inner: C,
    runtime: tokio::runtime::Runtime,
}

impl<C: Client + std::fmt::Debug> std::fmt::Debug for BlockingClient<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockingClient")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

impl BlockingClient<XrplHttpClient> {
    /// A blocking JSON-RPC client for the server at `url`.
    ///
    /// # Errors
    ///
    /// [`ClientError::Transport`] (`Setup`) if the HTTP stack or the runtime
    /// cannot start. The URL
    /// is not checked here: a bad one fails on the first request.
    pub fn new(url: &str) -> Result<Self, ClientError> {
        Self::with_client(XrplHttpClient::new(url)?)
    }
}

impl<C: Client> BlockingClient<C> {
    /// Block on any [`Client`] (an [`XrplHttpClient`], or your own transport).
    ///
    /// # Errors
    ///
    /// [`ClientError::UnexpectedResponse`] if the runtime cannot start.
    pub fn with_client(inner: C) -> Result<Self, ClientError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                ClientError::transport(
                    crate::error::TransportErrorKind::Setup,
                    format!("the blocking client's runtime: {e}"),
                )
            })?;
        Ok(Self { inner, runtime })
    }

    /// The async client underneath.
    pub fn inner(&self) -> &C {
        &self.inner
    }

    /// Run any async method of the client to completion:
    /// `client.run(|c| c.account_lines_all(addr, ledger))`.
    ///
    /// # Panics
    ///
    /// Inside an async runtime, as Tokio refuses to block one. The other
    /// methods return [`ClientError::InsideAsyncRuntime`] instead.
    pub fn run<'a, F: Future>(&'a self, call: impl FnOnce(&'a C) -> F) -> F::Output {
        self.runtime.block_on(call(&self.inner))
    }

    /// Run any future to completion on this client's runtime (for example a
    /// `faucet` call).
    ///
    /// # Panics
    ///
    /// Inside an async runtime (see [`BlockingClient::run`]).
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }

    /// [`BlockingClient::run`] for a call that returns a `Result`, refusing
    /// (instead of panicking) inside an async runtime.
    fn call<'a, T, F: Future<Output = Result<T, ClientError>>>(
        &'a self,
        call: impl FnOnce(&'a C) -> F,
    ) -> Result<T, ClientError> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(ClientError::InsideAsyncRuntime);
        }
        self.runtime.block_on(call(&self.inner))
    }

    /// [`Client::request`]: any typed request.
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::request`]: [`ClientError::RpcError`]
    /// when the server answers with an error, [`ClientError::Json`] when the
    /// answer does not decode as `R::Response`, and a [`ClientError`] when the
    /// request fails.
    pub fn request<R: Request>(&self, req: &R) -> Result<R::Response, ClientError> {
        self.call(|c| c.request(req))
    }

    /// [`Client::request_all`]: every page of a paginated request.
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::request_all`]:
    /// [`ClientError::PaginationLimitReached`] past
    /// [`MAX_PAGES`](crate::client::MAX_PAGES) pages, and the error of a page's
    /// request.
    pub fn request_all<R: Paginated>(&self, req: R) -> Result<Vec<R::Item>, ClientError> {
        self.call(|c| c.request_all(req))
    }

    /// [`Client::pages`]: the pages of a paginated request one at a time, as
    /// an iterator, with no page limit. Inside an async runtime it yields
    /// one [`ClientError::InsideAsyncRuntime`] and ends.
    pub fn pages<'a, R: Paginated + 'a>(
        &'a self,
        req: R,
    ) -> impl Iterator<Item = Result<Vec<R::Item>, ClientError>> + 'a {
        let mut stream = Box::pin(self.inner.pages(req));
        let mut done = false;
        std::iter::from_fn(move || {
            if done {
                return None;
            }
            if tokio::runtime::Handle::try_current().is_ok() {
                done = true;
                return Some(Err(ClientError::InsideAsyncRuntime));
            }
            self.runtime.block_on(futures::StreamExt::next(&mut stream))
        })
    }

    /// [`Client::raw_request`]: any method, JSON in and out.
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::raw_request`]:
    /// [`ClientError::RpcError`] when the server answers with an error, and a
    /// [`ClientError`] when the request fails.
    pub fn raw_request(&self, method: &str, params: Value) -> Result<Value, ClientError> {
        self.call(|c| c.raw_request(method, params))
    }

    /// [`Client::account_info`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::account_info`]:
    /// [`ClientError::RpcError`] (`actNotFound`) for an account that does not
    /// exist, and a [`ClientError`] when the request fails.
    pub fn account_info(
        &self,
        account: &str,
        ledger: LedgerIndex,
    ) -> Result<AccountInfo, ClientError> {
        self.call(|c| c.account_info(account, ledger))
    }

    /// [`Client::account_lines`] (one page).
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::account_lines`]:
    /// [`ClientError::RpcError`] (`actNotFound`) for an account that does not
    /// exist, and a [`ClientError`] when the request fails.
    pub fn account_lines(
        &self,
        account: &str,
        ledger: LedgerIndex,
    ) -> Result<AccountLinesResult, ClientError> {
        self.call(|c| c.account_lines(account, ledger))
    }

    /// [`Client::fee`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::fee`]: a [`ClientError`] when the
    /// request fails.
    pub fn fee(&self) -> Result<FeeResult, ClientError> {
        self.call(|c| c.fee())
    }

    /// [`Client::ledger_current`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::ledger_current`]: a [`ClientError`]
    /// when the request fails.
    pub fn ledger_current(&self) -> Result<u32, ClientError> {
        self.call(|c| c.ledger_current())
    }

    /// [`Client::ledger`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::ledger`]: [`ClientError::RpcError`]
    /// (`lgrNotFound`) for a ledger the server does not have, and a
    /// [`ClientError`] when the request fails.
    pub fn ledger(&self, ledger: LedgerIndex) -> Result<LedgerResult, ClientError> {
        self.call(|c| c.ledger(ledger))
    }

    /// [`Client::server_info`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::server_info`]: a [`ClientError`] when
    /// the request fails.
    pub fn server_info(&self) -> Result<ServerInfoResult, ClientError> {
        self.call(|c| c.server_info())
    }

    /// [`Client::tx`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::tx`]: [`ClientError::RpcError`]
    /// (`txnNotFound`) for a transaction the server cannot find, and a
    /// [`ClientError`] when the request fails.
    pub fn tx(&self, hash: &str) -> Result<TxResult, ClientError> {
        self.call(|c| c.tx(hash))
    }

    /// [`Client::autofill`]: `Sequence`, `Fee`, `LastLedgerSequence` (and a
    /// `Batch`'s inner transactions).
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::autofill`]:
    /// [`ClientError::FeeTooHigh`] above the fee cap,
    /// [`ClientError::UnexpectedResponse`] for a transaction with no `Account`,
    /// [`ClientError::Core`] for one that cannot be priced or names an
    /// X-address for the other network, and a [`ClientError`] when a request
    /// fails.
    pub fn autofill(&self, tx: &mut Value) -> Result<(), ClientError> {
        self.call(|c| c.autofill(tx))
    }

    /// [`Client::autofill_for_multisigning`].
    ///
    /// # Errors
    ///
    /// As [`BlockingClient::autofill`].
    pub fn autofill_for_multisigning(
        &self,
        tx: &mut Value,
        signers: u32,
    ) -> Result<(), ClientError> {
        self.call(|c| c.autofill_for_multisigning(tx, signers))
    }

    /// [`Client::autofill_batch`]: the inner-transaction part of
    /// [`BlockingClient::autofill`] for a `Batch`.
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::autofill_batch`]:
    /// [`ClientError::FeeTooHigh`] above the fee cap,
    /// [`ClientError::UnexpectedResponse`] for a batch with no `Sequence` or
    /// `RawTransactions` or an inner transaction with no `Account`,
    /// [`ClientError::Core`] for one that cannot be priced, and a
    /// [`ClientError`] when a request fails.
    pub fn autofill_batch(
        &self,
        tx: &mut Value,
        outer: &str,
        open_ledger_fee: Option<u64>,
        outer_signers: u32,
    ) -> Result<(), ClientError> {
        self.call(|c| c.autofill_batch(tx, outer, open_ledger_fee, outer_signers))
    }

    /// [`Client::verified_ledger`]: a ledger with its transactions, checked
    /// against its hash.
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::verified_ledger`]:
    /// [`ClientError::Core`] when the answer does not check out against the
    /// ledger's hash, and a [`ClientError`] when the request fails.
    pub fn verified_ledger(
        &self,
        ledger: LedgerIndex,
    ) -> Result<xrpl_core::ledger::VerifiedLedger, ClientError> {
        self.call(|c| c.verified_ledger(ledger))
    }

    /// [`Client::validated_ledger_index`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::validated_ledger_index`]:
    /// [`ClientError::UnexpectedResponse`] for an answer with no
    /// `ledger_index`, and a [`ClientError`] when the request fails.
    pub fn validated_ledger_index(&self) -> Result<u32, ClientError> {
        self.call(|c| c.validated_ledger_index())
    }

    /// [`Client::simulate`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::simulate`]: [`ClientError::RpcError`]
    /// from a server that refuses it (`unknownCmd` before rippled 2.4), and a
    /// [`ClientError`] when the request fails.
    pub fn simulate(&self, tx: &Value) -> Result<SimulateResult, ClientError> {
        self.call(|c| c.simulate(tx))
    }

    /// [`Client::submit`]: a signed blob, once.
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::submit`]:
    /// [`ClientError::TransactionFailed`], [`ClientError::TransactionRetry`] or
    /// [`ClientError::TransactionClaimed`] for a preliminary result other than
    /// `tesSUCCESS`, and a [`ClientError`] when the request fails.
    pub fn submit(&self, tx_blob: &str) -> Result<SubmitResult, ClientError> {
        self.call(|c| c.submit(tx_blob))
    }

    /// [`Client::submit_multisigned`].
    ///
    /// # Errors
    ///
    /// As [`BlockingClient::submit`].
    pub fn submit_multisigned(&self, tx_json: &Value) -> Result<SubmitResult, ClientError> {
        self.call(|c| c.submit_multisigned(tx_json))
    }

    /// [`Client::submit_and_wait`]: submit, then wait for the final outcome.
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::submit_and_wait`]: every ending but a
    /// validated `tesSUCCESS` ([`ClientError::TransactionClaimed`],
    /// [`ClientError::TransactionFailed`], [`ClientError::TransactionExpired`],
    /// [`ClientError::OutcomeUnknown`]); [`ClientError::Core`] for a blob that
    /// does not decode, is not signed or has no `LastLedgerSequence`; and a
    /// [`ClientError`] when the transaction was never sent: the first request
    /// failed, or the server refused the submit request
    /// ([`ClientError::RpcError`]).
    pub fn submit_and_wait(&self, tx_blob: &str) -> Result<TxResult, ClientError> {
        self.call(|c| c.submit_and_wait(tx_blob))
    }

    /// [`Client::account_channels`] (one page).
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::account_channels`]:
    /// [`ClientError::RpcError`] (`actNotFound`) for an account that does not
    /// exist, and a [`ClientError`] when the request fails.
    pub fn account_channels(
        &self,
        account: &str,
        destination_account: Option<&str>,
        ledger: LedgerIndex,
    ) -> Result<AccountChannelsResult, ClientError> {
        self.call(|c| c.account_channels(account, destination_account, ledger))
    }

    /// [`Client::account_currencies`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::account_currencies`]:
    /// [`ClientError::RpcError`] (`actNotFound`) for an account that does not
    /// exist, and a [`ClientError`] when the request fails.
    pub fn account_currencies(
        &self,
        account: &str,
        ledger: LedgerIndex,
    ) -> Result<AccountCurrenciesResult, ClientError> {
        self.call(|c| c.account_currencies(account, ledger))
    }

    /// [`Client::account_lines_all`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::account_lines_all`]:
    /// [`ClientError::RpcError`] (`actNotFound`) for an account that does not
    /// exist, [`ClientError::PaginationLimitReached`] past
    /// [`MAX_PAGES`](crate::client::MAX_PAGES) pages, and a [`ClientError`]
    /// when a request fails.
    pub fn account_lines_all(
        &self,
        account: &str,
        ledger: LedgerIndex,
    ) -> Result<Vec<TrustLine>, ClientError> {
        self.call(|c| c.account_lines_all(account, ledger))
    }

    /// [`Client::account_nfts`] (one page).
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::account_nfts`]:
    /// [`ClientError::RpcError`] (`actNotFound`) for an account that does not
    /// exist, and a [`ClientError`] when the request fails.
    pub fn account_nfts(
        &self,
        account: &str,
        ledger: LedgerIndex,
    ) -> Result<AccountNftsResult, ClientError> {
        self.call(|c| c.account_nfts(account, ledger))
    }

    /// [`Client::account_nfts_all`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::account_nfts_all`]:
    /// [`ClientError::RpcError`] (`actNotFound`) for an account that does not
    /// exist, [`ClientError::PaginationLimitReached`] past
    /// [`MAX_PAGES`](crate::client::MAX_PAGES) pages, and a [`ClientError`]
    /// when a request fails.
    pub fn account_nfts_all(
        &self,
        account: &str,
        ledger: LedgerIndex,
    ) -> Result<Vec<NFToken>, ClientError> {
        self.call(|c| c.account_nfts_all(account, ledger))
    }

    /// [`Client::account_objects`] (one page).
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::account_objects`]:
    /// [`ClientError::RpcError`] (`actNotFound`) for an account that does not
    /// exist, and a [`ClientError`] when the request fails.
    pub fn account_objects(
        &self,
        account: &str,
        object_type: Option<&str>,
        ledger: LedgerIndex,
        limit: Option<u32>,
        marker: Option<Value>,
    ) -> Result<AccountObjectsResult, ClientError> {
        self.call(|c| c.account_objects(account, object_type, ledger, limit, marker))
    }

    /// [`Client::account_objects_all`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::account_objects_all`]:
    /// [`ClientError::RpcError`] (`actNotFound`) for an account that does not
    /// exist, [`ClientError::PaginationLimitReached`] past
    /// [`MAX_PAGES`](crate::client::MAX_PAGES) pages, and a [`ClientError`]
    /// when a request fails.
    pub fn account_objects_all(
        &self,
        account: &str,
        object_type: Option<&str>,
        ledger: LedgerIndex,
    ) -> Result<Vec<LedgerObject>, ClientError> {
        self.call(|c| c.account_objects_all(account, object_type, ledger))
    }

    /// [`Client::account_offers`] (one page).
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::account_offers`]:
    /// [`ClientError::RpcError`] (`actNotFound`) for an account that does not
    /// exist, and a [`ClientError`] when the request fails.
    pub fn account_offers(
        &self,
        account: &str,
        ledger: LedgerIndex,
    ) -> Result<AccountOffersResult, ClientError> {
        self.call(|c| c.account_offers(account, ledger))
    }

    /// [`Client::account_offers_all`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::account_offers_all`]:
    /// [`ClientError::RpcError`] (`actNotFound`) for an account that does not
    /// exist, [`ClientError::PaginationLimitReached`] past
    /// [`MAX_PAGES`](crate::client::MAX_PAGES) pages, and a [`ClientError`]
    /// when a request fails.
    pub fn account_offers_all(
        &self,
        account: &str,
        ledger: LedgerIndex,
    ) -> Result<Vec<AccountOffer>, ClientError> {
        self.call(|c| c.account_offers_all(account, ledger))
    }

    /// [`Client::account_tx`] (one page).
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::account_tx`]: a [`ClientError`] when
    /// the request fails.
    pub fn account_tx(&self, account: &str) -> Result<AccountTxResult, ClientError> {
        self.call(|c| c.account_tx(account))
    }

    /// [`Client::account_tx_all`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::account_tx_all`]:
    /// [`ClientError::PaginationLimitReached`] past
    /// [`MAX_PAGES`](crate::client::MAX_PAGES) pages, and a [`ClientError`]
    /// when a request fails.
    pub fn account_tx_all(&self, account: &str) -> Result<Vec<TxResult>, ClientError> {
        self.call(|c| c.account_tx_all(account))
    }

    /// [`Client::amm_info`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::amm_info`]: [`ClientError::RpcError`]
    /// (`actNotFound`) when no pool holds the pair, and a [`ClientError`] when
    /// the request fails.
    pub fn amm_info(
        &self,
        asset: &xrpl_core::types::Asset,
        asset2: &xrpl_core::types::Asset,
    ) -> Result<AmmInfoResult, ClientError> {
        self.call(|c| c.amm_info(asset, asset2))
    }

    /// [`Client::balances`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::balances`]: [`ClientError::RpcError`]
    /// (`actNotFound`) for an account that does not exist, and a
    /// [`ClientError`] when a request fails or an answer holds an amount that
    /// does not parse.
    pub fn balances(
        &self,
        account: &str,
        ledger: LedgerIndex,
    ) -> Result<Vec<xrpl_core::types::Amount>, ClientError> {
        self.call(|c| c.balances(account, ledger))
    }

    /// [`Client::book_offers`] (one page).
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::book_offers`]: a [`ClientError`] when
    /// the request fails.
    pub fn book_offers(
        &self,
        taker_gets: &xrpl_core::types::Asset,
        taker_pays: &xrpl_core::types::Asset,
    ) -> Result<BookOffersResult, ClientError> {
        self.call(|c| c.book_offers(taker_gets, taker_pays))
    }

    /// [`Client::order_book`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::order_book`].
    pub fn order_book(
        &self,
        base: &xrpl_core::types::Asset,
        quote: &xrpl_core::types::Asset,
        limit: u32,
    ) -> Result<OrderBook, ClientError> {
        self.call(|c| c.order_book(base, quote, limit))
    }

    /// [`Client::gateway_balances`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::gateway_balances`]: a [`ClientError`]
    /// when the request fails.
    pub fn gateway_balances(
        &self,
        account: &str,
        hotwallet: Option<Vec<&str>>,
        ledger: LedgerIndex,
    ) -> Result<GatewayBalancesResult, ClientError> {
        self.call(|c| c.gateway_balances(account, hotwallet, ledger))
    }

    /// [`Client::ledger_entry`].
    ///
    /// # Errors
    ///
    /// [`ClientError::InsideAsyncRuntime`] when called inside a Tokio runtime;
    /// otherwise the errors of [`Client::ledger_entry`]:
    /// [`ClientError::RpcError`] (`entryNotFound`) for an entry the ledger does
    /// not hold, and a [`ClientError`] when the request fails.
    pub fn ledger_entry(
        &self,
        index: &str,
        ledger: LedgerIndex,
    ) -> Result<LedgerEntryResult, ClientError> {
        self.call(|c| c.ledger_entry(index, ledger))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use serde_json::json;

    struct Fixed;
    impl crate::client::Transport for Fixed {
        fn send_raw<'a>(&'a self, method: &'a str, _: Value) -> crate::client::TransportFuture<'a> {
            Box::pin(async move {
                Ok(match method {
                    "ledger_current" => json!({"ledger_current_index": 42}),
                    "server_info" => json!({"info": {"network_id": 1}}),
                    "fee" => {
                        json!({"drops": {"base_fee": "10", "median_fee": "10", "minimum_fee": "10", "open_ledger_fee": "12"}})
                    }
                    _ => json!({}),
                })
            })
        }
    }

    /// Plain `#[test]`, no async runtime: the client brings its own.
    #[test]
    fn calls_return_values_without_an_async_runtime() {
        let client = BlockingClient::with_client(Fixed).unwrap();
        assert_eq!(client.ledger_current().unwrap(), 42);
        assert_eq!(client.fee().unwrap().drops.open_ledger_fee, "12");
        let mut tx = json!({"TransactionType": "AccountSet", "Account": "rA", "Sequence": 1});
        client.autofill(&mut tx).unwrap();
        assert_eq!(
            (tx["Fee"].clone(), tx["LastLedgerSequence"].clone()),
            (json!("12"), json!(62))
        );
        assert_eq!(client.block_on(async { 7 }), 7);
    }

    /// Inside an async runtime the calls refuse instead of panicking.
    #[tokio::test]
    async fn refuses_inside_an_async_runtime() {
        let client = std::thread::spawn(|| BlockingClient::with_client(Fixed).unwrap())
            .join()
            .unwrap();
        assert!(matches!(
            client.ledger_current(),
            Err(ClientError::InsideAsyncRuntime)
        ));
        // The runtime cannot be dropped inside another one either.
        std::thread::spawn(move || drop(client)).join().unwrap();
    }
    /// The module's promise, "every `Client` method as a plain call", is
    /// kept: each async method of the trait has a blocking one here.
    #[test]
    fn every_client_method_has_a_blocking_one() {
        // A Windows checkout may have CRLF line ends.
        let client = include_str!("client.rs").replace("\r\n", "\n");
        let start = client.find("pub trait Client: Transport {").unwrap();
        let end = start + client[start..].find("\n}\n").unwrap();
        let blocking = include_str!("blocking.rs");
        let mut checked = 0;
        for line in client[start..end].lines() {
            let Some(rest) = line.strip_prefix("    fn ") else {
                continue;
            };
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            assert!(
                blocking.contains(&format!("pub fn {name}("))
                    || blocking.contains(&format!("pub fn {name}<")),
                "BlockingClient has no {name}"
            );
            checked += 1;
        }
        assert!(checked > 30, "{checked}");
    }
}
