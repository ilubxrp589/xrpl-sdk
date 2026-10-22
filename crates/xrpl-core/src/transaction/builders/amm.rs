use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// AMMCreate transaction.
#[derive(Debug, Clone)]
pub struct AMMCreate {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// How much (`Amount`).
    pub amount: Value,
    /// The second asset's amount (`Amount2`).
    pub amount2: Value,
    /// The trading fee, in 1/100,000 (1000 = 1%) (`TradingFee`).
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
/// Builds a [`AMMCreate`] (from [`AMMCreate::builder`]); `build` checks the required fields.
pub struct AMMCreateBuilder {
    common: TxCommon,
    amount: Option<Value>,
    amount2: Option<Value>,
    trading_fee: Option<u16>,
}
impl AMMCreateBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            amount: None,
            amount2: None,
            trading_fee: None,
        }
    }
    /// Set `Amount`.
    pub fn amount(mut self, a: Value) -> Self {
        self.amount = Some(a);
        self
    }
    /// Set `Amount2`.
    pub fn amount2(mut self, a: Value) -> Self {
        self.amount2 = Some(a);
        self
    }
    /// Set `TradingFee`.
    pub fn trading_fee(mut self, f: u16) -> Self {
        self.trading_fee = Some(f);
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
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> AMMCreateBuilder {
        AMMCreateBuilder::new(account)
    }
}

/// AMMDeposit transaction.
#[derive(Debug, Clone)]
pub struct AMMDeposit {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// The pool's first asset (`Asset`).
    pub asset: Value,
    /// The pool's second asset (`Asset2`).
    pub asset2: Value,
    /// How much (`Amount`).
    pub amount: Option<Value>,
    /// The second asset's amount (`Amount2`).
    pub amount2: Option<Value>,
    /// The LP tokens to receive (`LPTokenOut`).
    pub lp_token_out: Option<Value>,
    /// The effective price limit (`EPrice`).
    pub e_price: Option<Value>,
    /// The trading fee, in 1/100,000 (1000 = 1%) (`TradingFee`).
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
/// Builds a [`AMMDeposit`] (from [`AMMDeposit::builder`]); `build` checks the required fields.
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
    /// A builder for the transaction from `account`.
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
    /// Set `Asset`.
    pub fn asset(mut self, a: Value) -> Self {
        self.asset = Some(a);
        self
    }
    /// Set `Asset2`.
    pub fn asset2(mut self, a: Value) -> Self {
        self.asset2 = Some(a);
        self
    }
    /// Set `Amount`.
    pub fn amount(mut self, a: Value) -> Self {
        self.amount = Some(a);
        self
    }
    /// Set `Amount2`.
    pub fn amount2(mut self, a: Value) -> Self {
        self.amount2 = Some(a);
        self
    }
    /// Set `LPTokenOut`.
    pub fn lp_token_out(mut self, l: Value) -> Self {
        self.lp_token_out = Some(l);
        self
    }
    /// Set `EPrice`.
    pub fn e_price(mut self, e: Value) -> Self {
        self.e_price = Some(e);
        self
    }
    /// Set `TradingFee`.
    pub fn trading_fee(mut self, f: u16) -> Self {
        self.trading_fee = Some(f);
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
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> AMMDepositBuilder {
        AMMDepositBuilder::new(account)
    }
}

/// AMMWithdraw transaction.
#[derive(Debug, Clone)]
pub struct AMMWithdraw {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// The pool's first asset (`Asset`).
    pub asset: Value,
    /// The pool's second asset (`Asset2`).
    pub asset2: Value,
    /// How much (`Amount`).
    pub amount: Option<Value>,
    /// The second asset's amount (`Amount2`).
    pub amount2: Option<Value>,
    /// The LP tokens to return (`LPTokenIn`).
    pub lp_token_in: Option<Value>,
    /// The effective price limit (`EPrice`).
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
/// Builds a [`AMMWithdraw`] (from [`AMMWithdraw::builder`]); `build` checks the required fields.
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
    /// A builder for the transaction from `account`.
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
    /// Set `Asset`.
    pub fn asset(mut self, a: Value) -> Self {
        self.asset = Some(a);
        self
    }
    /// Set `Asset2`.
    pub fn asset2(mut self, a: Value) -> Self {
        self.asset2 = Some(a);
        self
    }
    /// Set `Amount`.
    pub fn amount(mut self, a: Value) -> Self {
        self.amount = Some(a);
        self
    }
    /// Set `Amount2`.
    pub fn amount2(mut self, a: Value) -> Self {
        self.amount2 = Some(a);
        self
    }
    /// Set `LPTokenIn`.
    pub fn lp_token_in(mut self, l: Value) -> Self {
        self.lp_token_in = Some(l);
        self
    }
    /// Set `EPrice`.
    pub fn e_price(mut self, e: Value) -> Self {
        self.e_price = Some(e);
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
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> AMMWithdrawBuilder {
        AMMWithdrawBuilder::new(account)
    }
}

/// AMMVote transaction.
#[derive(Debug, Clone)]
pub struct AMMVote {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// The pool's first asset (`Asset`).
    pub asset: Value,
    /// The pool's second asset (`Asset2`).
    pub asset2: Value,
    /// The trading fee, in 1/100,000 (1000 = 1%) (`TradingFee`).
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
/// Builds a [`AMMVote`] (from [`AMMVote::builder`]); `build` checks the required fields.
pub struct AMMVoteBuilder {
    common: TxCommon,
    asset: Option<Value>,
    asset2: Option<Value>,
    trading_fee: Option<u16>,
}
impl AMMVoteBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            asset: None,
            asset2: None,
            trading_fee: None,
        }
    }
    /// Set `Asset`.
    pub fn asset(mut self, a: Value) -> Self {
        self.asset = Some(a);
        self
    }
    /// Set `Asset2`.
    pub fn asset2(mut self, a: Value) -> Self {
        self.asset2 = Some(a);
        self
    }
    /// Set `TradingFee`.
    pub fn trading_fee(mut self, f: u16) -> Self {
        self.trading_fee = Some(f);
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
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> AMMVoteBuilder {
        AMMVoteBuilder::new(account)
    }
}

/// AMMBid transaction.
#[derive(Debug, Clone)]
pub struct AMMBid {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// The pool's first asset (`Asset`).
    pub asset: Value,
    /// The pool's second asset (`Asset2`).
    pub asset2: Value,
    /// The least to bid for the auction slot, in LP tokens (`BidMin`).
    pub bid_min: Option<Value>,
    /// The most to bid for the auction slot, in LP tokens (`BidMax`).
    pub bid_max: Option<Value>,
    /// Accounts that may trade at the auction slot's discounted fee (`AuthAccounts`).
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
/// Builds a [`AMMBid`] (from [`AMMBid::builder`]); `build` checks the required fields.
pub struct AMMBidBuilder {
    common: TxCommon,
    asset: Option<Value>,
    asset2: Option<Value>,
    bid_min: Option<Value>,
    bid_max: Option<Value>,
    auth_accounts: Option<Vec<Value>>,
}
impl AMMBidBuilder {
    /// A builder for the transaction from `account`.
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
    /// Set `Asset`.
    pub fn asset(mut self, a: Value) -> Self {
        self.asset = Some(a);
        self
    }
    /// Set `Asset2`.
    pub fn asset2(mut self, a: Value) -> Self {
        self.asset2 = Some(a);
        self
    }
    /// Set `BidMin`.
    pub fn bid_min(mut self, b: Value) -> Self {
        self.bid_min = Some(b);
        self
    }
    /// Set `BidMax`.
    pub fn bid_max(mut self, b: Value) -> Self {
        self.bid_max = Some(b);
        self
    }
    /// Set `AuthAccounts`.
    pub fn auth_accounts(mut self, aa: Vec<Value>) -> Self {
        self.auth_accounts = Some(aa);
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
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> AMMBidBuilder {
        AMMBidBuilder::new(account)
    }
}
