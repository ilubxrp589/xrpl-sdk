//! [`ClientError`]: transport and JSON failures, `rippled` errors, and the
//! sorted outcome of a submitted transaction.

/// Everything a client call can fail with. New variants may be added in
/// minor releases (`#[non_exhaustive]`): match with a `_` arm.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ClientError {
    /// The server could not be reached, or the exchange with it broke off
    /// before an answer: what went wrong is [`ClientError::Transport::kind`],
    /// and the message says why (never the URL, which may carry a token).
    /// A timeout is [`ClientError::Timeout`].
    #[error("{kind}: {message}")]
    Transport {
        /// What part of the exchange failed.
        kind: TransportErrorKind,
        /// What the network stack said (with no URL).
        message: String,
    },

    /// The server answered with an HTTP error status, after any retries the
    /// client's [`RetryPolicy`](crate::http::RetryPolicy) allows.
    #[error("HTTP status {status}: {excerpt}")]
    HttpStatus {
        /// The status code (429 Too Many Requests, 503 Service Unavailable, …).
        status: u16,
        /// How long the server asked to wait (`Retry-After`), when it said.
        retry_after: Option<std::time::Duration>,
        /// The start of the body (at most 200 characters).
        excerpt: String,
    },

    /// An answer, or a typed request, was not the JSON expected.
    #[error("unexpected JSON: {0}")]
    Json(serde_json::Error),

    /// `rippled` answered with an error: `code` is its token (e.g. `actNotFound`).
    #[error("RPC error: {code} — {message}")]
    RpcError {
        /// rippled's error token (`actNotFound`, `invalidParams`, …).
        code: String,
        /// Its message.
        message: String,
        /// The whole error response: `rippled` puts details beside the error
        /// (`tx`'s `searched_all`, the `request` it refused, …).
        response: Box<serde_json::Value>,
    },

    /// Refused. From [`Client::submit`](crate::Client::submit) this is the
    /// server's preliminary `tem`, `tef` or `tel`; only a `tem` (malformed)
    /// is sure never to reach a ledger. From
    /// [`Client::submit_and_wait`](crate::Client::submit_and_wait) it is
    /// final: a `tem`, or a `tef`/`tel` for a transaction no ledger holds.
    #[error("transaction failed: {engine_result} — {message}")]
    TransactionFailed {
        /// The engine result.
        engine_result: String,
        /// The engine result in words.
        message: String,
    },

    /// A preliminary result from [`Client::submit`](crate::Client::submit)
    /// that is not an outcome yet (`ter…`, `telINSUF_FEE_P`,
    /// `telCAN_NOT_QUEUE…`): the transaction was held, queued or dropped,
    /// and may still be applied. Resubmitting the same blob is safe; signing
    /// a new transaction for the same payment is not until this one's
    /// `LastLedgerSequence` has passed.
    #[error("transaction not applied yet: {engine_result}")]
    TransactionRetry {
        /// The engine result.
        engine_result: String,
    },

    /// A `tec`: the transaction failed, but it was (or, preliminary, would
    /// be) put in a ledger and its fee charged. From
    /// [`Client::submit_and_wait`](crate::Client::submit_and_wait) it is
    /// final and `result` holds the validated transaction and metadata; from
    /// [`Client::submit`](crate::Client::submit) it is preliminary (`result`
    /// is `None`) and the final result can still differ.
    #[error("transaction failed, fee charged: {engine_result}")]
    TransactionClaimed {
        /// The engine result.
        engine_result: String,
        /// The validated transaction, when the result is final.
        result: Option<Box<crate::types::TxResult>>,
    },

    /// A transaction was sent, but whether it is in a ledger could not be
    /// found out (the connection failed, or the server lacks the ledgers to
    /// prove it is in none). Look it up by `hash` before sending anything
    /// that replaces it.
    #[error("transaction {hash} was sent; its outcome is unknown ({reason})")]
    OutcomeUnknown {
        /// The transaction's ID.
        hash: String,
        /// The last ledger it can be in.
        last_ledger_sequence: u32,
        /// What stopped the check.
        reason: String,
    },

    /// [`Client::autofill`](crate::Client::autofill) would set a `Fee` above
    /// the client's cap (see `with_max_fee_drops` on each client).
    #[error("the fee {fee_drops} drops is above the cap of {max_fee_drops} drops")]
    FeeTooHigh {
        /// The fee the server's prices come to.
        fee_drops: u64,
        /// The cap.
        max_fee_drops: u64,
    },

    /// The WebSocket connection is closed.
    #[error("connection closed unexpectedly")]
    Disconnected,

    /// No answer in time.
    #[error("request timeout after {0:?}")]
    Timeout(std::time::Duration),

    /// The server's answer was not as `rippled` answers (a missing field, a
    /// number that does not parse, a body too large): the server's fault, or a
    /// server this crate does not know.
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    /// The request cannot be sent as given: the caller's mistake (a
    /// transaction with no `Account`, a `Batch` without its inner
    /// transactions, a header HTTP does not allow, …). Nothing was sent.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// Its `LastLedgerSequence` passed, and the server holds every ledger up
    /// to it: the transaction is in none and never will be.
    #[error("transaction expired: LastLedgerSequence {0} passed")]
    TransactionExpired(u32),

    /// `request_all` stopped after this many pages; read them one at a time
    /// with [`crate::Client::pages`], which has no limit.
    #[error(
        "pagination limit reached after {0} pages; read the pages one at a time with Client::pages"
    )]
    PaginationLimitReached(usize),

    /// A `BlockingClient` method (native targets only) was called inside an
    /// async runtime, which blocking would stall; use the async client.
    #[error("BlockingClient used inside an async runtime; use the async client there")]
    InsideAsyncRuntime,

    /// A core error (codec, keys, amounts, …), shown as itself.
    #[error(transparent)]
    Core(#[from] xrpl_core::CoreError),
}

/// Not `#[from]`: the message is in the variant's `Display`, so `source`
/// does not repeat it.
impl From<serde_json::Error> for ClientError {
    fn from(e: serde_json::Error) -> Self {
        ClientError::Json(e)
    }
}

/// What part of an exchange with the server failed: see
/// [`ClientError::Transport`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TransportErrorKind {
    /// Connecting failed (no route, refused, DNS, TLS).
    Connect,
    /// Sending the request failed.
    Request,
    /// Reading the answer's body failed.
    Body,
    /// The server redirected too often, or somewhere not allowed.
    Redirect,
    /// The client could not start (its HTTP stack, or the blocking client's
    /// runtime).
    Setup,
    /// Anything else the network stack reported.
    Other,
}

impl std::fmt::Display for TransportErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TransportErrorKind::Connect => "could not connect",
            TransportErrorKind::Request => "could not send the request",
            TransportErrorKind::Body => "could not read the answer",
            TransportErrorKind::Redirect => "redirected",
            TransportErrorKind::Setup => "could not start the client",
            TransportErrorKind::Other => "transport error",
        })
    }
}

impl ClientError {
    /// A [`ClientError::Transport`] of `kind` with `message`.
    #[cfg(any(feature = "http", feature = "ws"))]
    pub(crate) fn transport(kind: TransportErrorKind, message: impl std::fmt::Display) -> Self {
        ClientError::Transport {
            kind,
            message: message.to_string(),
        }
    }
}
