//! WebSocket client for WASM environments using browser-native WebSocket via web-sys.

use crate::client::Client;
use crate::error::ClientError;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

type PendingMap =
    Arc<Mutex<HashMap<u64, futures::channel::oneshot::Sender<Result<Value, ClientError>>>>>;

/// WebSocket client for XRPL nodes (WASM version using browser WebSocket API).
pub struct XrplWsClient {
    ws: WebSocket,
    pending: PendingMap,
    id_counter: Arc<AtomicU64>,
    url: String,
    connected: Arc<AtomicBool>,
    active_subs: Arc<Mutex<Vec<Value>>>,
    max_fee_drops: u64,
    // Event closures stored to prevent GC
    _on_message: Closure<dyn FnMut(MessageEvent)>,
    _on_error: Closure<dyn FnMut(ErrorEvent)>,
    _on_close: Closure<dyn FnMut(web_sys::CloseEvent)>,
}

impl XrplWsClient {
    /// The same client with `drops` as the most [`Client::autofill`] sets as
    /// a `Fee` (default [`crate::client::DEFAULT_MAX_FEE_DROPS`], 2 XRP).
    pub fn with_max_fee_drops(mut self, drops: u64) -> Self {
        self.max_fee_drops = drops;
        self
    }

    /// Connect to an XRPL WebSocket endpoint.
    /// Uses the browser's native WebSocket API.
    pub async fn connect(url: &str) -> Result<Self, ClientError> {
        let ws = WebSocket::new(url).map_err(|e| {
            ClientError::UnexpectedResponse(format!("WebSocket creation failed: {:?}", e))
        })?;

        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));
        let connected = Arc::new(AtomicBool::new(false));
        let active_subs: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));

        // Wait for connection to open
        let (open_tx, open_rx) = futures::channel::oneshot::channel::<Result<(), ClientError>>();
        let open_tx = Arc::new(Mutex::new(Some(open_tx)));

        let open_tx_clone = open_tx.clone();
        let on_open = Closure::once(move || {
            if let Ok(mut guard) = open_tx_clone.lock() {
                if let Some(tx) = guard.take() {
                    let _ = tx.send(Ok(()));
                }
            }
        });
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        on_open.forget();

        let open_tx_clone2 = open_tx.clone();
        let on_open_error = Closure::once(move |_e: ErrorEvent| {
            if let Ok(mut guard) = open_tx_clone2.lock() {
                if let Some(tx) = guard.take() {
                    let _ = tx.send(Err(ClientError::UnexpectedResponse(
                        "WebSocket open error".into(),
                    )));
                }
            }
        });
        ws.set_onerror(Some(on_open_error.as_ref().unchecked_ref()));
        on_open_error.forget();

        // Await the open event
        open_rx.await.map_err(|_| ClientError::Disconnected)??;

        connected.store(true, Ordering::Relaxed);

        // Set up persistent message handler
        let pending_msg = pending.clone();
        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
            let text = if let Ok(s) = e.data().dyn_into::<js_sys::JsString>() {
                String::from(s)
            } else {
                return;
            };

            let value: Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => return,
            };

            if let Some(id) = value.get("id").and_then(|v| v.as_u64()) {
                if id == u64::MAX {
                    return; // ping response
                }
                let Ok(mut map) = pending_msg.lock() else {
                    return;
                };
                if let Some(sender) = map.remove(&id) {
                    let _ = sender.send(crate::client::ws_response(&value));
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        // Persistent error handler
        let on_error = Closure::wrap(Box::new(move |_e: ErrorEvent| {
            web_sys::console::warn_1(&JsValue::from_str("XRPL WebSocket error"));
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));

        // Persistent close handler
        let connected_close = connected.clone();
        let on_close = Closure::wrap(Box::new(move |_: web_sys::CloseEvent| {
            connected_close.store(false, Ordering::Relaxed);
            web_sys::console::warn_1(&JsValue::from_str(
                "XRPL WebSocket disconnected. Call connect() again to reconnect.",
            ));
        }) as Box<dyn FnMut(web_sys::CloseEvent)>);
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));

        Ok(Self {
            ws,
            pending,
            id_counter: Arc::new(AtomicU64::new(1)),
            url: url.to_string(),
            connected,
            active_subs,
            max_fee_drops: crate::client::DEFAULT_MAX_FEE_DROPS,
            _on_message: on_message,
            _on_error: on_error,
            _on_close: on_close,
        })
    }

    /// Subscribe to ledger events.
    pub async fn subscribe_ledger(&self) -> Result<(), ClientError> {
        let params = json!({"streams": ["ledger"]});
        self.raw_request("subscribe", params.clone()).await?;
        let mut subs = self
            .active_subs
            .lock()
            .map_err(|_| ClientError::UnexpectedResponse("mutex lock poisoned".into()))?;
        if !subs.contains(&params) {
            subs.push(params);
        }
        Ok(())
    }

    /// Subscribe to all validated transactions.
    pub async fn subscribe_transactions(&self) -> Result<(), ClientError> {
        let params = json!({"streams": ["transactions"]});
        self.raw_request("subscribe", params.clone()).await?;
        let mut subs = self
            .active_subs
            .lock()
            .map_err(|_| ClientError::UnexpectedResponse("mutex lock poisoned".into()))?;
        if !subs.contains(&params) {
            subs.push(params);
        }
        Ok(())
    }

    /// Subscribe to a specific account's transactions.
    pub async fn subscribe_account(&self, account: &str) -> Result<(), ClientError> {
        let params = json!({"accounts": [account]});
        self.raw_request("subscribe", params.clone()).await?;
        let mut subs = self
            .active_subs
            .lock()
            .map_err(|_| ClientError::UnexpectedResponse("mutex lock poisoned".into()))?;
        if !subs.contains(&params) {
            subs.push(params);
        }
        Ok(())
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
    fn max_fee_drops(&self) -> u64 {
        self.max_fee_drops
    }

    async fn raw_request(&self, command: &str, params: Value) -> Result<Value, ClientError> {
        if !self.connected.load(Ordering::Relaxed) {
            return Err(ClientError::Disconnected);
        }

        let id = self.id_counter.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = futures::channel::oneshot::channel();

        {
            let mut map = self
                .pending
                .lock()
                .map_err(|_| ClientError::UnexpectedResponse("mutex lock poisoned".into()))?;
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

        self.ws
            .send_with_str(&msg.to_string())
            .map_err(|_| ClientError::Disconnected)?;

        rx.await.map_err(|_| ClientError::Disconnected)?
    }
}
