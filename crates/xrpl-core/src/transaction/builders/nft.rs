use super::{merge_common, Transaction, TxCommon};
use crate::CoreError;
use serde_json::{json, Value};

/// NFTokenMint transaction.
#[derive(Debug, Clone)]
pub struct NFTokenMint {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// A number the issuer uses to group its NFTs (`NFTokenTaxon`).
    pub nf_token_taxon: u32,
    /// The issuer (`Issuer`).
    pub issuer: Option<String>,
    /// The fee on secondary sales, in 1/100,000 (5000 = 5%); for tokens, see `TransferRate` (`TransferFee`).
    pub transfer_fee: Option<u16>,
    /// A URI, hex (`URI`).
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

/// Builds a [`NFTokenMint`] (from [`NFTokenMint::builder`]); `build` checks the required fields.
pub struct NFTokenMintBuilder {
    common: TxCommon,
    nf_token_taxon: Option<u32>,
    issuer: Option<String>,
    transfer_fee: Option<u16>,
    uri: Option<String>,
}
impl NFTokenMintBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            nf_token_taxon: None,
            issuer: None,
            transfer_fee: None,
            uri: None,
        }
    }
    /// Set `NFTokenTaxon`.
    pub fn nf_token_taxon(mut self, t: u32) -> Self {
        self.nf_token_taxon = Some(t);
        self
    }
    /// Set `Issuer`.
    pub fn issuer(mut self, i: impl Into<String>) -> Self {
        self.issuer = Some(i.into());
        self
    }
    /// Set `TransferFee`.
    pub fn transfer_fee(mut self, f: u16) -> Self {
        self.transfer_fee = Some(f);
        self
    }
    /// Set `URI`.
    pub fn uri(mut self, u: impl Into<String>) -> Self {
        self.uri = Some(u.into());
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
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> NFTokenMintBuilder {
        NFTokenMintBuilder::new(account)
    }
}

/// NFTokenBurn transaction.
#[derive(Debug, Clone)]
pub struct NFTokenBurn {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// An NFT's ID (`NFTokenID`).
    pub nf_token_id: String,
    /// The account that owns the entry (`Owner`).
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
/// Builds a [`NFTokenBurn`] (from [`NFTokenBurn::builder`]); `build` checks the required fields.
pub struct NFTokenBurnBuilder {
    common: TxCommon,
    nf_token_id: Option<String>,
    owner: Option<String>,
}
impl NFTokenBurnBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            nf_token_id: None,
            owner: None,
        }
    }
    /// Set `NFTokenID`.
    pub fn nf_token_id(mut self, id: impl Into<String>) -> Self {
        self.nf_token_id = Some(id.into());
        self
    }
    /// Set `Owner`.
    pub fn owner(mut self, o: impl Into<String>) -> Self {
        self.owner = Some(o.into());
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
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> NFTokenBurnBuilder {
        NFTokenBurnBuilder::new(account)
    }
}

/// NFTokenCreateOffer transaction.
#[derive(Debug, Clone)]
pub struct NFTokenCreateOffer {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// An NFT's ID (`NFTokenID`).
    pub nf_token_id: String,
    /// How much (`Amount`).
    pub amount: Value,
    /// The account that owns the entry (`Owner`).
    pub owner: Option<String>,
    /// When it expires (ledger time: seconds since 2000-01-01) (`Expiration`).
    pub expiration: Option<u32>,
    /// The account that receives it (`Destination`).
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
/// Builds a [`NFTokenCreateOffer`] (from [`NFTokenCreateOffer::builder`]); `build` checks the required fields.
pub struct NFTokenCreateOfferBuilder {
    common: TxCommon,
    nf_token_id: Option<String>,
    amount: Option<Value>,
    owner: Option<String>,
    expiration: Option<u32>,
    destination: Option<String>,
}
impl NFTokenCreateOfferBuilder {
    /// A builder for the transaction from `account`.
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
    /// Set `NFTokenID`.
    pub fn nf_token_id(mut self, id: impl Into<String>) -> Self {
        self.nf_token_id = Some(id.into());
        self
    }
    /// Set `Amount`.
    pub fn amount(mut self, a: Value) -> Self {
        self.amount = Some(a);
        self
    }
    /// Set `Owner`.
    pub fn owner(mut self, o: impl Into<String>) -> Self {
        self.owner = Some(o.into());
        self
    }
    /// Set `Expiration`.
    pub fn expiration(mut self, e: u32) -> Self {
        self.expiration = Some(e);
        self
    }
    /// Set `Destination`.
    pub fn destination(mut self, d: impl Into<String>) -> Self {
        self.destination = Some(d.into());
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
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> NFTokenCreateOfferBuilder {
        NFTokenCreateOfferBuilder::new(account)
    }
}

/// NFTokenCancelOffer transaction.
#[derive(Debug, Clone)]
pub struct NFTokenCancelOffer {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// The offers to cancel (`NFTokenOffers`).
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
/// Builds a [`NFTokenCancelOffer`] (from [`NFTokenCancelOffer::builder`]); `build` checks the required fields.
pub struct NFTokenCancelOfferBuilder {
    common: TxCommon,
    nf_token_offers: Option<Vec<String>>,
}
impl NFTokenCancelOfferBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            nf_token_offers: None,
        }
    }
    /// Set `NFTokenOffers`.
    pub fn nf_token_offers(mut self, offers: Vec<String>) -> Self {
        self.nf_token_offers = Some(offers);
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
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> NFTokenCancelOfferBuilder {
        NFTokenCancelOfferBuilder::new(account)
    }
}

/// NFTokenAcceptOffer transaction.
#[derive(Debug, Clone)]
pub struct NFTokenAcceptOffer {
    /// The fields every transaction has.
    pub common: TxCommon,
    /// The buy offer to accept (`NFTokenBuyOffer`).
    pub nf_token_buy_offer: Option<String>,
    /// The sell offer to accept (`NFTokenSellOffer`).
    pub nf_token_sell_offer: Option<String>,
    /// A broker's cut when matching a buy and a sell offer (`NFTokenBrokerFee`).
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
/// Builds a [`NFTokenAcceptOffer`] (from [`NFTokenAcceptOffer::builder`]); `build` checks the required fields.
pub struct NFTokenAcceptOfferBuilder {
    common: TxCommon,
    nf_token_buy_offer: Option<String>,
    nf_token_sell_offer: Option<String>,
    nf_token_broker_fee: Option<Value>,
}
impl NFTokenAcceptOfferBuilder {
    /// A builder for the transaction from `account`.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: TxCommon::new(account),
            nf_token_buy_offer: None,
            nf_token_sell_offer: None,
            nf_token_broker_fee: None,
        }
    }
    /// Set `NFTokenBuyOffer`.
    pub fn nf_token_buy_offer(mut self, o: impl Into<String>) -> Self {
        self.nf_token_buy_offer = Some(o.into());
        self
    }
    /// Set `NFTokenSellOffer`.
    pub fn nf_token_sell_offer(mut self, o: impl Into<String>) -> Self {
        self.nf_token_sell_offer = Some(o.into());
        self
    }
    /// Set `NFTokenBrokerFee`.
    pub fn nf_token_broker_fee(mut self, f: Value) -> Self {
        self.nf_token_broker_fee = Some(f);
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
    /// Start building one from `account`.
    pub fn builder(account: impl Into<String>) -> NFTokenAcceptOfferBuilder {
        NFTokenAcceptOfferBuilder::new(account)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
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
