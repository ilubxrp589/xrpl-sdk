//! One interface over the HTTP and WebSocket clients. A transport implements
//! [`Client::raw_request`]; every rippled method here is written once.

use crate::error::ClientError;
use crate::requests::*;
use crate::types::*;
use serde_json::{json, Value};
use std::time::Duration;
use xrpl_core::types::LedgerObject;

/// The most pages [`Client::request_all`] follows before it gives up.
pub const MAX_PAGES: usize = 50;

/// A connection to a rippled server. `XrplHttpClient` and `XrplWsClient`
/// implement it; bring it into scope (`use xrpl_client::Client;`) to call
/// the methods below on either.
#[allow(async_fn_in_trait)]
pub trait Client {
    /// Send `method` with `params` and return the response's `result`.
    /// A rippled error (`"status": "error"`) is [`ClientError::RpcError`].
    async fn raw_request(&self, method: &str, params: Value) -> Result<Value, ClientError>;

    /// Send a typed request and decode its `result`.
    async fn request<R: Request>(&self, req: &R) -> Result<R::Response, ClientError> {
        let result = self
            .raw_request(R::METHOD, serde_json::to_value(req)?)
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Send a paginated request and follow its `marker` to the last page
    /// (at most [`MAX_PAGES`] pages, else [`ClientError::PaginationLimitReached`]).
    async fn request_all<R: Paginated>(&self, req: R) -> Result<Vec<R::Item>, ClientError> {
        let mut all = Vec::new();
        let mut req = req;
        for _ in 0..MAX_PAGES {
            let (items, marker) = R::page(self.request(&req).await?);
            all.extend(items);
            match marker {
                Some(marker) => req = req.with_marker(marker),
                None => return Ok(all),
            }
        }
        Err(ClientError::PaginationLimitReached(MAX_PAGES))
    }

    /// Get account information.
    async fn account_info(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountInfo, ClientError> {
        self.request(&AccountInfoRequest {
            account: account.into(),
            ledger_index: ledger.clone(),
        })
        .await
    }

    /// Get current fee information.
    async fn fee(&self) -> Result<FeeResult, ClientError> {
        self.request(&FeeRequest {}).await
    }

    /// Get the current (open) ledger index.
    async fn ledger_current(&self) -> Result<u32, ClientError> {
        Ok(self
            .request(&LedgerCurrentRequest {})
            .await?
            .ledger_current_index)
    }

    /// Get ledger information.
    async fn ledger(&self, ledger: &LedgerIndex) -> Result<LedgerResult, ClientError> {
        self.request(&LedgerRequest {
            ledger_index: ledger.clone(),
        })
        .await
    }

    /// Submit a signed transaction blob (hex). `tem`/`tef` and most `tel` results
    /// are [`ClientError::TransactionFailed`]; `ter` and the load/queue `tel`s
    /// (`telINSUF_FEE_P`, `telCAN_NOT_QUEUE*`) are [`ClientError::TransactionRetry`];
    /// `tec` is [`ClientError::TransactionClaimed`].
    async fn submit(&self, tx_blob: &str) -> Result<SubmitResult, ClientError> {
        classify_submit(
            self.request(&SubmitRequest {
                tx_blob: tx_blob.into(),
            })
            .await?,
        )
    }

    /// Look up a transaction by hash.
    async fn tx(&self, hash: &str) -> Result<TxResult, ClientError> {
        self.request(&TxRequest {
            transaction: hash.into(),
            binary: false,
        })
        .await
    }

    /// Get trust lines for an account (one page of up to 400).
    async fn account_lines(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountLinesResult, ClientError> {
        self.request(&account_lines_request(account, ledger)).await
    }

    /// Get open offers for an account (one page of up to 400).
    async fn account_offers(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountOffersResult, ClientError> {
        self.request(&account_offers_request(account, ledger)).await
    }

    /// Get NFTs owned by an account (one page of up to 400).
    async fn account_nfts(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountNftsResult, ClientError> {
        self.request(&account_nfts_request(account, ledger)).await
    }

    /// Get order book offers (up to 20).
    async fn book_offers(
        &self,
        taker_pays: &Value,
        taker_gets: &Value,
    ) -> Result<BookOffersResult, ClientError> {
        self.request(&BookOffersRequest {
            taker_pays: taker_pays.clone(),
            taker_gets: taker_gets.clone(),
            limit: Some(20),
        })
        .await
    }

    /// Get AMM pool information.
    async fn amm_info(&self, asset: &Value, asset2: &Value) -> Result<AmmInfoResult, ClientError> {
        self.request(&AmmInfoRequest {
            asset: asset.clone(),
            asset2: asset2.clone(),
        })
        .await
    }

    /// Get transaction history for an account (bounds default to all
    /// validated ledgers, limit to 200, order to newest first).
    async fn account_tx(
        &self,
        account: &str,
        ledger_index_min: Option<i32>,
        ledger_index_max: Option<i32>,
        limit: Option<u32>,
        marker: Option<Value>,
        forward: Option<bool>,
    ) -> Result<AccountTxResult, ClientError> {
        self.request(&AccountTxRequest {
            account: account.into(),
            ledger_index_min: ledger_index_min.unwrap_or(-1),
            ledger_index_max: ledger_index_max.unwrap_or(-1),
            limit: Some(limit.unwrap_or(200)),
            marker,
            forward: forward.unwrap_or(false),
        })
        .await
    }

    /// Get objects owned by an account (limit defaults to 200).
    async fn account_objects(
        &self,
        account: &str,
        object_type: Option<&str>,
        ledger: &LedgerIndex,
        limit: Option<u32>,
        marker: Option<Value>,
    ) -> Result<AccountObjectsResult, ClientError> {
        self.request(&AccountObjectsRequest {
            limit: Some(limit.unwrap_or(200)),
            marker,
            ..account_objects_request(account, object_type, ledger)
        })
        .await
    }

    /// Get currencies an account can send or receive.
    async fn account_currencies(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountCurrenciesResult, ClientError> {
        self.request(&AccountCurrenciesRequest {
            account: account.into(),
            ledger_index: ledger.clone(),
        })
        .await
    }

    /// Get payment channels for an account.
    async fn account_channels(
        &self,
        account: &str,
        destination_account: Option<&str>,
        ledger: &LedgerIndex,
    ) -> Result<AccountChannelsResult, ClientError> {
        self.request(&AccountChannelsRequest {
            account: account.into(),
            destination_account: destination_account.map(Into::into),
            ledger_index: ledger.clone(),
        })
        .await
    }

    /// Get a single ledger entry by its object ID.
    async fn ledger_entry(
        &self,
        index: &str,
        ledger: &LedgerIndex,
    ) -> Result<LedgerEntryResult, ClientError> {
        self.request(&LedgerEntryRequest {
            index: index.into(),
            ledger_index: ledger.clone(),
        })
        .await
    }

    /// Get server information.
    async fn server_info(&self) -> Result<ServerInfoResult, ClientError> {
        self.request(&ServerInfoRequest {}).await
    }

    /// Get gateway balances for an issuing account.
    async fn gateway_balances(
        &self,
        account: &str,
        hotwallet: Option<Vec<&str>>,
        ledger: &LedgerIndex,
    ) -> Result<GatewayBalancesResult, ClientError> {
        self.request(&GatewayBalancesRequest {
            account: account.into(),
            hotwallet: hotwallet.map(|h| h.into_iter().map(Into::into).collect()),
            ledger_index: ledger.clone(),
        })
        .await
    }

    /// Fill in Sequence, Fee and LastLedgerSequence where missing or null,
    /// fetching them concurrently.
    async fn autofill(&self, tx: &mut Value) -> Result<(), ClientError> {
        let account = tx
            .get("Account")
            .and_then(Value::as_str)
            .ok_or_else(|| ClientError::UnexpectedResponse("tx missing Account field".into()))?
            .to_string();
        let missing = |tx: &Value, field: &str| tx.get(field).is_none_or(Value::is_null);
        let need_seq = missing(tx, "Sequence");
        let need_fee = missing(tx, "Fee");
        let need_lls = missing(tx, "LastLedgerSequence");
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

        let obj = tx
            .as_object_mut()
            .ok_or_else(|| ClientError::UnexpectedResponse("tx is not a JSON object".into()))?;
        if let Some(info) = acct_info {
            obj.insert("Sequence".into(), json!(info.account_data.sequence));
        }
        if let Some(fee) = fee_result {
            obj.insert("Fee".into(), json!(fee.drops.open_ledger_fee));
        }
        if let Some(idx) = ledger_idx {
            obj.insert("LastLedgerSequence".into(), json!(idx + 4));
        }
        Ok(())
    }

    /// Submit a signed blob (resubmitting up to twice on a retry result) and poll
    /// until it is validated or `last_ledger_sequence` passes.
    async fn submit_and_wait(
        &self,
        tx_blob: &str,
        last_ledger_sequence: u32,
    ) -> Result<TxResult, ClientError> {
        let mut submit_result = None;
        for attempt in 0..3 {
            match self.submit(tx_blob).await {
                Ok(sr) => {
                    submit_result = Some(sr);
                    break;
                }
                Err(ClientError::TransactionRetry { .. }) if attempt < 2 => {
                    sleep_secs(4).await;
                }
                Err(e) => return Err(e),
            }
        }
        let sr = submit_result.ok_or_else(|| ClientError::TransactionFailed {
            engine_result: "retry_exhausted".into(),
            message: "retry result after 3 submit attempts".into(),
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

        for _ in 0..20 {
            let current = self.ledger_current().await?;
            if current > last_ledger_sequence {
                return Err(ClientError::TransactionExpired(last_ledger_sequence));
            }
            match self.tx(&hash).await {
                Ok(tx_result) if tx_result.validated == Some(true) => return Ok(tx_result),
                Ok(_) => {}
                // Not in a ledger yet: keep polling.
                Err(ClientError::RpcError { ref code, .. }) if code == "txnNotFound" => {}
                Err(e) => return Err(e),
            }
            sleep_secs(4).await;
        }
        Err(ClientError::Timeout(Duration::from_secs(80)))
    }

    /// All trust lines of an account, following the marker.
    async fn account_lines_all(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<Vec<TrustLine>, ClientError> {
        self.request_all(account_lines_request(account, ledger))
            .await
    }

    /// All offers of an account, following the marker.
    async fn account_offers_all(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<Vec<AccountOffer>, ClientError> {
        self.request_all(account_offers_request(account, ledger))
            .await
    }

    /// All NFTs of an account, following the marker.
    async fn account_nfts_all(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<Vec<NFToken>, ClientError> {
        self.request_all(account_nfts_request(account, ledger))
            .await
    }

    /// All objects of an account, following the marker.
    async fn account_objects_all(
        &self,
        account: &str,
        object_type: Option<&str>,
        ledger: &LedgerIndex,
    ) -> Result<Vec<LedgerObject>, ClientError> {
        self.request_all(account_objects_request(account, object_type, ledger))
            .await
    }

    /// All transactions of an account, newest first, following the marker.
    async fn account_tx_all(
        &self,
        account: &str,
        ledger_index_min: Option<i32>,
        ledger_index_max: Option<i32>,
    ) -> Result<Vec<AccountTxEntry>, ClientError> {
        self.request_all(AccountTxRequest {
            account: account.into(),
            ledger_index_min: ledger_index_min.unwrap_or(-1),
            ledger_index_max: ledger_index_max.unwrap_or(-1),
            limit: Some(200),
            marker: None,
            forward: false,
        })
        .await
    }
}

fn account_lines_request(account: &str, ledger: &LedgerIndex) -> AccountLinesRequest {
    AccountLinesRequest {
        account: account.into(),
        ledger_index: ledger.clone(),
        limit: Some(400),
        marker: None,
    }
}

fn account_offers_request(account: &str, ledger: &LedgerIndex) -> AccountOffersRequest {
    AccountOffersRequest {
        account: account.into(),
        ledger_index: ledger.clone(),
        limit: Some(400),
        marker: None,
    }
}

fn account_nfts_request(account: &str, ledger: &LedgerIndex) -> AccountNftsRequest {
    AccountNftsRequest {
        account: account.into(),
        ledger_index: ledger.clone(),
        limit: Some(400),
        marker: None,
    }
}

fn account_objects_request(
    account: &str,
    object_type: Option<&str>,
    ledger: &LedgerIndex,
) -> AccountObjectsRequest {
    AccountObjectsRequest {
        account: account.into(),
        object_type: object_type.map(Into::into),
        ledger_index: ledger.clone(),
        limit: Some(200),
        marker: None,
    }
}

/// Sort a submit's engine result: `tes` is Ok; `tem`/`tef` failed; `ter` retry;
/// `tec` claimed a fee. A `tel` was neither applied nor relayed, so it never
/// reaches a ledger: the load and queue ones may pass on a resubmit, the rest
/// fail. Anything unrecognised is Ok.
fn classify_submit(sr: SubmitResult) -> Result<SubmitResult, ClientError> {
    let r = sr.engine_result.as_str();
    let tel_retry = r == "telINSUF_FEE_P" || r.starts_with("telCAN_NOT_QUEUE");
    if r.starts_with("tes") {
        Ok(sr)
    } else if r.starts_with("ter") || tel_retry {
        Err(ClientError::TransactionRetry {
            engine_result: sr.engine_result,
        })
    } else if r.starts_with("tem") || r.starts_with("tef") || r.starts_with("tel") {
        Err(ClientError::TransactionFailed {
            engine_result: sr.engine_result,
            message: sr.engine_result_message,
        })
    } else if r.starts_with("tec") {
        Err(ClientError::TransactionClaimed {
            engine_result: sr.engine_result,
        })
    } else {
        Ok(sr)
    }
}

/// Platform-aware async sleep.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn sleep_secs(secs: u64) {
    tokio::time::sleep(Duration::from_secs(secs)).await;
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn sleep_secs(secs: u64) {
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

#[cfg(all(test, not(target_arch = "wasm32")))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use std::collections::{HashMap, VecDeque};
    use std::sync::Mutex;

    /// A transport that answers each method from a queue and records the calls.
    #[derive(Default)]
    struct Mock {
        calls: Mutex<Vec<(String, Value)>>,
        replies: Mutex<HashMap<String, VecDeque<Value>>>,
    }

    impl Mock {
        fn reply(self, method: &str, result: Value) -> Self {
            self.replies
                .lock()
                .unwrap()
                .entry(method.into())
                .or_default()
                .push_back(result);
            self
        }
        fn calls(&self) -> Vec<(String, Value)> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl Client for Mock {
        async fn raw_request(&self, method: &str, params: Value) -> Result<Value, ClientError> {
            self.calls.lock().unwrap().push((method.into(), params));
            let reply = self
                .replies
                .lock()
                .unwrap()
                .get_mut(method)
                .and_then(VecDeque::pop_front);
            reply.ok_or_else(|| ClientError::UnexpectedResponse(format!("no reply for {method}")))
        }
    }

    #[test]
    fn requests_carry_the_params_the_methods_always_sent() {
        let p = |r: &dyn erased::Params| r.params();
        assert_eq!(
            p(&account_lines_request("rA", &LedgerIndex::Validated)),
            json!({"account": "rA", "ledger_index": "validated", "limit": 400})
        );
        assert_eq!(
            p(&account_objects_request(
                "rA",
                Some("offer"),
                &LedgerIndex::Index(7)
            )),
            json!({"account": "rA", "type": "offer", "ledger_index": 7, "limit": 200})
        );
        assert_eq!(
            p(&TxRequest {
                transaction: "AB".into(),
                binary: false
            }),
            json!({"transaction": "AB", "binary": false})
        );
        assert_eq!(p(&FeeRequest {}), json!({}));
        assert_eq!(
            p(&AccountChannelsRequest {
                account: "rA".into(),
                destination_account: None,
                ledger_index: LedgerIndex::Current,
            }),
            json!({"account": "rA", "ledger_index": "current"})
        );
    }

    mod erased {
        pub trait Params {
            fn params(&self) -> serde_json::Value;
        }
        impl<R: super::Request> Params for R {
            fn params(&self) -> serde_json::Value {
                serde_json::to_value(self).unwrap()
            }
        }
    }

    #[tokio::test]
    async fn request_all_follows_the_marker_to_the_last_page() {
        let line = |c: &str| json!({"account": "rB", "balance": "1", "currency": c, "limit": "0", "limit_peer": "0"});
        let mock = Mock::default()
            .reply(
                "account_lines",
                json!({"account": "rA", "lines": [line("USD")], "marker": "M1"}),
            )
            .reply(
                "account_lines",
                json!({"account": "rA", "lines": [line("EUR")]}),
            );
        let lines = mock
            .account_lines_all("rA", &LedgerIndex::Validated)
            .await
            .unwrap();
        assert_eq!(
            lines
                .iter()
                .map(|l| l.currency.as_str())
                .collect::<Vec<_>>(),
            ["USD", "EUR"]
        );
        let calls = mock.calls();
        assert_eq!(calls[0].1.get("marker"), None);
        assert_eq!(calls[1].1["marker"], "M1");
    }

    #[tokio::test]
    async fn request_all_stops_at_max_pages() {
        let mut mock = Mock::default();
        for _ in 0..MAX_PAGES {
            mock = mock.reply(
                "account_offers",
                json!({"account": "rA", "offers": [], "marker": "M"}),
            );
        }
        let err = mock
            .account_offers_all("rA", &LedgerIndex::Validated)
            .await
            .unwrap_err();
        assert!(
            matches!(err, ClientError::PaginationLimitReached(MAX_PAGES)),
            "{err}"
        );
    }

    #[tokio::test]
    async fn submit_sorts_engine_results() {
        let reply = |r: &str| json!({"engine_result": r, "engine_result_code": 0, "engine_result_message": "m", "tx_blob": "00"});
        let mock = Mock::default()
            .reply("submit", reply("tesSUCCESS"))
            .reply("submit", reply("temMALFORMED"))
            .reply("submit", reply("terQUEUED"))
            .reply("submit", reply("tecNO_DST"))
            .reply("submit", reply("telINSUF_FEE_P"))
            .reply("submit", reply("telCAN_NOT_QUEUE_FULL"))
            .reply("submit", reply("telBAD_PUBLIC_KEY"));
        assert!(mock.submit("00").await.is_ok());
        assert!(matches!(
            mock.submit("00").await,
            Err(ClientError::TransactionFailed { .. })
        ));
        assert!(matches!(
            mock.submit("00").await,
            Err(ClientError::TransactionRetry { .. })
        ));
        assert!(matches!(
            mock.submit("00").await,
            Err(ClientError::TransactionClaimed { .. })
        ));
        // A tel never reaches a ledger: load/queue ones retry, the rest fail.
        assert!(matches!(
            mock.submit("00").await,
            Err(ClientError::TransactionRetry { .. })
        ));
        assert!(matches!(
            mock.submit("00").await,
            Err(ClientError::TransactionRetry { .. })
        ));
        assert!(matches!(
            mock.submit("00").await,
            Err(ClientError::TransactionFailed { .. })
        ));
    }

    #[tokio::test]
    async fn autofill_fills_only_what_is_missing() {
        let mock = Mock::default()
            .reply("fee", json!({"drops": {"base_fee": "10", "median_fee": "10", "minimum_fee": "10", "open_ledger_fee": "12"}}))
            .reply("ledger_current", json!({"ledger_current_index": 100}));
        let mut tx =
            json!({"TransactionType": "Payment", "Account": "rA", "Sequence": 5, "Fee": null});
        mock.autofill(&mut tx).await.unwrap();
        assert_eq!(tx["Sequence"], 5);
        assert_eq!(tx["Fee"], "12");
        assert_eq!(tx["LastLedgerSequence"], 104);
        assert!(mock.calls().iter().all(|(m, _)| m != "account_info"));
    }

    /// The trait's futures stay Send for the real clients (tokio::spawn needs it).
    #[allow(dead_code)]
    fn client_futures_are_send(
        http: &crate::XrplHttpClient,
        ws: &crate::XrplWsClient,
        tx: &mut Value,
    ) {
        fn send<T: Send>(_: T) {}
        send(http.account_info("rA", &LedgerIndex::Current));
        send(http.account_lines_all("rA", &LedgerIndex::Current));
        send(http.submit_and_wait("00", 0));
        send(ws.account_info("rA", &LedgerIndex::Current));
        send(ws.account_tx_all("rA", None, None));
        send(ws.autofill(tx));
    }
}
