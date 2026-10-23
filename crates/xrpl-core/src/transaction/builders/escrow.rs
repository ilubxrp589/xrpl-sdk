use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// EscrowCreate transaction.
#[derive(Debug, Clone)]
pub struct EscrowCreate {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// How much (`Amount`): XRP as a string of **drops** (`"1000000"` is 1 XRP; `utils::xrp_to_drops`), or a token `{"currency", "issuer", "value"}`.
    pub amount: String,
    /// The account that receives it (`Destination`).
    pub destination: String,
    /// The time after which it can be cancelled (`CancelAfter`). Ledger time: seconds since 2000-01-01 UTC, not Unix time (`utils::unix_to_ripple_time`).
    pub cancel_after: Option<u32>,
    /// The time after which it can be finished (`FinishAfter`). Ledger time: seconds since 2000-01-01 UTC, not Unix time (`utils::unix_to_ripple_time`).
    pub finish_after: Option<u32>,
    /// A PREIMAGE-SHA-256 crypto-condition that must be fulfilled (`Condition`).
    pub condition: Option<String>,
    /// A tag the destination uses to tell its senders apart (`DestinationTag`).
    pub destination_tag: Option<u32>,
}

impl Transaction for EscrowCreate {
    fn transaction_type(&self) -> &'static str {
        "EscrowCreate"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("EscrowCreate"));
        obj.insert("Amount".into(), json!(self.amount));
        obj.insert("Destination".into(), json!(self.destination));
        if let Some(ca) = self.cancel_after {
            obj.insert("CancelAfter".into(), json!(ca));
        }
        if let Some(fa) = self.finish_after {
            obj.insert("FinishAfter".into(), json!(fa));
        }
        if let Some(ref c) = self.condition {
            obj.insert("Condition".into(), json!(c));
        }
        if let Some(dt) = self.destination_tag {
            obj.insert("DestinationTag".into(), json!(dt));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        if self.amount.is_empty() {
            return Err(CoreError::ValidationError(
                "EscrowCreate: amount is required".into(),
            ));
        }
        if self.destination.is_empty() {
            return Err(CoreError::ValidationError(
                "EscrowCreate: destination is required".into(),
            ));
        }
        Ok(())
    }
}

/// Builds a [`EscrowCreate`] (from [`EscrowCreate::builder`]); `build` checks the required fields.
#[derive(Debug)]
pub struct EscrowCreateBuilder {
    common: TxCommon,
    amount: Option<String>,
    destination: Option<String>,
    cancel_after: Option<u32>,
    finish_after: Option<u32>,
    condition: Option<String>,
    destination_tag: Option<u32>,
}

impl EscrowCreateBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            amount: None,
            destination: None,
            cancel_after: None,
            finish_after: None,
            condition: None,
            destination_tag: None,
        }
    }
    /// Set `Amount`.
    pub fn amount(mut self, a: impl Into<String>) -> Self {
        self.amount = Some(a.into());
        self
    }
    /// Set `Destination`.
    pub fn destination(mut self, d: impl Into<String>) -> Self {
        self.destination = Some(d.into());
        self
    }
    /// Set `CancelAfter`.
    pub fn cancel_after(mut self, ca: u32) -> Self {
        self.cancel_after = Some(ca);
        self
    }
    /// Set `FinishAfter`.
    pub fn finish_after(mut self, fa: u32) -> Self {
        self.finish_after = Some(fa);
        self
    }
    /// Set `Condition`.
    pub fn condition(mut self, c: impl Into<String>) -> Self {
        self.condition = Some(c.into());
        self
    }
    /// Set `DestinationTag`.
    pub fn destination_tag(mut self, dt: u32) -> Self {
        self.destination_tag = Some(dt);
        self
    }
    /// Set `Fee`.
    pub fn fee(mut self, f: impl Into<String>) -> Self {
        self.common.fee = Some(f.into());
        self
    }
    /// Set `Sequence`.
    pub fn sequence(mut self, s: u32) -> Self {
        self.common.sequence = Some(s);
        self
    }
    /// Set `LastLedgerSequence`.
    pub fn last_ledger_sequence(mut self, l: u32) -> Self {
        self.common.last_ledger_sequence = Some(l);
        self
    }
    /// Check the required fields and build the transaction.
    ///
    /// # Errors
    ///
    /// [`CoreError::ValidationError`] if `amount` or `destination` is missing
    /// or empty.
    pub fn build(self) -> Result<EscrowCreate, CoreError> {
        let amount = self
            .amount
            .ok_or_else(|| CoreError::ValidationError("EscrowCreate: amount is required".into()))?;
        let destination = self.destination.ok_or_else(|| {
            CoreError::ValidationError("EscrowCreate: destination is required".into())
        })?;
        let tx = EscrowCreate {
            common: self.common,
            amount,
            destination,
            cancel_after: self.cancel_after,
            finish_after: self.finish_after,
            condition: self.condition,
            destination_tag: self.destination_tag,
        };
        tx.validate()?;
        Ok(tx)
    }
}

impl EscrowCreate {
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> EscrowCreateBuilder {
        EscrowCreateBuilder::new(account)
    }
}

/// EscrowFinish transaction.
#[derive(Debug, Clone)]
pub struct EscrowFinish {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// The account that owns the entry (`Owner`).
    pub owner: String,
    /// The sequence of the transaction that created the entry (`OfferSequence`).
    pub offer_sequence: u32,
    /// A PREIMAGE-SHA-256 crypto-condition that must be fulfilled (`Condition`).
    pub condition: Option<String>,
    /// The fulfillment of the escrow's condition (`Fulfillment`).
    pub fulfillment: Option<String>,
}

impl Transaction for EscrowFinish {
    fn transaction_type(&self) -> &'static str {
        "EscrowFinish"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("EscrowFinish"));
        obj.insert("Owner".into(), json!(self.owner));
        obj.insert("OfferSequence".into(), json!(self.offer_sequence));
        if let Some(ref c) = self.condition {
            obj.insert("Condition".into(), json!(c));
        }
        if let Some(ref f) = self.fulfillment {
            obj.insert("Fulfillment".into(), json!(f));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}

/// Builds a [`EscrowFinish`] (from [`EscrowFinish::builder`]); `build` checks the required fields.
#[derive(Debug)]
pub struct EscrowFinishBuilder {
    common: TxCommon,
    owner: Option<String>,
    offer_sequence: Option<u32>,
    condition: Option<String>,
    fulfillment: Option<String>,
}
impl EscrowFinishBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            owner: None,
            offer_sequence: None,
            condition: None,
            fulfillment: None,
        }
    }
    /// Set `Owner`.
    pub fn owner(mut self, o: impl Into<String>) -> Self {
        self.owner = Some(o.into());
        self
    }
    /// Set `OfferSequence`.
    pub fn offer_sequence(mut self, s: u32) -> Self {
        self.offer_sequence = Some(s);
        self
    }
    /// Set `Condition`.
    pub fn condition(mut self, c: impl Into<String>) -> Self {
        self.condition = Some(c.into());
        self
    }
    /// Set `Fulfillment`.
    pub fn fulfillment(mut self, f: impl Into<String>) -> Self {
        self.fulfillment = Some(f.into());
        self
    }
    /// Set `Fee`.
    pub fn fee(mut self, f: impl Into<String>) -> Self {
        self.common.fee = Some(f.into());
        self
    }
    /// Set `Sequence`.
    pub fn sequence(mut self, s: u32) -> Self {
        self.common.sequence = Some(s);
        self
    }
    /// Set `LastLedgerSequence`.
    pub fn last_ledger_sequence(mut self, l: u32) -> Self {
        self.common.last_ledger_sequence = Some(l);
        self
    }
    /// Check the required fields and build the transaction.
    ///
    /// # Errors
    ///
    /// [`CoreError::ValidationError`] if `owner` or `offer_sequence` is
    /// missing.
    pub fn build(self) -> Result<EscrowFinish, CoreError> {
        let owner = self
            .owner
            .ok_or_else(|| CoreError::ValidationError("EscrowFinish: owner is required".into()))?;
        let offer_sequence = self.offer_sequence.ok_or_else(|| {
            CoreError::ValidationError("EscrowFinish: offer_sequence is required".into())
        })?;
        Ok(EscrowFinish {
            common: self.common,
            owner,
            offer_sequence,
            condition: self.condition,
            fulfillment: self.fulfillment,
        })
    }
}
impl EscrowFinish {
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> EscrowFinishBuilder {
        EscrowFinishBuilder::new(account)
    }
}

/// EscrowCancel transaction.
#[derive(Debug, Clone)]
pub struct EscrowCancel {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// The account that owns the entry (`Owner`).
    pub owner: String,
    /// The sequence of the transaction that created the entry (`OfferSequence`).
    pub offer_sequence: u32,
}

impl Transaction for EscrowCancel {
    fn transaction_type(&self) -> &'static str {
        "EscrowCancel"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("EscrowCancel"));
        obj.insert("Owner".into(), json!(self.owner));
        obj.insert("OfferSequence".into(), json!(self.offer_sequence));
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}

/// Builds a [`EscrowCancel`] (from [`EscrowCancel::builder`]); `build` checks the required fields.
#[derive(Debug)]
pub struct EscrowCancelBuilder {
    common: TxCommon,
    owner: Option<String>,
    offer_sequence: Option<u32>,
}
impl EscrowCancelBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            owner: None,
            offer_sequence: None,
        }
    }
    /// Set `Owner`.
    pub fn owner(mut self, o: impl Into<String>) -> Self {
        self.owner = Some(o.into());
        self
    }
    /// Set `OfferSequence`.
    pub fn offer_sequence(mut self, s: u32) -> Self {
        self.offer_sequence = Some(s);
        self
    }
    /// Set `Fee`.
    pub fn fee(mut self, f: impl Into<String>) -> Self {
        self.common.fee = Some(f.into());
        self
    }
    /// Set `Sequence`.
    pub fn sequence(mut self, s: u32) -> Self {
        self.common.sequence = Some(s);
        self
    }
    /// Set `LastLedgerSequence`.
    pub fn last_ledger_sequence(mut self, l: u32) -> Self {
        self.common.last_ledger_sequence = Some(l);
        self
    }
    /// Check the required fields and build the transaction.
    ///
    /// # Errors
    ///
    /// [`CoreError::ValidationError`] if `owner` or `offer_sequence` is
    /// missing.
    pub fn build(self) -> Result<EscrowCancel, CoreError> {
        let owner = self
            .owner
            .ok_or_else(|| CoreError::ValidationError("EscrowCancel: owner is required".into()))?;
        let offer_sequence = self.offer_sequence.ok_or_else(|| {
            CoreError::ValidationError("EscrowCancel: offer_sequence is required".into())
        })?;
        Ok(EscrowCancel {
            common: self.common,
            owner,
            offer_sequence,
        })
    }
}
impl EscrowCancel {
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> EscrowCancelBuilder {
        EscrowCancelBuilder::new(account)
    }
}
