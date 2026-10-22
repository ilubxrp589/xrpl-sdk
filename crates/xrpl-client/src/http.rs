//! [`XrplHttpClient`]: JSON-RPC over HTTPS (rustls on native targets, the
//! browser's `fetch` on `wasm32`), with a 30-second timeout.

use crate::client::Client;
use crate::error::ClientError;
use serde_json::{json, Value};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

/// HTTP JSON-RPC client for XRPL nodes.
pub struct XrplHttpClient {
    base_url: String,
    inner: reqwest::Client,
}

impl XrplHttpClient {
    /// Create a new HTTP client connecting to the given XRPL node URL.
    pub fn new(url: &str) -> Result<Self, ClientError> {
        #[cfg(not(target_arch = "wasm32"))]
        let inner = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(concat!("halcyon-xrpl-client/", env!("CARGO_PKG_VERSION")))
            .build()?;

        #[cfg(target_arch = "wasm32")]
        let inner = reqwest::Client::builder().build()?;

        Ok(Self {
            base_url: url.trim_end_matches('/').to_string(),
            inner,
        })
    }
}

impl Client for XrplHttpClient {
    async fn raw_request(&self, method: &str, params: Value) -> Result<Value, ClientError> {
        tracing::debug!(method = method, url = %self.base_url, "sending RPC request");

        let body = json!({
            "method": method,
            "params": [params]
        });

        let response = self.inner.post(&self.base_url).json(&body).send().await?;

        let status = response.status();
        let text = response.text().await?;

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

        // Check for RPC error
        if let Some(status_str) = result.get("status").and_then(|s| s.as_str()) {
            if status_str == "error" {
                let error = result
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let message = result
                    .get("error_message")
                    .and_then(|e| e.as_str())
                    .unwrap_or("")
                    .to_string();
                return Err(ClientError::RpcError {
                    code: error,
                    message,
                });
            }
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
