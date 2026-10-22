//! [`ClientError`]: transport and JSON failures, `rippled` errors, and the
//! sorted outcome of a submitted transaction.

/// Everything a client call can fail with.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// The HTTP request failed.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// A response was not the JSON expected.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// `rippled` answered with an error: `code` is its token (e.g. `actNotFound`).
    #[error("RPC error: {code} — {message}")]
    RpcError {
        /// rippled's error token (`actNotFound`, `invalidParams`, …).
        code: String,
        /// Its message.
        message: String,
    },

    /// Refused (`tem`, `tef`, most `tel`): it never reaches a ledger.
    #[error("transaction failed: {engine_result} — {message}")]
    TransactionFailed {
        /// The engine result.
        engine_result: String,
        /// The engine result in words.
        message: String,
    },

    /// May succeed if submitted again (`ter`, `telINSUF_FEE_P`, `telCAN_NOT_QUEUE…`).
    #[error("transaction retry: {engine_result}")]
    TransactionRetry {
        /// The engine result.
        engine_result: String,
    },

    /// Failed, but the fee was charged (`tec`).
    #[error("transaction fee claimed: {engine_result}")]
    TransactionClaimed {
        /// The engine result.
        engine_result: String,
    },

    /// The WebSocket connection is closed.
    #[error("connection closed unexpectedly")]
    Disconnected,

    /// No answer in time.
    #[error("request timeout after {0:?}")]
    Timeout(std::time::Duration),

    /// A response, or an input, was not as expected.
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    /// Its `LastLedgerSequence` passed without it being validated.
    #[error("transaction expired: LastLedgerSequence {0} passed")]
    TransactionExpired(u32),

    /// `request_all` stopped after this many pages; page on with `marker`.
    #[error("pagination limit reached: {0} pages fetched, use marker manually for more")]
    PaginationLimitReached(usize),

    /// A core error (codec, keys, amounts, …).
    #[error("core error: {0}")]
    Core(#[from] xrpl_core::CoreError),
}
