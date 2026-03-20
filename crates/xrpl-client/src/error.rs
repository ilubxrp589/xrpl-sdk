#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("RPC error: {code} — {message}")]
    RpcError { code: String, message: String },

    #[error("transaction failed: {engine_result} — {message}")]
    TransactionFailed {
        engine_result: String,
        message: String,
    },

    #[error("transaction retry: {engine_result}")]
    TransactionRetry { engine_result: String },

    #[error("transaction fee claimed: {engine_result}")]
    TransactionClaimed { engine_result: String },

    #[error("connection closed unexpectedly")]
    Disconnected,

    #[error("request timeout after {0:?}")]
    Timeout(std::time::Duration),

    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("transaction expired: LastLedgerSequence {0} passed")]
    TransactionExpired(u32),

    #[error("pagination limit reached: {0} pages fetched, use marker manually for more")]
    PaginationLimitReached(usize),

    #[error("core error: {0}")]
    Core(#[from] xrpl_core::CoreError),
}
