use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// PaymentChannelCreate transaction.
#[derive(Debug, Clone)]
pub struct PaymentChannelCreate {
    pub common: TxCommon,
    pub amount: String,
    pub destination: String,
    pub settle_delay: u32,
    pub public_key: String,
    pub cancel_after: Option<u32>,
    pub destination_tag: Option<u32>,
}
impl Transaction for PaymentChannelCreate {
    fn transaction_type(&self) -> &'static str {
        "PaymentChannelCreate"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("PaymentChannelCreate"));
        obj.insert("Amount".into(), json!(self.amount));
        obj.insert("Destination".into(), json!(self.destination));
        obj.insert("SettleDelay".into(), json!(self.settle_delay));
        obj.insert("PublicKey".into(), json!(self.public_key));
        if let Some(ca) = self.cancel_after {
            obj.insert("CancelAfter".into(), json!(ca));
        }
        if let Some(dt) = self.destination_tag {
            obj.insert("DestinationTag".into(), json!(dt));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
pub struct PaymentChannelCreateBuilder {
    common: TxCommon,
    amount: Option<String>,
    destination: Option<String>,
    settle_delay: Option<u32>,
    public_key: Option<String>,
    cancel_after: Option<u32>,
    destination_tag: Option<u32>,
}
impl PaymentChannelCreateBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            amount: None,
            destination: None,
            settle_delay: None,
            public_key: None,
            cancel_after: None,
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
    pub fn settle_delay(mut self, s: u32) -> Self {
        self.settle_delay = Some(s);
        self
    }
    pub fn public_key(mut self, p: impl Into<String>) -> Self {
        self.public_key = Some(p.into());
        self
    }
    pub fn cancel_after(mut self, ca: u32) -> Self {
        self.cancel_after = Some(ca);
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
    pub fn build(self) -> Result<PaymentChannelCreate, CoreError> {
        let amount = self.amount.ok_or_else(|| {
            CoreError::ValidationError("PaymentChannelCreate: amount is required".into())
        })?;
        let destination = self.destination.ok_or_else(|| {
            CoreError::ValidationError("PaymentChannelCreate: destination is required".into())
        })?;
        let settle_delay = self.settle_delay.ok_or_else(|| {
            CoreError::ValidationError("PaymentChannelCreate: settle_delay is required".into())
        })?;
        let public_key = self.public_key.ok_or_else(|| {
            CoreError::ValidationError("PaymentChannelCreate: public_key is required".into())
        })?;
        Ok(PaymentChannelCreate {
            common: self.common,
            amount,
            destination,
            settle_delay,
            public_key,
            cancel_after: self.cancel_after,
            destination_tag: self.destination_tag,
        })
    }
}
impl PaymentChannelCreate {
    pub fn builder(account: impl Into<String>) -> PaymentChannelCreateBuilder {
        PaymentChannelCreateBuilder::new(account)
    }
}

/// PaymentChannelFund transaction.
#[derive(Debug, Clone)]
pub struct PaymentChannelFund {
    pub common: TxCommon,
    pub channel: String,
    pub amount: String,
    pub expiration: Option<u32>,
}
impl Transaction for PaymentChannelFund {
    fn transaction_type(&self) -> &'static str {
        "PaymentChannelFund"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("PaymentChannelFund"));
        obj.insert("Channel".into(), json!(self.channel));
        obj.insert("Amount".into(), json!(self.amount));
        if let Some(e) = self.expiration {
            obj.insert("Expiration".into(), json!(e));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
pub struct PaymentChannelFundBuilder {
    common: TxCommon,
    channel: Option<String>,
    amount: Option<String>,
    expiration: Option<u32>,
}
impl PaymentChannelFundBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            channel: None,
            amount: None,
            expiration: None,
        }
    }
    pub fn channel(mut self, c: impl Into<String>) -> Self {
        self.channel = Some(c.into());
        self
    }
    pub fn amount(mut self, a: impl Into<String>) -> Self {
        self.amount = Some(a.into());
        self
    }
    pub fn expiration(mut self, e: u32) -> Self {
        self.expiration = Some(e);
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
    pub fn build(self) -> Result<PaymentChannelFund, CoreError> {
        let channel = self.channel.ok_or_else(|| {
            CoreError::ValidationError("PaymentChannelFund: channel is required".into())
        })?;
        let amount = self.amount.ok_or_else(|| {
            CoreError::ValidationError("PaymentChannelFund: amount is required".into())
        })?;
        Ok(PaymentChannelFund {
            common: self.common,
            channel,
            amount,
            expiration: self.expiration,
        })
    }
}
impl PaymentChannelFund {
    pub fn builder(account: impl Into<String>) -> PaymentChannelFundBuilder {
        PaymentChannelFundBuilder::new(account)
    }
}

/// PaymentChannelClaim transaction.
#[derive(Debug, Clone)]
pub struct PaymentChannelClaim {
    pub common: TxCommon,
    pub channel: String,
    pub balance: Option<String>,
    pub amount: Option<String>,
    pub signature: Option<String>,
    pub public_key: Option<String>,
}
impl Transaction for PaymentChannelClaim {
    fn transaction_type(&self) -> &'static str {
        "PaymentChannelClaim"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("PaymentChannelClaim"));
        obj.insert("Channel".into(), json!(self.channel));
        if let Some(ref b) = self.balance {
            obj.insert("Balance".into(), json!(b));
        }
        if let Some(ref a) = self.amount {
            obj.insert("Amount".into(), json!(a));
        }
        if let Some(ref s) = self.signature {
            obj.insert("Signature".into(), json!(s));
        }
        if let Some(ref p) = self.public_key {
            obj.insert("PublicKey".into(), json!(p));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
pub struct PaymentChannelClaimBuilder {
    common: TxCommon,
    channel: Option<String>,
    balance: Option<String>,
    amount: Option<String>,
    signature: Option<String>,
    public_key: Option<String>,
}
impl PaymentChannelClaimBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            channel: None,
            balance: None,
            amount: None,
            signature: None,
            public_key: None,
        }
    }
    pub fn channel(mut self, c: impl Into<String>) -> Self {
        self.channel = Some(c.into());
        self
    }
    pub fn balance(mut self, b: impl Into<String>) -> Self {
        self.balance = Some(b.into());
        self
    }
    pub fn amount(mut self, a: impl Into<String>) -> Self {
        self.amount = Some(a.into());
        self
    }
    pub fn signature(mut self, s: impl Into<String>) -> Self {
        self.signature = Some(s.into());
        self
    }
    pub fn public_key(mut self, p: impl Into<String>) -> Self {
        self.public_key = Some(p.into());
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
    pub fn flags(mut self, f: u32) -> Self {
        self.common.flags = Some(f);
        self
    }
    pub fn build(self) -> Result<PaymentChannelClaim, CoreError> {
        let channel = self.channel.ok_or_else(|| {
            CoreError::ValidationError("PaymentChannelClaim: channel is required".into())
        })?;
        Ok(PaymentChannelClaim {
            common: self.common,
            channel,
            balance: self.balance,
            amount: self.amount,
            signature: self.signature,
            public_key: self.public_key,
        })
    }
}
impl PaymentChannelClaim {
    pub fn builder(account: impl Into<String>) -> PaymentChannelClaimBuilder {
        PaymentChannelClaimBuilder::new(account)
    }
}
