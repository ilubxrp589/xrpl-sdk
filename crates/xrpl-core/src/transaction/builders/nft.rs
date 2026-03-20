use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// NFTokenMint transaction.
#[derive(Debug, Clone)]
pub struct NFTokenMint {
    pub common: TxCommon,
    pub nf_token_taxon: u32,
    pub issuer: Option<String>,
    pub transfer_fee: Option<u16>,
    pub uri: Option<String>,
}

impl Transaction for NFTokenMint {
    fn transaction_type(&self) -> &'static str {
        "NFTokenMint"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("NFTokenMint"));
        obj.insert("NFTokenTaxon".into(), json!(self.nf_token_taxon));
        if let Some(ref i) = self.issuer {
            obj.insert("Issuer".into(), json!(i));
        }
        if let Some(tf) = self.transfer_fee {
            obj.insert("TransferFee".into(), json!(tf));
        }
        if let Some(ref u) = self.uri {
            obj.insert("URI".into(), json!(u));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}

pub struct NFTokenMintBuilder {
    common: TxCommon,
    nf_token_taxon: Option<u32>,
    issuer: Option<String>,
    transfer_fee: Option<u16>,
    uri: Option<String>,
}
impl NFTokenMintBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            nf_token_taxon: None,
            issuer: None,
            transfer_fee: None,
            uri: None,
        }
    }
    pub fn nf_token_taxon(mut self, t: u32) -> Self {
        self.nf_token_taxon = Some(t);
        self
    }
    pub fn issuer(mut self, i: impl Into<String>) -> Self {
        self.issuer = Some(i.into());
        self
    }
    pub fn transfer_fee(mut self, f: u16) -> Self {
        self.transfer_fee = Some(f);
        self
    }
    pub fn uri(mut self, u: impl Into<String>) -> Self {
        self.uri = Some(u.into());
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
    pub fn build(self) -> Result<NFTokenMint, CoreError> {
        let nf_token_taxon = self.nf_token_taxon.ok_or_else(|| {
            CoreError::ValidationError("NFTokenMint: nf_token_taxon is required".into())
        })?;
        Ok(NFTokenMint {
            common: self.common,
            nf_token_taxon,
            issuer: self.issuer,
            transfer_fee: self.transfer_fee,
            uri: self.uri,
        })
    }
}
impl NFTokenMint {
    pub fn builder(account: impl Into<String>) -> NFTokenMintBuilder {
        NFTokenMintBuilder::new(account)
    }
}

/// NFTokenBurn transaction.
#[derive(Debug, Clone)]
pub struct NFTokenBurn {
    pub common: TxCommon,
    pub nf_token_id: String,
    pub owner: Option<String>,
}
impl Transaction for NFTokenBurn {
    fn transaction_type(&self) -> &'static str {
        "NFTokenBurn"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("NFTokenBurn"));
        obj.insert("NFTokenID".into(), json!(self.nf_token_id));
        if let Some(ref o) = self.owner {
            obj.insert("Owner".into(), json!(o));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
pub struct NFTokenBurnBuilder {
    common: TxCommon,
    nf_token_id: Option<String>,
    owner: Option<String>,
}
impl NFTokenBurnBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            nf_token_id: None,
            owner: None,
        }
    }
    pub fn nf_token_id(mut self, id: impl Into<String>) -> Self {
        self.nf_token_id = Some(id.into());
        self
    }
    pub fn owner(mut self, o: impl Into<String>) -> Self {
        self.owner = Some(o.into());
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
    pub fn build(self) -> Result<NFTokenBurn, CoreError> {
        let nf_token_id = self.nf_token_id.ok_or_else(|| {
            CoreError::ValidationError("NFTokenBurn: nf_token_id is required".into())
        })?;
        Ok(NFTokenBurn {
            common: self.common,
            nf_token_id,
            owner: self.owner,
        })
    }
}
impl NFTokenBurn {
    pub fn builder(account: impl Into<String>) -> NFTokenBurnBuilder {
        NFTokenBurnBuilder::new(account)
    }
}

/// NFTokenCreateOffer transaction.
#[derive(Debug, Clone)]
pub struct NFTokenCreateOffer {
    pub common: TxCommon,
    pub nf_token_id: String,
    pub amount: Value,
    pub owner: Option<String>,
    pub expiration: Option<u32>,
    pub destination: Option<String>,
}
impl Transaction for NFTokenCreateOffer {
    fn transaction_type(&self) -> &'static str {
        "NFTokenCreateOffer"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("NFTokenCreateOffer"));
        obj.insert("NFTokenID".into(), json!(self.nf_token_id));
        obj.insert("Amount".into(), self.amount.clone());
        if let Some(ref o) = self.owner {
            obj.insert("Owner".into(), json!(o));
        }
        if let Some(e) = self.expiration {
            obj.insert("Expiration".into(), json!(e));
        }
        if let Some(ref d) = self.destination {
            obj.insert("Destination".into(), json!(d));
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
pub struct NFTokenCreateOfferBuilder {
    common: TxCommon,
    nf_token_id: Option<String>,
    amount: Option<Value>,
    owner: Option<String>,
    expiration: Option<u32>,
    destination: Option<String>,
}
impl NFTokenCreateOfferBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            nf_token_id: None,
            amount: None,
            owner: None,
            expiration: None,
            destination: None,
        }
    }
    pub fn nf_token_id(mut self, id: impl Into<String>) -> Self {
        self.nf_token_id = Some(id.into());
        self
    }
    pub fn amount(mut self, a: Value) -> Self {
        self.amount = Some(a);
        self
    }
    pub fn owner(mut self, o: impl Into<String>) -> Self {
        self.owner = Some(o.into());
        self
    }
    pub fn expiration(mut self, e: u32) -> Self {
        self.expiration = Some(e);
        self
    }
    pub fn destination(mut self, d: impl Into<String>) -> Self {
        self.destination = Some(d.into());
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
    pub fn build(self) -> Result<NFTokenCreateOffer, CoreError> {
        let nf_token_id = self.nf_token_id.ok_or_else(|| {
            CoreError::ValidationError("NFTokenCreateOffer: nf_token_id is required".into())
        })?;
        let amount = self.amount.ok_or_else(|| {
            CoreError::ValidationError("NFTokenCreateOffer: amount is required".into())
        })?;
        Ok(NFTokenCreateOffer {
            common: self.common,
            nf_token_id,
            amount,
            owner: self.owner,
            expiration: self.expiration,
            destination: self.destination,
        })
    }
}
impl NFTokenCreateOffer {
    pub fn builder(account: impl Into<String>) -> NFTokenCreateOfferBuilder {
        NFTokenCreateOfferBuilder::new(account)
    }
}

/// NFTokenCancelOffer transaction.
#[derive(Debug, Clone)]
pub struct NFTokenCancelOffer {
    pub common: TxCommon,
    pub nf_token_offers: Vec<String>,
}
impl Transaction for NFTokenCancelOffer {
    fn transaction_type(&self) -> &'static str {
        "NFTokenCancelOffer"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("NFTokenCancelOffer"));
        obj.insert("NFTokenOffers".into(), json!(self.nf_token_offers));
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        Ok(())
    }
}
pub struct NFTokenCancelOfferBuilder {
    common: TxCommon,
    nf_token_offers: Option<Vec<String>>,
}
impl NFTokenCancelOfferBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            nf_token_offers: None,
        }
    }
    pub fn nf_token_offers(mut self, offers: Vec<String>) -> Self {
        self.nf_token_offers = Some(offers);
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
    pub fn build(self) -> Result<NFTokenCancelOffer, CoreError> {
        let nf_token_offers = self.nf_token_offers.ok_or_else(|| {
            CoreError::ValidationError("NFTokenCancelOffer: nf_token_offers is required".into())
        })?;
        Ok(NFTokenCancelOffer {
            common: self.common,
            nf_token_offers,
        })
    }
}
impl NFTokenCancelOffer {
    pub fn builder(account: impl Into<String>) -> NFTokenCancelOfferBuilder {
        NFTokenCancelOfferBuilder::new(account)
    }
}

/// NFTokenAcceptOffer transaction.
#[derive(Debug, Clone)]
pub struct NFTokenAcceptOffer {
    pub common: TxCommon,
    pub nf_token_buy_offer: Option<String>,
    pub nf_token_sell_offer: Option<String>,
    pub nf_token_broker_fee: Option<Value>,
}
impl Transaction for NFTokenAcceptOffer {
    fn transaction_type(&self) -> &'static str {
        "NFTokenAcceptOffer"
    }
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("TransactionType".into(), json!("NFTokenAcceptOffer"));
        if let Some(ref b) = self.nf_token_buy_offer {
            obj.insert("NFTokenBuyOffer".into(), json!(b));
        }
        if let Some(ref s) = self.nf_token_sell_offer {
            obj.insert("NFTokenSellOffer".into(), json!(s));
        }
        if let Some(ref bf) = self.nf_token_broker_fee {
            obj.insert("NFTokenBrokerFee".into(), bf.clone());
        }
        merge_common(&mut obj, &self.common);
        Value::Object(obj)
    }
    fn validate(&self) -> Result<(), CoreError> {
        if self.nf_token_buy_offer.is_none() && self.nf_token_sell_offer.is_none() {
            return Err(CoreError::ValidationError(
                "NFTokenAcceptOffer: at least one of buy_offer or sell_offer is required".into(),
            ));
        }
        Ok(())
    }
}
pub struct NFTokenAcceptOfferBuilder {
    common: TxCommon,
    nf_token_buy_offer: Option<String>,
    nf_token_sell_offer: Option<String>,
    nf_token_broker_fee: Option<Value>,
}
impl NFTokenAcceptOfferBuilder {
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            nf_token_buy_offer: None,
            nf_token_sell_offer: None,
            nf_token_broker_fee: None,
        }
    }
    pub fn nf_token_buy_offer(mut self, o: impl Into<String>) -> Self {
        self.nf_token_buy_offer = Some(o.into());
        self
    }
    pub fn nf_token_sell_offer(mut self, o: impl Into<String>) -> Self {
        self.nf_token_sell_offer = Some(o.into());
        self
    }
    pub fn nf_token_broker_fee(mut self, f: Value) -> Self {
        self.nf_token_broker_fee = Some(f);
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
    pub fn build(self) -> Result<NFTokenAcceptOffer, CoreError> {
        let tx = NFTokenAcceptOffer {
            common: self.common,
            nf_token_buy_offer: self.nf_token_buy_offer,
            nf_token_sell_offer: self.nf_token_sell_offer,
            nf_token_broker_fee: self.nf_token_broker_fee,
        };
        tx.validate()?;
        Ok(tx)
    }
}
impl NFTokenAcceptOffer {
    pub fn builder(account: impl Into<String>) -> NFTokenAcceptOfferBuilder {
        NFTokenAcceptOfferBuilder::new(account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nft_mint_build_success() {
        let tx = NFTokenMint::builder("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
            .nf_token_taxon(0)
            .uri("68747470733A2F2F6578616D706C652E636F6D")
            .flags(8)
            .build()
            .unwrap();
        assert_eq!(tx.to_json()["TransactionType"], "NFTokenMint");
        assert_eq!(tx.to_json()["NFTokenTaxon"], 0);
    }

    #[test]
    fn nft_mint_missing_taxon() {
        let r = NFTokenMint::builder("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").build();
        assert!(r.is_err());
    }
}
