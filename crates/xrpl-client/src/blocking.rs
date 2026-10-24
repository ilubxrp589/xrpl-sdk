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
///
/// ```
/// # use serde_json::{json, Value};
/// # use xrpl_client::{Transport, TransportFuture};
/// # struct Canned(Value);
/// # impl Transport for Canned {
/// #     fn send_raw<'a>(&'a self, _: &'a str, _: Value) -> TransportFuture<'a> {
/// #         let answer = self.0.clone();
/// #         Box::pin(async move { Ok(answer) })
/// #     }
/// # }
/// use xrpl_client::blocking::BlockingClient;
///
/// // Any transport: here one that answers every request with the same fees.
/// let fees = json!({"drops": {"base_fee": "10", "median_fee": "5000", "minimum_fee": "10", "open_ledger_fee": "12"}});
/// let client = BlockingClient::with_client(Canned(fees))?;
/// assert_eq!(client.fee()?.drops.open_ledger_fee, "12");
/// # Ok::<(), xrpl_client::ClientError>(())
/// ```
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
    /// ```
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::client::DEFAULT_MAX_FEE_DROPS;
    /// use xrpl_client::Transport;
    ///
    /// // Nothing is sent until the first call.
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// assert_eq!(client.inner().max_fee_drops(), DEFAULT_MAX_FEE_DROPS);
    /// # Ok::<(), xrpl_client::ClientError>(())
    /// ```
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
    /// [`ClientError::Transport`] (`Setup`) if the runtime cannot start.
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
    ///
    /// ```
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::{Transport, XrplHttpClient};
    ///
    /// let http = XrplHttpClient::new("https://s.altnet.rippletest.net:51234")?.with_max_fee_drops(100_000);
    /// let client = BlockingClient::with_client(http)?;
    /// assert_eq!(client.inner().max_fee_drops(), 100_000);
    /// # Ok::<(), xrpl_client::ClientError>(())
    /// ```
    pub fn inner(&self) -> &C {
        &self.inner
    }

    /// Run any async method of the client to completion:
    /// `client.run(|c| c.account_lines_all(addr, ledger))`.
    ///
    /// ```
    /// # use serde_json::{json, Value};
    /// # use xrpl_client::{Transport, TransportFuture};
    /// # struct Canned(Value);
    /// # impl Transport for Canned {
    /// #     fn send_raw<'a>(&'a self, _: &'a str, _: Value) -> TransportFuture<'a> {
    /// #         let answer = self.0.clone();
    /// #         Box::pin(async move { Ok(answer) })
    /// #     }
    /// # }
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    /// use xrpl_client::Client;
    ///
    /// // A transport whose every answer is a last page with no trust lines.
    /// let client = BlockingClient::with_client(Canned(json!({"lines": []})))?;
    /// let lines = client.run(|c| c.account_lines_all("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", LedgerIndex::Validated))?;
    /// assert!(lines.is_empty());
    /// # Ok::<(), xrpl_client::ClientError>(())
    /// ```
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
    /// ```
    /// # use serde_json::{json, Value};
    /// # use xrpl_client::{Transport, TransportFuture};
    /// # struct Canned(Value);
    /// # impl Transport for Canned {
    /// #     fn send_raw<'a>(&'a self, _: &'a str, _: Value) -> TransportFuture<'a> {
    /// #         let answer = self.0.clone();
    /// #         Box::pin(async move { Ok(answer) })
    /// #     }
    /// # }
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::Client;
    ///
    /// // Any future, such as a faucet call; here the async client's own.
    /// let client = BlockingClient::with_client(Canned(json!({"ledger_current_index": 7})))?;
    /// assert_eq!(client.block_on(client.inner().ledger_current())?, 7);
    /// # Ok::<(), xrpl_client::ClientError>(())
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::requests::LedgerClosedRequest;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let closed = client.request(&LedgerClosedRequest::new())?;
    /// println!("ledger {} ({}) closed", closed.ledger_index, closed.ledger_hash);
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::requests::AccountObjectsRequest;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let objects = client.request_all(AccountObjectsRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"))?;
    /// println!("{} ledger entries", objects.len());
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::requests::AccountTxRequest;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// // The newest 1,000 or so transactions, page by page.
    /// let mut seen = 0;
    /// for page in client.pages(AccountTxRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")) {
    ///     seen += page?.len();
    ///     if seen >= 1_000 {
    ///         break;
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let closed = client.raw_request("ledger_closed", serde_json::json!({}))?;
    /// println!("ledger {} closed", closed["ledger_index"]);
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let info = client.account_info("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", LedgerIndex::Validated)?;
    /// println!("{}, next sequence {}", info.account_data.balance, info.account_data.sequence);
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// for line in client.account_lines("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", LedgerIndex::Validated)?.lines {
    ///     println!("{} {} with {}", line.balance, line.currency, line.account);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// println!("open ledger fee: {} drops", client.fee()?.drops.open_ledger_fee);
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```
    /// # use serde_json::{json, Value};
    /// # use xrpl_client::{Transport, TransportFuture};
    /// # struct Canned(Value);
    /// # impl Transport for Canned {
    /// #     fn send_raw<'a>(&'a self, _: &'a str, _: Value) -> TransportFuture<'a> {
    /// #         let answer = self.0.clone();
    /// #         Box::pin(async move { Ok(answer) })
    /// #     }
    /// # }
    /// use xrpl_client::blocking::BlockingClient;
    ///
    /// // A transport that answers as a server whose open ledger is 7.
    /// let client = BlockingClient::with_client(Canned(json!({"ledger_current_index": 7})))?;
    /// assert_eq!(client.ledger_current()?, 7);
    /// # Ok::<(), xrpl_client::ClientError>(())
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let ledger = client.ledger(LedgerIndex::Validated)?;
    /// println!("ledger {:?}: {:?}", ledger.ledger_index, ledger.ledger_hash);
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let info = client.server_info()?.info;
    /// println!("{:?} on network {:?}", info.build_version, info.network_id);
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let tx = client.tx("E17E9EB903392000BC727251524391893D147E280B64F02EE0B5B9CFE7968CDA")?;
    /// println!("{:?} in ledger {:?}", tx.transaction_result(), tx.ledger_index);
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let mut tx = serde_json::json!({"TransactionType": "AccountSet", "Account": "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD"});
    /// client.autofill(&mut tx)?;
    /// println!("Sequence {}, Fee {}, LastLedgerSequence {}", tx["Sequence"], tx["Fee"], tx["LastLedgerSequence"]);
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// // For two signers: the fee is three times the open-ledger fee.
    /// let mut tx = serde_json::json!({"TransactionType": "AccountSet", "Account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"});
    /// client.autofill_for_multisigning(&mut tx, 2)?;
    /// assert_eq!(tx["SigningPubKey"], "");
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run(mut batch: serde_json::Value) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// // `batch`: a Batch with its Sequence and RawTransactions; price it at the
    /// // open-ledger fee, for no multi-signers of its own.
    /// let open_ledger_fee = client.fee()?.drops.open_ledger_fee.parse().ok();
    /// client.autofill_batch(&mut batch, "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD", open_ledger_fee, 0)?;
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let ledger = client.verified_ledger(LedgerIndex::Validated)?;
    /// println!("ledger {} ({}): {} transactions", ledger.header.ledger_index, ledger.hash, ledger.transactions.len());
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// println!("ledger {} is validated", client.validated_ledger_index()?);
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let payment = serde_json::json!({
    ///     "TransactionType": "Payment",
    ///     "Account": "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD",
    ///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
    ///     "Amount": "1000000",
    /// });
    /// println!("{}", client.simulate(&payment)?.engine_result);
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run(tx_blob: &str) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// // `tx_blob`: a signed transaction, in hex.
    /// let preliminary = client.submit(tx_blob)?;
    /// println!("sent {:?}; not final until a validated ledger holds it", preliminary.hash());
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run(tx_json: serde_json::Value) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// // `tx_json`: the transaction with its `Signers` and an empty `SigningPubKey`.
    /// println!("sent {:?}", client.submit_multisigned(&tx_json)?.hash());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// As [`BlockingClient::submit`].
    pub fn submit_multisigned(&self, tx_json: &Value) -> Result<SubmitResult, ClientError> {
        self.call(|c| c.submit_multisigned(tx_json))
    }

    /// [`Client::submit_and_wait`]: submit, then wait for the final outcome.
    ///
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_core::crypto::{sign_transaction, Keypair, Seed};
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let keys = Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")?)?;
    /// let mut tx = serde_json::json!({"TransactionType": "Payment", "Account": keys.classic_address(),
    ///     "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "Amount": "1000000"});
    /// client.autofill(&mut tx)?;
    /// let blob = xrpl_core::codec::encode_transaction_json(&sign_transaction(&tx, &keys)?)?;
    /// let result = client.submit_and_wait(&hex::encode_upper(blob))?;
    /// println!("validated in ledger {:?}", result.ledger_index);
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let page = client.account_channels("rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD", None, LedgerIndex::Validated)?;
    /// println!("{} channels", page.channels.len());
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let currencies = client.account_currencies("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", LedgerIndex::Validated)?;
    /// println!("receives {:?}", currencies.receive_currencies);
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let lines = client.account_lines_all("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", LedgerIndex::Validated)?;
    /// println!("{} trust lines", lines.len());
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// for nft in client.account_nfts("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", LedgerIndex::Validated)?.account_nfts {
    ///     println!("{}", nft.nf_token_id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let nfts = client.account_nfts_all("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", LedgerIndex::Validated)?;
    /// println!("{} NFTs", nfts.len());
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// // The first page of the account's escrows.
    /// let account = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
    /// let page = client.account_objects(account, Some("escrow"), LedgerIndex::Validated, None, None)?;
    /// println!("{} escrows", page.account_objects.len());
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let account = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
    /// let checks = client.account_objects_all(account, Some("check"), LedgerIndex::Validated)?;
    /// println!("{} checks", checks.len());
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// for offer in client.account_offers("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", LedgerIndex::Validated)?.offers {
    ///     println!("#{}: {:?} for {:?}", offer.seq, offer.taker_gets, offer.taker_pays);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let offers = client.account_offers_all("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", LedgerIndex::Validated)?;
    /// println!("{} open offers", offers.len());
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// for tx in client.account_tx("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")?.transactions {
    ///     println!("{:?}: {:?}", tx.hash(), tx.transaction_result());
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let history = client.account_tx_all("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")?;
    /// println!("{} transactions", history.len());
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_core::types::Asset;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let usd = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
    /// let pool = client.amm_info(&Asset::xrp(), &usd)?.amm;
    /// println!("{:?} and {:?}", pool.amount, pool.amount2);
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// for amount in client.balances("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", LedgerIndex::Validated)? {
    ///     println!("{amount}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_core::types::Asset;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let usd = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
    /// println!("{} offers", client.book_offers(&Asset::xrp(), &usd)?.offers.len());
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_core::types::Asset;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let usd = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
    /// let book = client.order_book(&Asset::xrp(), &usd, 10)?;
    /// println!("bid {:?}, ask {:?}", book.best_bid(), book.best_ask());
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let balances = client.gateway_balances("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", None, LedgerIndex::Validated)?;
    /// println!("{:?}", balances.obligations);
    /// # Ok(())
    /// # }
    /// ```
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
    /// ```no_run
    /// # fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::blocking::BlockingClient;
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let client = BlockingClient::new("https://s.altnet.rippletest.net:51234")?;
    /// let id = xrpl_core::hashes::keylet::account("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe")?;
    /// println!("{:?}", client.ledger_entry(&id.to_string(), LedgerIndex::Validated)?.node);
    /// # Ok(())
    /// # }
    /// ```
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
