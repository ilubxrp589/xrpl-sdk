use crate::client::Client;
use crate::error::ClientError;
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

    /// Subscribe to ledger events.
    pub async fn subscribe_ledger(&self) -> Result<broadcast::Receiver<WsEvent>, ClientError> {
        let params = json!({"streams": ["ledger"]});
        self.raw_request("subscribe", params.clone()).await?;
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
        self.raw_request("subscribe", params.clone()).await?;
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
        self.raw_request("subscribe", params.clone()).await?;
        let mut subs = self.active_subs.lock().await;
        if !subs.contains(&params) {
            subs.push(params);
        }
        Ok(self.event_tx.subscribe())
    }

    /// Unsubscribe from a ledger stream.
    pub async fn unsubscribe_ledger(&self) -> Result<(), ClientError> {
        let params = json!({"streams": ["ledger"]});
        self.raw_request("unsubscribe", params.clone()).await?;
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

    /// Get the URL this client is connected to.
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Client for XrplWsClient {
    async fn raw_request(&self, command: &str, params: Value) -> Result<Value, ClientError> {
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
}

#[cfg(all(test, feature = "live-tests"))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
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
