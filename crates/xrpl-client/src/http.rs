//! [`XrplHttpClient`]: JSON-RPC over HTTPS (rustls on native targets, the
//! browser's `fetch` on `wasm32`), with a 30-second timeout.

use crate::client::Client;
use crate::error::ClientError;
use serde_json::{json, Value};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

/// The largest response body read (128 MiB): a `ledger` with every
/// transaction expanded fits many times over; a server that sends more is
/// refused rather than allowed to fill memory.
pub const MAX_RESPONSE_BYTES: usize = 128 * 1024 * 1024;

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

/// HTTP JSON-RPC client for XRPL nodes.
pub struct XrplHttpClient {
    base_url: String,
    inner: reqwest::Client,
    max_fee_drops: u64,
}

impl XrplHttpClient {
    /// Create a new HTTP client connecting to the given XRPL node URL.
    pub fn new(url: &str) -> Result<Self, ClientError> {
        #[cfg(not(target_arch = "wasm32"))]
        let inner = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(concat!("halcyon-xrpl-client/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(http_error)?;

        #[cfg(target_arch = "wasm32")]
        let inner = reqwest::Client::builder().build().map_err(http_error)?;

        Ok(Self {
            base_url: url.trim_end_matches('/').to_string(),
            inner,
            max_fee_drops: crate::client::DEFAULT_MAX_FEE_DROPS,
        })
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
        tracing::debug!(method = method, "sending RPC request");

        let body = json!({
            "method": method,
            "params": [params]
        });

        let response = self
            .inner
            .post(&self.base_url)
            .json(&body)
            .send()
            .await
            .map_err(http_error)?;

        let status = response.status();
        let text = read_capped(response, MAX_RESPONSE_BYTES).await?;

        if !status.is_success() {
            // At most 200 characters of the body (a char boundary, never mid-UTF-8).
            let excerpt: String = text.chars().take(200).collect();
            return Err(ClientError::UnexpectedResponse(format!(
                "HTTP {status}: {excerpt}"
            )));
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
