use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// AMMCreate transaction.
#[derive(Debug, Clone)]
pub struct AMMCreate {
    pub common: TxCommon,
    pub amount: Value,
    pub amount2: Value,
    pub trading_fee: u16,
}
impl Transaction for AMMCreate {
    fn transaction_type(&self) -> &'static str {
        "AMMCreate"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("AMMCreate"));
        obj.insert("Amount".into(), self.amount.clone());
        obj.insert("Amount2".into(), self.amount2.clone());
        obj.insert("TradingFee".into(), json!(self.trading_fee));
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
pub struct AMMCreateBuilder {
    common: TxCommon,
    amount: Option<Value>,
    amount2: Option<Value>,
    trading_fee: Option<u16>,
}
impl AMMCreateBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            amount: None,
            amount2: None,
            trading_fee: None,
        }
    }
    pub fn amount(mut self, a: Value) -> Self {
        self.amount = Some(a);
        self
    }
    pub fn amount2(mut self, a: Value) -> Self {
        self.amount2 = Some(a);
        self
    }
    pub fn trading_fee(mut self, f: u16) -> Self {
        self.trading_fee = Some(f);
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
    pub fn build(self) -> Result<AMMCreate, CoreError> {
        let amount = self
            .amount
            .ok_or_else(|| CoreError::ValidationError("AMMCreate: amount is required".into()))?;
        let amount2 = self
            .amount2
            .ok_or_else(|| CoreError::ValidationError("AMMCreate: amount2 is required".into()))?;
        let trading_fee = self.trading_fee.ok_or_else(|| {
            CoreError::ValidationError("AMMCreate: trading_fee is required".into())
        })?;
        Ok(AMMCreate {
            common: self.common,
            amount,
            amount2,
            trading_fee,
        })
    }
}
impl AMMCreate {
    pub fn builder(account: impl Into<String>) -> AMMCreateBuilder {
        AMMCreateBuilder::new(account)
    }
}

/// AMMDeposit transaction.
#[derive(Debug, Clone)]
pub struct AMMDeposit {
    pub common: TxCommon,
    pub asset: Value,
    pub asset2: Value,
    pub amount: Option<Value>,
    pub amount2: Option<Value>,
    pub lp_token_out: Option<Value>,
    pub e_price: Option<Value>,
    pub trading_fee: Option<u16>,
}
impl Transaction for AMMDeposit {
    fn transaction_type(&self) -> &'static str {
        "AMMDeposit"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("AMMDeposit"));
        obj.insert("Asset".into(), self.asset.clone());
        obj.insert("Asset2".into(), self.asset2.clone());
        if let Some(ref a) = self.amount {
            obj.insert("Amount".into(), a.clone());
        }
        if let Some(ref a) = self.amount2 {
            obj.insert("Amount2".into(), a.clone());
        }
        if let Some(ref l) = self.lp_token_out {
            obj.insert("LPTokenOut".into(), l.clone());
        }
        if let Some(ref e) = self.e_price {
            obj.insert("EPrice".into(), e.clone());
        }
        if let Some(tf) = self.trading_fee {
            obj.insert("TradingFee".into(), json!(tf));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
pub struct AMMDepositBuilder {
    common: TxCommon,
    asset: Option<Value>,
    asset2: Option<Value>,
    amount: Option<Value>,
    amount2: Option<Value>,
    lp_token_out: Option<Value>,
    e_price: Option<Value>,
    trading_fee: Option<u16>,
}
impl AMMDepositBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            asset: None,
            asset2: None,
            amount: None,
            amount2: None,
            lp_token_out: None,
            e_price: None,
            trading_fee: None,
        }
    }
    pub fn asset(mut self, a: Value) -> Self {
        self.asset = Some(a);
        self
    }
    pub fn asset2(mut self, a: Value) -> Self {
        self.asset2 = Some(a);
        self
    }
    pub fn amount(mut self, a: Value) -> Self {
        self.amount = Some(a);
        self
    }
    pub fn amount2(mut self, a: Value) -> Self {
        self.amount2 = Some(a);
        self
    }
    pub fn lp_token_out(mut self, l: Value) -> Self {
        self.lp_token_out = Some(l);
        self
    }
    pub fn e_price(mut self, e: Value) -> Self {
        self.e_price = Some(e);
        self
    }
    pub fn trading_fee(mut self, f: u16) -> Self {
        self.trading_fee = Some(f);
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
    pub fn build(self) -> Result<AMMDeposit, CoreError> {
        let asset = self
            .asset
            .ok_or_else(|| CoreError::ValidationError("AMMDeposit: asset is required".into()))?;
        let asset2 = self
            .asset2
            .ok_or_else(|| CoreError::ValidationError("AMMDeposit: asset2 is required".into()))?;
        Ok(AMMDeposit {
            common: self.common,
            asset,
            asset2,
            amount: self.amount,
            amount2: self.amount2,
            lp_token_out: self.lp_token_out,
            e_price: self.e_price,
            trading_fee: self.trading_fee,
        })
    }
}
impl AMMDeposit {
    pub fn builder(account: impl Into<String>) -> AMMDepositBuilder {
        AMMDepositBuilder::new(account)
    }
}

/// AMMWithdraw transaction.
#[derive(Debug, Clone)]
pub struct AMMWithdraw {
    pub common: TxCommon,
    pub asset: Value,
    pub asset2: Value,
    pub amount: Option<Value>,
    pub amount2: Option<Value>,
    pub lp_token_in: Option<Value>,
    pub e_price: Option<Value>,
}
impl Transaction for AMMWithdraw {
    fn transaction_type(&self) -> &'static str {
        "AMMWithdraw"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("AMMWithdraw"));
        obj.insert("Asset".into(), self.asset.clone());
        obj.insert("Asset2".into(), self.asset2.clone());
        if let Some(ref a) = self.amount {
            obj.insert("Amount".into(), a.clone());
        }
        if let Some(ref a) = self.amount2 {
            obj.insert("Amount2".into(), a.clone());
        }
        if let Some(ref l) = self.lp_token_in {
            obj.insert("LPTokenIn".into(), l.clone());
        }
        if let Some(ref e) = self.e_price {
            obj.insert("EPrice".into(), e.clone());
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
pub struct AMMWithdrawBuilder {
    common: TxCommon,
    asset: Option<Value>,
    asset2: Option<Value>,
    amount: Option<Value>,
    amount2: Option<Value>,
    lp_token_in: Option<Value>,
    e_price: Option<Value>,
}
impl AMMWithdrawBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            asset: None,
            asset2: None,
            amount: None,
            amount2: None,
            lp_token_in: None,
            e_price: None,
        }
    }
    pub fn asset(mut self, a: Value) -> Self {
        self.asset = Some(a);
        self
    }
    pub fn asset2(mut self, a: Value) -> Self {
        self.asset2 = Some(a);
        self
    }
    pub fn amount(mut self, a: Value) -> Self {
        self.amount = Some(a);
        self
    }
    pub fn amount2(mut self, a: Value) -> Self {
        self.amount2 = Some(a);
        self
    }
    pub fn lp_token_in(mut self, l: Value) -> Self {
        self.lp_token_in = Some(l);
        self
    }
    pub fn e_price(mut self, e: Value) -> Self {
        self.e_price = Some(e);
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
    pub fn build(self) -> Result<AMMWithdraw, CoreError> {
        let asset = self
            .asset
            .ok_or_else(|| CoreError::ValidationError("AMMWithdraw: asset is required".into()))?;
        let asset2 = self
            .asset2
            .ok_or_else(|| CoreError::ValidationError("AMMWithdraw: asset2 is required".into()))?;
        Ok(AMMWithdraw {
            common: self.common,
            asset,
            asset2,
            amount: self.amount,
            amount2: self.amount2,
            lp_token_in: self.lp_token_in,
            e_price: self.e_price,
        })
    }
}
impl AMMWithdraw {
    pub fn builder(account: impl Into<String>) -> AMMWithdrawBuilder {
        AMMWithdrawBuilder::new(account)
    }
}

/// AMMVote transaction.
#[derive(Debug, Clone)]
pub struct AMMVote {
    pub common: TxCommon,
    pub asset: Value,
    pub asset2: Value,
    pub trading_fee: u16,
}
impl Transaction for AMMVote {
    fn transaction_type(&self) -> &'static str {
        "AMMVote"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("AMMVote"));
        obj.insert("Asset".into(), self.asset.clone());
        obj.insert("Asset2".into(), self.asset2.clone());
        obj.insert("TradingFee".into(), json!(self.trading_fee));
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
pub struct AMMVoteBuilder {
    common: TxCommon,
    asset: Option<Value>,
    asset2: Option<Value>,
    trading_fee: Option<u16>,
}
impl AMMVoteBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            asset: None,
            asset2: None,
            trading_fee: None,
        }
    }
    pub fn asset(mut self, a: Value) -> Self {
        self.asset = Some(a);
        self
    }
    pub fn asset2(mut self, a: Value) -> Self {
        self.asset2 = Some(a);
        self
    }
    pub fn trading_fee(mut self, f: u16) -> Self {
        self.trading_fee = Some(f);
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
    pub fn build(self) -> Result<AMMVote, CoreError> {
        let asset = self
            .asset
            .ok_or_else(|| CoreError::ValidationError("AMMVote: asset is required".into()))?;
        let asset2 = self
            .asset2
            .ok_or_else(|| CoreError::ValidationError("AMMVote: asset2 is required".into()))?;
        let trading_fee = self
            .trading_fee
            .ok_or_else(|| CoreError::ValidationError("AMMVote: trading_fee is required".into()))?;
        Ok(AMMVote {
            common: self.common,
            asset,
            asset2,
            trading_fee,
        })
    }
}
impl AMMVote {
    pub fn builder(account: impl Into<String>) -> AMMVoteBuilder {
        AMMVoteBuilder::new(account)
    }
}

/// AMMBid transaction.
#[derive(Debug, Clone)]
pub struct AMMBid {
    pub common: TxCommon,
    pub asset: Value,
    pub asset2: Value,
    pub bid_min: Option<Value>,
    pub bid_max: Option<Value>,
    pub auth_accounts: Option<Vec<Value>>,
}
impl Transaction for AMMBid {
    fn transaction_type(&self) -> &'static str {
        "AMMBid"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("AMMBid"));
        obj.insert("Asset".into(), self.asset.clone());
        obj.insert("Asset2".into(), self.asset2.clone());
        if let Some(ref bm) = self.bid_min {
            obj.insert("BidMin".into(), bm.clone());
        }
        if let Some(ref bx) = self.bid_max {
            obj.insert("BidMax".into(), bx.clone());
        }
        if let Some(ref aa) = self.auth_accounts {
            obj.insert("AuthAccounts".into(), json!(aa));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
pub struct AMMBidBuilder {
    common: TxCommon,
    asset: Option<Value>,
    asset2: Option<Value>,
    bid_min: Option<Value>,
    bid_max: Option<Value>,
    auth_accounts: Option<Vec<Value>>,
}
impl AMMBidBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            asset: None,
            asset2: None,
            bid_min: None,
            bid_max: None,
            auth_accounts: None,
        }
    }
    pub fn asset(mut self, a: Value) -> Self {
        self.asset = Some(a);
        self
    }
    pub fn asset2(mut self, a: Value) -> Self {
        self.asset2 = Some(a);
        self
    }
    pub fn bid_min(mut self, b: Value) -> Self {
        self.bid_min = Some(b);
        self
    }
    pub fn bid_max(mut self, b: Value) -> Self {
        self.bid_max = Some(b);
        self
    }
    pub fn auth_accounts(mut self, aa: Vec<Value>) -> Self {
        self.auth_accounts = Some(aa);
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
    pub fn build(self) -> Result<AMMBid, CoreError> {
        let asset = self
            .asset
            .ok_or_else(|| CoreError::ValidationError("AMMBid: asset is required".into()))?;
        let asset2 = self
            .asset2
            .ok_or_else(|| CoreError::ValidationError("AMMBid: asset2 is required".into()))?;
        Ok(AMMBid {
            common: self.common,
            asset,
            asset2,
            bid_min: self.bid_min,
            bid_max: self.bid_max,
            auth_accounts: self.auth_accounts,
        })
    }
}
impl AMMBid {
    pub fn builder(account: impl Into<String>) -> AMMBidBuilder {
        AMMBidBuilder::new(account)
    }
}
