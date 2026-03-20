# WebSocket Client

Reference: https://xrpl.org/websocket-api-tool.html

## Public WS Endpoints
| Network | URL |
|---|---|
| Mainnet | `wss://s1.ripple.com:51233` or `wss://xrplcluster.com` |
| Testnet | `wss://s.altnet.rippletest.net:51233` |

---

## Wire Protocol
All messages are JSON text frames.

**Request:**
```json
{ "command": "method_name", "id": 1, ...params... }
```
Note: for WS, params are inlined (not wrapped in array like HTTP).

**Response:**
```json
{ "id": 1, "status": "success", "type": "response", "result": { ... } }
```

**Stream message (no id):**
```json
{ "type": "ledgerClosed" | "transaction" | "validationReceived", ...fields... }
```

---

## Client Architecture

```
XrplWsClient {
    sender:   mpsc::Sender<WsMessage>,       // to write loop
    pending:  Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value>>>>>,
    subs:     Arc<Mutex<HashMap<SubType, broadcast::Sender<SubEvent>>>>,
    id_counter: Arc<AtomicU64>,
}
```

**Spawn two tasks on connect:**
1. **Write loop**: reads from `mpsc::Receiver`, sends text frames
2. **Read loop**: reads frames, routes:
   - Has `id` field → lookup in `pending`, send to oneshot
   - No `id`, has `type` field → route to subscription broadcast channel

---

## Request/Response Flow
```rust
async fn request<Req: Serialize, Resp: DeserializeOwned>(
    &self, command: &str, params: Req
) -> Result<Resp> {
    let id = self.id_counter.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = oneshot::channel();
    self.pending.lock().insert(id, tx);
    let msg = json!({ "command": command, "id": id, ...params });
    self.sender.send(msg.to_string()).await?;
    let raw = rx.await?;   // wait for read loop to route response
    parse_result(raw)
}
```

---

## Subscriptions

### Subscribe to ledger stream
```json
{ "command": "subscribe", "streams": ["ledger"] }
```
Events arrive as:
```json
{
  "type": "ledgerClosed",
  "fee_base": 10,
  "fee_ref": 10,
  "ledger_hash": "...",
  "ledger_index": 12345678,
  "ledger_time": 123456789,
  "reserve_base": 10000000,
  "reserve_inc": 2000000,
  "txn_count": 42,
  "validated_ledgers": "12345000-12345678"
}
```

### Subscribe to all transactions
```json
{ "command": "subscribe", "streams": ["transactions"] }
```

### Subscribe to specific account
```json
{ "command": "subscribe", "accounts": ["r...", "r..."] }
```
Events:
```json
{
  "type": "transaction",
  "transaction": { ...tx fields... },
  "meta": { ...metadata... },
  "validated": true,
  "ledger_index": 12345678
}
```

### Subscribe to order book
```json
{
  "command": "subscribe",
  "books": [{
    "taker_pays": { "currency": "XRP" },
    "taker_gets": { "currency": "USD", "issuer": "r..." },
    "snapshot": true,
    "both": false
  }]
}
```

### Unsubscribe
Replace `"subscribe"` with `"unsubscribe"` using same params.

---

## Reconnection Logic
```
on_disconnect:
    wait: min(2^attempt * 1s, 60s)
    reconnect()
    re_subscribe(all active subscriptions)
    notify subscribers of reconnection gap
```
Maintain a set of active subscription params. On reconnect, replay all `subscribe` calls.
Emit a `SubEvent::Reconnected { gap_start_ledger }` to all subscription channels.

---

## Keepalive
Send a ping frame every 30 seconds.
If no pong received within 10 seconds, close connection and trigger reconnect.

```rust
// tokio-tungstenite ping
ws_sink.send(Message::Ping(vec![])).await?;
```

---

## Stream Types (Rust)
```rust
pub enum SubEvent {
    Ledger(LedgerEvent),
    Transaction(TransactionEvent),
    Account(AccountEvent),
    Book(BookUpdateEvent),
    Reconnected { gap_start_ledger: u32 },
}

pub struct LedgerEvent {
    pub ledger_index: u32,
    pub ledger_hash: Hash256,
    pub ledger_time: u32,
    pub txn_count: u32,
    pub fee_base: u64,
    pub reserve_base: u64,
    pub reserve_inc: u64,
}

pub struct TransactionEvent {
    pub transaction: serde_json::Value,  // decoded later by caller
    pub meta: TransactionMeta,
    pub validated: bool,
    pub ledger_index: u32,
}
```

---

## Error Handling
- Parse error from server: `{ "status": "error", "error": "...", "error_message": "..." }`
- Map to `XrplError::RpcError { code: String, message: String }`
- Connection closed unexpectedly → trigger reconnect, return `XrplError::Disconnected` to pending requests
