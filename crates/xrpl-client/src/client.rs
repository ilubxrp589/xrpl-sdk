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

/// `fee`, or [`ClientError::FeeTooHigh`] above `max`.
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
    /// - `Fee`: the open ledger's fee for it, refused with
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
        let is_batch = tx.get("TransactionType").and_then(Value::as_str) == Some("Batch");

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
        let network_fut = async {
            if need_network {
                self.server_info().await.map(|r| r.info.network_id)
            } else {
                Ok(None)
            }
        };
        let (acct_info, fee_result, ledger_idx, network_id) =
            futures::future::try_join4(seq_fut, fee_fut, lls_fut, network_fut).await?;
        let open_ledger_fee = fee_result
            .map(|f| {
                f.drops.open_ledger_fee.parse::<u64>().map_err(|_| {
                    ClientError::UnexpectedResponse(format!(
                        "open_ledger_fee {:?} is not a whole number of drops",
                        f.drops.open_ledger_fee
                    ))
                })
            })
            .transpose()?;
        // Refuse a fee above the cap before changing anything.
        let fee = match open_ledger_fee {
            Some(fee) if !is_batch => Some(capped_fee(
                fee.saturating_mul(u64::from(signers) + 1),
                self.max_fee_drops(),
            )?),
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
            return self.autofill_batch(tx, &account, open_ledger_fee).await;
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
    /// was missing, charge the batch for all of it: the open-ledger fee ×
    /// (2 + inner transactions + other accounts that must sign).
    async fn autofill_batch(
        &self,
        tx: &mut Value,
        outer: &str,
        open_ledger_fee: Option<u64>,
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
        let mut others = std::collections::BTreeSet::new();
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
            if account != outer {
                others.insert(account.clone());
            }
            if inner.get("Sequence").is_some() || inner.get("TicketSequence").is_some() {
                continue;
            }
            let seq = match next.get(&account) {
                Some(s) => *s,
                None => {
                    self.account_info(&account, &LedgerIndex::Current)
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
        if let Some(fee) = open_ledger_fee {
            let inner_count = raw.len();
            let total = xrpl_core::transaction::batch::batch_fee(fee, inner_count, others.len());
            let total = capped_fee(total, self.max_fee_drops())?;
            if let Some(obj) = tx.as_object_mut() {
                obj.insert("Fee".into(), json!(total.to_string()));
            }
        }
        Ok(())
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
#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn sleep_secs(secs: u64) {
    tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
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
        let mock = Mock::default()
            .reply("account_info", json!({"account_data": {"Account": "rA", "Balance": "1", "Flags": 0, "OwnerCount": 0, "Sequence": 5}}))
            .reply("fee", json!({"drops": {"base_fee": "10", "median_fee": "10", "minimum_fee": "10", "open_ledger_fee": "12"}}))
            .reply("ledger_current", json!({"ledger_current_index": 100}))
            .reply("server_info", json!({"info": {"network_id": 1}}))
            .reply("account_info", json!({"account_data": {"Account": "rB", "Balance": "1", "Flags": 0, "OwnerCount": 0, "Sequence": 9}}));
        let pay = |from: &str| json!({"RawTransaction": {"TransactionType": "Payment", "Account": from, "Destination": "rC", "Amount": "1"}});
        let mut tx = json!({"TransactionType": "Batch", "Account": "rA", "Flags": 0x0001_0000, "RawTransactions": [pay("rA"), pay("rB"), pay("rA")]});
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
}
