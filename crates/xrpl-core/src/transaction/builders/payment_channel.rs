use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// PaymentChannelCreate transaction.
#[derive(Debug, Clone)]
pub struct PaymentChannelCreate {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// How much (`Amount`): XRP as a string of **drops** (`"1000000"` is 1 XRP; `utils::xrp_to_drops`), or a token `{"currency", "issuer", "value"}`.
    pub amount: String,
    /// The account that receives it (`Destination`).
    pub destination: String,
    /// Seconds the source must wait before closing a channel with unclaimed XRP (`SettleDelay`).
    pub settle_delay: u32,
    /// A public key, hex (`PublicKey`).
    pub public_key: String,
    /// The time after which it can be cancelled (`CancelAfter`). Ledger time: seconds since 2000-01-01 UTC, not Unix time (`utils::unix_to_ripple_time`).
    pub cancel_after: Option<u32>,
    /// A tag the destination uses to tell its senders apart (`DestinationTag`).
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
/// Builds a [`PaymentChannelCreate`] (from [`PaymentChannelCreate::builder`]); `build` checks the required fields.
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
    /// A builder for the transaction from `account`.
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
    /// Set `SettleDelay`.
    pub fn settle_delay(mut self, s: u32) -> Self {
        self.settle_delay = Some(s);
        self
    }
    /// Set `PublicKey`.
    pub fn public_key(mut self, p: impl Into<String>) -> Self {
        self.public_key = Some(p.into());
        self
    }
    /// Set `CancelAfter`.
    pub fn cancel_after(mut self, ca: u32) -> Self {
        self.cancel_after = Some(ca);
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
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> PaymentChannelCreateBuilder {
        PaymentChannelCreateBuilder::new(account)
    }
}

/// PaymentChannelFund transaction.
#[derive(Debug, Clone)]
pub struct PaymentChannelFund {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// The payment channel's ID (`Channel`).
    pub channel: String,
    /// How much (`Amount`): XRP as a string of **drops** (`"1000000"` is 1 XRP; `utils::xrp_to_drops`), or a token `{"currency", "issuer", "value"}`.
    pub amount: String,
    /// When it expires (`Expiration`). Ledger time: seconds since 2000-01-01 UTC, not Unix time (`utils::unix_to_ripple_time`).
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
/// Builds a [`PaymentChannelFund`] (from [`PaymentChannelFund::builder`]); `build` checks the required fields.
pub struct PaymentChannelFundBuilder {
    common: TxCommon,
    channel: Option<String>,
    amount: Option<String>,
    expiration: Option<u32>,
}
impl PaymentChannelFundBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            channel: None,
            amount: None,
            expiration: None,
        }
    }
    /// Set `Channel`.
    pub fn channel(mut self, c: impl Into<String>) -> Self {
        self.channel = Some(c.into());
        self
    }
    /// Set `Amount`.
    pub fn amount(mut self, a: impl Into<String>) -> Self {
        self.amount = Some(a.into());
        self
    }
    /// Set `Expiration`.
    pub fn expiration(mut self, e: u32) -> Self {
        self.expiration = Some(e);
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
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> PaymentChannelFundBuilder {
        PaymentChannelFundBuilder::new(account)
    }
}

/// PaymentChannelClaim transaction.
#[derive(Debug, Clone)]
pub struct PaymentChannelClaim {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// The payment channel's ID (`Channel`).
    pub channel: String,
    /// The balance (`Balance`): XRP as a string of **drops** (`"1000000"` is 1 XRP; `utils::xrp_to_drops`), or a token `{"currency", "issuer", "value"}`.
    pub balance: Option<String>,
    /// How much (`Amount`): XRP as a string of **drops** (`"1000000"` is 1 XRP; `utils::xrp_to_drops`), or a token `{"currency", "issuer", "value"}`.
    pub amount: Option<String>,
    /// A signature, hex (`Signature`).
    pub signature: Option<String>,
    /// A public key, hex (`PublicKey`).
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
/// Builds a [`PaymentChannelClaim`] (from [`PaymentChannelClaim::builder`]); `build` checks the required fields.
pub struct PaymentChannelClaimBuilder {
    common: TxCommon,
    channel: Option<String>,
    balance: Option<String>,
    amount: Option<String>,
    signature: Option<String>,
    public_key: Option<String>,
}
impl PaymentChannelClaimBuilder {
    /// A builder for the transaction from `account`.
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
    /// Set `Channel`.
    pub fn channel(mut self, c: impl Into<String>) -> Self {
        self.channel = Some(c.into());
        self
    }
    /// Set `Balance`.
    pub fn balance(mut self, b: impl Into<String>) -> Self {
        self.balance = Some(b.into());
        self
    }
    /// Set `Amount`.
    pub fn amount(mut self, a: impl Into<String>) -> Self {
        self.amount = Some(a.into());
        self
    }
    /// Set `Signature`.
    pub fn signature(mut self, s: impl Into<String>) -> Self {
        self.signature = Some(s.into());
        self
    }
    /// Set `PublicKey`.
    pub fn public_key(mut self, p: impl Into<String>) -> Self {
        self.public_key = Some(p.into());
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
    /// Set `Flags`.
    pub fn flags(mut self, f: u32) -> Self {
        self.common.flags = Some(f);
        self
    }
    /// Check the required fields and build the transaction.
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
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> PaymentChannelClaimBuilder {
        PaymentChannelClaimBuilder::new(account)
    }
}
