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
//!     let account = client.account_info("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", &LedgerIndex::Validated)?;
//!     println!("balance: {} drops", account.account_data.balance);
//!
//!     // Any typed request, or any async method with `run`.
//!     let changes = client.request(&BookChangesRequest::new(LedgerIndex::Validated))?;
//!     let lines = client.run(|c| c.account_lines_all("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", &LedgerIndex::Validated))?;
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
    pub fn new(url: &str) -> Result<Self, ClientError> {
        Self::with_client(XrplHttpClient::new(url)?)
    }
}

impl<C: Client> BlockingClient<C> {
    /// Block on any [`Client`] (an [`XrplHttpClient`], or your own transport).
    pub fn with_client(inner: C) -> Result<Self, ClientError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                ClientError::UnexpectedResponse(format!("could not start a runtime: {e}"))
            })?;
        Ok(Self { inner, runtime })
    }

    /// The async client underneath.
    pub fn inner(&self) -> &C {
        &self.inner
    }

    /// Run any async method of the client to completion:
    /// `client.run(|c| c.account_lines_all(addr, &ledger))`.
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
    pub fn request<R: Request>(&self, req: &R) -> Result<R::Response, ClientError> {
        self.call(|c| c.request(req))
    }

    /// [`Client::request_all`]: every page of a paginated request.
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
    pub fn raw_request(&self, method: &str, params: Value) -> Result<Value, ClientError> {
        self.call(|c| c.raw_request(method, params))
    }

    /// [`Client::account_info`].
    pub fn account_info(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountInfo, ClientError> {
        self.call(|c| c.account_info(account, ledger))
    }

    /// [`Client::account_lines`] (one page).
    pub fn account_lines(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountLinesResult, ClientError> {
        self.call(|c| c.account_lines(account, ledger))
    }

    /// [`Client::fee`].
    pub fn fee(&self) -> Result<FeeResult, ClientError> {
        self.call(|c| c.fee())
    }

    /// [`Client::ledger_current`].
    pub fn ledger_current(&self) -> Result<u32, ClientError> {
        self.call(|c| c.ledger_current())
    }

    /// [`Client::ledger`].
    pub fn ledger(&self, ledger: &LedgerIndex) -> Result<LedgerResult, ClientError> {
        self.call(|c| c.ledger(ledger))
    }

    /// [`Client::server_info`].
    pub fn server_info(&self) -> Result<ServerInfoResult, ClientError> {
        self.call(|c| c.server_info())
    }

    /// [`Client::tx`].
    pub fn tx(&self, hash: &str) -> Result<TxResult, ClientError> {
        self.call(|c| c.tx(hash))
    }

    /// [`Client::autofill`]: `Sequence`, `Fee`, `LastLedgerSequence` (and a
    /// `Batch`'s inner transactions).
    pub fn autofill(&self, tx: &mut Value) -> Result<(), ClientError> {
        self.call(|c| c.autofill(tx))
    }

    /// [`Client::autofill_for_multisigning`].
    pub fn autofill_for_multisigning(
        &self,
        tx: &mut Value,
        signers: u32,
    ) -> Result<(), ClientError> {
        self.call(|c| c.autofill_for_multisigning(tx, signers))
    }

    /// [`Client::autofill_batch`]: the inner-transaction part of
    /// [`BlockingClient::autofill`] for a `Batch`.
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
    pub fn verified_ledger(
        &self,
        ledger: &LedgerIndex,
    ) -> Result<xrpl_core::ledger::VerifiedLedger, ClientError> {
        self.call(|c| c.verified_ledger(ledger))
    }

    /// [`Client::validated_ledger_index`].
    pub fn validated_ledger_index(&self) -> Result<u32, ClientError> {
        self.call(|c| c.validated_ledger_index())
    }

    /// [`Client::simulate`].
    pub fn simulate(&self, tx: &Value) -> Result<SimulateResult, ClientError> {
        self.call(|c| c.simulate(tx))
    }

    /// [`Client::submit`]: a signed blob, once.
    pub fn submit(&self, tx_blob: &str) -> Result<SubmitResult, ClientError> {
        self.call(|c| c.submit(tx_blob))
    }

    /// [`Client::submit_multisigned`].
    pub fn submit_multisigned(&self, tx_json: &Value) -> Result<SubmitResult, ClientError> {
        self.call(|c| c.submit_multisigned(tx_json))
    }

    /// [`Client::submit_and_wait`]: submit, then wait for the final outcome.
    pub fn submit_and_wait(&self, tx_blob: &str) -> Result<TxResult, ClientError> {
        self.call(|c| c.submit_and_wait(tx_blob))
    }

    /// [`Client::account_channels`] (one page).
    pub fn account_channels(
        &self,
        account: &str,
        destination_account: Option<&str>,
        ledger: &LedgerIndex,
    ) -> Result<AccountChannelsResult, ClientError> {
        self.call(|c| c.account_channels(account, destination_account, ledger))
    }

    /// [`Client::account_currencies`].
    pub fn account_currencies(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountCurrenciesResult, ClientError> {
        self.call(|c| c.account_currencies(account, ledger))
    }

    /// [`Client::account_lines_all`].
    pub fn account_lines_all(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<Vec<TrustLine>, ClientError> {
        self.call(|c| c.account_lines_all(account, ledger))
    }

    /// [`Client::account_nfts`] (one page).
    pub fn account_nfts(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountNftsResult, ClientError> {
        self.call(|c| c.account_nfts(account, ledger))
    }

    /// [`Client::account_nfts_all`].
    pub fn account_nfts_all(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<Vec<NFToken>, ClientError> {
        self.call(|c| c.account_nfts_all(account, ledger))
    }

    /// [`Client::account_objects`] (one page).
    pub fn account_objects(
        &self,
        account: &str,
        object_type: Option<&str>,
        ledger: &LedgerIndex,
        limit: Option<u32>,
        marker: Option<Value>,
    ) -> Result<AccountObjectsResult, ClientError> {
        self.call(|c| c.account_objects(account, object_type, ledger, limit, marker))
    }

    /// [`Client::account_objects_all`].
    pub fn account_objects_all(
        &self,
        account: &str,
        object_type: Option<&str>,
        ledger: &LedgerIndex,
    ) -> Result<Vec<LedgerObject>, ClientError> {
        self.call(|c| c.account_objects_all(account, object_type, ledger))
    }

    /// [`Client::account_offers`] (one page).
    pub fn account_offers(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountOffersResult, ClientError> {
        self.call(|c| c.account_offers(account, ledger))
    }

    /// [`Client::account_offers_all`].
    pub fn account_offers_all(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<Vec<AccountOffer>, ClientError> {
        self.call(|c| c.account_offers_all(account, ledger))
    }

    /// [`Client::account_tx`] (one page).
    pub fn account_tx(
        &self,
        account: &str,
        ledger_index_min: Option<i32>,
        ledger_index_max: Option<i32>,
        limit: Option<u32>,
        marker: Option<Value>,
        forward: Option<bool>,
    ) -> Result<AccountTxResult, ClientError> {
        self.call(|c| {
            c.account_tx(
                account,
                ledger_index_min,
                ledger_index_max,
                limit,
                marker,
                forward,
            )
        })
    }

    /// [`Client::account_tx_all`].
    pub fn account_tx_all(
        &self,
        account: &str,
        ledger_index_min: Option<i32>,
        ledger_index_max: Option<i32>,
    ) -> Result<Vec<AccountTxEntry>, ClientError> {
        self.call(|c| c.account_tx_all(account, ledger_index_min, ledger_index_max))
    }

    /// [`Client::amm_info`].
    pub fn amm_info(&self, asset: &Value, asset2: &Value) -> Result<AmmInfoResult, ClientError> {
        self.call(|c| c.amm_info(asset, asset2))
    }

    /// [`Client::balances`].
    pub fn balances(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<Vec<xrpl_core::types::Amount>, ClientError> {
        self.call(|c| c.balances(account, ledger))
    }

    /// [`Client::book_offers`] (one page).
    pub fn book_offers(
        &self,
        taker_pays: &Value,
        taker_gets: &Value,
    ) -> Result<BookOffersResult, ClientError> {
        self.call(|c| c.book_offers(taker_pays, taker_gets))
    }

    /// [`Client::gateway_balances`].
    pub fn gateway_balances(
        &self,
        account: &str,
        hotwallet: Option<Vec<&str>>,
        ledger: &LedgerIndex,
    ) -> Result<GatewayBalancesResult, ClientError> {
        self.call(|c| c.gateway_balances(account, hotwallet, ledger))
    }

    /// [`Client::ledger_entry`].
    pub fn ledger_entry(
        &self,
        index: &str,
        ledger: &LedgerIndex,
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
    impl Client for Fixed {
        async fn raw_request(&self, method: &str, _: Value) -> Result<Value, ClientError> {
            Ok(match method {
                "ledger_current" => json!({"ledger_current_index": 42}),
                "server_info" => json!({"info": {"network_id": 1}}),
                "fee" => {
                    json!({"drops": {"base_fee": "10", "median_fee": "10", "minimum_fee": "10", "open_ledger_fee": "12"}})
                }
                _ => json!({}),
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
        let start = client.find("pub trait Client {").unwrap();
        let end = start + client[start..].find("\n}\n").unwrap();
        let blocking = include_str!("blocking.rs");
        let mut checked = 0;
        for line in client[start..end].lines() {
            let Some(rest) = line.strip_prefix("    async fn ") else {
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
