use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// TrustSet transaction: set a trust line limit.
#[derive(Debug, Clone)]
pub struct TrustSet {
    pub common: TxCommon,
    pub limit_amount: Value,
    pub quality_in: Option<u32>,
    pub quality_out: Option<u32>,
}

impl Transaction for TrustSet {
    fn transaction_type(&self) -> &'static str {
        "TrustSet"
    }

    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("TrustSet"));
        obj.insert("LimitAmount".into(), self.limit_amount.clone());
        if let Some(qi) = self.quality_in {
            obj.insert("QualityIn".into(), json!(qi));
        }
        if let Some(qo) = self.quality_out {
            obj.insert("QualityOut".into(), json!(qo));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }

    fn validate(&self) -> Result<(), CoreError> {
        let obj = self.limit_amount.as_object().ok_or_else(|| {
            CoreError::ValidationError("TrustSet: limit_amount must be an IOU object".into())
        })?;
        if !obj.contains_key("currency")
            || !obj.contains_key("issuer")
            || !obj.contains_key("value")
        {
            return Err(CoreError::ValidationError(
                "TrustSet: limit_amount must have currency, issuer, and value".into(),
            ));
        }
        Ok(())
    }
}

pub struct TrustSetBuilder {
    common: TxCommon,
    limit_amount: Option<Value>,
    quality_in: Option<u32>,
    quality_out: Option<u32>,
}

impl TrustSetBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            limit_amount: None,
            quality_in: None,
            quality_out: None,
        }
    }
    pub fn limit_amount(mut self, v: Value) -> Self {
        self.limit_amount = Some(v);
        self
    }
    pub fn quality_in(mut self, q: u32) -> Self {
        self.quality_in = Some(q);
        self
    }
    pub fn quality_out(mut self, q: u32) -> Self {
        self.quality_out = Some(q);
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

    pub fn build(self) -> Result<TrustSet, CoreError> {
        let limit_amount = self.limit_amount.ok_or_else(|| {
            CoreError::ValidationError("TrustSet: limit_amount is required".into())
        })?;
        let tx = TrustSet {
            common: self.common,
            limit_amount,
            quality_in: self.quality_in,
            quality_out: self.quality_out,
        };
        tx.validate()?;
        Ok(tx)
    }
}

impl TrustSet {
    pub fn builder(account: impl Into<String>) -> TrustSetBuilder {
        TrustSetBuilder::new(account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_set_build_success() {
        let tx = TrustSet::builder("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
            .limit_amount(json!({"currency": "USD", "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe", "value": "1000"}))
            .build()
            .unwrap();
        assert_eq!(tx.to_json()["TransactionType"], "TrustSet");
    }

    #[test]
    fn trust_set_invalid_limit() {
        let r = TrustSet::builder("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
            .limit_amount(json!("not_an_object"))
            .build();
        assert!(r.is_err());
    }
}
