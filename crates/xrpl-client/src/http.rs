use crate::error::ClientError;
use crate::types::*;
use serde_json::{json, Value};
use std::time::Duration;

/// Platform-aware async sleep.
#[cfg(not(target_arch = "wasm32"))]
async fn platform_sleep_secs(secs: u64) {
    tokio::time::sleep(Duration::from_secs(secs)).await;
}

#[cfg(target_arch = "wasm32")]
async fn platform_sleep_secs(secs: u64) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                (secs * 1000) as i32,
            );
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

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
            .user_agent("xrpl-rust-sdk/0.1")
            .build()?;

        #[cfg(target_arch = "wasm32")]
        let inner = reqwest::Client::builder().build()?;

        Ok(Self {
            base_url: url.trim_end_matches('/').to_string(),
            inner,
        })
    }

    /// Send a generic JSON-RPC request.
    async fn request(&self, method: &str, params: Value) -> Result<Value, ClientError> {
        tracing::debug!(method = method, url = %self.base_url, "sending RPC request");

        let body = json!({
            "method": method,
            "params": [params]
        });

        let response = self.inner.post(&self.base_url).json(&body).send().await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(ClientError::UnexpectedResponse(format!(
                "HTTP {}: {}",
                status,
                &text[..200.min(text.len())]
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

    /// Get account information.
    pub async fn account_info(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountInfo, ClientError> {
        let result = self
            .request(
                "account_info",
                json!({
                    "account": account,
                    "ledger_index": ledger.as_value()
                }),
            )
            .await?;

        Ok(serde_json::from_value(result)?)
    }

    /// Get current fee information.
    pub async fn fee(&self) -> Result<FeeResult, ClientError> {
        let result = self.request("fee", json!({})).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get current ledger index.
    pub async fn ledger_current(&self) -> Result<u32, ClientError> {
        let result = self.request("ledger_current", json!({})).await?;
        let lr: LedgerCurrentResult = serde_json::from_value(result)?;
        Ok(lr.ledger_current_index)
    }

    /// Get ledger information.
    pub async fn ledger(&self, ledger: &LedgerIndex) -> Result<LedgerResult, ClientError> {
        let result = self
            .request(
                "ledger",
                json!({
                    "ledger_index": ledger.as_value()
                }),
            )
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Submit a signed transaction blob (hex string).
    pub async fn submit(&self, tx_blob: &str) -> Result<SubmitResult, ClientError> {
        let result = self
            .request("submit", json!({ "tx_blob": tx_blob }))
            .await?;

        let sr: SubmitResult = serde_json::from_value(result)?;

        // Classify engine result
        if sr.engine_result == "tesSUCCESS" || sr.engine_result.starts_with("tes") {
            return Ok(sr);
        }

        if sr.engine_result.starts_with("tem") || sr.engine_result.starts_with("tef") {
            return Err(ClientError::TransactionFailed {
                engine_result: sr.engine_result,
                message: sr.engine_result_message,
            });
        }

        if sr.engine_result.starts_with("ter") {
            return Err(ClientError::TransactionRetry {
                engine_result: sr.engine_result,
            });
        }

        if sr.engine_result.starts_with("tec") {
            return Err(ClientError::TransactionClaimed {
                engine_result: sr.engine_result,
            });
        }

        Ok(sr)
    }

    /// Look up a transaction by hash.
    pub async fn tx(&self, hash: &str) -> Result<TxResult, ClientError> {
        let result = self
            .request(
                "tx",
                json!({
                    "transaction": hash,
                    "binary": false
                }),
            )
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get trust lines for an account.
    pub async fn account_lines(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountLinesResult, ClientError> {
        let result = self
            .request(
                "account_lines",
                json!({
                    "account": account,
                    "ledger_index": ledger.as_value(),
                    "limit": 400
                }),
            )
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get open offers for an account.
    pub async fn account_offers(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountOffersResult, ClientError> {
        let result = self
            .request(
                "account_offers",
                json!({
                    "account": account,
                    "ledger_index": ledger.as_value(),
                    "limit": 400
                }),
            )
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get NFTs owned by an account.
    pub async fn account_nfts(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountNftsResult, ClientError> {
        let result = self
            .request(
                "account_nfts",
                json!({
                    "account": account,
                    "ledger_index": ledger.as_value(),
                    "limit": 400
                }),
            )
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get order book offers.
    pub async fn book_offers(
        &self,
        taker_pays: &Value,
        taker_gets: &Value,
    ) -> Result<BookOffersResult, ClientError> {
        let result = self
            .request(
                "book_offers",
                json!({
                    "taker_pays": taker_pays,
                    "taker_gets": taker_gets,
                    "limit": 20
                }),
            )
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get AMM pool information.
    pub async fn amm_info(
        &self,
        asset: &Value,
        asset2: &Value,
    ) -> Result<AmmInfoResult, ClientError> {
        let result = self
            .request(
                "amm_info",
                json!({
                    "asset": asset,
                    "asset2": asset2
                }),
            )
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get transaction history for an account.
    pub async fn account_tx(
        &self,
        account: &str,
        ledger_index_min: Option<i32>,
        ledger_index_max: Option<i32>,
        limit: Option<u32>,
        marker: Option<Value>,
        forward: Option<bool>,
    ) -> Result<AccountTxResult, ClientError> {
        let result = self
            .request(
                "account_tx",
                json!({
                    "account": account,
                    "ledger_index_min": ledger_index_min.unwrap_or(-1),
                    "ledger_index_max": ledger_index_max.unwrap_or(-1),
                    "limit": limit.unwrap_or(200),
                    "marker": marker,
                    "forward": forward.unwrap_or(false)
                }),
            )
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get objects owned by an account.
    pub async fn account_objects(
        &self,
        account: &str,
        object_type: Option<&str>,
        ledger: &LedgerIndex,
        limit: Option<u32>,
        marker: Option<Value>,
    ) -> Result<AccountObjectsResult, ClientError> {
        let mut params = json!({
            "account": account,
            "ledger_index": ledger.as_value(),
            "limit": limit.unwrap_or(200)
        });
        if let Some(obj) = params.as_object_mut() {
            if let Some(t) = object_type {
                obj.insert("type".to_string(), Value::String(t.to_string()));
            }
            if let Some(m) = marker {
                obj.insert("marker".to_string(), m);
            }
        }
        let result = self.request("account_objects", params).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get currencies an account can send or receive.
    pub async fn account_currencies(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountCurrenciesResult, ClientError> {
        let result = self
            .request(
                "account_currencies",
                json!({
                    "account": account,
                    "ledger_index": ledger.as_value()
                }),
            )
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get payment channels for an account.
    pub async fn account_channels(
        &self,
        account: &str,
        destination_account: Option<&str>,
        ledger: &LedgerIndex,
    ) -> Result<AccountChannelsResult, ClientError> {
        let mut params = json!({
            "account": account,
            "ledger_index": ledger.as_value()
        });
        if let (Some(dest), Some(obj)) = (destination_account, params.as_object_mut()) {
            obj.insert(
                "destination_account".to_string(),
                Value::String(dest.to_string()),
            );
        }
        let result = self.request("account_channels", params).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get a single ledger entry by index.
    pub async fn ledger_entry(
        &self,
        index: &str,
        ledger: &LedgerIndex,
    ) -> Result<LedgerEntryResult, ClientError> {
        let result = self
            .request(
                "ledger_entry",
                json!({
                    "index": index,
                    "ledger_index": ledger.as_value()
                }),
            )
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get server information.
    pub async fn server_info(&self) -> Result<ServerInfoResult, ClientError> {
        let result = self.request("server_info", json!({})).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get gateway balances for an issuing account.
    pub async fn gateway_balances(
        &self,
        account: &str,
        hotwallet: Option<Vec<&str>>,
        ledger: &LedgerIndex,
    ) -> Result<GatewayBalancesResult, ClientError> {
        let mut params = json!({
            "account": account,
            "ledger_index": ledger.as_value()
        });
        if let (Some(hw), Some(obj)) = (hotwallet, params.as_object_mut()) {
            obj.insert(
                "hotwallet".to_string(),
                Value::Array(
                    hw.into_iter()
                        .map(|s| Value::String(s.to_string()))
                        .collect(),
                ),
            );
        }
        let result = self.request("gateway_balances", params).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Autofill missing transaction fields (Sequence, Fee, LastLedgerSequence).
    /// Only fills fields that are missing or null. Fetches concurrently via tokio::try_join!.
    pub async fn autofill(&self, tx: &mut Value) -> Result<(), ClientError> {
        let account = tx["Account"]
            .as_str()
            .ok_or_else(|| ClientError::UnexpectedResponse("tx missing Account field".into()))?
            .to_string();

        let need_seq = tx.get("Sequence").is_none() || tx["Sequence"].is_null();
        let need_fee = tx.get("Fee").is_none() || tx["Fee"].is_null();
        let need_lls = tx.get("LastLedgerSequence").is_none() || tx["LastLedgerSequence"].is_null();

        if !need_seq && !need_fee && !need_lls {
            return Ok(());
        }

        let seq_fut = async {
            if need_seq {
                self.account_info(&account, &LedgerIndex::Current)
                    .await
                    .map(Some)
            } else {
                Ok(None)
            }
        };
        let fee_fut = async {
            if need_fee {
                self.fee().await.map(Some)
            } else {
                Ok(None)
            }
        };
        let lls_fut = async {
            if need_lls {
                self.ledger_current().await.map(Some)
            } else {
                Ok(None)
            }
        };
        let (acct_info, fee_result, ledger_idx) =
            futures::future::try_join3(seq_fut, fee_fut, lls_fut).await?;

        if let Some(info) = acct_info {
            tx["Sequence"] = json!(info.account_data.sequence);
        }
        if let Some(fee) = fee_result {
            tx["Fee"] = json!(fee.drops.open_ledger_fee);
        }
        if let Some(idx) = ledger_idx {
            tx["LastLedgerSequence"] = json!(idx + 4);
        }

        Ok(())
    }

    /// Submit a signed blob and wait until the transaction is validated or expires.
    pub async fn submit_and_wait(
        &self,
        tx_blob: &str,
        last_ledger_sequence: u32,
    ) -> Result<TxResult, ClientError> {
        // Submit with retry for ter* results
        let mut submit_result = None;
        for attempt in 0..3 {
            match self.submit(tx_blob).await {
                Ok(sr) => {
                    submit_result = Some(sr);
                    break;
                }
                Err(ClientError::TransactionRetry { .. }) if attempt < 2 => {
                    platform_sleep_secs(4).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        let sr = submit_result.ok_or_else(|| ClientError::TransactionFailed {
            engine_result: "ter_retry_exhausted".into(),
            message: "ter result after 3 resubmit attempts".into(),
        })?;

        let hash = sr
            .tx_json
            .as_ref()
            .and_then(|j| j.get("hash").or_else(|| j.get("Hash")))
            .and_then(|h| h.as_str())
            .ok_or_else(|| {
                ClientError::UnexpectedResponse("submit response missing tx hash".into())
            })?
            .to_string();

        // Poll until validated
        for _ in 0..20 {
            let current = self.ledger_current().await?;
            if current > last_ledger_sequence {
                return Err(ClientError::TransactionExpired(last_ledger_sequence));
            }

            match self.tx(&hash).await {
                Ok(tx_result) => {
                    if tx_result.validated == Some(true) {
                        return Ok(tx_result);
                    }
                }
                Err(ClientError::RpcError { ref code, .. }) if code == "txnNotFound" => {
                    // Not in ledger yet, keep polling
                }
                Err(e) => return Err(e),
            }

            platform_sleep_secs(4).await;
        }

        Err(ClientError::Timeout(Duration::from_secs(80)))
    }

    /// Raw JSON-RPC request for any method.
    pub async fn raw_request(&self, method: &str, params: Value) -> Result<Value, ClientError> {
        self.request(method, params).await
    }

    // --- Pagination helpers ---

    /// Get all trust lines for an account, following marker chain.
    pub async fn account_lines_all(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<Vec<TrustLine>, ClientError> {
        let mut all = Vec::new();
        let mut marker: Option<Value> = None;
        for page in 0..50 {
            let result = self
                .request(
                    "account_lines",
                    json!({
                        "account": account,
                        "ledger_index": ledger.as_value(),
                        "limit": 400,
                        "marker": marker
                    }),
                )
                .await?;
            let page_result: AccountLinesResult = serde_json::from_value(result)?;
            all.extend(page_result.lines);
            marker = page_result.marker;
            if marker.is_none() {
                break;
            }
            if page == 49 {
                return Err(ClientError::PaginationLimitReached(50));
            }
        }
        Ok(all)
    }

    /// Get all offers for an account, following marker chain.
    pub async fn account_offers_all(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<Vec<AccountOffer>, ClientError> {
        let mut all = Vec::new();
        let mut marker: Option<Value> = None;
        for page in 0..50 {
            let result = self
                .request(
                    "account_offers",
                    json!({
                        "account": account,
                        "ledger_index": ledger.as_value(),
                        "limit": 400,
                        "marker": marker
                    }),
                )
                .await?;
            let page_result: AccountOffersResult = serde_json::from_value(result)?;
            all.extend(page_result.offers);
            marker = page_result.marker;
            if marker.is_none() {
                break;
            }
            if page == 49 {
                return Err(ClientError::PaginationLimitReached(50));
            }
        }
        Ok(all)
    }

    /// Get all NFTs for an account, following marker chain.
    pub async fn account_nfts_all(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<Vec<NFToken>, ClientError> {
        let mut all = Vec::new();
        let mut marker: Option<Value> = None;
        for page in 0..50 {
            let result = self
                .request(
                    "account_nfts",
                    json!({
                        "account": account,
                        "ledger_index": ledger.as_value(),
                        "limit": 400,
                        "marker": marker
                    }),
                )
                .await?;
            let page_result: AccountNftsResult = serde_json::from_value(result)?;
            all.extend(page_result.account_nfts);
            marker = page_result.marker;
            if marker.is_none() {
                break;
            }
            if page == 49 {
                return Err(ClientError::PaginationLimitReached(50));
            }
        }
        Ok(all)
    }

    /// Get all objects for an account, following marker chain.
    pub async fn account_objects_all(
        &self,
        account: &str,
        object_type: Option<&str>,
        ledger: &LedgerIndex,
    ) -> Result<Vec<Value>, ClientError> {
        let mut all = Vec::new();
        let mut marker: Option<Value> = None;
        for page in 0..50 {
            let mut params = json!({
                "account": account,
                "ledger_index": ledger.as_value(),
                "limit": 200,
                "marker": marker
            });
            if let (Some(t), Some(obj)) = (object_type, params.as_object_mut()) {
                obj.insert("type".to_string(), Value::String(t.to_string()));
            }
            let result = self.request("account_objects", params).await?;
            let page_result: AccountObjectsResult = serde_json::from_value(result)?;
            all.extend(page_result.account_objects);
            marker = page_result.marker;
            if marker.is_none() {
                break;
            }
            if page == 49 {
                return Err(ClientError::PaginationLimitReached(50));
            }
        }
        Ok(all)
    }

    /// Get all transaction history for an account, following marker chain.
    pub async fn account_tx_all(
        &self,
        account: &str,
        ledger_index_min: Option<i32>,
        ledger_index_max: Option<i32>,
    ) -> Result<Vec<AccountTxEntry>, ClientError> {
        let mut all = Vec::new();
        let mut marker: Option<Value> = None;
        for page in 0..50 {
            let result = self
                .request(
                    "account_tx",
                    json!({
                        "account": account,
                        "ledger_index_min": ledger_index_min.unwrap_or(-1),
                        "ledger_index_max": ledger_index_max.unwrap_or(-1),
                        "limit": 200,
                        "marker": marker,
                        "forward": false
                    }),
                )
                .await?;
            let page_result: AccountTxResult = serde_json::from_value(result)?;
            all.extend(page_result.transactions);
            marker = page_result.marker;
            if marker.is_none() {
                break;
            }
            if page == 49 {
                return Err(ClientError::PaginationLimitReached(50));
            }
        }
        Ok(all)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

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
