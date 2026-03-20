use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// EscrowCreate transaction.
#[derive(Debug, Clone)]
pub struct EscrowCreate {
    pub common: TxCommon,
    pub amount: String,
    pub destination: String,
    pub cancel_after: Option<u32>,
    pub finish_after: Option<u32>,
    pub condition: Option<String>,
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
    pub fn amount(mut self, a: impl Into<String>) -> Self {
        self.amount = Some(a.into());
        self
    }
    pub fn destination(mut self, d: impl Into<String>) -> Self {
        self.destination = Some(d.into());
        self
    }
    pub fn cancel_after(mut self, ca: u32) -> Self {
        self.cancel_after = Some(ca);
        self
    }
    pub fn finish_after(mut self, fa: u32) -> Self {
        self.finish_after = Some(fa);
        self
    }
    pub fn condition(mut self, c: impl Into<String>) -> Self {
        self.condition = Some(c.into());
        self
    }
    pub fn destination_tag(mut self, dt: u32) -> Self {
        self.destination_tag = Some(dt);
        self
    }
    pub fn fee(mut self, f: impl Into<String>) -> Self {
        self.common.fee = Some(f.into());
        self
    }
    pub fn sequence(mut self, s: u32) -> Self {
        self.common.sequence = Some(s);
        self
    }
    pub fn last_ledger_sequence(mut self, l: u32) -> Self {
        self.common.last_ledger_sequence = Some(l);
        self
    }
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
    pub fn builder(account: impl Into<String>) -> EscrowCreateBuilder {
        EscrowCreateBuilder::new(account)
    }
}

/// EscrowFinish transaction.
#[derive(Debug, Clone)]
pub struct EscrowFinish {
    pub common: TxCommon,
    pub owner: String,
    pub offer_sequence: u32,
    pub condition: Option<String>,
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

pub struct EscrowFinishBuilder {
    common: TxCommon,
    owner: Option<String>,
    offer_sequence: Option<u32>,
    condition: Option<String>,
    fulfillment: Option<String>,
}
impl EscrowFinishBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            owner: None,
            offer_sequence: None,
            condition: None,
            fulfillment: None,
        }
    }
    pub fn owner(mut self, o: impl Into<String>) -> Self {
        self.owner = Some(o.into());
        self
    }
    pub fn offer_sequence(mut self, s: u32) -> Self {
        self.offer_sequence = Some(s);
        self
    }
    pub fn condition(mut self, c: impl Into<String>) -> Self {
        self.condition = Some(c.into());
        self
    }
    pub fn fulfillment(mut self, f: impl Into<String>) -> Self {
        self.fulfillment = Some(f.into());
        self
    }
    pub fn fee(mut self, f: impl Into<String>) -> Self {
        self.common.fee = Some(f.into());
        self
    }
    pub fn sequence(mut self, s: u32) -> Self {
        self.common.sequence = Some(s);
        self
    }
    pub fn last_ledger_sequence(mut self, l: u32) -> Self {
        self.common.last_ledger_sequence = Some(l);
        self
    }
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
    pub fn builder(account: impl Into<String>) -> EscrowFinishBuilder {
        EscrowFinishBuilder::new(account)
    }
}

/// EscrowCancel transaction.
#[derive(Debug, Clone)]
pub struct EscrowCancel {
    pub common: TxCommon,
    pub owner: String,
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

pub struct EscrowCancelBuilder {
    common: TxCommon,
    owner: Option<String>,
    offer_sequence: Option<u32>,
}
impl EscrowCancelBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            owner: None,
            offer_sequence: None,
        }
    }
    pub fn owner(mut self, o: impl Into<String>) -> Self {
        self.owner = Some(o.into());
        self
    }
    pub fn offer_sequence(mut self, s: u32) -> Self {
        self.offer_sequence = Some(s);
        self
    }
    pub fn fee(mut self, f: impl Into<String>) -> Self {
        self.common.fee = Some(f.into());
        self
    }
    pub fn sequence(mut self, s: u32) -> Self {
        self.common.sequence = Some(s);
        self
    }
    pub fn last_ledger_sequence(mut self, l: u32) -> Self {
        self.common.last_ledger_sequence = Some(l);
        self
    }
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
    pub fn builder(account: impl Into<String>) -> EscrowCancelBuilder {
        EscrowCancelBuilder::new(account)
    }
}
