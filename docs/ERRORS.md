# Error Handling

## Philosophy
- All public functions return `Result<T, E>` — no panics in library code
- Each crate defines its own error enum
- `xrpl-sdk` wraps all lower errors into `XrplSdkError`
- Use `thiserror` for all error types

---

## `xrpl-core` Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("invalid base58: {0}")]
    InvalidBase58(String),

    #[error("invalid checksum: expected {expected:?}, got {got:?}")]
    InvalidChecksum { expected: [u8; 4], got: [u8; 4] },

    #[error("invalid account address: {0}")]
    InvalidAddress(String),

    #[error("invalid seed: {0}")]
    InvalidSeed(String),

    #[error("codec error: {0}")]
    CodecError(String),

    #[error("invalid amount: {0}")]
    InvalidAmount(String),

    #[error("unknown field: {0}")]
    UnknownField(String),

    #[error("key derivation failed")]
    KeyDerivationFailed,

    #[error("signing failed: {0}")]
    SigningFailed(String),

    #[error("invalid public key length: expected {expected}, got {got}")]
    InvalidKeyLength { expected: usize, got: usize },
}
```

---

## `xrpl-client` Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("RPC error: {code} — {message}")]
    RpcError { code: String, message: String },

    #[error("transaction failed: {engine_result} — {message}")]
    TransactionFailed { engine_result: String, message: String },

    #[error("transaction retry: {engine_result}")]
    TransactionRetry { engine_result: String },

    #[error("transaction fee claimed: {engine_result}")]
    TransactionClaimed { engine_result: String },

    #[error("connection closed unexpectedly")]
    Disconnected,

    #[error("request timeout after {0:?}")]
    Timeout(std::time::Duration),

    #[error("rate limited — retry after {0:?}")]
    RateLimited(Option<std::time::Duration>),

    #[error("unexpected response shape: {0}")]
    UnexpectedResponse(String),
}
```

---

## `xrpl-sdk` Top-Level Error

```rust
#[derive(Debug, thiserror::Error)]
pub enum XrplSdkError {
    #[error("core: {0}")]
    Core(#[from] xrpl_core::CoreError),

    #[error("client: {0}")]
    Client(#[from] xrpl_client::ClientError),

    #[error("wallet error: {0}")]
    Wallet(String),

    #[error("autofill failed: {0}")]
    AutofillFailed(String),

    #[error("build error: {0}")]
    Build(String),
}
```

---

## Engine Result Classification

When `submit` returns, classify the `engine_result` string:

| Pattern | Maps to |
|---|---|
| `tesSUCCESS` | `Ok(result)` |
| starts with `tem` | `TransactionFailed` — malformed, don't retry |
| starts with `tef` | `TransactionFailed` — failed, don't retry |
| starts with `ter` | `TransactionRetry` — transient, may retry |
| starts with `tec` | `TransactionClaimed` — fee charged, nothing applied |

---

## Retry Guidance for Callers

| Error | Action |
|---|---|
| `TransactionRetry(terQUEUED)` | Poll `tx` until validated or expired |
| `TransactionRetry(terNO_ACCOUNT)` | Account not funded; stop |
| `RateLimited` | Back off, retry after indicated duration |
| `Disconnected` | Client handles reconnect internally; retry request |
| `Timeout` | Retry once; if fails again, escalate |
| `TransactionFailed` | Do NOT retry; fix transaction |
| `TransactionClaimed(tecPATH_DRY)` | Paths exhausted; rebuild tx |

---

## Never Panic Contract
- No `unwrap()` or `expect()` outside of `#[cfg(test)]` and `examples/`
- Array indexing only after explicit bounds check
- All `HashMap::get` results handled
- All `from_utf8` and parse results propagated as errors
