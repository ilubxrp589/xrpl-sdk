//! WebSocket client for WASM environments using browser-native WebSocket via web-sys.

use crate::error::ClientError;
use crate::types::*;
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
    // Event closures stored to prevent GC
    _on_message: Closure<dyn FnMut(MessageEvent)>,
    _on_error: Closure<dyn FnMut(ErrorEvent)>,
    _on_close: Closure<dyn FnMut(web_sys::CloseEvent)>,
}

impl XrplWsClient {
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
                    let result = if let Some(result) = value.get("result") {
                        Ok(result.clone())
                    } else if value.get("error").is_some() {
                        let code = value["error"].as_str().unwrap_or("unknown").to_string();
                        let message = value["error_message"].as_str().unwrap_or("").to_string();
                        Err(ClientError::RpcError { code, message })
                    } else {
                        Ok(value.clone())
                    };
                    let _ = sender.send(result);
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
            _on_message: on_message,
            _on_error: on_error,
            _on_close: on_close,
        })
    }

    /// Send a request and wait for the response.
    pub async fn request(&self, command: &str, params: Value) -> Result<Value, ClientError> {
        if !self.connected.load(Ordering::Relaxed) {
            return Err(ClientError::Disconnected);
        }

        let id = self.id_counter.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = futures::channel::oneshot::channel();

        {
            let mut map = self.pending.lock().map_err(|_| {
                ClientError::UnexpectedResponse("mutex lock poisoned".into())
            })?;
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

    /// Subscribe to ledger events.
    pub async fn subscribe_ledger(&self) -> Result<(), ClientError> {
        let params = json!({"streams": ["ledger"]});
        self.request("subscribe", params.clone()).await?;
        let mut subs = self.active_subs.lock().map_err(|_| {
            ClientError::UnexpectedResponse("mutex lock poisoned".into())
        })?;
        if !subs.contains(&params) {
            subs.push(params);
        }
        Ok(())
    }

    /// Subscribe to all validated transactions.
    pub async fn subscribe_transactions(&self) -> Result<(), ClientError> {
        let params = json!({"streams": ["transactions"]});
        self.request("subscribe", params.clone()).await?;
        let mut subs = self.active_subs.lock().map_err(|_| {
            ClientError::UnexpectedResponse("mutex lock poisoned".into())
        })?;
        if !subs.contains(&params) {
            subs.push(params);
        }
        Ok(())
    }

    /// Subscribe to a specific account's transactions.
    pub async fn subscribe_account(&self, account: &str) -> Result<(), ClientError> {
        let params = json!({"accounts": [account]});
        self.request("subscribe", params.clone()).await?;
        let mut subs = self.active_subs.lock().map_err(|_| {
            ClientError::UnexpectedResponse("mutex lock poisoned".into())
        })?;
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
        let sr: SubmitResult = serde_json::from_value(result)?;
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
            .request("tx", json!({"transaction": hash, "binary": false}))
            .await?;
        Ok(serde_json::from_value(result)?)
    }
}
