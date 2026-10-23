//! [`XrplHttpClient`]: JSON-RPC over HTTPS (rustls on native targets, the
//! browser's `fetch` on `wasm32`), with a 30-second timeout. Set timeouts,
//! headers (an API key), the user agent, the fee cap and the
//! [`RetryPolicy`] with [`XrplHttpClient::builder`].

use crate::client::Client;
use crate::error::ClientError;
use serde_json::{json, Value};
use std::time::Duration;

/// When the client sends a request again: only when the server says it is
/// busy and did not take the request (HTTP 429 or 503, or `rippled`'s
/// `slowDown` or `tooBusy`), so a retry can never send a transaction twice.
/// A timeout or a dropped connection is never retried here: whether the
/// request arrived is unknown (`submit_and_wait` handles that case for
/// transactions by looking the transaction up).
///
/// The wait is the server's `Retry-After` when it gives one, else
/// `base_delay` doubled on each retry, never more than `max_delay`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Retries after the first attempt (0: never retry).
    pub max_retries: u32,
    /// The wait before the first retry when the server names none.
    pub base_delay: Duration,
    /// The longest wait, whatever the server asks.
    pub max_delay: Duration,
}

impl RetryPolicy {
    /// Never retry.
    pub const NONE: Self = Self {
        max_retries: 0,
        base_delay: Duration::ZERO,
        max_delay: Duration::ZERO,
    };

    /// The wait before retry number `attempt` (from 0), given the server's
    /// `Retry-After`.
    fn delay(&self, attempt: u32, retry_after: Option<Duration>) -> Duration {
        let backoff = self
            .base_delay
            .saturating_mul(2u32.saturating_pow(attempt.min(16)));
        retry_after.unwrap_or(backoff).min(self.max_delay)
    }
}

impl Default for RetryPolicy {
    /// Two retries, waiting 1 s then 2 s (or what `Retry-After` asks, up to
    /// 10 s).
    fn default() -> Self {
        Self {
            max_retries: 2,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
        }
    }
}

/// The settings of an [`XrplHttpClient`], from [`XrplHttpClient::builder`].
///
/// ```
/// use std::time::Duration;
/// use xrpl_client::http::{RetryPolicy, XrplHttpClient};
///
/// let client = XrplHttpClient::builder("https://xrplcluster.com")
///     .timeout(Duration::from_secs(10))
///     .header("x-api-key", "your-key")
///     .retry(RetryPolicy { max_retries: 5, ..RetryPolicy::default() })
///     .max_fee_drops(500_000)
///     .build()?;
/// # Ok::<(), xrpl_client::ClientError>(())
/// ```
#[derive(Clone)]
pub struct HttpClientBuilder {
    url: String,
    timeout: Duration,
    connect_timeout: Duration,
    headers: Vec<(String, String)>,
    user_agent: String,
    max_fee_drops: u64,
    retry: RetryPolicy,
}

impl std::fmt::Debug for HttpClientBuilder {
    /// The host and header names only: a URL or a header may carry a key.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpClientBuilder")
            .field("host", &crate::host_of(&self.url))
            .field("timeout", &self.timeout)
            .field("connect_timeout", &self.connect_timeout)
            .field(
                "headers",
                &self
                    .headers
                    .iter()
                    .map(|(name, _)| name)
                    .collect::<Vec<_>>(),
            )
            .field("user_agent", &self.user_agent)
            .field("max_fee_drops", &self.max_fee_drops)
            .field("retry", &self.retry)
            .finish()
    }
}

impl HttpClientBuilder {
    /// The longest a request may take, connection included (default 30 s).
    /// Not applied on `wasm32`, where the browser's `fetch` decides.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// The longest connecting may take (default 10 s). Not applied on
    /// `wasm32`.
    #[must_use]
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// A header sent with every request, for example a provider's API key.
    /// `Debug` shows its name, never its value.
    #[must_use]
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// The `User-Agent` (default `halcyon-xrpl-client/<version>`).
    #[must_use]
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// The most [`Client::autofill`] sets as a `Fee` (default
    /// [`crate::client::DEFAULT_MAX_FEE_DROPS`], 2 XRP).
    #[must_use]
    pub fn max_fee_drops(mut self, drops: u64) -> Self {
        self.max_fee_drops = drops;
        self
    }

    /// When a request is sent again (default [`RetryPolicy::default`];
    /// [`RetryPolicy::NONE`] never).
    #[must_use]
    pub fn retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    /// The client.
    ///
    /// # Errors
    ///
    /// [`ClientError::UnexpectedResponse`] for a header name or value HTTP
    /// does not allow; [`ClientError::Http`] if the HTTP stack cannot start.
    pub fn build(self) -> Result<XrplHttpClient, ClientError> {
        let mut headers = reqwest::header::HeaderMap::new();
        for (name, value) in &self.headers {
            let bad =
                || ClientError::UnexpectedResponse(format!("header {name:?} is not valid HTTP"));
            let header_name =
                reqwest::header::HeaderName::from_bytes(name.as_bytes()).map_err(|_| bad())?;
            let mut header_value =
                reqwest::header::HeaderValue::from_str(value).map_err(|_| bad())?;
            header_value.set_sensitive(true);
            headers.insert(header_name, header_value);
        }
        let builder = reqwest::Client::builder().default_headers(headers);
        #[cfg(not(target_arch = "wasm32"))]
        let builder = builder
            .timeout(self.timeout)
            .connect_timeout(self.connect_timeout)
            .user_agent(self.user_agent.clone());
        let inner = builder.build().map_err(http_error)?;
        Ok(XrplHttpClient {
            base_url: self.url.trim_end_matches('/').to_string(),
            inner,
            max_fee_drops: self.max_fee_drops,
            retry: self.retry,
        })
    }
}

/// The largest response body read (128 MiB): a `ledger` with every
/// transaction expanded fits many times over; a server that sends more is
/// refused rather than allowed to fill memory.
pub const MAX_RESPONSE_BYTES: usize = 128 * 1024 * 1024;

/// A `Retry-After` in seconds (the HTTP-date form is not read).
pub(crate) fn retry_after(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    headers
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
        .map(Duration::from_secs)
}

/// A transport error without the request URL, which may carry an API key.
pub(crate) fn http_error(e: reqwest::Error) -> ClientError {
    ClientError::Http(e.without_url())
}

/// The body, as text, refused past `max` bytes.
pub(crate) async fn read_capped(
    response: reqwest::Response,
    max: usize,
) -> Result<String, ClientError> {
    let too_big = || ClientError::UnexpectedResponse(format!("a response over {max} bytes"));
    if response
        .content_length()
        .is_some_and(|n| usize::try_from(n).map_or(true, |n| n > max))
    {
        return Err(too_big());
    }
    #[cfg(not(target_arch = "wasm32"))]
    let body = {
        let mut response = response;
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(http_error)? {
            if body.len() + chunk.len() > max {
                return Err(too_big());
            }
            body.extend_from_slice(&chunk);
        }
        body
    };
    // The browser's fetch has already read the body; refuse it all the same.
    #[cfg(target_arch = "wasm32")]
    let body = {
        let body = response.bytes().await.map_err(http_error)?.to_vec();
        if body.len() > max {
            return Err(too_big());
        }
        body
    };
    String::from_utf8(body)
        .map_err(|_| ClientError::UnexpectedResponse("a response that is not UTF-8".into()))
}

/// HTTP JSON-RPC client for XRPL nodes. Cloning is cheap: clones share one
/// connection pool.
#[derive(Clone)]
pub struct XrplHttpClient {
    base_url: String,
    inner: reqwest::Client,
    max_fee_drops: u64,
    retry: RetryPolicy,
}

impl std::fmt::Debug for XrplHttpClient {
    /// The server's host (never the URL, which may carry a token) and the
    /// fee cap.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XrplHttpClient")
            .field("host", &crate::host_of(&self.base_url))
            .field("max_fee_drops", &self.max_fee_drops)
            .field("retry", &self.retry)
            .finish()
    }
}

impl XrplHttpClient {
    /// Create a new HTTP client connecting to the given XRPL node URL, with
    /// the default settings (see [`XrplHttpClient::builder`]).
    pub fn new(url: &str) -> Result<Self, ClientError> {
        Self::builder(url).build()
    }

    /// Settings for a client of the server at `url`: timeouts, headers, the
    /// user agent, the fee cap and the [`RetryPolicy`].
    pub fn builder(url: &str) -> HttpClientBuilder {
        HttpClientBuilder {
            url: url.to_string(),
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            headers: Vec::new(),
            user_agent: concat!("halcyon-xrpl-client/", env!("CARGO_PKG_VERSION")).to_string(),
            max_fee_drops: crate::client::DEFAULT_MAX_FEE_DROPS,
            retry: RetryPolicy::default(),
        }
    }

    /// The same client with `drops` as the most [`Client::autofill`] sets as
    /// a `Fee` (default [`crate::client::DEFAULT_MAX_FEE_DROPS`], 2 XRP).
    pub fn with_max_fee_drops(mut self, drops: u64) -> Self {
        self.max_fee_drops = drops;
        self
    }
}

impl Client for XrplHttpClient {
    fn max_fee_drops(&self) -> u64 {
        self.max_fee_drops
    }

    async fn raw_request(&self, method: &str, params: Value) -> Result<Value, ClientError> {
        let body = json!({
            "method": method,
            "params": [params]
        });
        let mut attempt = 0;
        loop {
            let outcome = self.send_once(method, &body).await;
            let busy = match &outcome {
                Err(ClientError::HttpStatus {
                    status: 429 | 503,
                    retry_after,
                    ..
                }) => Some(*retry_after),
                Err(ClientError::RpcError { code, .. })
                    if code == "slowDown" || code == "tooBusy" =>
                {
                    Some(None)
                }
                _ => None,
            };
            match busy {
                Some(retry_after) if attempt < self.retry.max_retries => {
                    let wait = self.retry.delay(attempt, retry_after);
                    tracing::warn!(method, ?wait, attempt, "server busy; retrying");
                    crate::client::sleep_for(wait).await;
                    attempt += 1;
                }
                _ => return outcome,
            }
        }
    }
}

impl XrplHttpClient {
    /// One attempt at `body` (a JSON-RPC request for `method`).
    async fn send_once(&self, method: &str, body: &Value) -> Result<Value, ClientError> {
        tracing::debug!(method = method, "sending RPC request");

        let response = self
            .inner
            .post(&self.base_url)
            .json(body)
            .send()
            .await
            .map_err(http_error)?;

        let status = response.status();
        let retry_after = retry_after(response.headers());
        let text = read_capped(response, MAX_RESPONSE_BYTES).await?;

        if !status.is_success() {
            // At most 200 characters of the body (a char boundary, never mid-UTF-8).
            return Err(ClientError::HttpStatus {
                status: status.as_u16(),
                retry_after,
                excerpt: text.chars().take(200).collect(),
            });
        }

        let response_json: Value = serde_json::from_str(&text)?;

        let result = response_json
            .get("result")
            .ok_or_else(|| ClientError::UnexpectedResponse("missing 'result' field".into()))?;

        if result.get("status").and_then(Value::as_str) == Some("error") {
            return Err(crate::client::rpc_error(result));
        }

        tracing::debug!(method = method, "RPC request completed");
        Ok(result.clone())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    /// `{:?}` shows the server's host, never the rest of a URL that may
    /// carry credentials or an API key.
    #[test]
    fn debug_shows_the_host_only() {
        let client =
            XrplHttpClient::new("https://user:s3cret@node.example:51234/rpc?key=abc123").unwrap();
        let shown = format!("{client:?}");
        assert!(shown.contains("node.example:51234"), "{shown}");
        for secret in ["user", "s3cret", "rpc", "abc123"] {
            assert!(!shown.contains(secret), "{shown}");
        }
        let _copy: XrplHttpClient = client.clone();
    }

    use crate::types::*;

    #[test]
    fn mainnet_fee_deserializes() {
        // rippled 3.4.1 (.39) `fee`, 2026-10-22: the queue and ledger sizes are strings.
        let fee: FeeResult =
            serde_json::from_str(include_str!("../tests/fixtures/mainnet_fee.json")).unwrap();
        assert!(fee.current_ledger_size.is_some() && fee.max_queue_size.is_some());
        assert!(fee.ledger_current_index.unwrap() > 107_000_000);
        assert_eq!(fee.levels.unwrap().reference_level, "256");
    }

    #[test]
    fn mainnet_amounts_deserialize_typed() {
        // rippled 3.4.1 (.39) responses, 2026-10-22: XRP/RLUSD book, its AMM, a maker's offers.
        let book: BookOffersResult =
            serde_json::from_str(include_str!("../tests/fixtures/mainnet_book_offers.json"))
                .unwrap();
        assert!(book
            .offers
            .iter()
            .all(|o| o.taker_gets.is_xrp() && !o.taker_pays.is_xrp()));
        let amm: AmmInfoResult =
            serde_json::from_str(include_str!("../tests/fixtures/mainnet_amm_info.json")).unwrap();
        assert!(matches!(
            amm.amm.amount,
            Some(xrpl_core::types::Amount::Xrp(_))
        ));
        assert!(matches!(
            amm.amm.lp_token,
            Some(xrpl_core::types::Amount::Iou { .. })
        ));
        let offers: AccountOffersResult = serde_json::from_str(include_str!(
            "../tests/fixtures/mainnet_account_offers.json"
        ))
        .unwrap();
        assert!(!offers.offers.is_empty());
    }

    #[test]
    fn client_creation() {
        let client = XrplHttpClient::new("https://s.altnet.rippletest.net:51234");
        assert!(client.is_ok());
    }

    #[test]
    fn deserialize_account_tx_result() {
        let json = serde_json::json!({
            "account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "transactions": [
                {
                    "tx": {"TransactionType": "Payment", "Amount": "1000000"},
                    "meta": {"TransactionResult": "tesSUCCESS"},
                    "validated": true
                }
            ],
            "marker": null
        });
        let result: AccountTxResult = serde_json::from_value(json).unwrap();
        assert_eq!(result.account, "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
        assert_eq!(result.transactions.len(), 1);
        assert_eq!(result.transactions[0].validated, Some(true));
    }

    #[test]
    fn deserialize_account_objects_result() {
        let json = serde_json::json!({
            "account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "account_objects": [
                {"LedgerEntryType": "RippleState", "Balance": {"value": "100"}}
            ],
            "marker": null
        });
        let result: AccountObjectsResult = serde_json::from_value(json).unwrap();
        assert_eq!(result.account_objects.len(), 1);
    }

    #[test]
    fn deserialize_account_currencies_result() {
        let json = serde_json::json!({
            "receive_currencies": ["USD", "EUR"],
            "send_currencies": ["USD"],
            "ledger_index": 12345,
            "validated": true
        });
        let result: AccountCurrenciesResult = serde_json::from_value(json).unwrap();
        assert_eq!(result.receive_currencies.len(), 2);
        assert_eq!(result.send_currencies.len(), 1);
    }

    #[test]
    fn deserialize_account_channels_result() {
        let json = serde_json::json!({
            "account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "channels": [
                {
                    "channel_id": "ABCDEF0123456789",
                    "account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
                    "destination_account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
                    "amount": "1000000",
                    "balance": "500000",
                    "settle_delay": 3600
                }
            ]
        });
        let result: AccountChannelsResult = serde_json::from_value(json).unwrap();
        assert_eq!(result.channels.len(), 1);
        assert_eq!(result.channels[0].settle_delay, 3600);
    }

    #[test]
    fn deserialize_ledger_entry_result() {
        let json = serde_json::json!({
            "index": "ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789",
            "node": {"LedgerEntryType": "AccountRoot"},
            "ledger_index": 100,
            "validated": true
        });
        let result: LedgerEntryResult = serde_json::from_value(json).unwrap();
        assert!(result.validated.unwrap());
    }

    #[test]
    fn deserialize_server_info_result() {
        let json = serde_json::json!({
            "info": {
                "build_version": "1.12.0",
                "complete_ledgers": "32570-75000000",
                "server_state": "full",
                "uptime": 123456,
                "validated_ledger": {
                    "age": 2,
                    "base_fee_xrp": 0.00001,
                    "hash": "ABCDEF",
                    "reserve_base_xrp": 10.0,
                    "reserve_inc_xrp": 2.0,
                    "seq": 75000000
                }
            }
        });
        let result: ServerInfoResult = serde_json::from_value(json).unwrap();
        assert_eq!(result.info.build_version, Some("1.12.0".to_string()));
        let vl = result.info.validated_ledger.unwrap();
        assert_eq!(vl.seq, Some(75000000));
    }

    #[test]
    fn deserialize_gateway_balances_result() {
        let json = serde_json::json!({
            "account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "obligations": {"USD": "100.50"},
            "balances": {
                "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe": [
                    {"currency": "USD", "value": "50.25"}
                ]
            }
        });
        let result: GatewayBalancesResult = serde_json::from_value(json).unwrap();
        assert_eq!(result.obligations.unwrap()["USD"], "100.50");
    }
}

// Live testnet tests — only run with `cargo test --features live-tests`
#[cfg(all(test, not(target_arch = "wasm32")))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod transport_tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// A one-request HTTP server answering with `body` (and a
    /// Content-Length only when `declare` is set).
    async fn serve(body: Vec<u8>, declare: bool) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!(
            "http://user:secret-token@{}/rpc",
            listener.local_addr().unwrap()
        );
        tokio::spawn(async move {
            let (mut tcp, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 4096];
            let _ = tcp.read(&mut buf).await;
            let head = if declare {
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                )
            } else {
                "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n".to_string()
            };
            let _ = tcp.write_all(head.as_bytes()).await;
            let _ = tcp.write_all(&body).await;
        });
        url
    }

    #[tokio::test]
    async fn a_response_past_the_cap_is_refused() {
        for declare in [true, false] {
            let url = serve(vec![b'x'; 5000], declare).await;
            let response = reqwest::Client::new().get(&url).send().await.unwrap();
            assert!(
                read_capped(response, 1000).await.is_err(),
                "declared: {declare}"
            );
        }
        let url = serve(b"{\"result\":{}}".to_vec(), true).await;
        let response = reqwest::Client::new().get(&url).send().await.unwrap();
        assert_eq!(
            read_capped(response, 1000).await.unwrap(),
            "{\"result\":{}}"
        );
    }

    /// Transport errors do not carry the URL (it can hold an API key).
    #[tokio::test]
    async fn errors_leave_the_url_out() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener); // nothing listens there now
        let client = XrplHttpClient::new(&format!(
            "http://user:secret-token@{addr}/?key=secret-token"
        ))
        .unwrap();
        let error = client.fee().await.unwrap_err().to_string();
        assert!(!error.contains("secret-token"), "{error}");
    }
}

#[cfg(all(test, feature = "live-tests"))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod live_tests {
    use super::*;
    use crate::types::*;

    const TESTNET_URL: &str = "https://s.altnet.rippletest.net:51234";

    #[tokio::test]
    async fn fee_testnet() {
        let client = XrplHttpClient::new(TESTNET_URL).unwrap();
        let fee = client.fee().await.unwrap();
        let base_fee: u64 = fee.drops.base_fee.parse().unwrap();
        assert!(base_fee >= 10);
    }

    #[tokio::test]
    async fn ledger_current_testnet() {
        let client = XrplHttpClient::new(TESTNET_URL).unwrap();
        let idx = client.ledger_current().await.unwrap();
        assert!(idx > 0);
    }

    #[tokio::test]
    async fn account_info_testnet() {
        let client = XrplHttpClient::new(TESTNET_URL).unwrap();
        // Use the testnet faucet's known account
        let result = client
            .account_info(
                "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
                &LedgerIndex::Validated,
            )
            .await;
        // Account may or may not exist on testnet, but the request should succeed
        assert!(result.is_ok() || matches!(result, Err(ClientError::RpcError { .. })));
    }
}

/// The retry policy and headers, against a local HTTP server.
#[cfg(all(test, not(target_arch = "wasm32")))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod retry_tests {
    use super::*;

    /// A local HTTP server that answers each connection with the next of
    /// `replies` (status line, extra headers, body) and records the
    /// requests it read.
    async fn server(
        replies: Vec<(&'static str, &'static str, &'static str)>,
    ) -> (String, std::sync::Arc<std::sync::Mutex<Vec<String>>>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let log = seen.clone();
        tokio::spawn(async move {
            for (status, headers, body) in replies {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                let mut chunk = [0u8; 4096];
                loop {
                    let n = socket.read(&mut chunk).await.unwrap();
                    request.extend_from_slice(&chunk[..n]);
                    let text = String::from_utf8_lossy(&request).to_string();
                    if let Some(end) = text.find("\r\n\r\n") {
                        let length = text
                            .lines()
                            .find_map(|l| {
                                l.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .map(|v| v.trim().parse::<usize>().unwrap())
                            })
                            .unwrap_or(0);
                        if request.len() >= end + 4 + length || n == 0 {
                            break;
                        }
                    }
                }
                log.lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&request).to_string());
                let reply = format!(
                    "HTTP/1.1 {status}\r\n{headers}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                socket.write_all(reply.as_bytes()).await.unwrap();
                socket.shutdown().await.unwrap();
            }
        });
        (url, seen)
    }

    const OK: &str = r#"{"result":{"status":"success","ledger_current_index":7}}"#;
    const SLOW_DOWN: &str = r#"{"result":{"status":"error","error":"slowDown","error_message":"You are placing too much load on the server."}}"#;
    const QUICK: RetryPolicy = RetryPolicy {
        max_retries: 2,
        base_delay: Duration::ZERO,
        max_delay: Duration::ZERO,
    };

    /// A busy server (429, then rippled's slowDown) is asked again; the
    /// third answer is the result.
    #[tokio::test]
    async fn a_busy_server_is_asked_again() {
        let (url, seen) = server(vec![
            ("429 Too Many Requests", "Retry-After: 0\r\n", "busy"),
            ("200 OK", "", SLOW_DOWN),
            ("200 OK", "", OK),
        ])
        .await;
        let client = XrplHttpClient::builder(&url).retry(QUICK).build().unwrap();
        assert_eq!(client.ledger_current().await.unwrap(), 7);
        assert_eq!(seen.lock().unwrap().len(), 3);
    }

    /// After its retries the busy answer is the error, typed.
    #[tokio::test]
    async fn retries_end() {
        let (url, seen) = server(vec![
            ("503 Service Unavailable", "Retry-After: 0\r\n", "down"),
            ("503 Service Unavailable", "Retry-After: 0\r\n", "down"),
        ])
        .await;
        let retry = RetryPolicy {
            max_retries: 1,
            ..QUICK
        };
        let client = XrplHttpClient::builder(&url).retry(retry).build().unwrap();
        let err = client.ledger_current().await.unwrap_err();
        assert!(
            matches!(&err, ClientError::HttpStatus { status: 503, retry_after: Some(d), excerpt } if d.is_zero() && excerpt == "down"),
            "{err:?}"
        );
        assert_eq!(seen.lock().unwrap().len(), 2);
    }

    /// Any other failure is answered at once: it may not be safe to repeat.
    #[tokio::test]
    async fn other_errors_are_not_retried() {
        let (url, seen) = server(vec![("500 Internal Server Error", "", "oops")]).await;
        let client = XrplHttpClient::builder(&url).retry(QUICK).build().unwrap();
        assert!(matches!(
            client.ledger_current().await,
            Err(ClientError::HttpStatus { status: 500, .. })
        ));
        assert_eq!(seen.lock().unwrap().len(), 1);
    }

    /// A header is sent with every request, and never shown by `{:?}`.
    #[tokio::test]
    async fn headers_are_sent_not_shown() {
        let (url, seen) = server(vec![("200 OK", "", OK)]).await;
        let builder = XrplHttpClient::builder(&url)
            .header("x-api-key", "k3y-v4lue")
            .user_agent("tests/1");
        assert!(!format!("{builder:?}").contains("k3y-v4lue"));
        builder.build().unwrap().ledger_current().await.unwrap();
        let request = seen.lock().unwrap()[0].to_ascii_lowercase();
        assert!(request.contains("x-api-key: k3y-v4lue"), "{request}");
        assert!(request.contains("user-agent: tests/1"), "{request}");
    }

    #[test]
    fn delays_double_and_are_capped() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.delay(0, None), Duration::from_secs(1));
        assert_eq!(policy.delay(1, None), Duration::from_secs(2));
        assert_eq!(policy.delay(9, None), Duration::from_secs(10));
        assert_eq!(
            policy.delay(0, Some(Duration::from_secs(60))),
            Duration::from_secs(10)
        );
        assert_eq!(
            policy.delay(0, Some(Duration::from_secs(3))),
            Duration::from_secs(3)
        );
    }
}
