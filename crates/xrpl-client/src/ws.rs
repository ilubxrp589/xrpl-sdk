//! [`XrplWsClient`] for native targets: requests and subscriptions over one
//! WebSocket, matched by request ID, with automatic reconnection that restores
//! subscriptions.
//!
//! One supervisor task owns the connection: it writes requests, routes
//! replies and stream messages, pings the server, and reconnects (with
//! backoff) when the connection drops or goes silent. Dropping the client
//! stops it and closes the socket.

use crate::client::Transport;
use crate::error::ClientError;
use crate::host_of;
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc, oneshot, watch, Mutex};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::MaybeTlsStream;

type WsStream = tokio_tungstenite::WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

pub use crate::stream::*;

/// The ping's request ID; its replies are not routed.
const PING_ID: u64 = u64::MAX;

/// The settings of an [`XrplWsClient`], from [`XrplWsClient::builder`].
///
/// ```no_run
/// # async fn run() -> Result<(), xrpl_client::ClientError> {
/// use std::time::Duration;
/// use xrpl_client::XrplWsClient;
///
/// let client = XrplWsClient::builder("wss://xrplcluster.com")
///     .request_timeout(Duration::from_secs(10))
///     .ping_interval(Duration::from_secs(5))
///     .connect()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct WsClientBuilder {
    url: String,
    request_timeout: Duration,
    connect_timeout: Duration,
    ping_interval: Duration,
    silence_limit: Duration,
    event_buffer: usize,
    max_fee_drops: u64,
}

impl std::fmt::Debug for WsClientBuilder {
    /// The host only: a URL may carry a token.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsClientBuilder")
            .field("host", &host_of(&self.url))
            .field("request_timeout", &self.request_timeout)
            .field("connect_timeout", &self.connect_timeout)
            .field("ping_interval", &self.ping_interval)
            .field("silence_limit", &self.silence_limit)
            .field("event_buffer", &self.event_buffer)
            .field("max_fee_drops", &self.max_fee_drops)
            .finish()
    }
}

impl WsClientBuilder {
    /// How long a request waits for its reply (default 30 s), connection
    /// included while the client reconnects.
    ///
    /// ```
    /// use std::time::Duration;
    /// use xrpl_client::XrplWsClient;
    ///
    /// let builder = XrplWsClient::builder("wss://s.altnet.rippletest.net:51233")
    ///     .request_timeout(Duration::from_secs(10));
    /// assert!(format!("{builder:?}").contains("request_timeout: 10s"));
    /// // `builder.connect().await?` opens the connection with this setting.
    /// ```
    #[must_use]
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// How long opening a connection may take (default 15 s).
    ///
    /// ```
    /// use std::time::Duration;
    /// use xrpl_client::XrplWsClient;
    ///
    /// let builder = XrplWsClient::builder("wss://s.altnet.rippletest.net:51233")
    ///     .connect_timeout(Duration::from_secs(5));
    /// assert!(format!("{builder:?}").contains("connect_timeout: 5s"));
    /// // `builder.connect().await?` opens the connection with this setting.
    /// ```
    #[must_use]
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// How often the server is pinged (default 15 s).
    ///
    /// ```
    /// use std::time::Duration;
    /// use xrpl_client::XrplWsClient;
    ///
    /// let builder = XrplWsClient::builder("wss://s.altnet.rippletest.net:51233")
    ///     .ping_interval(Duration::from_secs(5));
    /// assert!(format!("{builder:?}").contains("ping_interval: 5s"));
    /// // `builder.connect().await?` opens the connection with this setting.
    /// ```
    #[must_use]
    pub fn ping_interval(mut self, interval: Duration) -> Self {
        self.ping_interval = interval;
        self
    }

    /// How long a connection may say nothing before it is taken as dead (a
    /// laptop that slept, a NAT that dropped it) and replaced (default 45 s;
    /// make it longer than the ping interval).
    ///
    /// ```
    /// use std::time::Duration;
    /// use xrpl_client::XrplWsClient;
    ///
    /// // For a server that may stay quiet for a while.
    /// let builder = XrplWsClient::builder("wss://s.altnet.rippletest.net:51233")
    ///     .silence_limit(Duration::from_secs(90));
    /// assert!(format!("{builder:?}").contains("silence_limit: 90s"));
    /// // `builder.connect().await?` opens the connection with this setting.
    /// ```
    #[must_use]
    pub fn silence_limit(mut self, limit: Duration) -> Self {
        self.silence_limit = limit;
        self
    }

    /// Events buffered for each subscription before the oldest are dropped
    /// and reported as [`WsEvent::Missed`] (default 1,024; at least 1).
    ///
    /// ```
    /// use xrpl_client::XrplWsClient;
    ///
    /// let builder = XrplWsClient::builder("wss://s.altnet.rippletest.net:51233")
    ///     .event_buffer(4096);
    /// assert!(format!("{builder:?}").contains("event_buffer: 4096"));
    /// // Never less than one.
    /// let builder = XrplWsClient::builder("wss://s.altnet.rippletest.net:51233")
    ///     .event_buffer(0);
    /// assert!(format!("{builder:?}").contains("event_buffer: 1"));
    /// ```
    #[must_use]
    pub fn event_buffer(mut self, events: usize) -> Self {
        self.event_buffer = events.max(1);
        self
    }

    /// The most [`Client::autofill`](crate::Client::autofill) sets as a `Fee` (default
    /// [`crate::client::DEFAULT_MAX_FEE_DROPS`], 2 XRP).
    ///
    /// ```
    /// use xrpl_client::XrplWsClient;
    ///
    /// let builder = XrplWsClient::builder("wss://s.altnet.rippletest.net:51233")
    ///     .max_fee_drops(100_000);
    /// assert!(format!("{builder:?}").contains("max_fee_drops: 100000"));
    /// // `builder.connect().await?` opens the connection with this setting.
    /// ```
    #[must_use]
    pub fn max_fee_drops(mut self, drops: u64) -> Self {
        self.max_fee_drops = drops;
        self
    }

    /// Connect, with automatic reconnection.
    ///
    /// ```no_run
    /// # async fn run() -> Result<(), xrpl_client::ClientError> {
    /// use std::time::Duration;
    /// use xrpl_client::XrplWsClient;
    ///
    /// let client = XrplWsClient::builder("wss://s.altnet.rippletest.net:51233")
    ///     .connect_timeout(Duration::from_secs(5))
    ///     .connect()
    ///     .await?;
    /// println!("connected: {}", client.is_connected());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`ClientError::Timeout`] when the connection does not open within the
    /// connect timeout, and [`ClientError::UnexpectedResponse`] when it is
    /// refused.
    pub async fn connect(self) -> Result<XrplWsClient, ClientError> {
        let settings = Settings {
            request_timeout: self.request_timeout,
            connect_timeout: self.connect_timeout,
            ping_interval: self.ping_interval,
            silence_limit: self.silence_limit,
        };
        let stream = open(&self.url, settings.connect_timeout).await?;
        let (event_tx, _) = broadcast::channel(self.event_buffer);
        let (shutdown, shutdown_rx) = watch::channel(false);
        let shared = Arc::new(Shared {
            pending: Mutex::new(HashMap::new()),
            event_tx,
            ids: AtomicU64::new(1),
            connected: AtomicBool::new(false),
            write_tx: Mutex::new(None),
            active_subs: Mutex::new(Vec::new()),
            started: Instant::now(),
            last_heard: AtomicU64::new(0),
            host: host_of(&self.url),
            shutdown: shutdown_rx,
            settings,
        });
        tokio::spawn(supervise(shared.clone(), self.url.clone(), stream));
        // The supervisor marks the connection up before it reads; wait for it.
        let waited = Instant::now();
        while !shared.connected.load(Ordering::Relaxed)
            && waited.elapsed() < settings.connect_timeout
        {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        Ok(XrplWsClient {
            shared,
            url: self.url,
            max_fee_drops: self.max_fee_drops,
            shutdown,
        })
    }
}

/// The timing a connection runs with.
#[derive(Debug, Clone, Copy)]
struct Settings {
    request_timeout: Duration,
    connect_timeout: Duration,
    ping_interval: Duration,
    silence_limit: Duration,
}

/// What the client and its supervisor task share.
struct Shared {
    pending: Mutex<HashMap<u64, oneshot::Sender<Result<Value, ClientError>>>>,
    event_tx: broadcast::Sender<WsEvent>,
    ids: AtomicU64,
    connected: AtomicBool,
    write_tx: Mutex<Option<mpsc::Sender<String>>>,
    active_subs: Mutex<Vec<Value>>,
    started: Instant,
    /// Milliseconds since `started` when the server last said anything.
    last_heard: AtomicU64,
    /// The server's host, for logs (never the URL: it may carry a token).
    host: String,
    shutdown: watch::Receiver<bool>,
    settings: Settings,
}

impl Shared {
    fn touch(&self) {
        let now = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.last_heard.store(now, Ordering::Relaxed);
    }

    fn silent_for(&self) -> Duration {
        let heard = Duration::from_millis(self.last_heard.load(Ordering::Relaxed));
        self.started.elapsed().saturating_sub(heard)
    }

    async fn fail_pending(&self) {
        for (_, sender) in self.pending.lock().await.drain() {
            let _ = sender.send(Err(ClientError::Disconnected));
        }
    }

    /// Send a request and wait for its reply (waiting up to the request
    /// timeout for a connection, while the supervisor reconnects).
    async fn request(&self, command: &str, params: Value) -> Result<Value, ClientError> {
        let waited = Instant::now();
        while !self.connected.load(Ordering::Relaxed) {
            if *self.shutdown.borrow() || waited.elapsed() >= self.settings.request_timeout {
                return Err(ClientError::Disconnected);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let id = self.ids.fetch_add(1, Ordering::Relaxed);
        tracing::debug!(command, id, host = %self.host, "sending WS request");
        let mut msg = match crate::client::with_api_version(params) {
            Value::Object(obj) => obj,
            _ => serde_json::Map::new(),
        };
        msg.insert("command".into(), Value::String(command.into()));
        msg.insert("id".into(), Value::Number(id.into()));

        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);
        let writer = self.write_tx.lock().await.clone();
        let sent = match writer {
            Some(w) => w.send(Value::Object(msg).to_string()).await.is_ok(),
            None => false,
        };
        if !sent {
            self.pending.lock().await.remove(&id);
            return Err(ClientError::Disconnected);
        }

        match tokio::time::timeout(self.settings.request_timeout, rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(ClientError::Disconnected),
            Err(_) => {
                self.pending.lock().await.remove(&id);
                Err(ClientError::Timeout(self.settings.request_timeout))
            }
        }
    }

    /// A reply goes to its request; a stream message to the subscribers.
    async fn route(&self, text: &str) {
        let Ok(value) = serde_json::from_str::<Value>(text) else {
            tracing::warn!(host = %self.host, "a WebSocket message that is not JSON");
            return;
        };
        match value.get("id").and_then(Value::as_u64) {
            Some(PING_ID) => {}
            Some(id) => {
                if let Some(sender) = self.pending.lock().await.remove(&id) {
                    let _ = sender.send(crate::client::ws_response(&value));
                }
            }
            None => {
                if let Some(event) = WsEvent::from_message(value) {
                    let _ = self.event_tx.send(event);
                }
            }
        }
    }
}

async fn open(url: &str, timeout: Duration) -> Result<WsStream, ClientError> {
    match tokio::time::timeout(timeout, tokio_tungstenite::connect_async(url)).await {
        Ok(Ok((stream, _))) => Ok(stream),
        Ok(Err(e)) => Err(ClientError::transport(
            crate::error::TransportErrorKind::Connect,
            format!("WebSocket to {}: {e}", host_of(url)),
        )),
        Err(_) => Err(ClientError::Timeout(timeout)),
    }
}

/// The supervisor: runs a connection until it ends, then reconnects with
/// backoff (1 s doubling to 60 s), restoring subscriptions, until shutdown.
async fn supervise(shared: Arc<Shared>, url: String, first: WsStream) {
    let mut shutdown = shared.shutdown.clone();
    let mut stream = Some(first);
    let mut reconnect = false;
    let mut attempt: u32 = 0;
    loop {
        if let Some(ws) = stream.take() {
            attempt = 0;
            run_connection(&shared, ws, &mut shutdown, reconnect).await;
            shared.connected.store(false, Ordering::Relaxed);
            *shared.write_tx.lock().await = None;
            shared.fail_pending().await;
            if *shutdown.borrow() {
                return;
            }
            tracing::warn!(host = %shared.host, "WebSocket disconnected; reconnecting");
            let _ = shared.event_tx.send(WsEvent::Disconnected);
            reconnect = true;
        }
        let delay = Duration::from_secs(1u64 << attempt.min(6)).min(Duration::from_secs(60));
        attempt = attempt.saturating_add(1);
        tokio::select! {
            _ = tokio::time::sleep(delay) => {}
            _ = shutdown.changed() => return,
        }
        match open(&url, shared.settings.connect_timeout).await {
            Ok(ws) => stream = Some(ws),
            Err(e) => {
                tracing::warn!(host = %shared.host, attempt, error = %e, "WebSocket reconnect failed")
            }
        }
    }
}

/// One connection's life: writes, reads, pings; returns when it ends, goes
/// silent, or the client is dropped.
async fn run_connection(
    shared: &Arc<Shared>,
    ws: WsStream,
    shutdown: &mut watch::Receiver<bool>,
    reconnect: bool,
) {
    let (mut sink, mut source) = ws.split();
    let (write_tx, mut write_rx) = mpsc::channel::<String>(64);
    *shared.write_tx.lock().await = Some(write_tx);
    shared.touch();
    shared.connected.store(true, Ordering::Relaxed);
    if reconnect {
        tokio::spawn(resubscribe(shared.clone()));
    }
    let mut ping = tokio::time::interval(shared.settings.ping_interval);
    ping.tick().await;
    loop {
        tokio::select! {
            _ = shutdown.changed() => {
                let _ = sink.send(Message::Close(None)).await;
                return;
            }
            Some(text) = write_rx.recv() => {
                if sink.send(Message::Text(text.into())).await.is_err() {
                    return;
                }
            }
            _ = ping.tick() => {
                let limit = shared.settings.silence_limit;
                if shared.silent_for() > limit {
                    tracing::warn!(host = %shared.host, "WebSocket silent for {:?}; replacing it", limit);
                    return;
                }
                let ping = json!({"command": "ping", "id": PING_ID}).to_string();
                if sink.send(Message::Text(ping.into())).await.is_err() {
                    return;
                }
            }
            message = source.next() => match message {
                Some(Ok(Message::Text(text))) => {
                    shared.touch();
                    shared.route(&text).await;
                }
                Some(Ok(Message::Close(_))) | Some(Err(_)) | None => return,
                Some(Ok(_)) => shared.touch(),
            },
        }
    }
}

/// After a reconnect: subscribe again to everything, then report it. A
/// subscription the server refuses is reported, not skipped silently.
async fn resubscribe(shared: Arc<Shared>) {
    let subs = shared.active_subs.lock().await.clone();
    let mut restored = true;
    for params in subs {
        if let Err(e) = shared.request("subscribe", params.clone()).await {
            restored = false;
            tracing::warn!(host = %shared.host, error = %e, "a subscription was not restored");
            let _ = shared.event_tx.send(WsEvent::ResubscribeFailed {
                subscription: params,
                error: e.to_string(),
            });
        }
    }
    if restored {
        tracing::info!(host = %shared.host, "WebSocket reconnected; subscriptions restored");
    }
    let _ = shared.event_tx.send(WsEvent::Reconnected);
}

/// WebSocket client for XRPL nodes with automatic reconnection.
///
/// On disconnect it sends [`WsEvent::Disconnected`], reconnects with
/// exponential backoff, subscribes again to everything (a refusal is a
/// [`WsEvent::ResubscribeFailed`]) and sends [`WsEvent::Reconnected`].
/// Events between the two were not seen; catch up with `account_tx` or
/// `ledger` if they matter. A connection that goes silent is replaced the
/// same way. Dropping the client closes the connection and stops its task.
///
/// ```no_run
/// # async fn run() -> Result<(), xrpl_client::ClientError> {
/// use xrpl_client::stream::WsEvent;
/// use xrpl_client::XrplWsClient;
///
/// let client = XrplWsClient::connect("wss://s.altnet.rippletest.net:51233").await?.with_max_fee_drops(100_000);
/// let mut ledgers = client.subscribe_ledger().await?;
/// while let Some(event) = ledgers.recv().await {
///     match event {
///         WsEvent::Ledger(ledger) => println!("ledger {} closed", ledger.ledger_index),
///         WsEvent::Disconnected => println!("reconnecting…"),
///         _ => {}
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct XrplWsClient {
    shared: Arc<Shared>,
    url: String,
    max_fee_drops: u64,
    shutdown: watch::Sender<bool>,
}

impl std::fmt::Debug for XrplWsClient {
    /// The server's host (never the URL, which may carry a token), whether
    /// connected, and the fee cap.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XrplWsClient")
            .field("host", &self.shared.host)
            .field("connected", &self.is_connected())
            .field("max_fee_drops", &self.max_fee_drops)
            .finish()
    }
}

impl Drop for XrplWsClient {
    fn drop(&mut self) {
        let _ = self.shutdown.send(true);
    }
}

impl XrplWsClient {
    /// The same client with `drops` as the most [`Client::autofill`](crate::Client::autofill) sets as
    /// a `Fee` (default [`crate::client::DEFAULT_MAX_FEE_DROPS`], 2 XRP).
    pub fn with_max_fee_drops(mut self, drops: u64) -> Self {
        self.max_fee_drops = drops;
        self
    }

    /// Connect to an XRPL WebSocket endpoint (within 15 s) with automatic
    /// reconnection and the default settings (see [`XrplWsClient::builder`]).
    ///
    /// ```no_run
    /// # async fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::{Client, XrplWsClient};
    ///
    /// let client = XrplWsClient::connect("wss://s.altnet.rippletest.net:51233").await?;
    /// // Every `Client` method works over the connection.
    /// println!("open ledger fee: {} drops", client.fee().await?.drops.open_ledger_fee);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`ClientError::Timeout`] when the connection does not open within
    /// 15 s, and [`ClientError::UnexpectedResponse`] when it is refused.
    pub async fn connect(url: &str) -> Result<Self, ClientError> {
        Self::builder(url).connect().await
    }

    /// Settings for a client of the server at `url`: request and connect
    /// timeouts, the ping interval, the silence limit, the event buffer and
    /// the fee cap.
    ///
    /// ```
    /// use xrpl_client::XrplWsClient;
    ///
    /// let builder = XrplWsClient::builder("wss://s.altnet.rippletest.net:51233");
    /// // The defaults: 30 s for a request, a ping every 15 s.
    /// let settings = format!("{builder:?}");
    /// assert!(settings.contains("request_timeout: 30s") && settings.contains("ping_interval: 15s"));
    /// ```
    pub fn builder(url: &str) -> WsClientBuilder {
        WsClientBuilder {
            url: url.to_string(),
            request_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(15),
            ping_interval: Duration::from_secs(15),
            silence_limit: Duration::from_secs(45),
            event_buffer: 1024,
            max_fee_drops: crate::client::DEFAULT_MAX_FEE_DROPS,
        }
    }

    /// Close the connection and stop reconnecting (the same as dropping the
    /// client). Requests in flight fail with [`ClientError::Disconnected`].
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// let mut ledgers = client.subscribe_ledger().await?;
    /// println!("{:?}", ledgers.recv().await);
    /// client.close();
    /// // The subscription ends once it has handed over what had already arrived.
    /// while let Some(event) = ledgers.recv().await {
    ///     println!("{event:?}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn close(self) {}

    fn subscription(
        &self,
        filter: impl Fn(&WsEvent) -> bool + Send + Sync + 'static,
    ) -> Subscription {
        Subscription {
            rx: self.shared.event_tx.subscribe(),
            filter: Box::new(filter),
        }
    }

    async fn subscribe_with(
        &self,
        params: Value,
        filter: impl Fn(&WsEvent) -> bool + Send + Sync + 'static,
    ) -> Result<(Value, Subscription), ClientError> {
        // Listen before asking, so no event between the reply and the
        // receiver is lost.
        let events = self.subscription(filter);
        let reply = self.shared.request("subscribe", params.clone()).await?;
        let mut subs = self.shared.active_subs.lock().await;
        if !subs.contains(&params) {
            subs.push(params);
        }
        Ok((reply, events))
    }

    /// Subscribe with any `subscribe` parameters (`streams`, `accounts`,
    /// `accounts_proposed`, `books`, …), returning `rippled`'s reply (an order
    /// book snapshot, for instance) and a [`Subscription`] to **every** event
    /// the connection carries (unfiltered: other subscriptions' too). The
    /// subscription is restored after a reconnect.
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// use serde_json::json;
    ///
    /// // Ledger closes and one account's transactions, on one receiver.
    /// let (reply, mut events) = client
    ///     .subscribe(json!({"streams": ["ledger"], "accounts": ["rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"]}))
    ///     .await?;
    /// println!("validated ledger {}", reply["ledger_index"]);
    /// while let Some(event) = events.recv().await {
    ///     println!("{event:?}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`ClientError::RpcError`] when the server refuses the subscription;
    /// [`ClientError::Disconnected`] when there is no connection within the
    /// request timeout, or it drops before the reply; and
    /// [`ClientError::Timeout`] when no reply comes within the request
    /// timeout.
    pub async fn subscribe(&self, params: Value) -> Result<(Value, Subscription), ClientError> {
        self.subscribe_with(params, |_| true).await
    }

    /// Undo a [`XrplWsClient::subscribe`] with the same parameters.
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// use serde_json::json;
    ///
    /// let params = json!({"streams": ["server"]});
    /// let (_reply, _events) = client.subscribe(params.clone()).await?;
    /// client.unsubscribe(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// The same as [`XrplWsClient::subscribe`], for the `unsubscribe`
    /// request.
    pub async fn unsubscribe(&self, params: Value) -> Result<(), ClientError> {
        self.shared.request("unsubscribe", params.clone()).await?;
        self.shared
            .active_subs
            .lock()
            .await
            .retain(|s| s != &params);
        Ok(())
    }

    async fn subscribe_stream(
        &self,
        stream: &str,
        filter: impl Fn(&WsEvent) -> bool + Send + Sync + 'static,
    ) -> Result<Subscription, ClientError> {
        Ok(self
            .subscribe_with(json!({ "streams": [stream] }), filter)
            .await?
            .1)
    }

    /// Subscribe to ledger closes ([`WsEvent::Ledger`] only).
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// let mut ledgers = client.subscribe_ledger().await?;
    /// while let Some(event) = ledgers.recv().await {
    ///     println!("{event:?}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// The same as [`XrplWsClient::subscribe`].
    pub async fn subscribe_ledger(&self) -> Result<Subscription, ClientError> {
        self.subscribe_stream("ledger", |e| matches!(e, WsEvent::Ledger(_)))
            .await
    }

    /// Subscribe to validations from every validator the server hears
    /// ([`WsEvent::Validation`] only).
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::stream::WsEvent;
    ///
    /// let mut validations = client.subscribe_validations().await?;
    /// while let Some(event) = validations.recv().await {
    ///     if let WsEvent::Validation(v) = event {
    ///         println!("{} validated {:?}", v.validation_public_key, v.ledger_index);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// The same as [`XrplWsClient::subscribe`].
    pub async fn subscribe_validations(&self) -> Result<Subscription, ClientError> {
        self.subscribe_stream("validations", |e| matches!(e, WsEvent::Validation(_)))
            .await
    }

    /// Subscribe to the server's load and fee changes ([`WsEvent::Server`] only).
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::stream::WsEvent;
    ///
    /// let mut status = client.subscribe_server().await?;
    /// while let Some(event) = status.recv().await {
    ///     if let WsEvent::Server(s) = event {
    ///         println!("{}: load factor {}/{}", s.server_status, s.load_factor, s.load_base);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// The same as [`XrplWsClient::subscribe`].
    pub async fn subscribe_server(&self) -> Result<Subscription, ClientError> {
        self.subscribe_stream("server", |e| matches!(e, WsEvent::Server(_)))
            .await
    }

    /// Subscribe to consensus phase changes ([`WsEvent::Consensus`] only).
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::stream::WsEvent;
    ///
    /// let mut phases = client.subscribe_consensus().await?;
    /// while let Some(event) = phases.recv().await {
    ///     if let WsEvent::Consensus(c) = event {
    ///         println!("consensus: {}", c.consensus);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// The same as [`XrplWsClient::subscribe`].
    pub async fn subscribe_consensus(&self) -> Result<Subscription, ClientError> {
        self.subscribe_stream("consensus", |e| matches!(e, WsEvent::Consensus(_)))
            .await
    }

    /// Subscribe to each validated ledger's order-book trading
    /// ([`WsEvent::BookChanges`] only).
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::stream::WsEvent;
    ///
    /// let mut trading = client.subscribe_book_changes().await?;
    /// while let Some(event) = trading.recv().await {
    ///     if let WsEvent::BookChanges(b) = event {
    ///         println!("ledger {}: {} books traded", b.ledger_index, b.changes.len());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// The same as [`XrplWsClient::subscribe`].
    pub async fn subscribe_book_changes(&self) -> Result<Subscription, ClientError> {
        self.subscribe_stream("book_changes", |e| matches!(e, WsEvent::BookChanges(_)))
            .await
    }

    /// Subscribe to validator manifests ([`WsEvent::Manifest`] only).
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::stream::WsEvent;
    ///
    /// let mut manifests = client.subscribe_manifests().await?;
    /// while let Some(event) = manifests.recv().await {
    ///     if let WsEvent::Manifest(m) = event {
    ///         println!("{} now signs with {}", m.master_key, m.signing_key);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// The same as [`XrplWsClient::subscribe`].
    pub async fn subscribe_manifests(&self) -> Result<Subscription, ClientError> {
        self.subscribe_stream("manifests", |e| matches!(e, WsEvent::Manifest(_)))
            .await
    }

    /// Subscribe to an order book: transactions ([`WsEvent::Transaction`];
    /// the receiver does not tell this book's from another book
    /// subscription's). `taker_gets` / `taker_pays` are `{"currency": "XRP"}`
    /// or `{"currency", "issuer"}`. With `snapshot`, returns the book's
    /// current offers first (`both` adds the other side).
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_core::types::Asset;
    ///
    /// // Offers that give XRP for USD issued by rHb9CJAW…, with the current ones first.
    /// let usd = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
    /// let (snapshot, mut trades) = client.subscribe_order_book(&Asset::xrp(), &usd, true, false).await?;
    /// println!("{} offers now", snapshot.offers.map_or(0, |o| o.len()));
    /// while let Some(event) = trades.recv().await {
    ///     println!("{event:?}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// The same as [`XrplWsClient::subscribe`] (the server refuses a book it
    /// cannot read), and [`ClientError::Json`] when the reply does not decode
    /// as an [`OrderBookSnapshot`].
    pub async fn subscribe_order_book(
        &self,
        taker_gets: &xrpl_core::types::Asset,
        taker_pays: &xrpl_core::types::Asset,
        snapshot: bool,
        both: bool,
    ) -> Result<(OrderBookSnapshot, Subscription), ClientError> {
        let book = json!({"taker_gets": taker_gets, "taker_pays": taker_pays, "snapshot": snapshot, "both": both});
        let (reply, events) = self
            .subscribe_with(json!({ "books": [book] }), |e| {
                matches!(e, WsEvent::Transaction(_))
            })
            .await?;
        Ok((serde_json::from_value(reply)?, events))
    }

    /// Subscribe to every validated transaction on the network
    /// ([`WsEvent::Transaction`] with `validated: true` only).
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::stream::WsEvent;
    ///
    /// let mut validated = client.subscribe_transactions().await?;
    /// while let Some(event) = validated.recv().await {
    ///     if let WsEvent::Transaction(tx) = event {
    ///         println!("{:?}: {}", tx.hash, tx.engine_result);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// The same as [`XrplWsClient::subscribe`].
    pub async fn subscribe_transactions(&self) -> Result<Subscription, ClientError> {
        self.subscribe_stream(
            "transactions",
            |e| matches!(e, WsEvent::Transaction(t) if t.validated),
        )
        .await
    }

    /// Subscribe to transactions as they are proposed, before validation
    /// (preliminary results; [`WsEvent::Transaction`], `validated: false`).
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::stream::WsEvent;
    ///
    /// let mut proposed = client.subscribe_transactions_proposed().await?;
    /// while let Some(event) = proposed.recv().await {
    ///     if let WsEvent::Transaction(tx) = event {
    ///         println!("{:?}: {} (validated: {})", tx.hash, tx.engine_result, tx.validated);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// The same as [`XrplWsClient::subscribe`].
    pub async fn subscribe_transactions_proposed(&self) -> Result<Subscription, ClientError> {
        self.subscribe_stream("transactions_proposed", |e| {
            matches!(e, WsEvent::Transaction(_))
        })
        .await
    }

    /// Subscribe to the validated transactions that affect `account`: the
    /// receiver gets only those ([`WsEvent::Transaction`] with
    /// `validated: true` naming the account in the transaction or its
    /// metadata), never another subscription's.
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::stream::WsEvent;
    ///
    /// let mut payments = client.subscribe_account("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe").await?;
    /// while let Some(event) = payments.recv().await {
    ///     if let WsEvent::Transaction(tx) = event {
    ///         println!("{:?} delivered {:?}", tx.hash, tx.delivered_amount());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// The same as [`XrplWsClient::subscribe`] ([`ClientError::RpcError`]
    /// `actMalformed` for an address that is not valid).
    pub async fn subscribe_account(&self, account: &str) -> Result<Subscription, ClientError> {
        let who = account.to_string();
        Ok(self
            .subscribe_with(
                json!({ "accounts": [account] }),
                move |e| matches!(e, WsEvent::Transaction(t) if t.validated && t.affects(&who)),
            )
            .await?
            .1)
    }

    /// Unsubscribe from ledger closes.
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// let mut ledgers = client.subscribe_ledger().await?;
    /// let first = ledgers.recv().await;
    /// println!("{first:?}");
    /// client.unsubscribe_ledger().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// The same as [`XrplWsClient::unsubscribe`].
    pub async fn unsubscribe_ledger(&self) -> Result<(), ClientError> {
        self.unsubscribe(json!({"streams": ["ledger"]})).await
    }

    /// Search for payment paths and keep the answer up to date: the first
    /// answer, and a [`Subscription`] to the updates ([`WsEvent::PathFind`],
    /// one per ledger while the paths change). One search per connection: a
    /// new one replaces the last; [`XrplWsClient::path_find_close`] ends it.
    /// A reconnect ends it too (search again after
    /// [`WsEvent::Reconnected`]).
    ///
    /// ```no_run
    /// # async fn run(ws: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::requests::PathFindRequest;
    /// use xrpl_client::stream::WsEvent;
    ///
    /// // Pay 10 USD (issued by rHb9…) to rPT1…, from rU6K…'s funds.
    /// let usd = serde_json::json!({"currency": "USD", "issuer": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "value": "10"});
    /// let search = PathFindRequest::new(
    ///     "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1",
    ///     "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
    ///     usd,
    /// );
    /// let (first, mut updates) = ws.path_find(&search).await?;
    /// println!("{} ways to pay", first.alternatives.len());
    /// while let Some(WsEvent::PathFind(update)) = updates.recv().await {
    ///     if update.full_reply {
    ///         println!("best: {:?}", update.alternatives.first().map(|a| &a.source_amount));
    ///         break;
    ///     }
    /// }
    /// ws.path_find_close().await?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// [`ClientError::RpcError`] when the server refuses the search (an
    /// account it does not know, an amount it cannot read), the errors of
    /// [`XrplWsClient::subscribe`] for the connection, and
    /// [`ClientError::Json`] when the answer does not decode.
    pub async fn path_find(
        &self,
        request: &crate::requests::PathFindRequest,
    ) -> Result<(crate::types::PathFindResult, Subscription), ClientError> {
        // Listen before asking, so no update between the reply and the
        // receiver is lost.
        let updates = self.subscription(|e| matches!(e, WsEvent::PathFind(_)));
        let reply = self
            .shared
            .request("path_find", serde_json::to_value(request)?)
            .await?;
        Ok((serde_json::from_value(reply)?, updates))
    }

    /// End the connection's [`XrplWsClient::path_find`] search.
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient, request: xrpl_client::requests::PathFindRequest) -> Result<(), xrpl_client::ClientError> {
    /// let (first, _updates) = client.path_find(&request).await?;
    /// println!("{} ways to pay", first.alternatives.len());
    /// client.path_find_close().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`ClientError::RpcError`] (`noPathRequest`) when no search is open,
    /// and the errors of [`XrplWsClient::subscribe`] for the connection.
    pub async fn path_find_close(&self) -> Result<(), ClientError> {
        self.shared
            .request(
                "path_find",
                serde_json::to_value(crate::requests::PathFindRequest::close())?,
            )
            .await?;
        Ok(())
    }

    /// Every event the connection carries, from now on.
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// // Everything the connection carries: every stream's events, and its own.
    /// let mut all = client.events();
    /// while let Some(event) = all.recv().await {
    ///     println!("{event:?}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn events(&self) -> Subscription {
        self.subscription(|_| true)
    }

    /// Returns true if the WebSocket is currently connected.
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::Client;
    ///
    /// if client.is_connected() {
    ///     println!("validated ledger {}", client.validated_ledger_index().await?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_connected(&self) -> bool {
        self.shared.connected.load(Ordering::Relaxed)
    }

    /// Get the URL this client is connected to.
    ///
    /// ```no_run
    /// # async fn run() -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::XrplWsClient;
    ///
    /// let client = XrplWsClient::connect("wss://s.altnet.rippletest.net:51233").await?;
    /// assert_eq!(client.url(), "wss://s.altnet.rippletest.net:51233");
    /// # Ok(())
    /// # }
    /// ```
    pub fn url(&self) -> &str {
        &self.url
    }
}

/// The events of one subscription: only the kind it asked for (an account's
/// transactions only for that account), plus the connection's
/// [`WsEvent::Disconnected`], [`WsEvent::Reconnected`] and
/// [`WsEvent::ResubscribeFailed`]. A receiver that falls more than 1,024
/// events behind gets [`WsEvent::Missed`] with the count, instead of the
/// stream ending.
///
/// ```no_run
/// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
/// use xrpl_client::stream::WsEvent;
///
/// let mut ledgers = client.subscribe_ledger().await?;
/// while let Some(event) = ledgers.recv().await {
///     match event {
///         WsEvent::Ledger(ledger) => println!("ledger {}", ledger.ledger_index),
///         WsEvent::Missed(n) => println!("fell behind: {n} events dropped"),
///         _ => {} // the connection's own events
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct Subscription {
    rx: broadcast::Receiver<WsEvent>,
    filter: Box<dyn Fn(&WsEvent) -> bool + Send + Sync>,
}

impl Subscription {
    /// The events as a [`futures::Stream`], for `StreamExt` combinators
    /// (`filter`, `take`, `for_each`, …). It ends when the client is dropped.
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// use futures::StreamExt;
    ///
    /// let ledgers = client.subscribe_ledger().await?.into_stream();
    /// ledgers.take(3).for_each(|event| async move { println!("{event:?}") }).await;
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_stream(self) -> impl futures::Stream<Item = WsEvent> + Send {
        futures::stream::unfold(self, |mut subscription| async move {
            subscription.recv().await.map(|event| (event, subscription))
        })
    }

    /// The next event; `None` once the client is dropped.
    ///
    /// ```no_run
    /// # async fn run(client: xrpl_client::XrplWsClient) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::stream::WsEvent;
    ///
    /// let mut ledgers = client.subscribe_ledger().await?;
    /// if let Some(WsEvent::Ledger(ledger)) = ledgers.recv().await {
    ///     println!("ledger {} closed with {:?} transactions", ledger.ledger_index, ledger.txn_count);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn recv(&mut self) -> Option<WsEvent> {
        loop {
            match self.rx.recv().await {
                Ok(event) if event.is_connection_event() || (self.filter)(&event) => {
                    return Some(event)
                }
                Ok(_) => {}
                Err(RecvError::Lagged(n)) => return Some(WsEvent::Missed(n)),
                Err(RecvError::Closed) => return None,
            }
        }
    }
}

impl std::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription").finish_non_exhaustive()
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
        Box::pin(async move { self.shared.request(command, params).await })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::client::Client;
    use tokio::net::TcpListener;
    use tokio_tungstenite::tungstenite::Message;

    #[test]
    fn logs_name_the_host_only() {
        assert_eq!(
            host_of("wss://user:token@node.example:443/ws?key=abc"),
            "node.example:443"
        );
        assert_eq!(host_of("ws://127.0.0.1:6006"), "127.0.0.1:6006");
        assert_eq!(host_of("wss://s1.ripple.com/"), "s1.ripple.com");
    }

    /// A local server: answers every request, and after the `subscribes`-th
    /// `subscribe` sends `events`.
    async fn server_after(
        subscribes: usize,
        events: Vec<Value>,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move {
            let (tcp, _) = listener.accept().await.unwrap();
            let mut ws = tokio_tungstenite::accept_async(tcp).await.unwrap();
            let mut seen = 0;
            while let Some(Ok(message)) = ws.next().await {
                let Message::Text(text) = message else {
                    if matches!(message, Message::Close(_)) {
                        break;
                    }
                    continue;
                };
                let request: Value = serde_json::from_str(&text).unwrap();
                let reply = json!({"id": request["id"], "result": {}, "status": "success", "type": "response"});
                ws.send(Message::Text(reply.to_string().into()))
                    .await
                    .unwrap();
                if request["command"] == "subscribe" {
                    seen += 1;
                }
                if request["command"] == "subscribe" && seen == subscribes {
                    for event in &events {
                        ws.send(Message::Text(event.to_string().into()))
                            .await
                            .unwrap();
                    }
                }
            }
        });
        (url, handle)
    }

    /// A local server that sends `events` after the first `subscribe`.
    async fn server(events: Vec<Value>) -> (String, tokio::task::JoinHandle<()>) {
        server_after(1, events).await
    }

    async fn next(s: &mut Subscription) -> WsEvent {
        tokio::time::timeout(Duration::from_secs(5), s.recv())
            .await
            .unwrap()
            .unwrap()
    }

    fn payment(from: &str, to: &str) -> Value {
        json!({"type": "transaction", "validated": true, "engine_result": "tesSUCCESS",
            "engine_result_code": 0, "engine_result_message": "ok", "ledger_index": 5,
            "tx_json": {"TransactionType": "Payment", "Account": from, "Destination": to,
                "DeliverMax": "1", "Fee": "12", "Sequence": 1, "SigningPubKey": "", "Flags": 0},
            "meta": {"TransactionResult": "tesSUCCESS", "TransactionIndex": 0, "AffectedNodes": []}})
    }

    /// An account subscription hears that account's transactions only, and
    /// a ledger subscription ledgers only, on one connection.
    #[tokio::test]
    async fn each_subscription_hears_only_its_own_events() {
        let a = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
        let b = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
        let c = "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1";
        let ledger = json!({"type": "ledgerClosed", "ledger_index": 6, "ledger_hash": "AB",
            "ledger_time": 1, "fee_base": 10, "reserve_base": 1, "reserve_inc": 1, "txn_count": 0});
        // The events come after both subscriptions exist: one made after
        // they were sent would not hear them.
        let (url, _server) = server_after(2, vec![payment(b, c), ledger, payment(a, c)]).await;
        let ws = XrplWsClient::connect(&url).await.unwrap();
        let mut all = ws.events();
        let mut account = ws.subscribe_account(a).await.unwrap();
        let mut ledgers = ws.subscribe_ledger().await.unwrap();
        match next(&mut account).await {
            WsEvent::Transaction(t) => assert!(t.affects(a) && !t.affects(b)),
            other => panic!("{other:?}"),
        }
        assert!(matches!(next(&mut ledgers).await, WsEvent::Ledger(_)));
        for _ in 0..3 {
            next(&mut all).await;
        }
    }

    /// The builder's request timeout applies: a server that never answers
    /// is a Timeout of that length, not of the 30 s default.
    #[tokio::test]
    async fn the_request_timeout_is_the_one_set() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        tokio::spawn(async move {
            let (tcp, _) = listener.accept().await.unwrap();
            let mut ws = tokio_tungstenite::accept_async(tcp).await.unwrap();
            while let Some(Ok(_)) = ws.next().await {}
        });
        let timeout = Duration::from_millis(300);
        let ws = XrplWsClient::builder(&url)
            .request_timeout(timeout)
            .connect()
            .await
            .unwrap();
        let started = Instant::now();
        let err = ws.raw_request("fee", json!({})).await.unwrap_err();
        assert!(
            matches!(err, ClientError::Timeout(t) if t == timeout),
            "{err:?}"
        );
        assert!(started.elapsed() < Duration::from_secs(3));
        assert!(!format!("{:?}", XrplWsClient::builder("wss://u:tok@h.example/x")).contains("tok"));
    }

    /// A subscription read as a Stream.
    #[tokio::test]
    async fn a_subscription_is_a_stream() {
        use futures::StreamExt;
        let ledger = |n: u32| {
            json!({"type": "ledgerClosed", "ledger_index": n, "ledger_hash": "AB",
            "ledger_time": 1, "fee_base": 10, "reserve_base": 1, "reserve_inc": 1, "txn_count": 0})
        };
        let (url, _server) = server(vec![ledger(6), ledger(7)]).await;
        let ws = XrplWsClient::connect(&url).await.unwrap();
        let events: Vec<WsEvent> = tokio::time::timeout(
            Duration::from_secs(5),
            ws.subscribe_ledger()
                .await
                .unwrap()
                .into_stream()
                .take(2)
                .collect(),
        )
        .await
        .unwrap();
        assert!(events.iter().all(|e| matches!(e, WsEvent::Ledger(_))));
    }

    /// Dropping the client closes the connection: the server sees it end.
    #[tokio::test]
    async fn dropping_the_client_closes_the_connection() {
        let (url, server) = server(vec![]).await;
        let ws = XrplWsClient::connect(&url).await.unwrap();
        ws.subscribe_ledger().await.unwrap();
        drop(ws);
        tokio::time::timeout(Duration::from_secs(5), server)
            .await
            .expect("the server's connection did not end")
            .unwrap();
    }

    /// The server drops the connection: the client says so, reconnects,
    /// subscribes again, says so, and the subscription keeps hearing events.
    #[tokio::test]
    async fn a_dropped_connection_is_restored_with_its_subscriptions() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let resubscribed = Arc::new(AtomicBool::new(false));
        let seen = resubscribed.clone();
        tokio::spawn(async move {
            for round in 0..2 {
                let (tcp, _) = listener.accept().await.unwrap();
                let mut ws = tokio_tungstenite::accept_async(tcp).await.unwrap();
                while let Some(Ok(Message::Text(text))) = ws.next().await {
                    let request: Value = serde_json::from_str(&text).unwrap();
                    let reply = json!({"id": request["id"], "result": {}, "status": "success", "type": "response"});
                    ws.send(Message::Text(reply.to_string().into()))
                        .await
                        .unwrap();
                    if request["command"] == "subscribe" {
                        if round == 0 {
                            break; // drop the first connection after the subscribe
                        }
                        seen.store(true, Ordering::Relaxed);
                        let ledger = json!({"type": "ledgerClosed", "ledger_index": 7});
                        ws.send(Message::Text(ledger.to_string().into()))
                            .await
                            .unwrap();
                    }
                }
            }
        });
        let ws = XrplWsClient::connect(&url).await.unwrap();
        let mut ledgers = ws.subscribe_ledger().await.unwrap();
        assert!(matches!(next(&mut ledgers).await, WsEvent::Disconnected));
        // Then the restored stream's ledger and the Reconnected marker, in
        // either order: the server sends the ledger as soon as it has taken
        // the subscription, while the client is still announcing it.
        let (a, b) = (next(&mut ledgers).await, next(&mut ledgers).await);
        assert!(resubscribed.load(Ordering::Relaxed));
        let reconnected = |e: &WsEvent| matches!(e, WsEvent::Reconnected);
        let ledger = |e: &WsEvent| matches!(e, WsEvent::Ledger(l) if l.ledger_index == 7);
        assert!(
            (reconnected(&a) && ledger(&b)) || (ledger(&a) && reconnected(&b)),
            "{a:?} then {b:?}"
        );
    }

    /// A receiver that falls behind is told how much it missed, and keeps going.
    #[tokio::test]
    async fn a_slow_receiver_is_told_what_it_missed() {
        let (tx, rx) = broadcast::channel(2);
        let mut sub = Subscription {
            rx,
            filter: Box::new(|_| true),
        };
        for _ in 0..5 {
            tx.send(WsEvent::Reconnected).unwrap();
        }
        assert!(matches!(sub.recv().await, Some(WsEvent::Missed(3))));
        assert!(matches!(sub.recv().await, Some(WsEvent::Reconnected)));
        drop(tx);
        assert!(matches!(sub.recv().await, Some(WsEvent::Reconnected)));
        assert!(sub.recv().await.is_none());
    }
}

#[cfg(all(test, xrpl_live_tests))]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod live_tests {
    use super::*;
    use crate::client::Client;

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
