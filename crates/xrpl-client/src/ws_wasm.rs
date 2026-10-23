//! WebSocket client for WASM environments using browser-native WebSocket via web-sys.
//!
//! Requests time out after 30 s; a closed connection fails every request in
//! flight and sends [`WsEvent::Disconnected`] to every subscription (the
//! browser client does not reconnect: connect again). Dropping the client
//! closes the socket.

use crate::client::{Client, Transport};
use crate::error::ClientError;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

pub use crate::stream::*;

type PendingMap =
    Arc<Mutex<HashMap<u64, futures::channel::oneshot::Sender<Result<Value, ClientError>>>>>;

/// How long a request waits for its reply.
const REQUEST_TIMEOUT_SECS: u64 = 30;
/// Events buffered per subscription before newer ones are counted as missed.
const EVENT_BUFFER: usize = 1024;

type Filter = Box<dyn Fn(&WsEvent) -> bool + Send + Sync>;

/// One subscriber: its filter, its channel, and how many events it missed.
struct Subscriber {
    filter: Filter,
    tx: futures::channel::mpsc::Sender<WsEvent>,
    missed: Arc<AtomicU64>,
}

type Subscribers = Arc<Mutex<Vec<Subscriber>>>;

fn deliver(subscribers: &Subscribers, event: &WsEvent) {
    let Ok(mut subs) = subscribers.lock() else {
        return;
    };
    subs.retain_mut(|s| {
        if !(event.is_connection_event() || (s.filter)(event)) {
            return true;
        }
        match s.tx.try_send(event.clone()) {
            Ok(()) => true,
            Err(e) if e.is_full() => {
                s.missed.fetch_add(1, Ordering::Relaxed);
                true
            }
            Err(_) => false, // the Subscription was dropped
        }
    });
}

/// The events of one subscription (see the native client's `Subscription`):
/// only its own kind, plus connection events; events dropped because the
/// receiver fell behind arrive as [`WsEvent::Missed`].
pub struct Subscription {
    rx: futures::channel::mpsc::Receiver<WsEvent>,
    missed: Arc<AtomicU64>,
}

impl Subscription {
    /// The next event; `None` once the client is dropped.
    pub async fn recv(&mut self) -> Option<WsEvent> {
        let missed = self.missed.swap(0, Ordering::Relaxed);
        if missed > 0 {
            return Some(WsEvent::Missed(missed));
        }
        futures::StreamExt::next(&mut self.rx).await
    }

    /// The events as a [`futures::Stream`] (see the native client's
    /// `Subscription::into_stream`). It ends when the client is dropped.
    pub fn into_stream(self) -> impl futures::Stream<Item = WsEvent> {
        futures::stream::unfold(self, |mut subscription| async move {
            subscription.recv().await.map(|event| (event, subscription))
        })
    }
}

impl std::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription").finish_non_exhaustive()
    }
}

impl std::fmt::Debug for XrplWsClient {
    /// The server's host (never the URL, which may carry a token), whether
    /// connected, and the fee cap.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XrplWsClient")
            .field("host", &crate::host_of(&self.url))
            .field("connected", &self.is_connected())
            .field("max_fee_drops", &self.max_fee_drops)
            .finish()
    }
}

/// WebSocket client for XRPL nodes (WASM version using browser WebSocket API).
pub struct XrplWsClient {
    ws: WebSocket,
    pending: PendingMap,
    id_counter: Arc<AtomicU64>,
    url: String,
    connected: Arc<AtomicBool>,
    active_subs: Arc<Mutex<Vec<Value>>>,
    subscribers: Subscribers,
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
    ///
    /// # Errors
    ///
    /// [`ClientError::UnexpectedResponse`] when the browser cannot create the
    /// WebSocket (a malformed URL, for one) or the connection fails before
    /// it opens.
    pub async fn connect(url: &str) -> Result<Self, ClientError> {
        // The browser's error may name the URL, which may carry a token: only
        // the host is reported.
        let ws = WebSocket::new(url).map_err(|_| {
            ClientError::transport(
                crate::error::TransportErrorKind::Connect,
                format!("the browser refused a WebSocket to {}", crate::host_of(url)),
            )
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
                    let _ = tx.send(Err(ClientError::transport(
                        crate::error::TransportErrorKind::Connect,
                        "the WebSocket did not open",
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
        let subscribers: Subscribers = Arc::new(Mutex::new(Vec::new()));
        let subscribers_msg = subscribers.clone();
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
            } else if let Some(event) = WsEvent::from_message(value) {
                deliver(&subscribers_msg, &event);
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
        let pending_close = pending.clone();
        let subscribers_close = subscribers.clone();
        let on_close = Closure::wrap(Box::new(move |_: web_sys::CloseEvent| {
            connected_close.store(false, Ordering::Relaxed);
            if let Ok(mut map) = pending_close.lock() {
                for (_, sender) in map.drain() {
                    let _ = sender.send(Err(ClientError::Disconnected));
                }
            }
            deliver(&subscribers_close, &WsEvent::Disconnected);
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
            subscribers,
            max_fee_drops: crate::client::DEFAULT_MAX_FEE_DROPS,
            _on_message: on_message,
            _on_error: on_error,
            _on_close: on_close,
        })
    }

    fn subscription(
        &self,
        filter: impl Fn(&WsEvent) -> bool + Send + Sync + 'static,
    ) -> Subscription {
        let (tx, rx) = futures::channel::mpsc::channel(EVENT_BUFFER);
        let missed = Arc::new(AtomicU64::new(0));
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.push(Subscriber {
                filter: Box::new(filter),
                tx,
                missed: missed.clone(),
            });
        }
        Subscription { rx, missed }
    }

    async fn subscribe_with(
        &self,
        params: Value,
        filter: impl Fn(&WsEvent) -> bool + Send + Sync + 'static,
    ) -> Result<Subscription, ClientError> {
        let events = self.subscription(filter);
        self.raw_request("subscribe", params.clone()).await?;
        let mut subs = self
            .active_subs
            .lock()
            .map_err(|_| ClientError::UnexpectedResponse("mutex lock poisoned".into()))?;
        if !subs.contains(&params) {
            subs.push(params);
        }
        Ok(events)
    }

    /// Subscribe to ledger closes ([`WsEvent::Ledger`] only).
    ///
    /// # Errors
    ///
    /// [`ClientError::RpcError`] when the server refuses the subscription;
    /// [`ClientError::Disconnected`] when the connection is closed, or closes
    /// before the reply; and [`ClientError::Timeout`] after 30 s without a
    /// reply.
    pub async fn subscribe_ledger(&self) -> Result<Subscription, ClientError> {
        self.subscribe_with(json!({"streams": ["ledger"]}), |e| {
            matches!(e, WsEvent::Ledger(_))
        })
        .await
    }

    /// Subscribe to every validated transaction ([`WsEvent::Transaction`],
    /// `validated: true`, only).
    ///
    /// # Errors
    ///
    /// The same as [`XrplWsClient::subscribe_ledger`].
    pub async fn subscribe_transactions(&self) -> Result<Subscription, ClientError> {
        self.subscribe_with(
            json!({"streams": ["transactions"]}),
            |e| matches!(e, WsEvent::Transaction(t) if t.validated),
        )
        .await
    }

    /// Subscribe to the validated transactions that affect `account` (only those).
    ///
    /// # Errors
    ///
    /// The same as [`XrplWsClient::subscribe_ledger`] ([`ClientError::RpcError`]
    /// `actMalformed` for an address that is not valid).
    pub async fn subscribe_account(&self, account: &str) -> Result<Subscription, ClientError> {
        let who = account.to_string();
        self.subscribe_with(
            json!({"accounts": [account]}),
            move |e| matches!(e, WsEvent::Transaction(t) if t.validated && t.affects(&who)),
        )
        .await
    }

    /// Every event the connection carries, from now on.
    pub fn events(&self) -> Subscription {
        self.subscription(|_| true)
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

impl Drop for XrplWsClient {
    fn drop(&mut self) {
        self.ws.set_onmessage(None);
        self.ws.set_onerror(None);
        self.ws.set_onclose(None);
        let _ = self.ws.close();
        if let Ok(mut map) = self.pending.lock() {
            for (_, sender) in map.drain() {
                let _ = sender.send(Err(ClientError::Disconnected));
            }
        }
    }
}

impl Transport for XrplWsClient {
    fn max_fee_drops(&self) -> u64 {
        self.max_fee_drops
    }

    fn send_raw<'a>(
        &'a self,
        command: &'a str,
        params: Value,
    ) -> crate::client::TransportFuture<'a> {
        Box::pin(async move {
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

            let params = crate::client::with_api_version(params);
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

            if self.ws.send_with_str(&msg.to_string()).is_err() {
                if let Ok(mut map) = self.pending.lock() {
                    map.remove(&id);
                }
                return Err(ClientError::Disconnected);
            }

            let timeout = Box::pin(crate::client::sleep_secs(REQUEST_TIMEOUT_SECS));
            match futures::future::select(rx, timeout).await {
                futures::future::Either::Left((reply, _)) => {
                    reply.map_err(|_| ClientError::Disconnected)?
                }
                futures::future::Either::Right(_) => {
                    if let Ok(mut map) = self.pending.lock() {
                        map.remove(&id);
                    }
                    Err(ClientError::Timeout(std::time::Duration::from_secs(
                        REQUEST_TIMEOUT_SECS,
                    )))
                }
            }
        })
    }
}
