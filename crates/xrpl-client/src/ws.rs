use crate::error::ClientError;
use crate::types::*;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex, Notify};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::MaybeTlsStream;

type WsStream = tokio_tungstenite::WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// WebSocket event from subscriptions.
#[derive(Debug, Clone)]
pub enum WsEvent {
    /// Ledger closed event.
    Ledger(LedgerEvent),
    /// Transaction event (from account or global subscription).
    Transaction(TransactionEvent),
    /// Connection was re-established after a disconnect.
    Reconnected,
}

/// Ledger closed event data.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LedgerEvent {
    pub ledger_index: u32,
    pub ledger_hash: Option<String>,
    pub ledger_time: Option<u64>,
    pub txn_count: Option<u32>,
    pub fee_base: Option<u64>,
    pub reserve_base: Option<u64>,
    pub reserve_inc: Option<u64>,
}

/// Transaction stream event data.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TransactionEvent {
    pub transaction: Value,
    pub meta: Option<Value>,
    pub validated: Option<bool>,
    pub ledger_index: Option<u32>,
}

type PendingMap = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value, ClientError>>>>>;

/// WebSocket client for XRPL nodes with automatic reconnection.
///
/// On disconnect, the client will reconnect with exponential backoff
/// and re-subscribe to all active subscriptions.
pub struct XrplWsClient {
    write_tx: Arc<Mutex<mpsc::Sender<String>>>,
    pending: PendingMap,
    id_counter: Arc<AtomicU64>,
    event_tx: broadcast::Sender<WsEvent>,
    url: String,
    active_subs: Arc<Mutex<Vec<Value>>>,
    connected: Arc<AtomicBool>,
    /// Kept alive to signal reconnection loop (used by spawned tasks, not directly by self).
    #[allow(dead_code)]
    reconnect_notify: Arc<Notify>,
}

impl XrplWsClient {
    /// Connect to an XRPL WebSocket endpoint with automatic reconnection.
    pub async fn connect(url: &str) -> Result<Self, ClientError> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(url)
            .await
            .map_err(|e| ClientError::UnexpectedResponse(format!("WS connect failed: {e}")))?;

        let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));
        let (event_tx, _) = broadcast::channel(256);
        let (write_tx, write_rx) = mpsc::channel::<String>(64);
        let connected = Arc::new(AtomicBool::new(true));
        let reconnect_notify = Arc::new(Notify::new());
        let active_subs: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));

        let (ws_write, ws_read) = futures::StreamExt::split(ws_stream);

        // Spawn write loop
        tokio::spawn(Self::write_loop(ws_write, write_rx));

        // Spawn read loop — on disconnect triggers reconnect
        let pending_read = pending.clone();
        let event_tx_read = event_tx.clone();
        let connected_read = connected.clone();
        let reconnect_notify_read = reconnect_notify.clone();
        tokio::spawn(async move {
            Self::read_loop(ws_read, pending_read, event_tx_read).await;
            connected_read.store(false, Ordering::Relaxed);
            reconnect_notify_read.notify_one();
        });

        // Spawn keepalive
        let write_tx_ping = write_tx.clone();
        let connected_ping = connected.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if !connected_ping.load(Ordering::Relaxed) {
                    break;
                }
                let ping = json!({"command": "ping", "id": u64::MAX}).to_string();
                if write_tx_ping.send(ping).await.is_err() {
                    break;
                }
            }
        });

        let write_tx = Arc::new(Mutex::new(write_tx));
        let id_counter = Arc::new(AtomicU64::new(1));

        let client = Self {
            write_tx: write_tx.clone(),
            pending: pending.clone(),
            id_counter: id_counter.clone(),
            event_tx: event_tx.clone(),
            url: url.to_string(),
            active_subs: active_subs.clone(),
            connected: connected.clone(),
            reconnect_notify: reconnect_notify.clone(),
        };

        // Spawn reconnection loop
        let url_owned = url.to_string();
        tokio::spawn(async move {
            Self::reconnection_loop(
                url_owned,
                write_tx,
                pending,
                event_tx,
                connected,
                reconnect_notify,
                active_subs,
                id_counter,
            )
            .await;
        });

        Ok(client)
    }

    /// Reconnection loop: waits for disconnect, then reconnects with exponential backoff.
    #[allow(clippy::too_many_arguments)]
    async fn reconnection_loop(
        url: String,
        write_tx: Arc<Mutex<mpsc::Sender<String>>>,
        pending: PendingMap,
        event_tx: broadcast::Sender<WsEvent>,
        connected: Arc<AtomicBool>,
        reconnect_notify: Arc<Notify>,
        active_subs: Arc<Mutex<Vec<Value>>>,
        id_counter: Arc<AtomicU64>,
    ) {
        loop {
            // Wait for a disconnect signal
            reconnect_notify.notified().await;

            if connected.load(Ordering::Relaxed) {
                continue;
            }

            let mut attempt = 0u32;
            loop {
                // Exponential backoff: 1s, 2s, 4s, 8s, ... capped at 60s
                let delay = Duration::from_secs((1u64 << attempt).min(60));
                tracing::warn!(attempt = attempt, url = %url, "WebSocket disconnected, attempting reconnect");
                tokio::time::sleep(delay).await;
                attempt += 1;

                match tokio_tungstenite::connect_async(&url).await {
                    Ok((ws_stream, _)) => {
                        let (ws_write, ws_read) = futures::StreamExt::split(ws_stream);

                        // Create new write channel
                        let (new_write_tx, new_write_rx) = mpsc::channel::<String>(64);
                        *write_tx.lock().await = new_write_tx.clone();

                        connected.store(true, Ordering::Relaxed);

                        // Spawn new write loop
                        tokio::spawn(Self::write_loop(ws_write, new_write_rx));

                        // Spawn new read loop
                        let pending_read = pending.clone();
                        let event_tx_read = event_tx.clone();
                        let connected_read = connected.clone();
                        let reconnect_notify_read = reconnect_notify.clone();
                        tokio::spawn(async move {
                            Self::read_loop(ws_read, pending_read, event_tx_read).await;
                            connected_read.store(false, Ordering::Relaxed);
                            reconnect_notify_read.notify_one();
                        });

                        // Spawn new keepalive
                        let connected_ping = connected.clone();
                        let write_tx_ping = new_write_tx;
                        tokio::spawn(async move {
                            let mut interval = tokio::time::interval(Duration::from_secs(30));
                            loop {
                                interval.tick().await;
                                if !connected_ping.load(Ordering::Relaxed) {
                                    break;
                                }
                                let ping = json!({"command": "ping", "id": u64::MAX}).to_string();
                                if write_tx_ping.send(ping).await.is_err() {
                                    break;
                                }
                            }
                        });

                        // Re-subscribe to all active subscriptions
                        let subs = active_subs.lock().await.clone();
                        for sub_params in &subs {
                            let id = id_counter.fetch_add(1, Ordering::Relaxed);
                            let mut msg = sub_params.clone();
                            if let Some(obj) = msg.as_object_mut() {
                                obj.insert("command".into(), Value::String("subscribe".into()));
                                obj.insert("id".into(), Value::Number(id.into()));
                            }
                            let wtx = write_tx.lock().await;
                            let _ = wtx.send(msg.to_string()).await;
                        }

                        // Notify subscribers of reconnection
                        tracing::info!(url = %url, "WebSocket reconnected successfully");
                        let _ = event_tx.send(WsEvent::Reconnected);
                        break;
                    }
                    Err(_) => {
                        tracing::error!(url = %url, "WebSocket reconnect attempt failed, retrying");
                        continue;
                    }
                }
            }
        }
    }

    /// Write loop: sends messages from the mpsc channel to the WebSocket.
    async fn write_loop(
        mut ws_write: SplitSink<WsStream, Message>,
        mut write_rx: mpsc::Receiver<String>,
    ) {
        while let Some(msg) = write_rx.recv().await {
            if ws_write.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    }

    /// Read loop: routes responses to pending requests and events to subscribers.
    async fn read_loop(
        mut ws_read: SplitStream<WsStream>,
        pending: PendingMap,
        event_tx: broadcast::Sender<WsEvent>,
    ) {
        while let Some(msg_result) = ws_read.next().await {
            let msg = match msg_result {
                Ok(m) => m,
                Err(_) => break,
            };

            let text = match msg {
                Message::Text(t) => t.to_string(),
                Message::Close(_) => break,
                _ => continue,
            };

            let value: Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => continue,
            };

            // Route based on whether this is a response (has "id") or stream event
            if let Some(id) = value.get("id").and_then(|v| v.as_u64()) {
                if id == u64::MAX {
                    continue; // ping response, ignore
                }
                let mut map = pending.lock().await;
                if let Some(sender) = map.remove(&id) {
                    let result = if let Some(result) = value.get("result") {
                        Ok(result.clone())
                    } else if value.get("error").is_some() {
                        let code = value
                            .get("error")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let message = value
                            .get("error_message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        Err(ClientError::RpcError { code, message })
                    } else {
                        Ok(value.clone())
                    };
                    let _ = sender.send(result);
                }
            } else if let Some(msg_type) = value.get("type").and_then(|v| v.as_str()) {
                let event = match msg_type {
                    "ledgerClosed" => serde_json::from_value::<LedgerEvent>(value)
                        .ok()
                        .map(WsEvent::Ledger),
                    "transaction" => serde_json::from_value::<TransactionEvent>(value)
                        .ok()
                        .map(WsEvent::Transaction),
                    _ => None,
                };
                if let Some(evt) = event {
                    let _ = event_tx.send(evt);
                }
            }
        }

        // Connection closed — fail all pending requests
        let mut map = pending.lock().await;
        for (_, sender) in map.drain() {
            let _ = sender.send(Err(ClientError::Disconnected));
        }
    }

    /// Send a request and wait for the response.
    pub async fn request(&self, command: &str, params: Value) -> Result<Value, ClientError> {
        let mut wait_iters = 0u32;
        while !self.connected.load(Ordering::Relaxed) {
            if wait_iters >= 300 {
                return Err(ClientError::Disconnected);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
            wait_iters += 1;
        }

        let id = self.id_counter.fetch_add(1, Ordering::Relaxed);
        tracing::debug!(command = command, id = id, "sending WS request");
        let (tx, rx) = oneshot::channel();

        {
            let mut map = self.pending.lock().await;
            map.insert(id, tx);
        }

        let msg = if let Some(obj) = params.as_object() {
            let mut m = serde_json::Map::new();
            m.insert("command".into(), Value::String(command.into()));
            m.insert("id".into(), Value::Number(id.into()));
            for (k, v) in obj {
                m.insert(k.clone(), v.clone());
            }
            Value::Object(m)
        } else {
            json!({"command": command, "id": id})
        };

        {
            let wtx = self.write_tx.lock().await;
            wtx.send(msg.to_string())
                .await
                .map_err(|_| ClientError::Disconnected)?;
        }

        match tokio::time::timeout(Duration::from_secs(30), rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(ClientError::Disconnected),
            Err(_) => {
                let mut map = self.pending.lock().await;
                map.remove(&id);
                Err(ClientError::Timeout(Duration::from_secs(30)))
            }
        }
    }

    /// Subscribe to ledger events.
    pub async fn subscribe_ledger(&self) -> Result<broadcast::Receiver<WsEvent>, ClientError> {
        let params = json!({"streams": ["ledger"]});
        self.request("subscribe", params.clone()).await?;
        let mut subs = self.active_subs.lock().await;
        if !subs.contains(&params) {
            subs.push(params);
        }
        Ok(self.event_tx.subscribe())
    }

    /// Subscribe to all validated transactions.
    pub async fn subscribe_transactions(
        &self,
    ) -> Result<broadcast::Receiver<WsEvent>, ClientError> {
        let params = json!({"streams": ["transactions"]});
        self.request("subscribe", params.clone()).await?;
        let mut subs = self.active_subs.lock().await;
        if !subs.contains(&params) {
            subs.push(params);
        }
        Ok(self.event_tx.subscribe())
    }

    /// Subscribe to a specific account's transactions.
    pub async fn subscribe_account(
        &self,
        account: &str,
    ) -> Result<broadcast::Receiver<WsEvent>, ClientError> {
        let params = json!({"accounts": [account]});
        self.request("subscribe", params.clone()).await?;
        let mut subs = self.active_subs.lock().await;
        if !subs.contains(&params) {
            subs.push(params);
        }
        Ok(self.event_tx.subscribe())
    }

    /// Unsubscribe from a ledger stream.
    pub async fn unsubscribe_ledger(&self) -> Result<(), ClientError> {
        let params = json!({"streams": ["ledger"]});
        self.request("unsubscribe", params.clone()).await?;
        let mut subs = self.active_subs.lock().await;
        subs.retain(|s| s != &params);
        Ok(())
    }

    /// Get a receiver for all subscription events.
    pub fn events(&self) -> broadcast::Receiver<WsEvent> {
        self.event_tx.subscribe()
    }

    /// Returns true if the WebSocket is currently connected.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    // --- Mirrored HTTP methods ---

    /// Get account information.
    pub async fn account_info(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountInfo, ClientError> {
        let result = self
            .request(
                "account_info",
                json!({"account": account, "ledger_index": ledger.as_value()}),
            )
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get account trust lines.
    pub async fn account_lines(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountLinesResult, ClientError> {
        let result = self
            .request(
                "account_lines",
                json!({"account": account, "ledger_index": ledger.as_value(), "limit": 400}),
            )
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get account offers.
    pub async fn account_offers(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<AccountOffersResult, ClientError> {
        let result = self
            .request(
                "account_offers",
                json!({"account": account, "ledger_index": ledger.as_value(), "limit": 400}),
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

    /// Submit a signed transaction blob.
    pub async fn submit(&self, tx_blob: &str) -> Result<SubmitResult, ClientError> {
        let result = self.request("submit", json!({"tx_blob": tx_blob})).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Look up a transaction by hash.
    pub async fn tx(&self, hash: &str) -> Result<TxResult, ClientError> {
        let result = self
            .request("tx", json!({"transaction": hash, "binary": false}))
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

        let (acct_info, fee_result, ledger_idx) = tokio::try_join!(
            async {
                if need_seq {
                    self.account_info(&account, &LedgerIndex::Current)
                        .await
                        .map(Some)
                } else {
                    Ok(None)
                }
            },
            async {
                if need_fee {
                    self.fee().await.map(Some)
                } else {
                    Ok(None)
                }
            },
            async {
                if need_lls {
                    self.ledger_current().await.map(Some)
                } else {
                    Ok(None)
                }
            },
        )?;

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
        let sr = self.submit(tx_blob).await?;

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
                Ok(tx_result) => {
                    if tx_result.validated == Some(true) {
                        return Ok(tx_result);
                    }
                }
                Err(ClientError::RpcError { ref code, .. }) if code == "txnNotFound" => {}
                Err(e) => return Err(e),
            }

            tokio::time::sleep(Duration::from_secs(4)).await;
        }

        Err(ClientError::Timeout(Duration::from_secs(80)))
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

    /// Get the URL this client is connected to.
    pub fn url(&self) -> &str {
        &self.url
    }
}

#[cfg(all(test, feature = "live-tests"))]
mod live_tests {
    use super::*;

    const TESTNET_WS: &str = "wss://s.altnet.rippletest.net:51233";

    #[tokio::test]
    async fn ws_fee_testnet() {
        let client = XrplWsClient::connect(TESTNET_WS).await.unwrap();
        assert!(client.is_connected());
        let fee = client.fee().await.unwrap();
        let base_fee: u64 = fee.drops.base_fee.parse().unwrap();
        assert!(base_fee >= 10);
    }

    #[tokio::test]
    async fn ws_ledger_current_testnet() {
        let client = XrplWsClient::connect(TESTNET_WS).await.unwrap();
        let idx = client.ledger_current().await.unwrap();
        assert!(idx > 0);
    }

    #[tokio::test]
    async fn ws_subscribe_ledger_testnet() {
        let client = XrplWsClient::connect(TESTNET_WS).await.unwrap();
        let mut rx = client.subscribe_ledger().await.unwrap();

        let event = tokio::time::timeout(Duration::from_secs(15), rx.recv())
            .await
            .expect("timeout waiting for ledger event")
            .expect("channel closed");

        match event {
            WsEvent::Ledger(le) => {
                assert!(le.ledger_index > 0);
            }
            _ => panic!("expected ledger event"),
        }
    }
}
