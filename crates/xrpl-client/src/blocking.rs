//! A blocking client: every [`Client`] method as a plain function call, for
//! scripts, command-line tools and programs without an async runtime.
//!
//! [`BlockingClient`] owns a small single-threaded Tokio runtime and runs each
//! call to completion on it. Do not use it from inside an async runtime
//! (Tokio refuses to block one); use the async client there.
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
    pub fn run<'a, F: Future>(&'a self, call: impl FnOnce(&'a C) -> F) -> F::Output {
        self.runtime.block_on(call(&self.inner))
    }

    /// Run any future to completion on this client's runtime (for example a
    /// `faucet` call).
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }

    /// [`Client::request`]: any typed request.
    pub fn request<R: Request>(&self, req: &R) -> Result<R::Response, ClientError> {
        self.run(|c| c.request(req))
    }

    /// [`Client::request_all`]: every page of a paginated request.
    pub fn request_all<R: Paginated>(&self, req: R) -> Result<Vec<R::Item>, ClientError> {
        self.run(|c| c.request_all(req))
    }

    /// [`Client::raw_request`]: any method, JSON in and out.
    pub fn raw_request(&self, method: &str, params: Value) -> Result<Value, ClientError> {
        self.run(|c| c.raw_request(method, params))
    }

    /// [`Client::account_info`].
    pub fn account_info(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountInfo, ClientError> {
        self.run(|c| c.account_info(account, ledger))
    }

    /// [`Client::account_lines`] (one page).
    pub fn account_lines(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountLinesResult, ClientError> {
        self.run(|c| c.account_lines(account, ledger))
    }

    /// [`Client::fee`].
    pub fn fee(&self) -> Result<FeeResult, ClientError> {
        self.run(|c| c.fee())
    }

    /// [`Client::ledger_current`].
    pub fn ledger_current(&self) -> Result<u32, ClientError> {
        self.run(|c| c.ledger_current())
    }

    /// [`Client::ledger`].
    pub fn ledger(&self, ledger: &LedgerIndex) -> Result<LedgerResult, ClientError> {
        self.run(|c| c.ledger(ledger))
    }

    /// [`Client::server_info`].
    pub fn server_info(&self) -> Result<ServerInfoResult, ClientError> {
        self.run(|c| c.server_info())
    }

    /// [`Client::tx`].
    pub fn tx(&self, hash: &str) -> Result<TxResult, ClientError> {
        self.run(|c| c.tx(hash))
    }

    /// [`Client::autofill`]: `Sequence`, `Fee`, `LastLedgerSequence` (and a
    /// `Batch`'s inner transactions).
    pub fn autofill(&self, tx: &mut Value) -> Result<(), ClientError> {
        self.run(|c| c.autofill(tx))
    }

    /// [`Client::autofill_for_multisigning`].
    pub fn autofill_for_multisigning(
        &self,
        tx: &mut Value,
        signers: u32,
    ) -> Result<(), ClientError> {
        self.run(|c| c.autofill_for_multisigning(tx, signers))
    }

    /// [`Client::validated_ledger_index`].
    pub fn validated_ledger_index(&self) -> Result<u32, ClientError> {
        self.run(|c| c.validated_ledger_index())
    }

    /// [`Client::simulate`].
    pub fn simulate(&self, tx: &Value) -> Result<SimulateResult, ClientError> {
        self.run(|c| c.simulate(tx))
    }

    /// [`Client::submit`]: a signed blob, once.
    pub fn submit(&self, tx_blob: &str) -> Result<SubmitResult, ClientError> {
        self.run(|c| c.submit(tx_blob))
    }

    /// [`Client::submit_multisigned`].
    pub fn submit_multisigned(&self, tx_json: &Value) -> Result<SubmitResult, ClientError> {
        self.run(|c| c.submit_multisigned(tx_json))
    }

    /// [`Client::submit_and_wait`]: submit, then wait for the final outcome.
    pub fn submit_and_wait(&self, tx_blob: &str) -> Result<TxResult, ClientError> {
        self.run(|c| c.submit_and_wait(tx_blob))
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
}
