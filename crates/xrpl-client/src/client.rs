//! One interface over the HTTP and WebSocket clients. A transport implements
//! [`Client::raw_request`]; every rippled method here is written once.

use crate::error::ClientError;
use crate::requests::*;
use crate::types::*;
use serde_json::{json, Value};
use xrpl_core::types::LedgerObject;

/// The most pages [`Client::request_all`] follows before it gives up.
pub const MAX_PAGES: usize = 50;

/// The fee cap a client starts with: 2 XRP (xrpl.js's default `maxFeeXRP`).
/// A normal fee is 10-20 drops; fees near the cap mean the network is
/// heavily loaded, or the server is lying.
pub const DEFAULT_MAX_FEE_DROPS: u64 = 2_000_000;

/// How many ledgers past the current one [`Client::autofill`] gives a
/// transaction (`LastLedgerSequence`): about a minute and a half.
pub const LAST_LEDGER_OFFSET: u32 = 20;

/// An X-address says whether it is for a test network (XLS-5d): one for
/// the other kind of network than the server's (mainnet is 0; Testnet 1 and
/// Devnet 2 are test networks) is refused, so a Testnet address cannot
/// receive a mainnet payment by mistake.
fn check_x_address_networks(tx: &Value, network_id: u32) -> Result<(), ClientError> {
    let mainnet = match network_id {
        0 => true,
        1 | 2 => false,
        _ => return Ok(()),
    };
    for (name, value) in tx.as_object().into_iter().flatten() {
        let Some(text) = value.as_str().filter(|s| s.starts_with(['X', 'T'])) else {
            continue;
        };
        let Ok((_, _, test)) = xrpl_core::address::decode_x_address(text) else {
            continue;
        };
        if test == mainnet {
            return Err(ClientError::Core(xrpl_core::CoreError::ValidationError(
                format!(
                    "{name} is an X-address for {}, but the server is on {}",
                    if test { "a test network" } else { "mainnet" },
                    if mainnet { "mainnet" } else { "a test network" }
                ),
            )));
        }
    }
    Ok(())
}

/// `fee`, or [`ClientError::FeeTooHigh`] above `max`.
/// [`Client::autofill_batch`] with the prices already known.
async fn fill_batch<C: Client + ?Sized>(
    client: &C,
    tx: &mut Value,
    outer: &str,
    prices: Option<Prices>,
    outer_signers: u32,
) -> Result<(), ClientError> {
    let bad = |m: &str| ClientError::UnexpectedResponse(format!("Batch: {m}"));
    let outer_seq = tx
        .get("Sequence")
        .and_then(Value::as_u64)
        .and_then(|s| u32::try_from(s).ok())
        .ok_or_else(|| bad("no Sequence that fits 32 bits"))?;
    let raw = tx
        .get_mut("RawTransactions")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| bad("no RawTransactions"))?;
    let mut next: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    next.insert(outer.to_string(), outer_seq.saturating_add(1));
    for wrapped in raw.iter_mut() {
        let inner = wrapped
            .get_mut("RawTransaction")
            .ok_or_else(|| bad("expected {\"RawTransaction\": …}"))?;
        *inner = xrpl_core::transaction::batch::inner_transaction(inner)?;
        let account = inner
            .get("Account")
            .and_then(Value::as_str)
            .ok_or_else(|| bad("an inner transaction has no Account"))?
            .to_string();
        if inner.get("Sequence").is_some() || inner.get("TicketSequence").is_some() {
            continue;
        }
        let seq = match next.get(&account) {
            Some(s) => *s,
            None => {
                client
                    .account_info(&account, &LedgerIndex::Current)
                    .await?
                    .account_data
                    .sequence
            }
        };
        next.insert(account, seq.saturating_add(1));
        if let Some(obj) = inner.as_object_mut() {
            obj.insert("Sequence".into(), json!(seq));
        }
    }
    if let Some(prices) = prices {
        let mut cost = xrpl_core::transaction::cost::transaction_cost(
            tx,
            usize::try_from(outer_signers).unwrap_or(usize::MAX),
        )?;
        if tx.get("BatchSigners").is_none() {
            let accounts = xrpl_core::transaction::batch::required_batch_signers(tx)?.len();
            cost.base_fees = cost
                .base_fees
                .saturating_add(u64::try_from(accounts).unwrap_or(u64::MAX));
        }
        let total = capped_fee(prices.of(cost), client.max_fee_drops())?;
        if let Some(obj) = tx.as_object_mut() {
            obj.insert("Fee".into(), json!(total.to_string()));
        }
    }
    Ok(())
}

/// What autofill charges now, in drops: one base fee at the open-ledger fee,
/// one owner reserve scaled the same way, and the extra base fees a
/// `LoanSet`'s counterparty will add.
#[derive(Debug, Clone, Copy)]
struct Prices {
    base_fee: u64,
    owner_reserve: u64,
    extra_base_fees: u64,
}

impl Prices {
    /// From the `fee` answer, the owner reserve (drops, when it is needed)
    /// and the counterparty's signatures.
    fn new(
        fee: &FeeResult,
        reserve: Option<u64>,
        extra_base_fees: u64,
    ) -> Result<Self, ClientError> {
        let open = parse_drops("open_ledger_fee", &fee.drops.open_ledger_fee)?;
        let base = match reserve {
            Some(_) => Some(parse_drops("base_fee", &fee.drops.base_fee)?),
            None => None,
        };
        Self::from_parts(open, base, reserve, extra_base_fees)
    }

    /// The open ledger's fee for a reference transaction prices one base
    /// fee; an owner reserve is scaled by the same ratio to the unescalated
    /// `base` fee, rounded up.
    fn from_parts(
        open: u64,
        base: Option<u64>,
        reserve: Option<u64>,
        extra_base_fees: u64,
    ) -> Result<Self, ClientError> {
        let owner_reserve = match (reserve, base) {
            (Some(_), Some(0)) => {
                return Err(ClientError::UnexpectedResponse(
                    "base_fee is 0, so the open-ledger fee cannot be scaled".into(),
                ))
            }
            (Some(reserve), Some(base)) => {
                let scaled = (u128::from(reserve) * u128::from(open)).div_ceil(u128::from(base));
                u64::try_from(scaled).unwrap_or(u64::MAX)
            }
            _ => 0,
        };
        Ok(Self {
            base_fee: open,
            owner_reserve,
            extra_base_fees,
        })
    }

    /// The fee for a transaction of this cost.
    fn of(&self, mut cost: xrpl_core::transaction::cost::TransactionCost) -> u64 {
        cost.base_fees = cost.base_fees.saturating_add(self.extra_base_fees);
        cost.drops(self.base_fee, self.owner_reserve)
    }
}

/// `value` (a field of the `fee` answer) as a whole number of drops.
fn parse_drops(name: &str, value: &str) -> Result<u64, ClientError> {
    value.parse::<u64>().map_err(|_| {
        ClientError::UnexpectedResponse(format!("{name} {value:?} is not a whole number of drops"))
    })
}

/// The owner reserve of the server's validated ledger, in drops
/// (`server_state`'s `validated_ledger.reserve_inc`).
async fn owner_reserve_drops<C: Client + ?Sized>(client: &C) -> Result<u64, ClientError> {
    client
        .request(&ServerStateRequest {})
        .await?
        .state
        .validated_ledger
        .and_then(|l| l.reserve_inc)
        .ok_or_else(|| {
            ClientError::UnexpectedResponse(
                "server_state has no validated ledger, so the owner reserve this \
                 transaction costs is unknown: set its Fee yourself"
                    .into(),
            )
        })
}

/// How many signatures a `LoanSet`'s counterparty will add: the size of its
/// signer list, or one. The counterparty is `Counterparty`, or else the owner
/// of the `LoanBrokerID` broker (as xrpl.js reads it).
async fn loan_counterparty_signers<C: Client + ?Sized>(
    client: &C,
    tx: &Value,
) -> Result<u64, ClientError> {
    let invalid = |m: &str| {
        ClientError::Core(xrpl_core::CoreError::ValidationError(format!(
            "LoanSet: {m}"
        )))
    };
    let counterparty = match tx.get("Counterparty").and_then(Value::as_str) {
        Some(account) => account.to_string(),
        None => {
            let broker = tx
                .get("LoanBrokerID")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid("it needs a Counterparty or a LoanBrokerID"))?;
            let entry = client
                .raw_request(
                    "ledger_entry",
                    json!({"index": broker, "ledger_index": "validated"}),
                )
                .await?;
            entry
                .get("node")
                .and_then(|n| n.get("Owner"))
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    ClientError::UnexpectedResponse("the loan broker entry has no Owner".into())
                })?
                .to_string()
        }
    };
    let info = client
        .raw_request(
            "account_info",
            json!({"account": counterparty, "ledger_index": "validated", "signer_lists": true}),
        )
        .await?;
    // API version 2 puts signer_lists beside account_data; version 1 inside it.
    let entries = info
        .get("signer_lists")
        .or_else(|| info.get("account_data").and_then(|a| a.get("signer_lists")))
        .and_then(|lists| lists.get(0))
        .and_then(|list| list.get("SignerEntries"))
        .and_then(Value::as_array)
        .map_or(1, Vec::len);
    Ok(u64::try_from(entries).unwrap_or(u64::MAX))
}

fn capped_fee(fee: u64, max: u64) -> Result<u64, ClientError> {
    if fee > max {
        return Err(ClientError::FeeTooHigh {
            fee_drops: fee,
            max_fee_drops: max,
        });
    }
    Ok(fee)
}

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

    /// The pages of a paginated request, one at a time: follow its `marker`
    /// with no page limit, and stop reading when you have enough (for an
    /// account's whole history, or a list longer than [`MAX_PAGES`] pages).
    /// Every page after the first asks for the ledger the first came from.
    /// The stream ends after the last page, or after an error.
    ///
    /// ```no_run
    /// # async fn run(client: impl xrpl_client::Client) -> Result<(), xrpl_client::ClientError> {
    /// use futures::{pin_mut, StreamExt};
    /// use xrpl_client::requests::AccountTxRequest;
    ///
    /// let pages = client.pages(AccountTxRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"));
    /// pin_mut!(pages);
    /// let mut seen = 0;
    /// while let Some(page) = pages.next().await {
    ///     seen += page?.len();
    ///     if seen >= 10_000 {
    ///         break;
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn pages<R: Paginated>(
        &self,
        req: R,
    ) -> impl futures::Stream<Item = Result<Vec<R::Item>, ClientError>> {
        futures::stream::unfold(Some(req), move |state| async move {
            let req = state?;
            match self.request(&req).await {
                Err(e) => Some((Err(e), None)),
                Ok(response) => {
                    let ledger = R::ledger_of(&response);
                    let (items, marker) = R::page(response);
                    // A marker is only good for the ledger it came from.
                    let next = marker.map(|marker| {
                        let next = req.with_marker(marker);
                        match ledger {
                            Some(index) => next.at_ledger(index),
                            None => next,
                        }
                    });
                    Some((Ok(items), next))
                }
            }
        })
    }

    /// Send a paginated request and follow its `marker` to the last page
    /// (at most [`MAX_PAGES`] pages, else [`ClientError::PaginationLimitReached`];
    /// for more, read them with [`Client::pages`]).
    /// Every page after the first asks for the ledger the first came from,
    /// so the result is one ledger's state, never a mix of two.
    async fn request_all<R: Paginated>(&self, req: R) -> Result<Vec<R::Item>, ClientError> {
        let mut all = Vec::new();
        let mut req = req;
        for _ in 0..MAX_PAGES {
            let response = self.request(&req).await?;
            let ledger = R::ledger_of(&response);
            let (items, marker) = R::page(response);
            all.extend(items);
            match marker {
                Some(marker) => {
                    req = req.with_marker(marker);
                    // A marker is only good for the ledger it came from.
                    if let Some(index) = ledger {
                        req = req.at_ledger(index);
                    }
                }
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
            ledger_index: *ledger,
            ..AccountInfoRequest::new(account)
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
        self.request(&LedgerRequest::new(*ledger)).await
    }

    /// Submit a signed transaction blob (hex) once and sort the server's
    /// **preliminary** result: `tem`/`tef` and most `tel` results are
    /// [`ClientError::TransactionFailed`]; `ter` and the load/queue `tel`s
    /// (`telINSUF_FEE_P`, `telCAN_NOT_QUEUE*`) are
    /// [`ClientError::TransactionRetry`]; `tec` is
    /// [`ClientError::TransactionClaimed`].
    ///
    /// None of these is final except a `tem`: a transaction that got `tec`
    /// or `terQUEUED` here can still succeed, and one that got `tesSUCCESS`
    /// can still fail. For the outcome, use [`Client::submit_and_wait`];
    /// never sign a replacement on the strength of this result alone.
    async fn submit(&self, tx_blob: &str) -> Result<SubmitResult, ClientError> {
        classify_submit(self.request(&SubmitRequest::new(tx_blob)).await?)
    }

    /// Submit a multi-signed transaction (JSON with `Signers` and an empty
    /// `SigningPubKey`); the engine result is sorted as for [`Client::submit`].
    async fn submit_multisigned(&self, tx_json: &Value) -> Result<SubmitResult, ClientError> {
        classify_submit(
            self.request(&SubmitMultisignedRequest::new(tx_json.clone()))
                .await?,
        )
    }

    /// What a transaction would do in the open ledger, without signing or
    /// submitting it (rippled 2.4+). Leave out `SigningPubKey` and
    /// `TxnSignature`; `Fee`, `Sequence` and the like are filled in.
    async fn simulate(&self, tx_json: &Value) -> Result<SimulateResult, ClientError> {
        self.request(&SimulateRequest::new(tx_json.clone())).await
    }

    /// Look up a transaction by hash.
    async fn tx(&self, hash: &str) -> Result<TxResult, ClientError> {
        self.request(&TxRequest::new(hash)).await
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
            limit: Some(20),
            ..BookOffersRequest::new(taker_gets.clone(), taker_pays.clone())
        })
        .await
    }

    /// Get AMM pool information.
    async fn amm_info(&self, asset: &Value, asset2: &Value) -> Result<AmmInfoResult, ClientError> {
        self.request(&AmmInfoRequest::new(asset.clone(), asset2.clone()))
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
            binary: None,
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
            ledger_index: *ledger,
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
            destination_account: destination_account.map(Into::into),
            ledger_index: *ledger,
            ..AccountChannelsRequest::new(account)
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
            ledger_index: *ledger,
            ..LedgerEntryRequest::new(index)
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
            hotwallet: hotwallet.map(|h| h.into_iter().map(Into::into).collect()),
            ledger_index: *ledger,
            ..GatewayBalancesRequest::new(account)
        })
        .await
    }

    /// The most [`Client::autofill`] sets as a `Fee`, in drops:
    /// [`DEFAULT_MAX_FEE_DROPS`] (2 XRP) unless the client was built with
    /// another (`with_max_fee_drops`).
    fn max_fee_drops(&self) -> u64 {
        DEFAULT_MAX_FEE_DROPS
    }

    /// Fill in what a transaction needs and does not have (a field that is
    /// missing or `null`), asking the server concurrently:
    ///
    /// - `Sequence`: the account's next sequence, or 0 when the transaction
    ///   uses a `TicketSequence`;
    /// - `Fee`: what the open ledger charges for it now: its base cost as
    ///   `rippled` prices its type
    ///   ([`transaction_cost`](xrpl_core::transaction::cost::transaction_cost):
    ///   one owner reserve for `AccountDelete` and `AMMCreate`, more for an
    ///   `EscrowFinish` with a fulfillment, …) at the open-ledger fee. A
    ///   `LoanSet` is priced for its counterparty's signatures too (the size
    ///   of its signer list, or one). The fee is refused with
    ///   [`ClientError::FeeTooHigh`] above [`Client::max_fee_drops`] (a
    ///   busy network, or a server that names any fee it likes, cannot make
    ///   it spend more);
    /// - `LastLedgerSequence`: the current ledger + [`LAST_LEDGER_OFFSET`];
    /// - `NetworkID`: on a network whose ID is above 1024, which requires it.
    ///
    /// A `Batch` also gets its inner transactions filled in (see
    /// `autofill_batch`). For a multi-signed transaction use
    /// [`Client::autofill_for_multisigning`], whose fee counts the signers.
    async fn autofill(&self, tx: &mut Value) -> Result<(), ClientError> {
        self.autofill_for_multisigning(tx, 0).await
    }

    /// [`Client::autofill`] for a transaction that `signers` accounts will
    /// sign: the fee is the open ledger's × (1 + `signers`), as `rippled`
    /// charges a multi-signed transaction, and `SigningPubKey` is set empty.
    /// With `signers` 0 it is exactly [`Client::autofill`].
    async fn autofill_for_multisigning(
        &self,
        tx: &mut Value,
        signers: u32,
    ) -> Result<(), ClientError> {
        let account = tx
            .get("Account")
            .and_then(Value::as_str)
            .ok_or_else(|| ClientError::UnexpectedResponse("tx missing Account field".into()))?
            .to_string();
        let missing = |tx: &Value, field: &str| tx.get(field).is_none_or(Value::is_null);
        let uses_ticket = !missing(tx, "TicketSequence");
        let need_seq = missing(tx, "Sequence");
        let need_fee = missing(tx, "Fee");
        let need_lls = missing(tx, "LastLedgerSequence");
        let need_network = missing(tx, "NetworkID");
        let kind = tx.get("TransactionType").and_then(Value::as_str);
        let is_batch = kind == Some("Batch");
        // What rippled charges for this type, to know what to look up.
        let cost = if need_fee {
            Some(xrpl_core::transaction::cost::transaction_cost(
                tx,
                usize::try_from(signers).unwrap_or(usize::MAX),
            )?)
        } else {
            None
        };
        let counterparty_signs =
            need_fee && kind == Some("LoanSet") && tx.get("CounterpartySignature").is_none();

        let seq_fut = async {
            if need_seq && !uses_ticket {
                self.account_info(&account, &LedgerIndex::Current)
                    .await
                    .map(Some)
            } else {
                Ok(None)
            }
        };
        let fee_fut = async {
            if !need_fee {
                return Ok(None);
            }
            let reserve = async {
                if cost.is_some_and(|c| c.owner_reserves > 0) {
                    owner_reserve_drops(self).await.map(Some)
                } else {
                    Ok(None)
                }
            };
            let counterparty = async {
                if counterparty_signs {
                    loan_counterparty_signers(self, tx).await
                } else {
                    Ok(0)
                }
            };
            let (fee, reserve, counterparty) =
                futures::future::try_join3(self.fee(), reserve, counterparty).await?;
            Ok(Some((fee, reserve, counterparty)))
        };
        let lls_fut = async {
            if need_lls {
                self.ledger_current().await.map(Some)
            } else {
                Ok(None)
            }
        };
        let network_fut = async {
            if need_network {
                self.server_info().await.map(|r| r.info.network_id)
            } else {
                Ok(None)
            }
        };
        let (acct_info, fee_result, ledger_idx, network_id) =
            futures::future::try_join4(seq_fut, fee_fut, lls_fut, network_fut).await?;
        if let Some(id) = network_id {
            check_x_address_networks(tx, id)?;
        }
        let prices = fee_result
            .map(|(fee, reserve, counterparty)| Prices::new(&fee, reserve, counterparty))
            .transpose()?;
        // Refuse a fee above the cap before changing anything.
        let fee = match (prices, cost) {
            (Some(p), Some(c)) if !is_batch => Some(capped_fee(p.of(c), self.max_fee_drops())?),
            _ => None,
        };

        let obj = tx
            .as_object_mut()
            .ok_or_else(|| ClientError::UnexpectedResponse("tx is not a JSON object".into()))?;
        if need_seq && uses_ticket {
            obj.insert("Sequence".into(), json!(0));
        }
        if let Some(info) = acct_info {
            obj.insert("Sequence".into(), json!(info.account_data.sequence));
        }
        if let Some(idx) = ledger_idx {
            obj.insert(
                "LastLedgerSequence".into(),
                json!(idx.saturating_add(LAST_LEDGER_OFFSET)),
            );
        }
        if let Some(id) = network_id.filter(|id| *id > 1024) {
            obj.insert("NetworkID".into(), json!(id));
        }
        if signers > 0 {
            obj.insert("SigningPubKey".into(), json!(""));
        }
        if is_batch {
            return fill_batch(self, tx, &account, prices, signers).await;
        }
        if let Some(fee) = fee {
            obj.insert("Fee".into(), json!(fee.to_string()));
        }
        Ok(())
    }

    /// The inner-transaction part of [`Client::autofill`] for a `Batch`: mark
    /// each inner transaction (`tfInnerBatchTxn`, `Fee` "0", empty
    /// `SigningPubKey`), number the ones without a `Sequence` or
    /// `TicketSequence` (the outer account's continue after the outer
    /// `Sequence`; other accounts' from their next sequence), and when the fee
    /// was missing, charge the batch for all of it as `rippled` prices a
    /// batch (two base fees, the outer transaction's own signers, each inner
    /// transaction's own cost, and one per account that must sign the
    /// batch), refused above the fee cap. (A BatchSigner that signs through
    /// its signer list needs one base fee per signer: set the `Fee` yourself
    /// for those.)
    async fn autofill_batch(
        &self,
        tx: &mut Value,
        outer: &str,
        open_ledger_fee: Option<u64>,
        outer_signers: u32,
    ) -> Result<(), ClientError> {
        let prices = match open_ledger_fee {
            Some(open) => {
                // An inner transaction that costs an owner reserve needs it,
                // scaled as the open-ledger fee is.
                let reserves =
                    xrpl_core::transaction::cost::transaction_cost(tx, 0)?.owner_reserves;
                let (base, reserve) = if reserves > 0 {
                    let (fee, reserve) =
                        futures::future::try_join(self.fee(), owner_reserve_drops(self)).await?;
                    (
                        Some(parse_drops("base_fee", &fee.drops.base_fee)?),
                        Some(reserve),
                    )
                } else {
                    (None, None)
                };
                Some(Prices::from_parts(open, base, reserve, 0)?)
            }
            None => None,
        };
        fill_batch(self, tx, outer, prices, outer_signers).await
    }

    /// Submit a signed blob and wait for its final outcome: the result of the
    /// validated ledger that holds it, or proof that no ledger ever will.
    ///
    /// - `Ok`: validated with `tesSUCCESS`.
    /// - [`ClientError::TransactionClaimed`] (with the validated `result`):
    ///   validated with a `tec`; the fee was charged.
    /// - [`ClientError::TransactionFailed`]: malformed (`tem`), or refused
    ///   (`tef`, `tel`) and in no ledger.
    /// - [`ClientError::TransactionExpired`]: its `LastLedgerSequence` is
    ///   validated and the server, holding every ledger up to it, finds it in
    ///   none. Only now is it safe to sign a replacement.
    /// - [`ClientError::OutcomeUnknown`]: it may have been sent but the
    ///   outcome could not be learned; look it up by `hash` before replacing.
    ///
    /// The blob must carry a `LastLedgerSequence` (autofill sets one). The
    /// transaction is looked up by its ID computed here, never by the
    /// server's word; a preliminary result is never taken as final; and a
    /// transaction the server dropped (`telINSUF_FEE_P`,
    /// `telCAN_NOT_QUEUE…`, or a failed connection) is resubmitted, as the
    /// same blob, until it is found or expires.
    async fn submit_and_wait(&self, tx_blob: &str) -> Result<TxResult, ClientError> {
        let tx = xrpl_core::codec::decode_transaction_binary(
            &hex::decode(tx_blob).map_err(|e| xrpl_core::CoreError::InvalidHex(e.to_string()))?,
        )?;
        let hash = xrpl_core::hashes::hash_signed_tx(&tx)?.to_string();
        let last_ledger_sequence = tx
            .get("LastLedgerSequence")
            .and_then(Value::as_u64)
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| {
                ClientError::Core(xrpl_core::CoreError::ValidationError(
                    "submit_and_wait needs a LastLedgerSequence, or it could wait forever".into(),
                ))
            })?;
        // Ledgers before this one were validated before the blob was sent.
        let first_ledger = self.validated_ledger_index().await?;
        let unknown = |reason: String| ClientError::OutcomeUnknown {
            hash: hash.clone(),
            last_ledger_sequence,
            reason,
        };

        let resubmit = match self.request(&SubmitRequest::new(tx_blob)).await {
            Ok(sr) => {
                let r = sr.engine_result.as_str();
                let failed = || ClientError::TransactionFailed {
                    engine_result: sr.engine_result.clone(),
                    message: sr.engine_result_message.clone(),
                };
                if r.starts_with("tem") {
                    return Err(failed());
                }
                let dropped = is_dropped(r);
                if (r.starts_with("tef") || r.starts_with("tel")) && !dropped {
                    // Not applied and not relayed, unless an earlier submit of
                    // this same blob already put it in a ledger.
                    match self.tx(&hash).await {
                        Err(ClientError::RpcError { ref code, .. }) if code == "txnNotFound" => {
                            return Err(failed())
                        }
                        Err(e) => return Err(unknown(e.to_string())),
                        Ok(_) => {}
                    }
                }
                dropped
            }
            // The server read the request and refused it: nothing was sent on.
            Err(e @ ClientError::RpcError { .. }) => return Err(e),
            // It may or may not have reached the server.
            Err(e) => {
                tracing::warn!(%hash, error = %e, "submit failed; looking the transaction up");
                true
            }
        };

        let mut failures = 0;
        loop {
            sleep_secs(POLL_SECS).await;
            match poll_outcome(self, &hash, first_ledger, last_ledger_sequence).await {
                Ok(Poll::Validated(result)) => {
                    let result = *result;
                    return match result.transaction_result() {
                        Some("tesSUCCESS") => Ok(result),
                        Some(code) => Err(ClientError::TransactionClaimed {
                            engine_result: code.to_string(),
                            result: Some(Box::new(result)),
                        }),
                        None => Err(unknown("the validated transaction has no result".into())),
                    };
                }
                Ok(Poll::Pending { found }) => {
                    failures = 0;
                    if resubmit && !found {
                        let _ = self.request(&SubmitRequest::new(tx_blob)).await;
                    }
                }
                Ok(Poll::Expired) => {
                    return Err(ClientError::TransactionExpired(last_ledger_sequence))
                }
                Ok(Poll::Unprovable) => {
                    return Err(unknown(format!(
                        "the server lacks some of ledgers {first_ledger}..={last_ledger_sequence}, \
                         so it cannot show the transaction is in none"
                    )))
                }
                Err(e) => {
                    failures += 1;
                    if failures >= MAX_POLL_FAILURES {
                        return Err(unknown(e.to_string()));
                    }
                }
            }
        }
    }

    /// The index of the latest validated ledger.
    async fn validated_ledger_index(&self) -> Result<u32, ClientError> {
        self.ledger(&LedgerIndex::Validated)
            .await?
            .ledger_index
            .ok_or_else(|| ClientError::UnexpectedResponse("ledger has no ledger_index".into()))
    }

    /// Everything `account` holds, as typed amounts read from one ledger:
    /// its XRP, the balance of each trust line (with the peer as the issuer;
    /// negative when the account owes it) and each MPT it holds. The first
    /// answer fixes the ledger the others are read from, so the list is
    /// consistent. Like xrpl.js's `getBalances`, plus MPTs.
    ///
    /// ```no_run
    /// # async fn run(client: impl xrpl_client::Client) -> Result<(), xrpl_client::error::ClientError> {
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// for amount in client.balances("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", &LedgerIndex::Validated).await? {
    ///     println!("{amount}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`ClientError::RpcError`] (`actNotFound`) for an account that does
    /// not exist, and a [`ClientError`] when a request fails or an answer
    /// holds an amount that does not parse.
    async fn balances(
        &self,
        account: &str,
        ledger: &LedgerIndex,
    ) -> Result<Vec<xrpl_core::types::Amount>, ClientError> {
        use xrpl_core::types::Amount;
        let info = self.account_info(account, ledger).await?;
        let pinned = info
            .ledger_index
            .or(info.ledger_current_index)
            .map_or(*ledger, LedgerIndex::Index);
        let drops = info.account_data.balance.parse::<u64>().map_err(|_| {
            ClientError::UnexpectedResponse(format!(
                "Balance {:?} is not a whole number of drops",
                info.account_data.balance
            ))
        })?;
        let (lines, objects) = futures::future::try_join(
            self.account_lines_all(account, &pinned),
            self.account_objects_all(account, Some("mptoken"), &pinned),
        )
        .await?;
        let mut amounts = vec![Amount::xrp(drops)?];
        for line in lines {
            amounts.push(Amount::token(&line.balance, &line.currency, &line.account)?);
        }
        for object in objects {
            if let LedgerObject::MPToken(token) = object {
                let raw = token.mpt_amount.as_deref().unwrap_or("0");
                let value = raw.parse::<i64>().map_err(|_| {
                    ClientError::UnexpectedResponse(format!("MPTAmount {raw:?} is not a number"))
                })?;
                amounts.push(Amount::mpt(value, &token.mp_token_issuance_id.to_string())?);
            }
        }
        Ok(amounts)
    }

    /// A ledger with its transactions, checked against its hash: the header
    /// hashes to the ledger hash the server names, and the transactions
    /// rebuild the header's transaction tree, so they are exactly that
    /// ledger's (see [`xrpl_core::ledger`]). Compare the returned `hash`
    /// with one from a source you trust (another server, or trusted
    /// validators) to know the ledger is the network's.
    ///
    /// ```no_run
    /// # async fn run(client: impl xrpl_client::Client) -> Result<(), xrpl_client::ClientError> {
    /// use xrpl_client::types::LedgerIndex;
    ///
    /// let ledger = client.verified_ledger(&LedgerIndex::Validated).await?;
    /// println!("ledger {} ({}): {} transactions", ledger.header.ledger_index, ledger.hash, ledger.transactions.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`ClientError::Core`] with a `ValidationError` when the answer does
    /// not check out (a transaction missing, added or changed, or a header
    /// that does not hash to the named ledger hash), and a [`ClientError`]
    /// when the request fails.
    async fn verified_ledger(
        &self,
        ledger: &LedgerIndex,
    ) -> Result<xrpl_core::ledger::VerifiedLedger, ClientError> {
        let result = self
            .raw_request(
                "ledger",
                json!({"ledger_index": ledger, "transactions": true, "expand": true, "binary": true}),
            )
            .await?;
        Ok(xrpl_core::ledger::verify_ledger_response(&result)?)
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
            binary: None,
        })
        .await
    }
}

fn account_lines_request(account: &str, ledger: &LedgerIndex) -> AccountLinesRequest {
    AccountLinesRequest {
        ledger_index: *ledger,
        limit: Some(400),
        ..AccountLinesRequest::new(account)
    }
}

fn account_offers_request(account: &str, ledger: &LedgerIndex) -> AccountOffersRequest {
    AccountOffersRequest {
        ledger_index: *ledger,
        limit: Some(400),
        ..AccountOffersRequest::new(account)
    }
}

fn account_nfts_request(account: &str, ledger: &LedgerIndex) -> AccountNftsRequest {
    AccountNftsRequest {
        ledger_index: *ledger,
        limit: Some(400),
        ..AccountNftsRequest::new(account)
    }
}

fn account_objects_request(
    account: &str,
    object_type: Option<&str>,
    ledger: &LedgerIndex,
) -> AccountObjectsRequest {
    AccountObjectsRequest {
        object_type: object_type.map(Into::into),
        ledger_index: *ledger,
        limit: Some(200),
        ..AccountObjectsRequest::new(account)
    }
}

/// Seconds between looks at a submitted transaction.
const POLL_SECS: u64 = 2;

/// Failed looks in a row before [`Client::submit_and_wait`] gives up with
/// [`ClientError::OutcomeUnknown`].
const MAX_POLL_FAILURES: u32 = 15;

/// What one look at a submitted transaction found.
enum Poll {
    /// In a validated ledger.
    Validated(Box<TxResult>),
    /// Not final yet; `found` if it is in a ledger not yet validated.
    Pending {
        /// In a ledger not validated yet.
        found: bool,
    },
    /// Its last ledger is validated and no ledger up to it holds it.
    Expired,
    /// Its last ledger is validated, but the server lacks ledgers it could be in.
    Unprovable,
}

/// One look at a submitted transaction (see [`Client::submit_and_wait`]).
async fn poll_outcome<C: Client + ?Sized>(
    client: &C,
    hash: &str,
    first_ledger: u32,
    last_ledger_sequence: u32,
) -> Result<Poll, ClientError> {
    // Read the validated index first: if it has reached the last ledger,
    // a miss below covers every ledger the transaction could be in.
    let validated = client.validated_ledger_index().await?;
    let lookup = TxRequest {
        // rippled searches at most 1,000 ledgers for `searched_all`; a
        // transaction found by its hash is returned wherever it is.
        min_ledger: Some(
            first_ledger
                .min(last_ledger_sequence)
                .max(last_ledger_sequence.saturating_sub(1000)),
        ),
        max_ledger: Some(last_ledger_sequence),
        ..TxRequest::new(hash)
    };
    match client.request(&lookup).await {
        Ok(result) if result.validated == Some(true) => Ok(Poll::Validated(Box::new(result))),
        Ok(_) => Ok(Poll::Pending { found: true }),
        Err(ClientError::RpcError {
            ref code,
            ref response,
            ..
        }) if code == "txnNotFound" => {
            if validated < last_ledger_sequence {
                Ok(Poll::Pending { found: false })
            } else if response.get("searched_all").and_then(Value::as_bool) == Some(true) {
                Ok(Poll::Expired)
            } else {
                Ok(Poll::Unprovable)
            }
        }
        Err(e) => Err(e),
    }
}

/// A preliminary result for a transaction the server neither applied, kept
/// nor relayed, which the same blob may pass when resubmitted.
fn is_dropped(engine_result: &str) -> bool {
    engine_result == "telINSUF_FEE_P" || engine_result.starts_with("telCAN_NOT_QUEUE")
}

/// Sort a submit's preliminary engine result: `tes` is Ok; `tem`, `tef` and
/// `tel` are [`ClientError::TransactionFailed`] except the load and queue
/// `tel`s, which with `ter` are [`ClientError::TransactionRetry`]; `tec` is
/// [`ClientError::TransactionClaimed`] (preliminary, no `result`). A code
/// with none of these prefixes is an unexpected response.
fn classify_submit(sr: SubmitResult) -> Result<SubmitResult, ClientError> {
    let r = sr.engine_result.as_str();
    if r.starts_with("tes") {
        Ok(sr)
    } else if r.starts_with("ter") || is_dropped(r) {
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
            result: None,
        })
    } else {
        Err(ClientError::UnexpectedResponse(format!(
            "engine result {r:?} is none of tes/tec/tem/tef/tel/ter"
        )))
    }
}

/// rippled's error, from the object that carries it (an HTTP response's
/// `result`, or a WebSocket message), with the whole object kept.
pub(crate) fn rpc_error(obj: &Value) -> ClientError {
    let text = |k: &str| obj.get(k).and_then(Value::as_str).map(str::to_string);
    ClientError::RpcError {
        code: text("error").unwrap_or_else(|| "unknown".to_string()),
        message: text("error_message").unwrap_or_default(),
        response: Box::new(obj.clone()),
    }
}

/// A WebSocket response's `result`, or the error it carries (`rippled` puts
/// a WebSocket error's fields at the top of the message).
pub(crate) fn ws_response(message: &Value) -> Result<Value, ClientError> {
    if message.get("status").and_then(Value::as_str) == Some("error")
        || message.get("error").is_some()
    {
        return Err(rpc_error(message));
    }
    message.get("result").cloned().ok_or_else(|| {
        ClientError::UnexpectedResponse("a response with neither a result nor an error".into())
    })
}

/// Platform-aware async sleep.
pub(crate) async fn sleep_secs(secs: u64) {
    sleep_for(std::time::Duration::from_secs(secs)).await;
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn sleep_for(duration: std::time::Duration) {
    tokio::time::sleep(duration).await;
}

/// Wait `duration` with the JavaScript global's `setTimeout`, which a
/// browser window, a Web Worker, a Service Worker, Node.js and Cloudflare
/// Workers all have (`web_sys::window()` exists only in a window: a worker
/// waited forever). In a runtime without `setTimeout` it returns at once.
#[cfg(target_arch = "wasm32")]
pub(crate) async fn sleep_for(duration: std::time::Duration) {
    use wasm_bindgen::{JsCast, JsValue};
    let millis = i32::try_from(duration.as_millis()).unwrap_or(i32::MAX);
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let global = js_sys::global();
        let set_timeout = js_sys::Reflect::get(&global, &JsValue::from_str("setTimeout"))
            .ok()
            .and_then(|f| f.dyn_into::<js_sys::Function>().ok());
        let started = set_timeout
            .map(|f| f.call2(&global, &resolve, &JsValue::from(millis)).is_ok())
            .unwrap_or(false);
        if !started {
            let _ = reject.call1(&JsValue::NULL, &JsValue::from_str("no setTimeout"));
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// The timer works outside a browser window: these run in Node.js, which,
/// like a Web Worker, has `setTimeout` but no `window`.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    async fn sleep_returns_without_a_window() {
        let has_window = js_sys::Reflect::has(&js_sys::global(), &"window".into()).unwrap_or(true);
        assert!(!has_window, "this test must run outside a browser window");
        let started = js_sys::Date::now();
        super::sleep_secs(1).await;
        let waited = js_sys::Date::now() - started;
        assert!((900.0..5_000.0).contains(&waited), "waited {waited} ms");
    }
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
        /// Replace the queued `server_state` answers with one whose owner
        /// reserve is `drops`.
        fn with_reserve(self, drops: u64) -> Self {
            self.replies.lock().unwrap().remove("server_state");
            self.reply(
                "server_state",
                json!({"state": {"validated_ledger": {"reserve_inc": drops}}}),
            )
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
            let reply = reply
                .ok_or_else(|| ClientError::UnexpectedResponse(format!("no reply for {method}")))?;
            // A rippled error, as the HTTP transport reads one.
            if reply.get("status").and_then(Value::as_str) == Some("error") {
                return Err(rpc_error(&reply));
            }
            Ok(reply)
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
            p(&TxRequest::new("AB")),
            json!({"transaction": "AB", "binary": false})
        );
        assert_eq!(p(&FeeRequest {}), json!({}));
        assert_eq!(
            p(&AccountChannelsRequest {
                ledger_index: LedgerIndex::Current,
                ..AccountChannelsRequest::new("rA")
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
                json!({"account": "rA", "lines": [line("USD")], "marker": "M1", "ledger_index": 100}),
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
        assert_eq!(calls[0].1["ledger_index"], "validated");
        assert_eq!(calls[1].1["marker"], "M1");
        // The second page is from the first page's ledger.
        assert_eq!(calls[1].1["ledger_index"], 100);
    }

    /// `pages` reads past `MAX_PAGES`, each page from the first one's
    /// ledger, and ends after the last.
    #[tokio::test]
    async fn pages_have_no_limit_and_stay_on_one_ledger() {
        use futures::StreamExt;
        let mut mock = Mock::default();
        let total = MAX_PAGES + 10;
        for n in 0..total {
            let mut page = json!({"account": "rA", "offers": [], "ledger_index": 100});
            if n + 1 < total {
                page["marker"] = json!(format!("M{n}"));
            }
            mock = mock.reply("account_offers", page);
        }
        let pages: Vec<_> = mock.pages(AccountOffersRequest::new("rA")).collect().await;
        assert_eq!(pages.len(), total);
        assert!(pages.iter().all(Result::is_ok));
        let calls = mock.calls();
        assert!(calls.iter().skip(1).all(|(_, p)| p["ledger_index"] == 100));
        assert_eq!(calls[total - 1].1["marker"], format!("M{}", total - 2));
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
            .reply("submit", reply("telBAD_PUBLIC_KEY"))
            .reply("submit", reply("xyzUNKNOWN"));
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
        // A result with none of the known prefixes is not success.
        assert!(matches!(
            mock.submit("00").await,
            Err(ClientError::UnexpectedResponse(_))
        ));
    }

    #[tokio::test]
    async fn autofill_numbers_and_prices_a_batch() {
        let (ra, rb, rc) = (
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1",
        );
        let mock = Mock::default()
            .reply("account_info", json!({"account_data": {"Account": ra, "Balance": "1", "Flags": 0, "OwnerCount": 0, "Sequence": 5}}))
            .reply("fee", json!({"drops": {"base_fee": "10", "median_fee": "10", "minimum_fee": "10", "open_ledger_fee": "12"}}))
            .reply("ledger_current", json!({"ledger_current_index": 100}))
            .reply("server_info", json!({"info": {"network_id": 1}}))
            .reply("account_info", json!({"account_data": {"Account": rb, "Balance": "1", "Flags": 0, "OwnerCount": 0, "Sequence": 9}}));
        let pay = |from: &str| json!({"RawTransaction": {"TransactionType": "Payment", "Account": from, "Destination": rc, "Amount": "1"}});
        let mut tx = json!({"TransactionType": "Batch", "Account": ra, "Flags": 0x0001_0000, "RawTransactions": [pay(ra), pay(rb), pay(ra)]});
        mock.autofill(&mut tx).await.unwrap();
        assert_eq!(tx["Sequence"], 5);
        let inner: Vec<&Value> = tx["RawTransactions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|w| &w["RawTransaction"])
            .collect();
        assert_eq!(
            (
                inner[0]["Sequence"].clone(),
                inner[1]["Sequence"].clone(),
                inner[2]["Sequence"].clone()
            ),
            (json!(6), json!(9), json!(7))
        );
        assert!(inner
            .iter()
            .all(|t| t["Fee"] == "0" && t["SigningPubKey"] == "" && t["Flags"] == 0x4000_0000));
        // 12 × (2 + 3 inner + 1 other signer)
        assert_eq!(tx["Fee"], "72");
    }

    #[tokio::test]
    async fn autofill_fills_only_what_is_missing() {
        let mock = Mock::default()
            .reply("fee", json!({"drops": {"base_fee": "10", "median_fee": "10", "minimum_fee": "10", "open_ledger_fee": "12"}}))
            .reply("ledger_current", json!({"ledger_current_index": 100}))
            .reply("server_info", json!({"info": {"network_id": 1}}));
        let mut tx =
            json!({"TransactionType": "Payment", "Account": "rA", "Sequence": 5, "Fee": null});
        mock.autofill(&mut tx).await.unwrap();
        assert_eq!(tx["Sequence"], 5);
        assert_eq!(tx["Fee"], "12");
        assert_eq!(tx["LastLedgerSequence"], 120);
        // Mainnet (0), Testnet (1) and Devnet (2) refuse a NetworkID.
        assert!(tx.get("NetworkID").is_none());
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
        send(http.submit_and_wait("00"));
        send(ws.account_info("rA", &LedgerIndex::Current));
        send(ws.account_tx_all("rA", None, None));
        send(ws.autofill(tx));
    }

    /// A Payment signed with a public test seed, `LastLedgerSequence` 100,
    /// as a blob, and its ID.
    fn signed_payment(last_ledger: Option<u32>) -> (String, String) {
        use xrpl_core::crypto::{sign_transaction, Keypair, Seed};
        let keypair =
            Keypair::from_seed(&Seed::from_base58("sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r").unwrap())
                .unwrap();
        let mut tx = json!({"TransactionType": "Payment", "Account": "rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD",
            "Destination": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "Amount": "1000000", "Fee": "12",
            "Sequence": 1});
        if let Some(l) = last_ledger {
            tx["LastLedgerSequence"] = json!(l);
        }
        let signed = sign_transaction(&tx, &keypair).unwrap();
        let blob =
            hex::encode_upper(xrpl_core::codec::encode_transaction_json(&signed, false).unwrap());
        let hash = xrpl_core::hashes::hash_signed_tx_blob(&blob)
            .unwrap()
            .to_string();
        (blob, hash)
    }

    fn validated(mock: Mock, indexes: &[u32]) -> Mock {
        indexes.iter().fold(mock, |m, i| {
            m.reply("ledger", json!({"ledger_index": i, "validated": true}))
        })
    }
    fn submitted(r: &str) -> Value {
        // The server's word for the hash is never used: ours is computed.
        json!({"engine_result": r, "engine_result_code": 0, "engine_result_message": "m",
            "tx_json": {"hash": "NOT_THE_HASH"}})
    }
    fn found(validated: bool, result: &str) -> Value {
        json!({"validated": validated, "ledger_index": 95, "meta": {"TransactionResult": result}})
    }
    fn not_found(searched_all: bool) -> Value {
        json!({"status": "error", "error": "txnNotFound", "error_message": "Transaction not found.",
            "searched_all": searched_all})
    }
    fn count(mock: &Mock, method: &str) -> usize {
        mock.calls().iter().filter(|(m, _)| m == method).count()
    }

    /// A preliminary tec is not the outcome: the validated result is.
    #[tokio::test(start_paused = true)]
    async fn submit_and_wait_takes_the_validated_result_not_the_preliminary_one() {
        let (blob, hash) = signed_payment(Some(100));
        let mock = validated(Mock::default(), &[90, 91])
            .reply("submit", submitted("tecPATH_DRY"))
            .reply("tx", found(true, "tesSUCCESS"));
        assert!(mock.submit_and_wait(&blob).await.is_ok());
        let lookup = &mock.calls().into_iter().find(|(m, _)| m == "tx").unwrap().1;
        assert_eq!(lookup["transaction"], hash.as_str());
        assert_eq!(
            (lookup["min_ledger"].clone(), lookup["max_ledger"].clone()),
            (json!(90), json!(100))
        );
    }

    /// terQUEUED is waited on, never resubmitted (a resubmit would answer tef).
    #[tokio::test(start_paused = true)]
    async fn a_queued_transaction_is_waited_on_not_resubmitted() {
        let (blob, _) = signed_payment(Some(100));
        let mock = validated(Mock::default(), &[90, 91, 92])
            .reply("submit", submitted("terQUEUED"))
            .reply("tx", not_found(false))
            .reply("tx", found(true, "tesSUCCESS"));
        assert!(mock.submit_and_wait(&blob).await.is_ok());
        assert_eq!(count(&mock, "submit"), 1);
    }

    /// A validated tec is an error that carries the validated transaction.
    #[tokio::test(start_paused = true)]
    async fn a_validated_tec_is_an_error_with_its_result() {
        let (blob, _) = signed_payment(Some(100));
        let mock = validated(Mock::default(), &[90, 91])
            .reply("submit", submitted("tesSUCCESS"))
            .reply("tx", found(true, "tecUNFUNDED_PAYMENT"));
        match mock.submit_and_wait(&blob).await {
            Err(ClientError::TransactionClaimed {
                engine_result,
                result: Some(result),
            }) => {
                assert_eq!(engine_result, "tecUNFUNDED_PAYMENT");
                assert_eq!(result.transaction_result(), Some("tecUNFUNDED_PAYMENT"));
            }
            other => panic!("{other:?}"),
        }
    }

    /// A transaction in the last ledger it allows is found, even though the
    /// network has moved past that ledger by the time it is looked up.
    #[tokio::test(start_paused = true)]
    async fn a_transaction_in_its_last_ledger_is_not_reported_expired() {
        let (blob, _) = signed_payment(Some(100));
        let mock = validated(Mock::default(), &[90, 101])
            .reply("submit", submitted("terQUEUED"))
            .reply("tx", found(true, "tesSUCCESS"));
        assert!(mock.submit_and_wait(&blob).await.is_ok());
    }

    /// Expired only when the last ledger is validated and the server searched
    /// every ledger up to it.
    #[tokio::test(start_paused = true)]
    async fn expired_needs_the_last_ledger_validated_and_searched() {
        let (blob, _) = signed_payment(Some(100));
        let mock = validated(Mock::default(), &[90, 99, 100])
            .reply("submit", submitted("tesSUCCESS"))
            .reply("tx", not_found(false))
            .reply("tx", not_found(true));
        assert!(matches!(
            mock.submit_and_wait(&blob).await,
            Err(ClientError::TransactionExpired(100))
        ));
        // A server missing some of those ledgers cannot prove it.
        let mock = validated(Mock::default(), &[90, 100])
            .reply("submit", submitted("tesSUCCESS"))
            .reply("tx", not_found(false));
        assert!(matches!(
            mock.submit_and_wait(&blob).await,
            Err(ClientError::OutcomeUnknown {
                last_ledger_sequence: 100,
                ..
            })
        ));
    }

    /// tem is final at once; a tef is final only for a transaction no ledger holds.
    #[tokio::test(start_paused = true)]
    async fn malformed_and_refused_transactions_fail_at_once() {
        let (blob, _) = signed_payment(Some(100));
        let mock = validated(Mock::default(), &[90]).reply("submit", submitted("temBAD_AMOUNT"));
        assert!(matches!(
            mock.submit_and_wait(&blob).await,
            Err(ClientError::TransactionFailed { .. })
        ));
        assert_eq!(count(&mock, "tx"), 0);

        let mock = validated(Mock::default(), &[90])
            .reply("submit", submitted("tefPAST_SEQ"))
            .reply("tx", not_found(false));
        assert!(matches!(
            mock.submit_and_wait(&blob).await,
            Err(ClientError::TransactionFailed { .. })
        ));
        // tefPAST_SEQ for a blob an earlier submit already put in a ledger.
        let mock = validated(Mock::default(), &[90, 91])
            .reply("submit", submitted("tefPAST_SEQ"))
            .reply("tx", found(true, "tesSUCCESS"))
            .reply("tx", found(true, "tesSUCCESS"));
        assert!(mock.submit_and_wait(&blob).await.is_ok());
    }

    /// A transaction the server dropped, or a submit whose answer was lost,
    /// is resubmitted as the same blob until it is found.
    #[tokio::test(start_paused = true)]
    async fn a_dropped_transaction_is_resubmitted_until_found() {
        let (blob, _) = signed_payment(Some(100));
        let mock = validated(Mock::default(), &[90, 91, 92])
            .reply("submit", submitted("telCAN_NOT_QUEUE_FULL"))
            .reply("submit", submitted("telCAN_NOT_QUEUE_FULL"))
            .reply("tx", not_found(false))
            .reply("tx", found(true, "tesSUCCESS"));
        assert!(mock.submit_and_wait(&blob).await.is_ok());
        assert_eq!(count(&mock, "submit"), 2);
        assert!(mock
            .calls()
            .iter()
            .filter(|(m, _)| m == "submit")
            .all(|(_, p)| p["tx_blob"] == blob.as_str()));

        // No answer to the submit at all (the mock has none): look it up.
        let mock = validated(Mock::default(), &[90, 91, 92])
            .reply("tx", not_found(false))
            .reply("tx", found(true, "tesSUCCESS"));
        assert!(mock.submit_and_wait(&blob).await.is_ok());
    }

    #[tokio::test(start_paused = true)]
    async fn submit_and_wait_needs_a_last_ledger_sequence() {
        let (blob, _) = signed_payment(None);
        let mock = Mock::default();
        assert!(matches!(
            mock.submit_and_wait(&blob).await,
            Err(ClientError::Core(_))
        ));
        assert!(mock.calls().is_empty(), "nothing may be sent");
    }

    /// The server's fee is capped, and a multi-signed transaction's fee
    /// counts its signers.
    #[tokio::test]
    async fn autofill_caps_the_fee_and_counts_signers() {
        let fee = |open: &str| {
            Mock::default()
                .reply("fee", json!({"drops": {"base_fee": "10", "median_fee": "10", "minimum_fee": "10", "open_ledger_fee": open}}))
                .reply("server_info", json!({"info": {"network_id": 1}}))
        };
        let tx = || json!({"TransactionType": "Payment", "Account": "rA", "Sequence": 5, "LastLedgerSequence": 9});
        let mut t = tx();
        assert!(matches!(
            fee("100000000000").autofill(&mut t).await,
            Err(ClientError::FeeTooHigh {
                fee_drops: 100_000_000_000,
                max_fee_drops: DEFAULT_MAX_FEE_DROPS
            })
        ));
        assert!(t.get("Fee").is_none(), "no fee is set above the cap");
        let mut t = tx();
        fee("2000000").autofill(&mut t).await.unwrap();
        assert_eq!(t["Fee"], "2000000");
        let mut t = tx();
        fee("12")
            .autofill_for_multisigning(&mut t, 3)
            .await
            .unwrap();
        assert_eq!(
            (t["Fee"].clone(), t["SigningPubKey"].clone()),
            (json!("48"), json!(""))
        );
        let mut t = tx();
        assert!(matches!(
            fee("12").autofill(&mut json!({"TransactionType": "Payment", "Account": "rA", "Sequence": 5, "LastLedgerSequence": 9, "Fee": null, "NetworkID": 1})).await,
            Ok(())
        ));
        assert!(fee("lots").autofill(&mut t).await.is_err());
    }

    /// `balances` reads XRP, trust lines and MPTs from the one ledger the
    /// first answer came from.
    #[tokio::test]
    async fn balances_are_read_from_one_ledger() {
        let issuance = "000004C463C52827307480341125DA0577DEFC38405B0E3E";
        let mock = Mock::default()
            .reply("account_info", json!({"account_data": {"Account": "rA", "Balance": "25000000", "Flags": 0, "OwnerCount": 2, "Sequence": 5}, "ledger_index": 100, "validated": true}))
            .reply("account_lines", json!({"account": "rA", "lines": [
                {"account": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "balance": "-12.5", "currency": "USD", "limit": "0", "limit_peer": "100"}]}))
            .reply("account_objects", json!({"account": "rA", "account_objects": [
                {"LedgerEntryType": "MPToken", "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", "MPTokenIssuanceID": issuance, "MPTAmount": "750", "Flags": 0, "OwnerNode": "0", "PreviousTxnID": "0000000000000000000000000000000000000000000000000000000000000000", "PreviousTxnLgrSeq": 1, "index": "0000000000000000000000000000000000000000000000000000000000000000"}]}));
        let amounts = mock.balances("rA", &LedgerIndex::Validated).await.unwrap();
        assert_eq!(
            amounts,
            vec![
                xrpl_core::types::Amount::xrp(25_000_000).unwrap(),
                xrpl_core::types::Amount::token(
                    "-12.5",
                    "USD",
                    "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"
                )
                .unwrap(),
                xrpl_core::types::Amount::mpt(750, issuance).unwrap(),
            ]
        );
        for (method, params) in mock.calls().iter().skip(1) {
            assert_eq!(params["ledger_index"], 100, "{method}");
        }
    }

    /// The fee follows rippled 3.4.1's price for each type: one owner
    /// reserve for AccountDelete (scaled when the open ledger escalates),
    /// more base fees for an EscrowFinish with a fulfillment and for a
    /// Confidential MPT transaction. (AccountDelete was priced at one base
    /// fee, which rippled refuses: telCAN_NOT_QUEUE, which submit_and_wait
    /// took as dropped and resubmitted until the transaction expired.)
    #[tokio::test]
    async fn autofill_prices_each_type_as_rippled_does() {
        let mock = |open: &str| {
            Mock::default()
                .reply("fee", json!({"drops": {"base_fee": "10", "median_fee": "10", "minimum_fee": "10", "open_ledger_fee": open}}))
                .reply("server_state", json!({"state": {"validated_ledger": {"base_fee": 10, "reserve_base": 1_000_000, "reserve_inc": 200_000, "seq": 9}}}))
        };
        let tx = |kind: &str| json!({"TransactionType": kind, "Account": "rA", "Sequence": 5, "LastLedgerSequence": 9, "NetworkID": 1});
        let fee_of = |m: Mock, mut t: Value, signers: u32| async move {
            m.autofill_for_multisigning(&mut t, signers).await.unwrap();
            (
                t["Fee"].as_str().unwrap().to_string(),
                m.calls()
                    .into_iter()
                    .map(|(method, _)| method)
                    .collect::<Vec<_>>(),
            )
        };
        // One owner reserve, whoever signs.
        let (fee, calls) = fee_of(mock("10"), tx("AccountDelete"), 0).await;
        assert_eq!(fee, "200000");
        assert!(calls.contains(&"server_state".to_string()));
        assert_eq!(fee_of(mock("10"), tx("AMMCreate"), 3).await.0, "200000");
        // An escalated open ledger (2.5 times the base fee) scales the reserve.
        assert_eq!(fee_of(mock("25"), tx("AccountDelete"), 0).await.0, "500000");
        // 100 bytes of fulfillment: 1 + 32 + 6 base fees.
        let mut escrow = tx("EscrowFinish");
        escrow["Fulfillment"] = json!("A0".repeat(100));
        let (fee, calls) = fee_of(mock("12"), escrow, 0).await;
        assert_eq!(fee, (12 * 39).to_string());
        assert!(
            !calls.contains(&"server_state".to_string()),
            "no reserve needed"
        );
        assert_eq!(
            fee_of(mock("12"), tx("ConfidentialMPTSend"), 1).await.0,
            (12 * 11).to_string()
        );
        // An owner reserve above the cap is refused like any fee.
        let mut t = tx("AccountDelete");
        assert!(matches!(
            mock("10").with_reserve(5_000_000).autofill(&mut t).await,
            Err(ClientError::FeeTooHigh {
                fee_drops: 5_000_000,
                ..
            })
        ));
    }

    /// A LoanSet is priced for its counterparty's signatures: the size of
    /// the counterparty's signer list (here the loan broker's owner's), or
    /// one when it has none.
    #[tokio::test]
    async fn autofill_prices_a_loan_set_for_its_counterparty() {
        let fee = json!({"drops": {"base_fee": "10", "median_fee": "10", "minimum_fee": "10", "open_ledger_fee": "10"}});
        let tx = json!({"TransactionType": "LoanSet", "Account": "rBorrower", "LoanBrokerID": "AB", "Sequence": 5, "LastLedgerSequence": 9, "NetworkID": 1});
        let mock = Mock::default()
            .reply("fee", fee.clone())
            .reply("ledger_entry", json!({"node": {"LedgerEntryType": "LoanBroker", "Owner": "rBroker"}}))
            .reply("account_info", json!({"account_data": {"Account": "rBroker", "signer_lists": [{"SignerEntries": [{}, {}, {}]}]}}));
        let mut t = tx.clone();
        mock.autofill(&mut t).await.unwrap();
        assert_eq!(t["Fee"], "40");
        let calls = mock.calls();
        let info = calls.iter().find(|(m, _)| m == "account_info").unwrap();
        assert_eq!(info.1["account"], "rBroker");
        // Named counterparty without a signer list (API version 2 shape): one signature.
        let mock = Mock::default().reply("fee", fee).reply(
            "account_info",
            json!({"account_data": {"Account": "rLender"}, "signer_lists": []}),
        );
        let mut t = tx;
        t["Counterparty"] = json!("rLender");
        mock.autofill(&mut t).await.unwrap();
        assert_eq!(t["Fee"], "20");
    }

    /// A batch's inner transactions are priced by their own type.
    #[tokio::test]
    async fn autofill_prices_a_batch_by_its_inner_types() {
        let ra = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
        let mock = Mock::default()
            .reply("fee", json!({"drops": {"base_fee": "10", "median_fee": "10", "minimum_fee": "10", "open_ledger_fee": "10"}}))
            .reply("server_state", json!({"state": {"validated_ledger": {"reserve_inc": 200_000}}}));
        let inner = |kind: &str, seq: u32| json!({"RawTransaction": {"TransactionType": kind, "Account": ra, "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "Amount": "1", "Sequence": seq}});
        let mut tx = json!({"TransactionType": "Batch", "Account": ra, "Flags": 0x0001_0000, "Sequence": 5, "LastLedgerSequence": 9, "NetworkID": 1,
            "RawTransactions": [inner("Payment", 6), inner("AccountDelete", 7)]});
        mock.autofill(&mut tx).await.unwrap();
        // 10 × (2 + 1 Payment) + one owner reserve.
        assert_eq!(tx["Fee"], "200030");
    }

    /// A Ticket transaction's Sequence is 0; a network above 1024 gets its ID.
    #[tokio::test]
    async fn autofill_handles_tickets_and_network_ids() {
        let mock = Mock::default()
            .reply("ledger_current", json!({"ledger_current_index": 100}))
            .reply("server_info", json!({"info": {"network_id": 21338}}));
        let mut tx = json!({"TransactionType": "Payment", "Account": "rA", "TicketSequence": 7, "Fee": "12"});
        mock.autofill(&mut tx).await.unwrap();
        assert_eq!(
            (tx["Sequence"].clone(), tx["NetworkID"].clone()),
            (json!(0), json!(21338))
        );
        assert_eq!(count(&mock, "account_info"), 0);
    }

    /// A Testnet X-address in a transaction for a mainnet server is refused.
    #[tokio::test]
    async fn autofill_refuses_an_x_address_for_the_other_network() {
        let classic = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
        let test_x = xrpl_core::address::classic_to_x_address(classic, Some(7), true).unwrap();
        let main_x = xrpl_core::address::classic_to_x_address(classic, Some(7), false).unwrap();
        let server =
            |id: u32| Mock::default().reply("server_info", json!({"info": {"network_id": id}}));
        let tx = |dest: &str| {
            json!({"TransactionType": "Payment", "Account": "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "Destination": dest, "Amount": "1", "Fee": "12", "Sequence": 1, "LastLedgerSequence": 9})
        };
        assert!(server(0).autofill(&mut tx(&test_x)).await.is_err());
        assert!(server(1).autofill(&mut tx(&main_x)).await.is_err());
        assert!(server(0).autofill(&mut tx(&main_x)).await.is_ok());
        assert!(server(1).autofill(&mut tx(&test_x)).await.is_ok());
    }
}
