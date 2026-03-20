use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// CheckCreate transaction.
#[derive(Debug, Clone)]
pub struct CheckCreate {
    pub common: TxCommon,
    pub destination: String,
    pub send_max: Value,
    pub expiration: Option<u32>,
    pub destination_tag: Option<u32>,
    pub invoice_id: Option<String>,
}
impl Transaction for CheckCreate {
    fn transaction_type(&self) -> &'static str {
        "CheckCreate"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("CheckCreate"));
        obj.insert("Destination".into(), json!(self.destination));
        obj.insert("SendMax".into(), self.send_max.clone());
        if let Some(e) = self.expiration {
            obj.insert("Expiration".into(), json!(e));
        }
        if let Some(dt) = self.destination_tag {
            obj.insert("DestinationTag".into(), json!(dt));
        }
        if let Some(ref id) = self.invoice_id {
            obj.insert("InvoiceID".into(), json!(id));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
pub struct CheckCreateBuilder {
    common: TxCommon,
    destination: Option<String>,
    send_max: Option<Value>,
    expiration: Option<u32>,
    destination_tag: Option<u32>,
    invoice_id: Option<String>,
}
impl CheckCreateBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            destination: None,
            send_max: None,
            expiration: None,
            destination_tag: None,
            invoice_id: None,
        }
    }
    pub fn destination(mut self, d: impl Into<String>) -> Self {
        self.destination = Some(d.into());
        self
    }
    pub fn send_max(mut self, s: Value) -> Self {
        self.send_max = Some(s);
        self
    }
    pub fn expiration(mut self, e: u32) -> Self {
        self.expiration = Some(e);
        self
    }
    pub fn destination_tag(mut self, dt: u32) -> Self {
        self.destination_tag = Some(dt);
        self
    }
    pub fn invoice_id(mut self, id: impl Into<String>) -> Self {
        self.invoice_id = Some(id.into());
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
    pub fn build(self) -> Result<CheckCreate, CoreError> {
        let destination = self.destination.ok_or_else(|| {
            CoreError::ValidationError("CheckCreate: destination is required".into())
        })?;
        let send_max = self.send_max.ok_or_else(|| {
            CoreError::ValidationError("CheckCreate: send_max is required".into())
        })?;
        Ok(CheckCreate {
            common: self.common,
            destination,
            send_max,
            expiration: self.expiration,
            destination_tag: self.destination_tag,
            invoice_id: self.invoice_id,
        })
    }
}
impl CheckCreate {
    pub fn builder(account: impl Into<String>) -> CheckCreateBuilder {
        CheckCreateBuilder::new(account)
    }
}

/// CheckCash transaction.
#[derive(Debug, Clone)]
pub struct CheckCash {
    pub common: TxCommon,
    pub check_id: String,
    pub amount: Option<Value>,
    pub deliver_min: Option<Value>,
}
impl Transaction for CheckCash {
    fn transaction_type(&self) -> &'static str {
        "CheckCash"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("CheckCash"));
        obj.insert("CheckID".into(), json!(self.check_id));
        if let Some(ref a) = self.amount {
            obj.insert("Amount".into(), a.clone());
        }
        if let Some(ref dm) = self.deliver_min {
            obj.insert("DeliverMin".into(), dm.clone());
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
pub struct CheckCashBuilder {
    common: TxCommon,
    check_id: Option<String>,
    amount: Option<Value>,
    deliver_min: Option<Value>,
}
impl CheckCashBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            check_id: None,
            amount: None,
            deliver_min: None,
        }
    }
    pub fn check_id(mut self, id: impl Into<String>) -> Self {
        self.check_id = Some(id.into());
        self
    }
    pub fn amount(mut self, a: Value) -> Self {
        self.amount = Some(a);
        self
    }
    pub fn deliver_min(mut self, dm: Value) -> Self {
        self.deliver_min = Some(dm);
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
    pub fn build(self) -> Result<CheckCash, CoreError> {
        let check_id = self
            .check_id
            .ok_or_else(|| CoreError::ValidationError("CheckCash: check_id is required".into()))?;
        Ok(CheckCash {
            common: self.common,
            check_id,
            amount: self.amount,
            deliver_min: self.deliver_min,
        })
    }
}
impl CheckCash {
    pub fn builder(account: impl Into<String>) -> CheckCashBuilder {
        CheckCashBuilder::new(account)
    }
}

/// CheckCancel transaction.
#[derive(Debug, Clone)]
pub struct CheckCancel {
    pub common: TxCommon,
    pub check_id: String,
}
impl Transaction for CheckCancel {
    fn transaction_type(&self) -> &'static str {
        "CheckCancel"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("CheckCancel"));
        obj.insert("CheckID".into(), json!(self.check_id));
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
pub struct CheckCancelBuilder {
    common: TxCommon,
    check_id: Option<String>,
}
impl CheckCancelBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            check_id: None,
        }
    }
    pub fn check_id(mut self, id: impl Into<String>) -> Self {
        self.check_id = Some(id.into());
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
    pub fn build(self) -> Result<CheckCancel, CoreError> {
        let check_id = self.check_id.ok_or_else(|| {
            CoreError::ValidationError("CheckCancel: check_id is required".into())
        })?;
        Ok(CheckCancel {
            common: self.common,
            check_id,
        })
    }
}
impl CheckCancel {
    pub fn builder(account: impl Into<String>) -> CheckCancelBuilder {
        CheckCancelBuilder::new(account)
    }
}
