# HTTP JSON-RPC Client

Reference: https://xrpl.org/http-websocket-apis.html

## Public Endpoints
| Network | URL |
|---|---|
| Mainnet | `https://s1.ripple.com:51234` or `https://s2.ripple.com:51234` |
| Testnet | `https://s.altnet.rippletest.net:51234` |
| Devnet | `https://s.devnet.rippletest.net:51234` |

## Request Format
All requests are HTTP POST to the root URL `/`.
```json
{
  "method": "method_name",
  "params": [{ ...params object... }]
}
```
Note: `params` is always a **single-element array** containing the params object.

## Response Format
```json
{
  "result": {
    "status": "success",
    ...fields...
  }
}
```
On error:
```json
{
  "result": {
    "status": "error",
    "error": "error_code_string",
    "error_message": "Human readable message",
    "request": { ...original request... }
  }
}
```

## Client Struct
```rust
pub struct XrplHttpClient {
    base_url: Url,
    inner: reqwest::Client,
    timeout: Duration,  // default: 30s
}
```
Set `User-Agent: xrpl-rust-sdk/0.1` on all requests.

---

## Methods to Implement

### account_info
```
method: "account_info"
params: { "account": "r...", "ledger_index": "validated" | "current" | "closed" | u32 }
```
Returns: `{ account_data: AccountRoot, ledger_current_index: u32, validated: bool }`

`AccountRoot` fields to map:
- `Account`, `Balance` (drops string), `Sequence` (u32), `Flags` (u32),
  `OwnerCount` (u32), `LedgerEntryType`, `index` (Hash256 string)

### account_lines
```
params: { "account": "r...", "ledger_index": "validated", "limit": 400 }
```
Returns paginated `lines` array. Handle `marker` field for pagination:
- If response has `marker`, make another request with `marker` param
- Continue until no `marker` in response

### account_offers
```
params: { "account": "r...", "limit": 400 }
```
Paginated. Returns `offers` array with fields: `flags`, `quality`, `seq`, `taker_gets`, `taker_pays`

### account_nfts
```
params: { "account": "r...", "limit": 400 }
```
Returns `account_nfts` array.

### account_tx
```
params: {
  "account": "r...",
  "binary": false,
  "forward": false,
  "limit": 400,
  "ledger_index_min": -1,    // -1 = earliest
  "ledger_index_max": -1,    // -1 = latest
  "marker": { ... }          // for pagination
}
```

### ledger
```
params: { "ledger_index": "validated" | u32, "transactions": false }
```

### ledger_current
```
params: {}
```
Returns `{ ledger_current_index: u32 }`

### submit
```
params: { "tx_blob": "AABBCC..." }  // uppercase hex
```
Returns:
```
{
  "engine_result": "tesSUCCESS" | "tem..." | "tef..." | ...,
  "engine_result_code": i32,
  "engine_result_message": string,
  "tx_blob": string,
  "tx_json": { ... }
}
```

### submit_and_wait
```
params: { "tx_blob": "...", "fail_hard": false }
```
Blocks server-side until validated. Use for simple payment flows.

### tx
```
params: { "transaction": "HASH256HEX", "binary": false }
```
Returns full transaction + metadata. Check `validated: true` before trusting.

### book_offers
```
params: {
  "taker_pays": { "currency": "XRP" } | { "currency": "USD", "issuer": "r..." },
  "taker_gets": { ... },
  "limit": 20
}
```
Returns `offers` array.

### amm_info
```
params: {
  "asset": { "currency": "XRP" } | { "currency": "...", "issuer": "r..." },
  "asset2": { ... }
}
```
Returns AMM account, LP token balance, trading fee, vote slots.

### fee
```
params: {}
```
Returns:
```
{
  "current_ledger_size": u32,
  "current_queue_size": u32,
  "drops": {
    "base_fee": "10",
    "median_fee": "5000",
    "minimum_fee": "10",
    "open_ledger_fee": "10"
  },
  "expected_ledger_size": u32,
  "ledger_current_index": u32,
  "levels": { ... }
}
```

---

## Pagination Helper
```rust
async fn paginate<T, F, Fut>(
    &self,
    command: &str,
    mut params: serde_json::Value,
    extract: F,
) -> Result<Vec<T>>
where
    F: Fn(serde_json::Value) -> Result<(Vec<T>, Option<serde_json::Value>)>,
    Fut: Future<Output = Result<(Vec<T>, Option<serde_json::Value>)>>,
```
Call with initial params; if result has `marker`, repeat with marker added to params.

---

## Engine Result Codes
| Prefix | Meaning |
|---|---|
| `tes` | Success |
| `tem` | Malformed transaction (local rejection) |
| `tef` | Failure (not applied, not queued) |
| `ter` | Retry (temporary failure) |
| `tec` | Claimed fee, transaction not applied |

Map `tem*`, `tef*` to `XrplError::TransactionFailed`.
Map `ter*` to `XrplError::TransactionRetry` (caller should retry).
Map `tec*` to `XrplError::TransactionClaimed` (fee charged, nothing else happened).

---

## Retry Policy
- On HTTP 429 or 503: exponential backoff, max 5 retries
- On `terQUEUED`: poll `tx` method until validated or ledger moves past `LastLedgerSequence`
- Timeout: 30s default, configurable
