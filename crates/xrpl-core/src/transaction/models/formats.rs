// @generated from rippled 3.4.1's server_definitions (TRANSACTION_FORMATS, TRANSACTION_FLAGS,
// ACCOUNT_SET_FLAGS) by crates/xrpl-core/scripts/gen_transactions.py. Do not edit; regenerate:
//     python3 gen_transactions.py <server_definitions.json> <this file>
// One struct per transaction type: the common fields in `common`, then the type's own fields as
// rippled lists them. Required fields are plain, optional and default ones `Option`.

#![allow(clippy::upper_case_acronyms, clippy::too_many_arguments)]

use super::*;

use crate::flags::flags_type;

flags_type!(
    /// The flags any transaction may carry (for the types with none of their own).
    UniversalFlags {
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// A value of [`AccountSet`]'s `SetFlag` / `ClearFlag` (`asf…`): one account setting to turn on
/// or off. `u32::from(flag)` is its number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AccountSetFlag {
    /// `asfRequireDest` (1).
    RequireDest = 1,
    /// `asfRequireAuth` (2).
    RequireAuth = 2,
    /// `asfDisallowXRP` (3).
    DisallowXRP = 3,
    /// `asfDisableMaster` (4).
    DisableMaster = 4,
    /// `asfAccountTxnID` (5).
    AccountTxnID = 5,
    /// `asfNoFreeze` (6).
    NoFreeze = 6,
    /// `asfGlobalFreeze` (7).
    GlobalFreeze = 7,
    /// `asfDefaultRipple` (8).
    DefaultRipple = 8,
    /// `asfDepositAuth` (9).
    DepositAuth = 9,
    /// `asfAuthorizedNFTokenMinter` (10).
    AuthorizedNFTokenMinter = 10,
    /// `asfDisallowIncomingNFTokenOffer` (12).
    DisallowIncomingNFTokenOffer = 12,
    /// `asfDisallowIncomingCheck` (13).
    DisallowIncomingCheck = 13,
    /// `asfDisallowIncomingPayChan` (14).
    DisallowIncomingPayChan = 14,
    /// `asfDisallowIncomingTrustline` (15).
    DisallowIncomingTrustline = 15,
    /// `asfAllowTrustLineClawback` (16).
    AllowTrustLineClawback = 16,
    /// `asfAllowTrustLineLocking` (17).
    AllowTrustLineLocking = 17,
}

impl From<AccountSetFlag> for u32 {
    fn from(flag: AccountSetFlag) -> u32 {
        flag as u32
    }
}

impl AccountSetFlag {
    /// The flag with this number, if this version knows it.
    pub fn from_value(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::RequireDest),
            2 => Some(Self::RequireAuth),
            3 => Some(Self::DisallowXRP),
            4 => Some(Self::DisableMaster),
            5 => Some(Self::AccountTxnID),
            6 => Some(Self::NoFreeze),
            7 => Some(Self::GlobalFreeze),
            8 => Some(Self::DefaultRipple),
            9 => Some(Self::DepositAuth),
            10 => Some(Self::AuthorizedNFTokenMinter),
            12 => Some(Self::DisallowIncomingNFTokenOffer),
            13 => Some(Self::DisallowIncomingCheck),
            14 => Some(Self::DisallowIncomingPayChan),
            15 => Some(Self::DisallowIncomingTrustline),
            16 => Some(Self::AllowTrustLineClawback),
            17 => Some(Self::AllowTrustLineLocking),
            _ => None,
        }
    }
}

/// [`AMMBid`](https://xrpl.org/docs/references/protocol/transactions/types/ammbid) transaction.
///
/// Required: `Asset`, `Asset2`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AMMBid {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The pool's first asset (`Asset`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset")]
    pub asset: Asset,
    /// The pool's second asset (`Asset2`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset2")]
    pub asset2: Asset,
    /// The least to bid for the auction slot, in LP tokens (`BidMin`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "BidMin", default, skip_serializing_if = "Option::is_none")]
    pub bid_min: Option<Amount>,
    /// The most to bid for the auction slot, in LP tokens (`BidMax`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "BidMax", default, skip_serializing_if = "Option::is_none")]
    pub bid_max: Option<Amount>,
    /// Accounts that may trade at the auction slot's discounted fee (`AuthAccounts`: a list). Optional.
    #[serde(
        rename = "AuthAccounts",
        default,
        skip_serializing_if = "Option::is_none",
        with = "auth_accounts_opt"
    )]
    pub auth_accounts: Option<Vec<AuthAccount>>,
}

impl AMMBid {
    /// `"AMMBid"`.
    pub const TRANSACTION_TYPE: &'static str = "AMMBid";

    /// A `AMMBid` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, asset: Asset, asset2: Asset) -> Self {
        Self {
            common: Common::new(account),
            asset,
            asset2,
            bid_min: None,
            bid_max: None,
            auth_accounts: None,
        }
    }

    /// With `BidMin` set (see [`Self::bid_min`]).
    #[must_use]
    pub fn with_bid_min(mut self, value: impl Into<Amount>) -> Self {
        self.bid_min = Some(value.into());
        self
    }

    /// With `BidMax` set (see [`Self::bid_max`]).
    #[must_use]
    pub fn with_bid_max(mut self, value: impl Into<Amount>) -> Self {
        self.bid_max = Some(value.into());
        self
    }

    /// With `AuthAccounts` set (see [`Self::auth_accounts`]).
    #[must_use]
    pub fn with_auth_accounts(mut self, value: impl Into<Vec<AuthAccount>>) -> Self {
        self.auth_accounts = Some(value.into());
        self
    }
}

impl TxModel for AMMBid {
    const TRANSACTION_TYPE: &'static str = "AMMBid";
    type Flags = UniversalFlags;
    fn validate(&self) -> Result<(), CoreError> {
        super::validate::amm_bid(self)
    }
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<AMMBid> for Tx {
    fn from(tx: AMMBid) -> Self {
        Tx::AMMBid(tx)
    }
}

flags_type!(
    /// The flags of a [`AMMClawback`] (`Flags`).
    AMMClawbackFlags {
        /// `tfClawTwoAssets`.
        CLAW_TWO_ASSETS = 0x00000001,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`AMMClawback`](https://xrpl.org/docs/references/protocol/transactions/types/ammclawback) transaction.
///
/// Required: `Holder`, `Asset`, `Asset2`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AMMClawback {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The holder (`Holder`: a classic address). Required.
    #[serde(rename = "Holder")]
    pub holder: String,
    /// The pool's first asset (`Asset`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset")]
    pub asset: Asset,
    /// The pool's second asset (`Asset2`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset2")]
    pub asset2: Asset,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "Amount", default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
}

impl AMMClawback {
    /// `"AMMClawback"`.
    pub const TRANSACTION_TYPE: &'static str = "AMMClawback";

    /// A `AMMClawback` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        holder: impl Into<String>,
        asset: Asset,
        asset2: Asset,
    ) -> Self {
        Self {
            common: Common::new(account),
            holder: holder.into(),
            asset,
            asset2,
            amount: None,
        }
    }

    /// With `Amount` set (see [`Self::amount`]).
    #[must_use]
    pub fn with_amount(mut self, value: impl Into<Amount>) -> Self {
        self.amount = Some(value.into());
        self
    }
}

impl TxModel for AMMClawback {
    const TRANSACTION_TYPE: &'static str = "AMMClawback";
    type Flags = AMMClawbackFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<AMMClawback> for Tx {
    fn from(tx: AMMClawback) -> Self {
        Tx::AMMClawback(tx)
    }
}

/// [`AMMCreate`](https://xrpl.org/docs/references/protocol/transactions/types/ammcreate) transaction.
///
/// Required: `Amount`, `Amount2`, `TradingFee`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AMMCreate {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// The second asset's amount (`Amount2`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount2")]
    pub amount2: Amount,
    /// The trading fee, in 1/100,000 (1000 = 1%) (`TradingFee`: a number). Required.
    #[serde(rename = "TradingFee")]
    pub trading_fee: u16,
}

impl AMMCreate {
    /// `"AMMCreate"`.
    pub const TRANSACTION_TYPE: &'static str = "AMMCreate";

    /// A `AMMCreate` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        amount: Amount,
        amount2: Amount,
        trading_fee: u16,
    ) -> Self {
        Self {
            common: Common::new(account),
            amount,
            amount2,
            trading_fee,
        }
    }
}

impl TxModel for AMMCreate {
    const TRANSACTION_TYPE: &'static str = "AMMCreate";
    type Flags = UniversalFlags;
    fn validate(&self) -> Result<(), CoreError> {
        super::validate::amm_create(self)
    }
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<AMMCreate> for Tx {
    fn from(tx: AMMCreate) -> Self {
        Tx::AMMCreate(tx)
    }
}

/// [`AMMDelete`](https://xrpl.org/docs/references/protocol/transactions/types/ammdelete) transaction.
///
/// Required: `Asset`, `Asset2`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AMMDelete {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The pool's first asset (`Asset`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset")]
    pub asset: Asset,
    /// The pool's second asset (`Asset2`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset2")]
    pub asset2: Asset,
}

impl AMMDelete {
    /// `"AMMDelete"`.
    pub const TRANSACTION_TYPE: &'static str = "AMMDelete";

    /// A `AMMDelete` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, asset: Asset, asset2: Asset) -> Self {
        Self {
            common: Common::new(account),
            asset,
            asset2,
        }
    }
}

impl TxModel for AMMDelete {
    const TRANSACTION_TYPE: &'static str = "AMMDelete";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<AMMDelete> for Tx {
    fn from(tx: AMMDelete) -> Self {
        Tx::AMMDelete(tx)
    }
}

flags_type!(
    /// The flags of a [`AMMDeposit`] (`Flags`).
    AMMDepositFlags {
        /// `tfLPToken`.
        LP_TOKEN = 0x00010000,
        /// `tfSingleAsset`.
        SINGLE_ASSET = 0x00080000,
        /// `tfTwoAsset`.
        TWO_ASSET = 0x00100000,
        /// `tfOneAssetLPToken`.
        ONE_ASSET_LP_TOKEN = 0x00200000,
        /// `tfLimitLPToken`.
        LIMIT_LP_TOKEN = 0x00400000,
        /// `tfTwoAssetIfEmpty`.
        TWO_ASSET_IF_EMPTY = 0x00800000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`AMMDeposit`](https://xrpl.org/docs/references/protocol/transactions/types/ammdeposit) transaction.
///
/// Required: `Asset`, `Asset2`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AMMDeposit {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The pool's first asset (`Asset`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset")]
    pub asset: Asset,
    /// The pool's second asset (`Asset2`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset2")]
    pub asset2: Asset,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "Amount", default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
    /// The second asset's amount (`Amount2`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "Amount2", default, skip_serializing_if = "Option::is_none")]
    pub amount2: Option<Amount>,
    /// The effective price limit (`EPrice`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "EPrice", default, skip_serializing_if = "Option::is_none")]
    pub e_price: Option<Amount>,
    /// The LP tokens to receive (`LPTokenOut`: XRP in drops, a token, or an MPT). Optional.
    #[serde(
        rename = "LPTokenOut",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub lp_token_out: Option<Amount>,
    /// The trading fee, in 1/100,000 (1000 = 1%) (`TradingFee`: a number). Optional.
    #[serde(
        rename = "TradingFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub trading_fee: Option<u16>,
}

impl AMMDeposit {
    /// `"AMMDeposit"`.
    pub const TRANSACTION_TYPE: &'static str = "AMMDeposit";

    /// A `AMMDeposit` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, asset: Asset, asset2: Asset) -> Self {
        Self {
            common: Common::new(account),
            asset,
            asset2,
            amount: None,
            amount2: None,
            e_price: None,
            lp_token_out: None,
            trading_fee: None,
        }
    }

    /// With `Amount` set (see [`Self::amount`]).
    #[must_use]
    pub fn with_amount(mut self, value: impl Into<Amount>) -> Self {
        self.amount = Some(value.into());
        self
    }

    /// With `Amount2` set (see [`Self::amount2`]).
    #[must_use]
    pub fn with_amount2(mut self, value: impl Into<Amount>) -> Self {
        self.amount2 = Some(value.into());
        self
    }

    /// With `EPrice` set (see [`Self::e_price`]).
    #[must_use]
    pub fn with_e_price(mut self, value: impl Into<Amount>) -> Self {
        self.e_price = Some(value.into());
        self
    }

    /// With `LPTokenOut` set (see [`Self::lp_token_out`]).
    #[must_use]
    pub fn with_lp_token_out(mut self, value: impl Into<Amount>) -> Self {
        self.lp_token_out = Some(value.into());
        self
    }

    /// With `TradingFee` set (see [`Self::trading_fee`]).
    #[must_use]
    pub fn with_trading_fee(mut self, value: impl Into<u16>) -> Self {
        self.trading_fee = Some(value.into());
        self
    }
}

impl TxModel for AMMDeposit {
    const TRANSACTION_TYPE: &'static str = "AMMDeposit";
    type Flags = AMMDepositFlags;
    fn validate(&self) -> Result<(), CoreError> {
        super::validate::amm_deposit(self)
    }
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<AMMDeposit> for Tx {
    fn from(tx: AMMDeposit) -> Self {
        Tx::AMMDeposit(tx)
    }
}

/// [`AMMVote`](https://xrpl.org/docs/references/protocol/transactions/types/ammvote) transaction.
///
/// Required: `Asset`, `Asset2`, `TradingFee`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AMMVote {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The pool's first asset (`Asset`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset")]
    pub asset: Asset,
    /// The pool's second asset (`Asset2`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset2")]
    pub asset2: Asset,
    /// The trading fee, in 1/100,000 (1000 = 1%) (`TradingFee`: a number). Required.
    #[serde(rename = "TradingFee")]
    pub trading_fee: u16,
}

impl AMMVote {
    /// `"AMMVote"`.
    pub const TRANSACTION_TYPE: &'static str = "AMMVote";

    /// A `AMMVote` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, asset: Asset, asset2: Asset, trading_fee: u16) -> Self {
        Self {
            common: Common::new(account),
            asset,
            asset2,
            trading_fee,
        }
    }
}

impl TxModel for AMMVote {
    const TRANSACTION_TYPE: &'static str = "AMMVote";
    type Flags = UniversalFlags;
    fn validate(&self) -> Result<(), CoreError> {
        super::validate::amm_vote(self)
    }
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<AMMVote> for Tx {
    fn from(tx: AMMVote) -> Self {
        Tx::AMMVote(tx)
    }
}

flags_type!(
    /// The flags of a [`AMMWithdraw`] (`Flags`).
    AMMWithdrawFlags {
        /// `tfLPToken`.
        LP_TOKEN = 0x00010000,
        /// `tfWithdrawAll`.
        WITHDRAW_ALL = 0x00020000,
        /// `tfOneAssetWithdrawAll`.
        ONE_ASSET_WITHDRAW_ALL = 0x00040000,
        /// `tfSingleAsset`.
        SINGLE_ASSET = 0x00080000,
        /// `tfTwoAsset`.
        TWO_ASSET = 0x00100000,
        /// `tfOneAssetLPToken`.
        ONE_ASSET_LP_TOKEN = 0x00200000,
        /// `tfLimitLPToken`.
        LIMIT_LP_TOKEN = 0x00400000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`AMMWithdraw`](https://xrpl.org/docs/references/protocol/transactions/types/ammwithdraw) transaction.
///
/// Required: `Asset`, `Asset2`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AMMWithdraw {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The pool's first asset (`Asset`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset")]
    pub asset: Asset,
    /// The pool's second asset (`Asset2`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset2")]
    pub asset2: Asset,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "Amount", default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
    /// The second asset's amount (`Amount2`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "Amount2", default, skip_serializing_if = "Option::is_none")]
    pub amount2: Option<Amount>,
    /// The effective price limit (`EPrice`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "EPrice", default, skip_serializing_if = "Option::is_none")]
    pub e_price: Option<Amount>,
    /// The LP tokens to return (`LPTokenIn`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "LPTokenIn", default, skip_serializing_if = "Option::is_none")]
    pub lp_token_in: Option<Amount>,
}

impl AMMWithdraw {
    /// `"AMMWithdraw"`.
    pub const TRANSACTION_TYPE: &'static str = "AMMWithdraw";

    /// A `AMMWithdraw` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, asset: Asset, asset2: Asset) -> Self {
        Self {
            common: Common::new(account),
            asset,
            asset2,
            amount: None,
            amount2: None,
            e_price: None,
            lp_token_in: None,
        }
    }

    /// With `Amount` set (see [`Self::amount`]).
    #[must_use]
    pub fn with_amount(mut self, value: impl Into<Amount>) -> Self {
        self.amount = Some(value.into());
        self
    }

    /// With `Amount2` set (see [`Self::amount2`]).
    #[must_use]
    pub fn with_amount2(mut self, value: impl Into<Amount>) -> Self {
        self.amount2 = Some(value.into());
        self
    }

    /// With `EPrice` set (see [`Self::e_price`]).
    #[must_use]
    pub fn with_e_price(mut self, value: impl Into<Amount>) -> Self {
        self.e_price = Some(value.into());
        self
    }

    /// With `LPTokenIn` set (see [`Self::lp_token_in`]).
    #[must_use]
    pub fn with_lp_token_in(mut self, value: impl Into<Amount>) -> Self {
        self.lp_token_in = Some(value.into());
        self
    }
}

impl TxModel for AMMWithdraw {
    const TRANSACTION_TYPE: &'static str = "AMMWithdraw";
    type Flags = AMMWithdrawFlags;
    fn validate(&self) -> Result<(), CoreError> {
        super::validate::amm_withdraw(self)
    }
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<AMMWithdraw> for Tx {
    fn from(tx: AMMWithdraw) -> Self {
        Tx::AMMWithdraw(tx)
    }
}

/// [`AccountDelete`](https://xrpl.org/docs/references/protocol/transactions/types/accountdelete) transaction.
///
/// Required: `Destination`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountDelete {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The account that receives it (`Destination`: a classic address). Required.
    #[serde(rename = "Destination")]
    pub destination: String,
    /// A tag the destination uses to tell its senders apart (`DestinationTag`: a number). Optional.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
    /// Credentials to present (`CredentialIDs`: a list of 256-bit IDs). Optional.
    #[serde(
        rename = "CredentialIDs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub credential_ids: Option<Vec<Hash256>>,
}

impl AccountDelete {
    /// `"AccountDelete"`.
    pub const TRANSACTION_TYPE: &'static str = "AccountDelete";

    /// A `AccountDelete` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, destination: impl Into<String>) -> Self {
        Self {
            common: Common::new(account),
            destination: destination.into(),
            destination_tag: None,
            credential_ids: None,
        }
    }

    /// With `DestinationTag` set (see [`Self::destination_tag`]).
    #[must_use]
    pub fn with_destination_tag(mut self, value: impl Into<u32>) -> Self {
        self.destination_tag = Some(value.into());
        self
    }

    /// With `CredentialIDs` set (see [`Self::credential_ids`]).
    #[must_use]
    pub fn with_credential_ids(mut self, value: impl Into<Vec<Hash256>>) -> Self {
        self.credential_ids = Some(value.into());
        self
    }
}

impl TxModel for AccountDelete {
    const TRANSACTION_TYPE: &'static str = "AccountDelete";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<AccountDelete> for Tx {
    fn from(tx: AccountDelete) -> Self {
        Tx::AccountDelete(tx)
    }
}

flags_type!(
    /// The flags of a [`AccountSet`] (`Flags`).
    AccountSetFlags {
        /// `tfRequireDestTag`.
        REQUIRE_DEST_TAG = 0x00010000,
        /// `tfOptionalDestTag`.
        OPTIONAL_DEST_TAG = 0x00020000,
        /// `tfRequireAuth`.
        REQUIRE_AUTH = 0x00040000,
        /// `tfOptionalAuth`.
        OPTIONAL_AUTH = 0x00080000,
        /// `tfDisallowXRP`.
        DISALLOW_XRP = 0x00100000,
        /// `tfAllowXRP`.
        ALLOW_XRP = 0x00200000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`AccountSet`](https://xrpl.org/docs/references/protocol/transactions/types/accountset) transaction.
///
/// Required: none beyond the common fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AccountSet {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The MD5 of an email address, for an avatar (`EmailHash`: 128 bits, hex). Optional.
    #[serde(rename = "EmailHash", default, skip_serializing_if = "Option::is_none")]
    pub email_hash: Option<Hash128>,
    /// `WalletLocator`: 256 bits, hex. Optional.
    #[serde(
        rename = "WalletLocator",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub wallet_locator: Option<Hash256>,
    /// `WalletSize`: a number. Optional.
    #[serde(
        rename = "WalletSize",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub wallet_size: Option<u32>,
    /// A public key for sending the account encrypted messages (`MessageKey`: bytes, hex). Optional.
    #[serde(
        rename = "MessageKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub message_key: Option<String>,
    /// A domain the account claims, hex (ASCII, lower case) (`Domain`: bytes, hex). Optional.
    #[serde(rename = "Domain", default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// The fee on transfers of the account's tokens, in billionths over 1,000,000,000 (`TransferRate`: a number). Optional.
    #[serde(
        rename = "TransferRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transfer_rate: Option<u32>,
    /// An account flag to turn on (`ASF_*`) (`SetFlag`: a number). Optional.
    #[serde(rename = "SetFlag", default, skip_serializing_if = "Option::is_none")]
    pub set_flag: Option<u32>,
    /// An account flag to turn off (`ASF_*`) (`ClearFlag`: a number). Optional.
    #[serde(rename = "ClearFlag", default, skip_serializing_if = "Option::is_none")]
    pub clear_flag: Option<u32>,
    /// Significant digits for offers of the account's tokens (3–15; 0 clears) (`TickSize`: a number). Optional.
    #[serde(rename = "TickSize", default, skip_serializing_if = "Option::is_none")]
    pub tick_size: Option<u8>,
    /// An account allowed to mint NFTs for this one (`NFTokenMinter`: a classic address). Optional.
    #[serde(
        rename = "NFTokenMinter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub nf_token_minter: Option<String>,
}

impl AccountSet {
    /// `"AccountSet"`.
    pub const TRANSACTION_TYPE: &'static str = "AccountSet";

    /// A `AccountSet` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: Common::new(account),
            email_hash: None,
            wallet_locator: None,
            wallet_size: None,
            message_key: None,
            domain: None,
            transfer_rate: None,
            set_flag: None,
            clear_flag: None,
            tick_size: None,
            nf_token_minter: None,
        }
    }

    /// With `EmailHash` set (see [`Self::email_hash`]).
    #[must_use]
    pub fn with_email_hash(mut self, value: impl Into<Hash128>) -> Self {
        self.email_hash = Some(value.into());
        self
    }

    /// With `WalletLocator` set (see [`Self::wallet_locator`]).
    #[must_use]
    pub fn with_wallet_locator(mut self, value: impl Into<Hash256>) -> Self {
        self.wallet_locator = Some(value.into());
        self
    }

    /// With `WalletSize` set (see [`Self::wallet_size`]).
    #[must_use]
    pub fn with_wallet_size(mut self, value: impl Into<u32>) -> Self {
        self.wallet_size = Some(value.into());
        self
    }

    /// With `MessageKey` set (see [`Self::message_key`]).
    #[must_use]
    pub fn with_message_key(mut self, value: impl Into<String>) -> Self {
        self.message_key = Some(value.into());
        self
    }

    /// With `Domain` set (see [`Self::domain`]).
    #[must_use]
    pub fn with_domain(mut self, value: impl Into<String>) -> Self {
        self.domain = Some(value.into());
        self
    }

    /// With `TransferRate` set (see [`Self::transfer_rate`]).
    #[must_use]
    pub fn with_transfer_rate(mut self, value: impl Into<u32>) -> Self {
        self.transfer_rate = Some(value.into());
        self
    }

    /// With `SetFlag` set (see [`Self::set_flag`]).
    #[must_use]
    pub fn with_set_flag(mut self, value: impl Into<u32>) -> Self {
        self.set_flag = Some(value.into());
        self
    }

    /// With `ClearFlag` set (see [`Self::clear_flag`]).
    #[must_use]
    pub fn with_clear_flag(mut self, value: impl Into<u32>) -> Self {
        self.clear_flag = Some(value.into());
        self
    }

    /// With `TickSize` set (see [`Self::tick_size`]).
    #[must_use]
    pub fn with_tick_size(mut self, value: impl Into<u8>) -> Self {
        self.tick_size = Some(value.into());
        self
    }

    /// With `NFTokenMinter` set (see [`Self::nf_token_minter`]).
    #[must_use]
    pub fn with_nf_token_minter(mut self, value: impl Into<String>) -> Self {
        self.nf_token_minter = Some(value.into());
        self
    }
}

impl TxModel for AccountSet {
    const TRANSACTION_TYPE: &'static str = "AccountSet";
    type Flags = AccountSetFlags;
    fn validate(&self) -> Result<(), CoreError> {
        super::validate::account_set(self)
    }
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<AccountSet> for Tx {
    fn from(tx: AccountSet) -> Self {
        Tx::AccountSet(tx)
    }
}

flags_type!(
    /// The flags of a [`Batch`] (`Flags`).
    BatchFlags {
        /// `tfAllOrNothing`.
        ALL_OR_NOTHING = 0x00010000,
        /// `tfOnlyOne`.
        ONLY_ONE = 0x00020000,
        /// `tfUntilFailure`.
        UNTIL_FAILURE = 0x00040000,
        /// `tfIndependent`.
        INDEPENDENT = 0x00080000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`Batch`](https://xrpl.org/docs/references/protocol/transactions/types/batch) transaction.
///
/// Required: `RawTransactions`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Batch {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The inner transactions (2 to 8) (`RawTransactions`: a list). Required.
    #[serde(rename = "RawTransactions", with = "raw_transactions")]
    pub raw_transactions: Vec<Tx>,
    /// Approvals from the other accounts with inner transactions (`BatchSigners`: a list). Optional.
    #[serde(
        rename = "BatchSigners",
        default,
        skip_serializing_if = "Option::is_none",
        with = "batch_signers_opt"
    )]
    pub batch_signers: Option<Vec<BatchSigner>>,
}

impl Batch {
    /// `"Batch"`.
    pub const TRANSACTION_TYPE: &'static str = "Batch";

    /// A `Batch` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, raw_transactions: Vec<Tx>) -> Self {
        Self {
            common: Common::new(account),
            raw_transactions,
            batch_signers: None,
        }
    }

    /// With `BatchSigners` set (see [`Self::batch_signers`]).
    #[must_use]
    pub fn with_batch_signers(mut self, value: impl Into<Vec<BatchSigner>>) -> Self {
        self.batch_signers = Some(value.into());
        self
    }
}

impl TxModel for Batch {
    const TRANSACTION_TYPE: &'static str = "Batch";
    type Flags = BatchFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<Batch> for Tx {
    fn from(tx: Batch) -> Self {
        Tx::Batch(tx)
    }
}

/// [`CheckCancel`](https://xrpl.org/docs/references/protocol/transactions/types/checkcancel) transaction.
///
/// Required: `CheckID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CheckCancel {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The check's ID (`CheckID`: 256 bits, hex). Required.
    #[serde(rename = "CheckID")]
    pub check_id: Hash256,
}

impl CheckCancel {
    /// `"CheckCancel"`.
    pub const TRANSACTION_TYPE: &'static str = "CheckCancel";

    /// A `CheckCancel` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, check_id: Hash256) -> Self {
        Self {
            common: Common::new(account),
            check_id,
        }
    }
}

impl TxModel for CheckCancel {
    const TRANSACTION_TYPE: &'static str = "CheckCancel";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<CheckCancel> for Tx {
    fn from(tx: CheckCancel) -> Self {
        Tx::CheckCancel(tx)
    }
}

/// [`CheckCash`](https://xrpl.org/docs/references/protocol/transactions/types/checkcash) transaction.
///
/// Required: `CheckID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CheckCash {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The check's ID (`CheckID`: 256 bits, hex). Required.
    #[serde(rename = "CheckID")]
    pub check_id: Hash256,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "Amount", default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
    /// The least a partial payment must deliver (`DeliverMin`: XRP in drops, a token, or an MPT). Optional.
    #[serde(
        rename = "DeliverMin",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub deliver_min: Option<Amount>,
}

impl CheckCash {
    /// `"CheckCash"`.
    pub const TRANSACTION_TYPE: &'static str = "CheckCash";

    /// A `CheckCash` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, check_id: Hash256) -> Self {
        Self {
            common: Common::new(account),
            check_id,
            amount: None,
            deliver_min: None,
        }
    }

    /// With `Amount` set (see [`Self::amount`]).
    #[must_use]
    pub fn with_amount(mut self, value: impl Into<Amount>) -> Self {
        self.amount = Some(value.into());
        self
    }

    /// With `DeliverMin` set (see [`Self::deliver_min`]).
    #[must_use]
    pub fn with_deliver_min(mut self, value: impl Into<Amount>) -> Self {
        self.deliver_min = Some(value.into());
        self
    }
}

impl TxModel for CheckCash {
    const TRANSACTION_TYPE: &'static str = "CheckCash";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<CheckCash> for Tx {
    fn from(tx: CheckCash) -> Self {
        Tx::CheckCash(tx)
    }
}

/// [`CheckCreate`](https://xrpl.org/docs/references/protocol/transactions/types/checkcreate) transaction.
///
/// Required: `Destination`, `SendMax`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CheckCreate {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The account that receives it (`Destination`: a classic address). Required.
    #[serde(rename = "Destination")]
    pub destination: String,
    /// The most the sender will spend, in any currency (`SendMax`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "SendMax")]
    pub send_max: Amount,
    /// When it expires (ledger time: seconds since 2000-01-01) (`Expiration`: a number). Optional.
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
    /// A tag the destination uses to tell its senders apart (`DestinationTag`: a number). Optional.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
    /// An ID of the sender's choosing, such as an invoice number (`InvoiceID`: 256 bits, hex). Optional.
    #[serde(rename = "InvoiceID", default, skip_serializing_if = "Option::is_none")]
    pub invoice_id: Option<Hash256>,
}

impl CheckCreate {
    /// `"CheckCreate"`.
    pub const TRANSACTION_TYPE: &'static str = "CheckCreate";

    /// A `CheckCreate` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        destination: impl Into<String>,
        send_max: Amount,
    ) -> Self {
        Self {
            common: Common::new(account),
            destination: destination.into(),
            send_max,
            expiration: None,
            destination_tag: None,
            invoice_id: None,
        }
    }

    /// With `Expiration` set (see [`Self::expiration`]).
    #[must_use]
    pub fn with_expiration(mut self, value: impl Into<u32>) -> Self {
        self.expiration = Some(value.into());
        self
    }

    /// With `DestinationTag` set (see [`Self::destination_tag`]).
    #[must_use]
    pub fn with_destination_tag(mut self, value: impl Into<u32>) -> Self {
        self.destination_tag = Some(value.into());
        self
    }

    /// With `InvoiceID` set (see [`Self::invoice_id`]).
    #[must_use]
    pub fn with_invoice_id(mut self, value: impl Into<Hash256>) -> Self {
        self.invoice_id = Some(value.into());
        self
    }
}

impl TxModel for CheckCreate {
    const TRANSACTION_TYPE: &'static str = "CheckCreate";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<CheckCreate> for Tx {
    fn from(tx: CheckCreate) -> Self {
        Tx::CheckCreate(tx)
    }
}

/// [`Clawback`](https://xrpl.org/docs/references/protocol/transactions/types/clawback) transaction.
///
/// Required: `Amount`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Clawback {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// The holder (`Holder`: a classic address). Optional.
    #[serde(rename = "Holder", default, skip_serializing_if = "Option::is_none")]
    pub holder: Option<String>,
}

impl Clawback {
    /// `"Clawback"`.
    pub const TRANSACTION_TYPE: &'static str = "Clawback";

    /// A `Clawback` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, amount: Amount) -> Self {
        Self {
            common: Common::new(account),
            amount,
            holder: None,
        }
    }

    /// With `Holder` set (see [`Self::holder`]).
    #[must_use]
    pub fn with_holder(mut self, value: impl Into<String>) -> Self {
        self.holder = Some(value.into());
        self
    }
}

impl TxModel for Clawback {
    const TRANSACTION_TYPE: &'static str = "Clawback";
    type Flags = UniversalFlags;
    fn validate(&self) -> Result<(), CoreError> {
        super::validate::clawback(self)
    }
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<Clawback> for Tx {
    fn from(tx: Clawback) -> Self {
        Tx::Clawback(tx)
    }
}

/// [`ConfidentialMPTClawback`](https://xrpl.org/docs/references/protocol/transactions/types/confidentialmptclawback) transaction.
///
/// Required: `MPTokenIssuanceID`, `Holder`, `MPTAmount`, `ZKProof`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConfidentialMPTClawback {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The MPT issuance's ID (`MPTokenIssuanceID`: 192 bits, hex). Required.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mp_token_issuance_id: Hash192,
    /// The holder (`Holder`: a classic address). Required.
    #[serde(rename = "Holder")]
    pub holder: String,
    /// `MPTAmount`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "MPTAmount")]
    pub mpt_amount: String,
    /// `ZKProof`: bytes, hex. Required.
    #[serde(rename = "ZKProof")]
    pub zk_proof: String,
}

impl ConfidentialMPTClawback {
    /// `"ConfidentialMPTClawback"`.
    pub const TRANSACTION_TYPE: &'static str = "ConfidentialMPTClawback";

    /// A `ConfidentialMPTClawback` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        mp_token_issuance_id: Hash192,
        holder: impl Into<String>,
        mpt_amount: impl Into<String>,
        zk_proof: impl Into<String>,
    ) -> Self {
        Self {
            common: Common::new(account),
            mp_token_issuance_id,
            holder: holder.into(),
            mpt_amount: mpt_amount.into(),
            zk_proof: zk_proof.into(),
        }
    }
}

impl TxModel for ConfidentialMPTClawback {
    const TRANSACTION_TYPE: &'static str = "ConfidentialMPTClawback";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<ConfidentialMPTClawback> for Tx {
    fn from(tx: ConfidentialMPTClawback) -> Self {
        Tx::ConfidentialMPTClawback(tx)
    }
}

/// [`ConfidentialMPTConvert`](https://xrpl.org/docs/references/protocol/transactions/types/confidentialmptconvert) transaction.
///
/// Required: `MPTokenIssuanceID`, `MPTAmount`, `HolderEncryptedAmount`, `IssuerEncryptedAmount`, `BlindingFactor`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConfidentialMPTConvert {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The MPT issuance's ID (`MPTokenIssuanceID`: 192 bits, hex). Required.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mp_token_issuance_id: Hash192,
    /// `MPTAmount`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "MPTAmount")]
    pub mpt_amount: String,
    /// `HolderEncryptionKey`: bytes, hex. Optional.
    #[serde(
        rename = "HolderEncryptionKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub holder_encryption_key: Option<String>,
    /// `HolderEncryptedAmount`: bytes, hex. Required.
    #[serde(rename = "HolderEncryptedAmount")]
    pub holder_encrypted_amount: String,
    /// `IssuerEncryptedAmount`: bytes, hex. Required.
    #[serde(rename = "IssuerEncryptedAmount")]
    pub issuer_encrypted_amount: String,
    /// `AuditorEncryptedAmount`: bytes, hex. Optional.
    #[serde(
        rename = "AuditorEncryptedAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auditor_encrypted_amount: Option<String>,
    /// `BlindingFactor`: 256 bits, hex. Required.
    #[serde(rename = "BlindingFactor")]
    pub blinding_factor: Hash256,
    /// `ZKProof`: bytes, hex. Optional.
    #[serde(rename = "ZKProof", default, skip_serializing_if = "Option::is_none")]
    pub zk_proof: Option<String>,
}

impl ConfidentialMPTConvert {
    /// `"ConfidentialMPTConvert"`.
    pub const TRANSACTION_TYPE: &'static str = "ConfidentialMPTConvert";

    /// A `ConfidentialMPTConvert` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        mp_token_issuance_id: Hash192,
        mpt_amount: impl Into<String>,
        holder_encrypted_amount: impl Into<String>,
        issuer_encrypted_amount: impl Into<String>,
        blinding_factor: Hash256,
    ) -> Self {
        Self {
            common: Common::new(account),
            mp_token_issuance_id,
            mpt_amount: mpt_amount.into(),
            holder_encryption_key: None,
            holder_encrypted_amount: holder_encrypted_amount.into(),
            issuer_encrypted_amount: issuer_encrypted_amount.into(),
            auditor_encrypted_amount: None,
            blinding_factor,
            zk_proof: None,
        }
    }

    /// With `HolderEncryptionKey` set (see [`Self::holder_encryption_key`]).
    #[must_use]
    pub fn with_holder_encryption_key(mut self, value: impl Into<String>) -> Self {
        self.holder_encryption_key = Some(value.into());
        self
    }

    /// With `AuditorEncryptedAmount` set (see [`Self::auditor_encrypted_amount`]).
    #[must_use]
    pub fn with_auditor_encrypted_amount(mut self, value: impl Into<String>) -> Self {
        self.auditor_encrypted_amount = Some(value.into());
        self
    }

    /// With `ZKProof` set (see [`Self::zk_proof`]).
    #[must_use]
    pub fn with_zk_proof(mut self, value: impl Into<String>) -> Self {
        self.zk_proof = Some(value.into());
        self
    }
}

impl TxModel for ConfidentialMPTConvert {
    const TRANSACTION_TYPE: &'static str = "ConfidentialMPTConvert";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<ConfidentialMPTConvert> for Tx {
    fn from(tx: ConfidentialMPTConvert) -> Self {
        Tx::ConfidentialMPTConvert(tx)
    }
}

/// [`ConfidentialMPTConvertBack`](https://xrpl.org/docs/references/protocol/transactions/types/confidentialmptconvertback) transaction.
///
/// Required: `MPTokenIssuanceID`, `MPTAmount`, `HolderEncryptedAmount`, `IssuerEncryptedAmount`, `BlindingFactor`, `ZKProof`, `BalanceCommitment`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConfidentialMPTConvertBack {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The MPT issuance's ID (`MPTokenIssuanceID`: 192 bits, hex). Required.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mp_token_issuance_id: Hash192,
    /// `MPTAmount`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "MPTAmount")]
    pub mpt_amount: String,
    /// `HolderEncryptedAmount`: bytes, hex. Required.
    #[serde(rename = "HolderEncryptedAmount")]
    pub holder_encrypted_amount: String,
    /// `IssuerEncryptedAmount`: bytes, hex. Required.
    #[serde(rename = "IssuerEncryptedAmount")]
    pub issuer_encrypted_amount: String,
    /// `AuditorEncryptedAmount`: bytes, hex. Optional.
    #[serde(
        rename = "AuditorEncryptedAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auditor_encrypted_amount: Option<String>,
    /// `BlindingFactor`: 256 bits, hex. Required.
    #[serde(rename = "BlindingFactor")]
    pub blinding_factor: Hash256,
    /// `ZKProof`: bytes, hex. Required.
    #[serde(rename = "ZKProof")]
    pub zk_proof: String,
    /// `BalanceCommitment`: bytes, hex. Required.
    #[serde(rename = "BalanceCommitment")]
    pub balance_commitment: String,
}

impl ConfidentialMPTConvertBack {
    /// `"ConfidentialMPTConvertBack"`.
    pub const TRANSACTION_TYPE: &'static str = "ConfidentialMPTConvertBack";

    /// A `ConfidentialMPTConvertBack` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        mp_token_issuance_id: Hash192,
        mpt_amount: impl Into<String>,
        holder_encrypted_amount: impl Into<String>,
        issuer_encrypted_amount: impl Into<String>,
        blinding_factor: Hash256,
        zk_proof: impl Into<String>,
        balance_commitment: impl Into<String>,
    ) -> Self {
        Self {
            common: Common::new(account),
            mp_token_issuance_id,
            mpt_amount: mpt_amount.into(),
            holder_encrypted_amount: holder_encrypted_amount.into(),
            issuer_encrypted_amount: issuer_encrypted_amount.into(),
            auditor_encrypted_amount: None,
            blinding_factor,
            zk_proof: zk_proof.into(),
            balance_commitment: balance_commitment.into(),
        }
    }

    /// With `AuditorEncryptedAmount` set (see [`Self::auditor_encrypted_amount`]).
    #[must_use]
    pub fn with_auditor_encrypted_amount(mut self, value: impl Into<String>) -> Self {
        self.auditor_encrypted_amount = Some(value.into());
        self
    }
}

impl TxModel for ConfidentialMPTConvertBack {
    const TRANSACTION_TYPE: &'static str = "ConfidentialMPTConvertBack";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<ConfidentialMPTConvertBack> for Tx {
    fn from(tx: ConfidentialMPTConvertBack) -> Self {
        Tx::ConfidentialMPTConvertBack(tx)
    }
}

/// [`ConfidentialMPTMergeInbox`](https://xrpl.org/docs/references/protocol/transactions/types/confidentialmptmergeinbox) transaction.
///
/// Required: `MPTokenIssuanceID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConfidentialMPTMergeInbox {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The MPT issuance's ID (`MPTokenIssuanceID`: 192 bits, hex). Required.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mp_token_issuance_id: Hash192,
}

impl ConfidentialMPTMergeInbox {
    /// `"ConfidentialMPTMergeInbox"`.
    pub const TRANSACTION_TYPE: &'static str = "ConfidentialMPTMergeInbox";

    /// A `ConfidentialMPTMergeInbox` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, mp_token_issuance_id: Hash192) -> Self {
        Self {
            common: Common::new(account),
            mp_token_issuance_id,
        }
    }
}

impl TxModel for ConfidentialMPTMergeInbox {
    const TRANSACTION_TYPE: &'static str = "ConfidentialMPTMergeInbox";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<ConfidentialMPTMergeInbox> for Tx {
    fn from(tx: ConfidentialMPTMergeInbox) -> Self {
        Tx::ConfidentialMPTMergeInbox(tx)
    }
}

/// [`ConfidentialMPTSend`](https://xrpl.org/docs/references/protocol/transactions/types/confidentialmptsend) transaction.
///
/// Required: `MPTokenIssuanceID`, `Destination`, `SenderEncryptedAmount`, `DestinationEncryptedAmount`, `IssuerEncryptedAmount`, `ZKProof`, `AmountCommitment`, `BalanceCommitment`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConfidentialMPTSend {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The MPT issuance's ID (`MPTokenIssuanceID`: 192 bits, hex). Required.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mp_token_issuance_id: Hash192,
    /// The account that receives it (`Destination`: a classic address). Required.
    #[serde(rename = "Destination")]
    pub destination: String,
    /// A tag the destination uses to tell its senders apart (`DestinationTag`: a number). Optional.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
    /// `SenderEncryptedAmount`: bytes, hex. Required.
    #[serde(rename = "SenderEncryptedAmount")]
    pub sender_encrypted_amount: String,
    /// `DestinationEncryptedAmount`: bytes, hex. Required.
    #[serde(rename = "DestinationEncryptedAmount")]
    pub destination_encrypted_amount: String,
    /// `IssuerEncryptedAmount`: bytes, hex. Required.
    #[serde(rename = "IssuerEncryptedAmount")]
    pub issuer_encrypted_amount: String,
    /// `AuditorEncryptedAmount`: bytes, hex. Optional.
    #[serde(
        rename = "AuditorEncryptedAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auditor_encrypted_amount: Option<String>,
    /// `ZKProof`: bytes, hex. Required.
    #[serde(rename = "ZKProof")]
    pub zk_proof: String,
    /// `AmountCommitment`: bytes, hex. Required.
    #[serde(rename = "AmountCommitment")]
    pub amount_commitment: String,
    /// `BalanceCommitment`: bytes, hex. Required.
    #[serde(rename = "BalanceCommitment")]
    pub balance_commitment: String,
    /// Credentials to present (`CredentialIDs`: a list of 256-bit IDs). Optional.
    #[serde(
        rename = "CredentialIDs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub credential_ids: Option<Vec<Hash256>>,
}

impl ConfidentialMPTSend {
    /// `"ConfidentialMPTSend"`.
    pub const TRANSACTION_TYPE: &'static str = "ConfidentialMPTSend";

    /// A `ConfidentialMPTSend` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        mp_token_issuance_id: Hash192,
        destination: impl Into<String>,
        sender_encrypted_amount: impl Into<String>,
        destination_encrypted_amount: impl Into<String>,
        issuer_encrypted_amount: impl Into<String>,
        zk_proof: impl Into<String>,
        amount_commitment: impl Into<String>,
        balance_commitment: impl Into<String>,
    ) -> Self {
        Self {
            common: Common::new(account),
            mp_token_issuance_id,
            destination: destination.into(),
            destination_tag: None,
            sender_encrypted_amount: sender_encrypted_amount.into(),
            destination_encrypted_amount: destination_encrypted_amount.into(),
            issuer_encrypted_amount: issuer_encrypted_amount.into(),
            auditor_encrypted_amount: None,
            zk_proof: zk_proof.into(),
            amount_commitment: amount_commitment.into(),
            balance_commitment: balance_commitment.into(),
            credential_ids: None,
        }
    }

    /// With `DestinationTag` set (see [`Self::destination_tag`]).
    #[must_use]
    pub fn with_destination_tag(mut self, value: impl Into<u32>) -> Self {
        self.destination_tag = Some(value.into());
        self
    }

    /// With `AuditorEncryptedAmount` set (see [`Self::auditor_encrypted_amount`]).
    #[must_use]
    pub fn with_auditor_encrypted_amount(mut self, value: impl Into<String>) -> Self {
        self.auditor_encrypted_amount = Some(value.into());
        self
    }

    /// With `CredentialIDs` set (see [`Self::credential_ids`]).
    #[must_use]
    pub fn with_credential_ids(mut self, value: impl Into<Vec<Hash256>>) -> Self {
        self.credential_ids = Some(value.into());
        self
    }
}

impl TxModel for ConfidentialMPTSend {
    const TRANSACTION_TYPE: &'static str = "ConfidentialMPTSend";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<ConfidentialMPTSend> for Tx {
    fn from(tx: ConfidentialMPTSend) -> Self {
        Tx::ConfidentialMPTSend(tx)
    }
}

/// [`CredentialAccept`](https://xrpl.org/docs/references/protocol/transactions/types/credentialaccept) transaction.
///
/// Required: `Issuer`, `CredentialType`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CredentialAccept {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The issuer (`Issuer`: a classic address). Required.
    #[serde(rename = "Issuer")]
    pub issuer: String,
    /// The credential's type, hex (`CredentialType`: bytes, hex). Required.
    #[serde(rename = "CredentialType")]
    pub credential_type: String,
}

impl CredentialAccept {
    /// `"CredentialAccept"`.
    pub const TRANSACTION_TYPE: &'static str = "CredentialAccept";

    /// A `CredentialAccept` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        issuer: impl Into<String>,
        credential_type: impl Into<String>,
    ) -> Self {
        Self {
            common: Common::new(account),
            issuer: issuer.into(),
            credential_type: credential_type.into(),
        }
    }
}

impl TxModel for CredentialAccept {
    const TRANSACTION_TYPE: &'static str = "CredentialAccept";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<CredentialAccept> for Tx {
    fn from(tx: CredentialAccept) -> Self {
        Tx::CredentialAccept(tx)
    }
}

/// [`CredentialCreate`](https://xrpl.org/docs/references/protocol/transactions/types/credentialcreate) transaction.
///
/// Required: `Subject`, `CredentialType`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CredentialCreate {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The account a credential is about (`Subject`: a classic address). Required.
    #[serde(rename = "Subject")]
    pub subject: String,
    /// The credential's type, hex (`CredentialType`: bytes, hex). Required.
    #[serde(rename = "CredentialType")]
    pub credential_type: String,
    /// When it expires (ledger time: seconds since 2000-01-01) (`Expiration`: a number). Optional.
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
    /// A URI, hex (`URI`: bytes, hex). Optional.
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

impl CredentialCreate {
    /// `"CredentialCreate"`.
    pub const TRANSACTION_TYPE: &'static str = "CredentialCreate";

    /// A `CredentialCreate` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        subject: impl Into<String>,
        credential_type: impl Into<String>,
    ) -> Self {
        Self {
            common: Common::new(account),
            subject: subject.into(),
            credential_type: credential_type.into(),
            expiration: None,
            uri: None,
        }
    }

    /// With `Expiration` set (see [`Self::expiration`]).
    #[must_use]
    pub fn with_expiration(mut self, value: impl Into<u32>) -> Self {
        self.expiration = Some(value.into());
        self
    }

    /// With `URI` set (see [`Self::uri`]).
    #[must_use]
    pub fn with_uri(mut self, value: impl Into<String>) -> Self {
        self.uri = Some(value.into());
        self
    }
}

impl TxModel for CredentialCreate {
    const TRANSACTION_TYPE: &'static str = "CredentialCreate";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<CredentialCreate> for Tx {
    fn from(tx: CredentialCreate) -> Self {
        Tx::CredentialCreate(tx)
    }
}

/// [`CredentialDelete`](https://xrpl.org/docs/references/protocol/transactions/types/credentialdelete) transaction.
///
/// Required: `CredentialType`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CredentialDelete {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The account a credential is about (`Subject`: a classic address). Optional.
    #[serde(rename = "Subject", default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The issuer (`Issuer`: a classic address). Optional.
    #[serde(rename = "Issuer", default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    /// The credential's type, hex (`CredentialType`: bytes, hex). Required.
    #[serde(rename = "CredentialType")]
    pub credential_type: String,
}

impl CredentialDelete {
    /// `"CredentialDelete"`.
    pub const TRANSACTION_TYPE: &'static str = "CredentialDelete";

    /// A `CredentialDelete` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, credential_type: impl Into<String>) -> Self {
        Self {
            common: Common::new(account),
            subject: None,
            issuer: None,
            credential_type: credential_type.into(),
        }
    }

    /// With `Subject` set (see [`Self::subject`]).
    #[must_use]
    pub fn with_subject(mut self, value: impl Into<String>) -> Self {
        self.subject = Some(value.into());
        self
    }

    /// With `Issuer` set (see [`Self::issuer`]).
    #[must_use]
    pub fn with_issuer(mut self, value: impl Into<String>) -> Self {
        self.issuer = Some(value.into());
        self
    }
}

impl TxModel for CredentialDelete {
    const TRANSACTION_TYPE: &'static str = "CredentialDelete";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<CredentialDelete> for Tx {
    fn from(tx: CredentialDelete) -> Self {
        Tx::CredentialDelete(tx)
    }
}

/// [`DIDDelete`](https://xrpl.org/docs/references/protocol/transactions/types/diddelete) transaction.
///
/// Required: none beyond the common fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DIDDelete {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
}

impl DIDDelete {
    /// `"DIDDelete"`.
    pub const TRANSACTION_TYPE: &'static str = "DIDDelete";

    /// A `DIDDelete` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: Common::new(account),
        }
    }
}

impl TxModel for DIDDelete {
    const TRANSACTION_TYPE: &'static str = "DIDDelete";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<DIDDelete> for Tx {
    fn from(tx: DIDDelete) -> Self {
        Tx::DIDDelete(tx)
    }
}

/// [`DIDSet`](https://xrpl.org/docs/references/protocol/transactions/types/didset) transaction.
///
/// Required: none beyond the common fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DIDSet {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The DID document, hex (`DIDDocument`: bytes, hex). Optional.
    #[serde(
        rename = "DIDDocument",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub did_document: Option<String>,
    /// A URI, hex (`URI`: bytes, hex). Optional.
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Data, hex (`Data`: bytes, hex). Optional.
    #[serde(rename = "Data", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

impl DIDSet {
    /// `"DIDSet"`.
    pub const TRANSACTION_TYPE: &'static str = "DIDSet";

    /// A `DIDSet` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: Common::new(account),
            did_document: None,
            uri: None,
            data: None,
        }
    }

    /// With `DIDDocument` set (see [`Self::did_document`]).
    #[must_use]
    pub fn with_did_document(mut self, value: impl Into<String>) -> Self {
        self.did_document = Some(value.into());
        self
    }

    /// With `URI` set (see [`Self::uri`]).
    #[must_use]
    pub fn with_uri(mut self, value: impl Into<String>) -> Self {
        self.uri = Some(value.into());
        self
    }

    /// With `Data` set (see [`Self::data`]).
    #[must_use]
    pub fn with_data(mut self, value: impl Into<String>) -> Self {
        self.data = Some(value.into());
        self
    }
}

impl TxModel for DIDSet {
    const TRANSACTION_TYPE: &'static str = "DIDSet";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<DIDSet> for Tx {
    fn from(tx: DIDSet) -> Self {
        Tx::DIDSet(tx)
    }
}

/// [`DelegateSet`](https://xrpl.org/docs/references/protocol/transactions/types/delegateset) transaction.
///
/// Required: `Authorize`, `Permissions`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DelegateSet {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The account to authorize (`Authorize`: a classic address). Required.
    #[serde(rename = "Authorize")]
    pub authorize: String,
    /// The transaction types or granular permissions to grant (`Permissions`: a list). Required.
    #[serde(rename = "Permissions", with = "permissions")]
    pub permissions: Vec<Permission>,
}

impl DelegateSet {
    /// `"DelegateSet"`.
    pub const TRANSACTION_TYPE: &'static str = "DelegateSet";

    /// A `DelegateSet` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        authorize: impl Into<String>,
        permissions: Vec<Permission>,
    ) -> Self {
        Self {
            common: Common::new(account),
            authorize: authorize.into(),
            permissions,
        }
    }
}

impl TxModel for DelegateSet {
    const TRANSACTION_TYPE: &'static str = "DelegateSet";
    type Flags = UniversalFlags;
    fn validate(&self) -> Result<(), CoreError> {
        super::validate::delegate_set(self)
    }
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<DelegateSet> for Tx {
    fn from(tx: DelegateSet) -> Self {
        Tx::DelegateSet(tx)
    }
}

/// [`DepositPreauth`](https://xrpl.org/docs/references/protocol/transactions/types/depositpreauth) transaction.
///
/// Required: none beyond the common fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DepositPreauth {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The account to authorize (`Authorize`: a classic address). Optional.
    #[serde(rename = "Authorize", default, skip_serializing_if = "Option::is_none")]
    pub authorize: Option<String>,
    /// The account to stop authorizing (`Unauthorize`: a classic address). Optional.
    #[serde(
        rename = "Unauthorize",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub unauthorize: Option<String>,
    /// Credentials to authorize (`AuthorizeCredentials`: a list). Optional.
    #[serde(
        rename = "AuthorizeCredentials",
        default,
        skip_serializing_if = "Option::is_none",
        with = "credentials_opt"
    )]
    pub authorize_credentials: Option<Vec<CredentialSpec>>,
    /// Credentials to stop authorizing (`UnauthorizeCredentials`: a list). Optional.
    #[serde(
        rename = "UnauthorizeCredentials",
        default,
        skip_serializing_if = "Option::is_none",
        with = "credentials_opt"
    )]
    pub unauthorize_credentials: Option<Vec<CredentialSpec>>,
}

impl DepositPreauth {
    /// `"DepositPreauth"`.
    pub const TRANSACTION_TYPE: &'static str = "DepositPreauth";

    /// A `DepositPreauth` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: Common::new(account),
            authorize: None,
            unauthorize: None,
            authorize_credentials: None,
            unauthorize_credentials: None,
        }
    }

    /// With `Authorize` set (see [`Self::authorize`]).
    #[must_use]
    pub fn with_authorize(mut self, value: impl Into<String>) -> Self {
        self.authorize = Some(value.into());
        self
    }

    /// With `Unauthorize` set (see [`Self::unauthorize`]).
    #[must_use]
    pub fn with_unauthorize(mut self, value: impl Into<String>) -> Self {
        self.unauthorize = Some(value.into());
        self
    }

    /// With `AuthorizeCredentials` set (see [`Self::authorize_credentials`]).
    #[must_use]
    pub fn with_authorize_credentials(mut self, value: impl Into<Vec<CredentialSpec>>) -> Self {
        self.authorize_credentials = Some(value.into());
        self
    }

    /// With `UnauthorizeCredentials` set (see [`Self::unauthorize_credentials`]).
    #[must_use]
    pub fn with_unauthorize_credentials(mut self, value: impl Into<Vec<CredentialSpec>>) -> Self {
        self.unauthorize_credentials = Some(value.into());
        self
    }
}

impl TxModel for DepositPreauth {
    const TRANSACTION_TYPE: &'static str = "DepositPreauth";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<DepositPreauth> for Tx {
    fn from(tx: DepositPreauth) -> Self {
        Tx::DepositPreauth(tx)
    }
}

flags_type!(
    /// The flags of a [`EnableAmendment`] (`Flags`).
    EnableAmendmentFlags {
        /// `tfGotMajority`.
        GOT_MAJORITY = 0x00010000,
        /// `tfLostMajority`.
        LOST_MAJORITY = 0x00020000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`EnableAmendment`](https://xrpl.org/docs/references/protocol/transactions/types/enableamendment) transaction.
///
/// Required: `LedgerSequence`, `Amendment`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EnableAmendment {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The ledger the pseudo-transaction applies to (`LedgerSequence`: a number). Required.
    #[serde(rename = "LedgerSequence")]
    pub ledger_sequence: u32,
    /// An amendment's ID (`Amendment`: 256 bits, hex). Required.
    #[serde(rename = "Amendment")]
    pub amendment: Hash256,
}

impl EnableAmendment {
    /// `"EnableAmendment"`.
    pub const TRANSACTION_TYPE: &'static str = "EnableAmendment";

    /// A `EnableAmendment` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, ledger_sequence: u32, amendment: Hash256) -> Self {
        Self {
            common: Common::new(account),
            ledger_sequence,
            amendment,
        }
    }
}

impl TxModel for EnableAmendment {
    const TRANSACTION_TYPE: &'static str = "EnableAmendment";
    type Flags = EnableAmendmentFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<EnableAmendment> for Tx {
    fn from(tx: EnableAmendment) -> Self {
        Tx::EnableAmendment(tx)
    }
}

/// [`EscrowCancel`](https://xrpl.org/docs/references/protocol/transactions/types/escrowcancel) transaction.
///
/// Required: `Owner`, `OfferSequence`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EscrowCancel {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The account that owns the entry (`Owner`: a classic address). Required.
    #[serde(rename = "Owner")]
    pub owner: String,
    /// The sequence of the transaction that created the entry (`OfferSequence`: a number). Required.
    #[serde(rename = "OfferSequence")]
    pub offer_sequence: u32,
}

impl EscrowCancel {
    /// `"EscrowCancel"`.
    pub const TRANSACTION_TYPE: &'static str = "EscrowCancel";

    /// A `EscrowCancel` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, owner: impl Into<String>, offer_sequence: u32) -> Self {
        Self {
            common: Common::new(account),
            owner: owner.into(),
            offer_sequence,
        }
    }
}

impl TxModel for EscrowCancel {
    const TRANSACTION_TYPE: &'static str = "EscrowCancel";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<EscrowCancel> for Tx {
    fn from(tx: EscrowCancel) -> Self {
        Tx::EscrowCancel(tx)
    }
}

/// [`EscrowCreate`](https://xrpl.org/docs/references/protocol/transactions/types/escrowcreate) transaction.
///
/// Required: `Destination`, `Amount`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EscrowCreate {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The account that receives it (`Destination`: a classic address). Required.
    #[serde(rename = "Destination")]
    pub destination: String,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// A PREIMAGE-SHA-256 crypto-condition that must be fulfilled (`Condition`: bytes, hex). Optional.
    #[serde(rename = "Condition", default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// The time after which it can be cancelled (ledger time) (`CancelAfter`: a number). Optional.
    #[serde(
        rename = "CancelAfter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cancel_after: Option<u32>,
    /// The time after which it can be finished (ledger time) (`FinishAfter`: a number). Optional.
    #[serde(
        rename = "FinishAfter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub finish_after: Option<u32>,
    /// A tag the destination uses to tell its senders apart (`DestinationTag`: a number). Optional.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
}

impl EscrowCreate {
    /// `"EscrowCreate"`.
    pub const TRANSACTION_TYPE: &'static str = "EscrowCreate";

    /// A `EscrowCreate` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, destination: impl Into<String>, amount: Amount) -> Self {
        Self {
            common: Common::new(account),
            destination: destination.into(),
            amount,
            condition: None,
            cancel_after: None,
            finish_after: None,
            destination_tag: None,
        }
    }

    /// With `Condition` set (see [`Self::condition`]).
    #[must_use]
    pub fn with_condition(mut self, value: impl Into<String>) -> Self {
        self.condition = Some(value.into());
        self
    }

    /// With `CancelAfter` set (see [`Self::cancel_after`]).
    #[must_use]
    pub fn with_cancel_after(mut self, value: impl Into<u32>) -> Self {
        self.cancel_after = Some(value.into());
        self
    }

    /// With `FinishAfter` set (see [`Self::finish_after`]).
    #[must_use]
    pub fn with_finish_after(mut self, value: impl Into<u32>) -> Self {
        self.finish_after = Some(value.into());
        self
    }

    /// With `DestinationTag` set (see [`Self::destination_tag`]).
    #[must_use]
    pub fn with_destination_tag(mut self, value: impl Into<u32>) -> Self {
        self.destination_tag = Some(value.into());
        self
    }
}

impl TxModel for EscrowCreate {
    const TRANSACTION_TYPE: &'static str = "EscrowCreate";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<EscrowCreate> for Tx {
    fn from(tx: EscrowCreate) -> Self {
        Tx::EscrowCreate(tx)
    }
}

/// [`EscrowFinish`](https://xrpl.org/docs/references/protocol/transactions/types/escrowfinish) transaction.
///
/// Required: `Owner`, `OfferSequence`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EscrowFinish {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The account that owns the entry (`Owner`: a classic address). Required.
    #[serde(rename = "Owner")]
    pub owner: String,
    /// The sequence of the transaction that created the entry (`OfferSequence`: a number). Required.
    #[serde(rename = "OfferSequence")]
    pub offer_sequence: u32,
    /// The fulfillment of the escrow's condition (`Fulfillment`: bytes, hex). Optional.
    #[serde(
        rename = "Fulfillment",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub fulfillment: Option<String>,
    /// A PREIMAGE-SHA-256 crypto-condition that must be fulfilled (`Condition`: bytes, hex). Optional.
    #[serde(rename = "Condition", default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// Credentials to present (`CredentialIDs`: a list of 256-bit IDs). Optional.
    #[serde(
        rename = "CredentialIDs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub credential_ids: Option<Vec<Hash256>>,
}

impl EscrowFinish {
    /// `"EscrowFinish"`.
    pub const TRANSACTION_TYPE: &'static str = "EscrowFinish";

    /// A `EscrowFinish` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, owner: impl Into<String>, offer_sequence: u32) -> Self {
        Self {
            common: Common::new(account),
            owner: owner.into(),
            offer_sequence,
            fulfillment: None,
            condition: None,
            credential_ids: None,
        }
    }

    /// With `Fulfillment` set (see [`Self::fulfillment`]).
    #[must_use]
    pub fn with_fulfillment(mut self, value: impl Into<String>) -> Self {
        self.fulfillment = Some(value.into());
        self
    }

    /// With `Condition` set (see [`Self::condition`]).
    #[must_use]
    pub fn with_condition(mut self, value: impl Into<String>) -> Self {
        self.condition = Some(value.into());
        self
    }

    /// With `CredentialIDs` set (see [`Self::credential_ids`]).
    #[must_use]
    pub fn with_credential_ids(mut self, value: impl Into<Vec<Hash256>>) -> Self {
        self.credential_ids = Some(value.into());
        self
    }
}

impl TxModel for EscrowFinish {
    const TRANSACTION_TYPE: &'static str = "EscrowFinish";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<EscrowFinish> for Tx {
    fn from(tx: EscrowFinish) -> Self {
        Tx::EscrowFinish(tx)
    }
}

/// [`LedgerStateFix`](https://xrpl.org/docs/references/protocol/transactions/types/ledgerstatefix) transaction.
///
/// Required: `LedgerFixType`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LedgerStateFix {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `LedgerFixType`: a number. Required.
    #[serde(rename = "LedgerFixType")]
    pub ledger_fix_type: u16,
    /// The account that owns the entry (`Owner`: a classic address). Optional.
    #[serde(rename = "Owner", default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// The order book directory page holding the offer (`BookDirectory`: 256 bits, hex). Optional.
    #[serde(
        rename = "BookDirectory",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub book_directory: Option<Hash256>,
}

impl LedgerStateFix {
    /// `"LedgerStateFix"`.
    pub const TRANSACTION_TYPE: &'static str = "LedgerStateFix";

    /// A `LedgerStateFix` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, ledger_fix_type: u16) -> Self {
        Self {
            common: Common::new(account),
            ledger_fix_type,
            owner: None,
            book_directory: None,
        }
    }

    /// With `Owner` set (see [`Self::owner`]).
    #[must_use]
    pub fn with_owner(mut self, value: impl Into<String>) -> Self {
        self.owner = Some(value.into());
        self
    }

    /// With `BookDirectory` set (see [`Self::book_directory`]).
    #[must_use]
    pub fn with_book_directory(mut self, value: impl Into<Hash256>) -> Self {
        self.book_directory = Some(value.into());
        self
    }
}

impl TxModel for LedgerStateFix {
    const TRANSACTION_TYPE: &'static str = "LedgerStateFix";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<LedgerStateFix> for Tx {
    fn from(tx: LedgerStateFix) -> Self {
        Tx::LedgerStateFix(tx)
    }
}

/// [`LoanBrokerCoverClawback`](https://xrpl.org/docs/references/protocol/transactions/types/loanbrokercoverclawback) transaction.
///
/// Required: none beyond the common fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LoanBrokerCoverClawback {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `LoanBrokerID`: 256 bits, hex. Optional.
    #[serde(
        rename = "LoanBrokerID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub loan_broker_id: Option<Hash256>,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "Amount", default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
}

impl LoanBrokerCoverClawback {
    /// `"LoanBrokerCoverClawback"`.
    pub const TRANSACTION_TYPE: &'static str = "LoanBrokerCoverClawback";

    /// A `LoanBrokerCoverClawback` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: Common::new(account),
            loan_broker_id: None,
            amount: None,
        }
    }

    /// With `LoanBrokerID` set (see [`Self::loan_broker_id`]).
    #[must_use]
    pub fn with_loan_broker_id(mut self, value: impl Into<Hash256>) -> Self {
        self.loan_broker_id = Some(value.into());
        self
    }

    /// With `Amount` set (see [`Self::amount`]).
    #[must_use]
    pub fn with_amount(mut self, value: impl Into<Amount>) -> Self {
        self.amount = Some(value.into());
        self
    }
}

impl TxModel for LoanBrokerCoverClawback {
    const TRANSACTION_TYPE: &'static str = "LoanBrokerCoverClawback";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<LoanBrokerCoverClawback> for Tx {
    fn from(tx: LoanBrokerCoverClawback) -> Self {
        Tx::LoanBrokerCoverClawback(tx)
    }
}

/// [`LoanBrokerCoverDeposit`](https://xrpl.org/docs/references/protocol/transactions/types/loanbrokercoverdeposit) transaction.
///
/// Required: `LoanBrokerID`, `Amount`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LoanBrokerCoverDeposit {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `LoanBrokerID`: 256 bits, hex. Required.
    #[serde(rename = "LoanBrokerID")]
    pub loan_broker_id: Hash256,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
}

impl LoanBrokerCoverDeposit {
    /// `"LoanBrokerCoverDeposit"`.
    pub const TRANSACTION_TYPE: &'static str = "LoanBrokerCoverDeposit";

    /// A `LoanBrokerCoverDeposit` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, loan_broker_id: Hash256, amount: Amount) -> Self {
        Self {
            common: Common::new(account),
            loan_broker_id,
            amount,
        }
    }
}

impl TxModel for LoanBrokerCoverDeposit {
    const TRANSACTION_TYPE: &'static str = "LoanBrokerCoverDeposit";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<LoanBrokerCoverDeposit> for Tx {
    fn from(tx: LoanBrokerCoverDeposit) -> Self {
        Tx::LoanBrokerCoverDeposit(tx)
    }
}

/// [`LoanBrokerCoverWithdraw`](https://xrpl.org/docs/references/protocol/transactions/types/loanbrokercoverwithdraw) transaction.
///
/// Required: `LoanBrokerID`, `Amount`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LoanBrokerCoverWithdraw {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `LoanBrokerID`: 256 bits, hex. Required.
    #[serde(rename = "LoanBrokerID")]
    pub loan_broker_id: Hash256,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// The account that receives it (`Destination`: a classic address). Optional.
    #[serde(
        rename = "Destination",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination: Option<String>,
    /// A tag the destination uses to tell its senders apart (`DestinationTag`: a number). Optional.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
    /// Credentials to present (`CredentialIDs`: a list of 256-bit IDs). Optional.
    #[serde(
        rename = "CredentialIDs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub credential_ids: Option<Vec<Hash256>>,
}

impl LoanBrokerCoverWithdraw {
    /// `"LoanBrokerCoverWithdraw"`.
    pub const TRANSACTION_TYPE: &'static str = "LoanBrokerCoverWithdraw";

    /// A `LoanBrokerCoverWithdraw` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, loan_broker_id: Hash256, amount: Amount) -> Self {
        Self {
            common: Common::new(account),
            loan_broker_id,
            amount,
            destination: None,
            destination_tag: None,
            credential_ids: None,
        }
    }

    /// With `Destination` set (see [`Self::destination`]).
    #[must_use]
    pub fn with_destination(mut self, value: impl Into<String>) -> Self {
        self.destination = Some(value.into());
        self
    }

    /// With `DestinationTag` set (see [`Self::destination_tag`]).
    #[must_use]
    pub fn with_destination_tag(mut self, value: impl Into<u32>) -> Self {
        self.destination_tag = Some(value.into());
        self
    }

    /// With `CredentialIDs` set (see [`Self::credential_ids`]).
    #[must_use]
    pub fn with_credential_ids(mut self, value: impl Into<Vec<Hash256>>) -> Self {
        self.credential_ids = Some(value.into());
        self
    }
}

impl TxModel for LoanBrokerCoverWithdraw {
    const TRANSACTION_TYPE: &'static str = "LoanBrokerCoverWithdraw";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<LoanBrokerCoverWithdraw> for Tx {
    fn from(tx: LoanBrokerCoverWithdraw) -> Self {
        Tx::LoanBrokerCoverWithdraw(tx)
    }
}

/// [`LoanBrokerDelete`](https://xrpl.org/docs/references/protocol/transactions/types/loanbrokerdelete) transaction.
///
/// Required: `LoanBrokerID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LoanBrokerDelete {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `LoanBrokerID`: 256 bits, hex. Required.
    #[serde(rename = "LoanBrokerID")]
    pub loan_broker_id: Hash256,
}

impl LoanBrokerDelete {
    /// `"LoanBrokerDelete"`.
    pub const TRANSACTION_TYPE: &'static str = "LoanBrokerDelete";

    /// A `LoanBrokerDelete` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, loan_broker_id: Hash256) -> Self {
        Self {
            common: Common::new(account),
            loan_broker_id,
        }
    }
}

impl TxModel for LoanBrokerDelete {
    const TRANSACTION_TYPE: &'static str = "LoanBrokerDelete";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<LoanBrokerDelete> for Tx {
    fn from(tx: LoanBrokerDelete) -> Self {
        Tx::LoanBrokerDelete(tx)
    }
}

/// [`LoanBrokerSet`](https://xrpl.org/docs/references/protocol/transactions/types/loanbrokerset) transaction.
///
/// Required: `VaultID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LoanBrokerSet {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `VaultID`: 256 bits, hex. Required.
    #[serde(rename = "VaultID")]
    pub vault_id: Hash256,
    /// `LoanBrokerID`: 256 bits, hex. Optional.
    #[serde(
        rename = "LoanBrokerID",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub loan_broker_id: Option<Hash256>,
    /// Data, hex (`Data`: bytes, hex). Optional.
    #[serde(rename = "Data", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// `ManagementFeeRate`: a number. Optional.
    #[serde(
        rename = "ManagementFeeRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub management_fee_rate: Option<u16>,
    /// `DebtMaximum`: a decimal number, as a string. Optional.
    #[serde(
        rename = "DebtMaximum",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub debt_maximum: Option<String>,
    /// `CoverRateMinimum`: a number. Optional.
    #[serde(
        rename = "CoverRateMinimum",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cover_rate_minimum: Option<u32>,
    /// `CoverRateLiquidation`: a number. Optional.
    #[serde(
        rename = "CoverRateLiquidation",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cover_rate_liquidation: Option<u32>,
}

impl LoanBrokerSet {
    /// `"LoanBrokerSet"`.
    pub const TRANSACTION_TYPE: &'static str = "LoanBrokerSet";

    /// A `LoanBrokerSet` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, vault_id: Hash256) -> Self {
        Self {
            common: Common::new(account),
            vault_id,
            loan_broker_id: None,
            data: None,
            management_fee_rate: None,
            debt_maximum: None,
            cover_rate_minimum: None,
            cover_rate_liquidation: None,
        }
    }

    /// With `LoanBrokerID` set (see [`Self::loan_broker_id`]).
    #[must_use]
    pub fn with_loan_broker_id(mut self, value: impl Into<Hash256>) -> Self {
        self.loan_broker_id = Some(value.into());
        self
    }

    /// With `Data` set (see [`Self::data`]).
    #[must_use]
    pub fn with_data(mut self, value: impl Into<String>) -> Self {
        self.data = Some(value.into());
        self
    }

    /// With `ManagementFeeRate` set (see [`Self::management_fee_rate`]).
    #[must_use]
    pub fn with_management_fee_rate(mut self, value: impl Into<u16>) -> Self {
        self.management_fee_rate = Some(value.into());
        self
    }

    /// With `DebtMaximum` set (see [`Self::debt_maximum`]).
    #[must_use]
    pub fn with_debt_maximum(mut self, value: impl Into<String>) -> Self {
        self.debt_maximum = Some(value.into());
        self
    }

    /// With `CoverRateMinimum` set (see [`Self::cover_rate_minimum`]).
    #[must_use]
    pub fn with_cover_rate_minimum(mut self, value: impl Into<u32>) -> Self {
        self.cover_rate_minimum = Some(value.into());
        self
    }

    /// With `CoverRateLiquidation` set (see [`Self::cover_rate_liquidation`]).
    #[must_use]
    pub fn with_cover_rate_liquidation(mut self, value: impl Into<u32>) -> Self {
        self.cover_rate_liquidation = Some(value.into());
        self
    }
}

impl TxModel for LoanBrokerSet {
    const TRANSACTION_TYPE: &'static str = "LoanBrokerSet";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<LoanBrokerSet> for Tx {
    fn from(tx: LoanBrokerSet) -> Self {
        Tx::LoanBrokerSet(tx)
    }
}

/// [`LoanDelete`](https://xrpl.org/docs/references/protocol/transactions/types/loandelete) transaction.
///
/// Required: `LoanID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LoanDelete {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `LoanID`: 256 bits, hex. Required.
    #[serde(rename = "LoanID")]
    pub loan_id: Hash256,
}

impl LoanDelete {
    /// `"LoanDelete"`.
    pub const TRANSACTION_TYPE: &'static str = "LoanDelete";

    /// A `LoanDelete` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, loan_id: Hash256) -> Self {
        Self {
            common: Common::new(account),
            loan_id,
        }
    }
}

impl TxModel for LoanDelete {
    const TRANSACTION_TYPE: &'static str = "LoanDelete";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<LoanDelete> for Tx {
    fn from(tx: LoanDelete) -> Self {
        Tx::LoanDelete(tx)
    }
}

flags_type!(
    /// The flags of a [`LoanManage`] (`Flags`).
    LoanManageFlags {
        /// `tfLoanDefault`.
        LOAN_DEFAULT = 0x00010000,
        /// `tfLoanImpair`.
        LOAN_IMPAIR = 0x00020000,
        /// `tfLoanUnimpair`.
        LOAN_UNIMPAIR = 0x00040000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`LoanManage`](https://xrpl.org/docs/references/protocol/transactions/types/loanmanage) transaction.
///
/// Required: `LoanID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LoanManage {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `LoanID`: 256 bits, hex. Required.
    #[serde(rename = "LoanID")]
    pub loan_id: Hash256,
}

impl LoanManage {
    /// `"LoanManage"`.
    pub const TRANSACTION_TYPE: &'static str = "LoanManage";

    /// A `LoanManage` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, loan_id: Hash256) -> Self {
        Self {
            common: Common::new(account),
            loan_id,
        }
    }
}

impl TxModel for LoanManage {
    const TRANSACTION_TYPE: &'static str = "LoanManage";
    type Flags = LoanManageFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<LoanManage> for Tx {
    fn from(tx: LoanManage) -> Self {
        Tx::LoanManage(tx)
    }
}

flags_type!(
    /// The flags of a [`LoanPay`] (`Flags`).
    LoanPayFlags {
        /// `tfLoanOverpayment`.
        LOAN_OVERPAYMENT = 0x00010000,
        /// `tfLoanFullPayment`.
        LOAN_FULL_PAYMENT = 0x00020000,
        /// `tfLoanLatePayment`.
        LOAN_LATE_PAYMENT = 0x00040000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`LoanPay`](https://xrpl.org/docs/references/protocol/transactions/types/loanpay) transaction.
///
/// Required: `LoanID`, `Amount`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LoanPay {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `LoanID`: 256 bits, hex. Required.
    #[serde(rename = "LoanID")]
    pub loan_id: Hash256,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
}

impl LoanPay {
    /// `"LoanPay"`.
    pub const TRANSACTION_TYPE: &'static str = "LoanPay";

    /// A `LoanPay` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, loan_id: Hash256, amount: Amount) -> Self {
        Self {
            common: Common::new(account),
            loan_id,
            amount,
        }
    }
}

impl TxModel for LoanPay {
    const TRANSACTION_TYPE: &'static str = "LoanPay";
    type Flags = LoanPayFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<LoanPay> for Tx {
    fn from(tx: LoanPay) -> Self {
        Tx::LoanPay(tx)
    }
}

flags_type!(
    /// The flags of a [`LoanSet`] (`Flags`).
    LoanSetFlags {
        /// `tfLoanOverpayment`.
        LOAN_OVERPAYMENT = 0x00010000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`LoanSet`](https://xrpl.org/docs/references/protocol/transactions/types/loanset) transaction.
///
/// Required: `LoanBrokerID`, `PrincipalRequested`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LoanSet {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `LoanBrokerID`: 256 bits, hex. Required.
    #[serde(rename = "LoanBrokerID")]
    pub loan_broker_id: Hash256,
    /// Data, hex (`Data`: bytes, hex). Optional.
    #[serde(rename = "Data", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// `Counterparty`: a classic address. Optional.
    #[serde(
        rename = "Counterparty",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub counterparty: Option<String>,
    /// `CounterpartySignature`: an object. Optional.
    #[serde(
        rename = "CounterpartySignature",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub counterparty_signature: Option<Value>,
    /// `LoanOriginationFee`: a decimal number, as a string. Optional.
    #[serde(
        rename = "LoanOriginationFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub loan_origination_fee: Option<String>,
    /// `LoanServiceFee`: a decimal number, as a string. Optional.
    #[serde(
        rename = "LoanServiceFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub loan_service_fee: Option<String>,
    /// `LatePaymentFee`: a decimal number, as a string. Optional.
    #[serde(
        rename = "LatePaymentFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub late_payment_fee: Option<String>,
    /// `ClosePaymentFee`: a decimal number, as a string. Optional.
    #[serde(
        rename = "ClosePaymentFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub close_payment_fee: Option<String>,
    /// `OverpaymentFee`: a number. Optional.
    #[serde(
        rename = "OverpaymentFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub overpayment_fee: Option<u32>,
    /// `InterestRate`: a number. Optional.
    #[serde(
        rename = "InterestRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub interest_rate: Option<u32>,
    /// `LateInterestRate`: a number. Optional.
    #[serde(
        rename = "LateInterestRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub late_interest_rate: Option<u32>,
    /// `CloseInterestRate`: a number. Optional.
    #[serde(
        rename = "CloseInterestRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub close_interest_rate: Option<u32>,
    /// `OverpaymentInterestRate`: a number. Optional.
    #[serde(
        rename = "OverpaymentInterestRate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub overpayment_interest_rate: Option<u32>,
    /// `PrincipalRequested`: a decimal number, as a string. Required.
    #[serde(rename = "PrincipalRequested")]
    pub principal_requested: String,
    /// `PaymentTotal`: a number. Optional.
    #[serde(
        rename = "PaymentTotal",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub payment_total: Option<u32>,
    /// `PaymentInterval`: a number. Optional.
    #[serde(
        rename = "PaymentInterval",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub payment_interval: Option<u32>,
    /// `GracePeriod`: a number. Optional.
    #[serde(
        rename = "GracePeriod",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub grace_period: Option<u32>,
}

impl LoanSet {
    /// `"LoanSet"`.
    pub const TRANSACTION_TYPE: &'static str = "LoanSet";

    /// A `LoanSet` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        loan_broker_id: Hash256,
        principal_requested: impl Into<String>,
    ) -> Self {
        Self {
            common: Common::new(account),
            loan_broker_id,
            data: None,
            counterparty: None,
            counterparty_signature: None,
            loan_origination_fee: None,
            loan_service_fee: None,
            late_payment_fee: None,
            close_payment_fee: None,
            overpayment_fee: None,
            interest_rate: None,
            late_interest_rate: None,
            close_interest_rate: None,
            overpayment_interest_rate: None,
            principal_requested: principal_requested.into(),
            payment_total: None,
            payment_interval: None,
            grace_period: None,
        }
    }

    /// With `Data` set (see [`Self::data`]).
    #[must_use]
    pub fn with_data(mut self, value: impl Into<String>) -> Self {
        self.data = Some(value.into());
        self
    }

    /// With `Counterparty` set (see [`Self::counterparty`]).
    #[must_use]
    pub fn with_counterparty(mut self, value: impl Into<String>) -> Self {
        self.counterparty = Some(value.into());
        self
    }

    /// With `CounterpartySignature` set (see [`Self::counterparty_signature`]).
    #[must_use]
    pub fn with_counterparty_signature(mut self, value: impl Into<Value>) -> Self {
        self.counterparty_signature = Some(value.into());
        self
    }

    /// With `LoanOriginationFee` set (see [`Self::loan_origination_fee`]).
    #[must_use]
    pub fn with_loan_origination_fee(mut self, value: impl Into<String>) -> Self {
        self.loan_origination_fee = Some(value.into());
        self
    }

    /// With `LoanServiceFee` set (see [`Self::loan_service_fee`]).
    #[must_use]
    pub fn with_loan_service_fee(mut self, value: impl Into<String>) -> Self {
        self.loan_service_fee = Some(value.into());
        self
    }

    /// With `LatePaymentFee` set (see [`Self::late_payment_fee`]).
    #[must_use]
    pub fn with_late_payment_fee(mut self, value: impl Into<String>) -> Self {
        self.late_payment_fee = Some(value.into());
        self
    }

    /// With `ClosePaymentFee` set (see [`Self::close_payment_fee`]).
    #[must_use]
    pub fn with_close_payment_fee(mut self, value: impl Into<String>) -> Self {
        self.close_payment_fee = Some(value.into());
        self
    }

    /// With `OverpaymentFee` set (see [`Self::overpayment_fee`]).
    #[must_use]
    pub fn with_overpayment_fee(mut self, value: impl Into<u32>) -> Self {
        self.overpayment_fee = Some(value.into());
        self
    }

    /// With `InterestRate` set (see [`Self::interest_rate`]).
    #[must_use]
    pub fn with_interest_rate(mut self, value: impl Into<u32>) -> Self {
        self.interest_rate = Some(value.into());
        self
    }

    /// With `LateInterestRate` set (see [`Self::late_interest_rate`]).
    #[must_use]
    pub fn with_late_interest_rate(mut self, value: impl Into<u32>) -> Self {
        self.late_interest_rate = Some(value.into());
        self
    }

    /// With `CloseInterestRate` set (see [`Self::close_interest_rate`]).
    #[must_use]
    pub fn with_close_interest_rate(mut self, value: impl Into<u32>) -> Self {
        self.close_interest_rate = Some(value.into());
        self
    }

    /// With `OverpaymentInterestRate` set (see [`Self::overpayment_interest_rate`]).
    #[must_use]
    pub fn with_overpayment_interest_rate(mut self, value: impl Into<u32>) -> Self {
        self.overpayment_interest_rate = Some(value.into());
        self
    }

    /// With `PaymentTotal` set (see [`Self::payment_total`]).
    #[must_use]
    pub fn with_payment_total(mut self, value: impl Into<u32>) -> Self {
        self.payment_total = Some(value.into());
        self
    }

    /// With `PaymentInterval` set (see [`Self::payment_interval`]).
    #[must_use]
    pub fn with_payment_interval(mut self, value: impl Into<u32>) -> Self {
        self.payment_interval = Some(value.into());
        self
    }

    /// With `GracePeriod` set (see [`Self::grace_period`]).
    #[must_use]
    pub fn with_grace_period(mut self, value: impl Into<u32>) -> Self {
        self.grace_period = Some(value.into());
        self
    }
}

impl TxModel for LoanSet {
    const TRANSACTION_TYPE: &'static str = "LoanSet";
    type Flags = LoanSetFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<LoanSet> for Tx {
    fn from(tx: LoanSet) -> Self {
        Tx::LoanSet(tx)
    }
}

flags_type!(
    /// The flags of a [`MPTokenAuthorize`] (`Flags`).
    MPTokenAuthorizeFlags {
        /// `tfMPTUnauthorize`.
        MPT_UNAUTHORIZE = 0x00000001,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`MPTokenAuthorize`](https://xrpl.org/docs/references/protocol/transactions/types/mptokenauthorize) transaction.
///
/// Required: `MPTokenIssuanceID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MPTokenAuthorize {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The MPT issuance's ID (`MPTokenIssuanceID`: 192 bits, hex). Required.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mp_token_issuance_id: Hash192,
    /// The holder (`Holder`: a classic address). Optional.
    #[serde(rename = "Holder", default, skip_serializing_if = "Option::is_none")]
    pub holder: Option<String>,
}

impl MPTokenAuthorize {
    /// `"MPTokenAuthorize"`.
    pub const TRANSACTION_TYPE: &'static str = "MPTokenAuthorize";

    /// A `MPTokenAuthorize` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, mp_token_issuance_id: Hash192) -> Self {
        Self {
            common: Common::new(account),
            mp_token_issuance_id,
            holder: None,
        }
    }

    /// With `Holder` set (see [`Self::holder`]).
    #[must_use]
    pub fn with_holder(mut self, value: impl Into<String>) -> Self {
        self.holder = Some(value.into());
        self
    }
}

impl TxModel for MPTokenAuthorize {
    const TRANSACTION_TYPE: &'static str = "MPTokenAuthorize";
    type Flags = MPTokenAuthorizeFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<MPTokenAuthorize> for Tx {
    fn from(tx: MPTokenAuthorize) -> Self {
        Tx::MPTokenAuthorize(tx)
    }
}

flags_type!(
    /// The flags of a [`MPTokenIssuanceCreate`] (`Flags`).
    MPTokenIssuanceCreateFlags {
        /// `tfMPTCanLock`.
        MPT_CAN_LOCK = 0x00000002,
        /// `tfMPTRequireAuth`.
        MPT_REQUIRE_AUTH = 0x00000004,
        /// `tfMPTCanEscrow`.
        MPT_CAN_ESCROW = 0x00000008,
        /// `tfMPTCanTrade`.
        MPT_CAN_TRADE = 0x00000010,
        /// `tfMPTCanTransfer`.
        MPT_CAN_TRANSFER = 0x00000020,
        /// `tfMPTCanClawback`.
        MPT_CAN_CLAWBACK = 0x00000040,
        /// `tfMPTCanHoldConfidentialBalance`.
        MPT_CAN_HOLD_CONFIDENTIAL_BALANCE = 0x00000080,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`MPTokenIssuanceCreate`](https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuancecreate) transaction.
///
/// Required: none beyond the common fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MPTokenIssuanceCreate {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// Decimal places: an MPT amount of 12345 at scale 2 is 123.45 (`AssetScale`: a number). Optional.
    #[serde(
        rename = "AssetScale",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub asset_scale: Option<u8>,
    /// The fee on secondary sales, in 1/100,000 (5000 = 5%); for tokens, see `TransferRate` (`TransferFee`: a number). Optional.
    #[serde(
        rename = "TransferFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transfer_fee: Option<u16>,
    /// The most of the MPT that can ever exist, in its smallest unit (`MaximumAmount`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts)). Optional.
    #[serde(
        rename = "MaximumAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub maximum_amount: Option<String>,
    /// Metadata, hex (XLS-89 JSON) (`MPTokenMetadata`: bytes, hex). Optional.
    #[serde(
        rename = "MPTokenMetadata",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mp_token_metadata: Option<String>,
    /// A permissioned domain's ID (`DomainID`: 256 bits, hex). Optional.
    #[serde(rename = "DomainID", default, skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<Hash256>,
    /// `ImmutableFlags`: a number. Optional.
    #[serde(
        rename = "ImmutableFlags",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub immutable_flags: Option<u32>,
}

impl MPTokenIssuanceCreate {
    /// `"MPTokenIssuanceCreate"`.
    pub const TRANSACTION_TYPE: &'static str = "MPTokenIssuanceCreate";

    /// A `MPTokenIssuanceCreate` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: Common::new(account),
            asset_scale: None,
            transfer_fee: None,
            maximum_amount: None,
            mp_token_metadata: None,
            domain_id: None,
            immutable_flags: None,
        }
    }

    /// With `AssetScale` set (see [`Self::asset_scale`]).
    #[must_use]
    pub fn with_asset_scale(mut self, value: impl Into<u8>) -> Self {
        self.asset_scale = Some(value.into());
        self
    }

    /// With `TransferFee` set (see [`Self::transfer_fee`]).
    #[must_use]
    pub fn with_transfer_fee(mut self, value: impl Into<u16>) -> Self {
        self.transfer_fee = Some(value.into());
        self
    }

    /// With `MaximumAmount` set (see [`Self::maximum_amount`]).
    #[must_use]
    pub fn with_maximum_amount(mut self, value: impl Into<String>) -> Self {
        self.maximum_amount = Some(value.into());
        self
    }

    /// With `MPTokenMetadata` set (see [`Self::mp_token_metadata`]).
    #[must_use]
    pub fn with_mp_token_metadata(mut self, value: impl Into<String>) -> Self {
        self.mp_token_metadata = Some(value.into());
        self
    }

    /// With `DomainID` set (see [`Self::domain_id`]).
    #[must_use]
    pub fn with_domain_id(mut self, value: impl Into<Hash256>) -> Self {
        self.domain_id = Some(value.into());
        self
    }

    /// With `ImmutableFlags` set (see [`Self::immutable_flags`]).
    #[must_use]
    pub fn with_immutable_flags(mut self, value: impl Into<u32>) -> Self {
        self.immutable_flags = Some(value.into());
        self
    }
}

impl TxModel for MPTokenIssuanceCreate {
    const TRANSACTION_TYPE: &'static str = "MPTokenIssuanceCreate";
    type Flags = MPTokenIssuanceCreateFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<MPTokenIssuanceCreate> for Tx {
    fn from(tx: MPTokenIssuanceCreate) -> Self {
        Tx::MPTokenIssuanceCreate(tx)
    }
}

/// [`MPTokenIssuanceDestroy`](https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuancedestroy) transaction.
///
/// Required: `MPTokenIssuanceID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MPTokenIssuanceDestroy {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The MPT issuance's ID (`MPTokenIssuanceID`: 192 bits, hex). Required.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mp_token_issuance_id: Hash192,
}

impl MPTokenIssuanceDestroy {
    /// `"MPTokenIssuanceDestroy"`.
    pub const TRANSACTION_TYPE: &'static str = "MPTokenIssuanceDestroy";

    /// A `MPTokenIssuanceDestroy` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, mp_token_issuance_id: Hash192) -> Self {
        Self {
            common: Common::new(account),
            mp_token_issuance_id,
        }
    }
}

impl TxModel for MPTokenIssuanceDestroy {
    const TRANSACTION_TYPE: &'static str = "MPTokenIssuanceDestroy";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<MPTokenIssuanceDestroy> for Tx {
    fn from(tx: MPTokenIssuanceDestroy) -> Self {
        Tx::MPTokenIssuanceDestroy(tx)
    }
}

flags_type!(
    /// The flags of a [`MPTokenIssuanceSet`] (`Flags`).
    MPTokenIssuanceSetFlags {
        /// `tfMPTLock`.
        MPT_LOCK = 0x00000001,
        /// `tfMPTUnlock`.
        MPT_UNLOCK = 0x00000002,
        /// `tfMPTSetCanLock`.
        MPT_SET_CAN_LOCK = 0x00000004,
        /// `tfMPTSetRequireAuth`.
        MPT_SET_REQUIRE_AUTH = 0x00000008,
        /// `tfMPTSetCanEscrow`.
        MPT_SET_CAN_ESCROW = 0x00000010,
        /// `tfMPTSetCanTrade`.
        MPT_SET_CAN_TRADE = 0x00000020,
        /// `tfMPTSetCanTransfer`.
        MPT_SET_CAN_TRANSFER = 0x00000040,
        /// `tfMPTSetCanClawback`.
        MPT_SET_CAN_CLAWBACK = 0x00000080,
        /// `tfMPTSetCanHoldConfidentialBalance`.
        MPT_SET_CAN_HOLD_CONFIDENTIAL_BALANCE = 0x00000100,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`MPTokenIssuanceSet`](https://xrpl.org/docs/references/protocol/transactions/types/mptokenissuanceset) transaction.
///
/// Required: `MPTokenIssuanceID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MPTokenIssuanceSet {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The MPT issuance's ID (`MPTokenIssuanceID`: 192 bits, hex). Required.
    #[serde(rename = "MPTokenIssuanceID")]
    pub mp_token_issuance_id: Hash192,
    /// The holder (`Holder`: a classic address). Optional.
    #[serde(rename = "Holder", default, skip_serializing_if = "Option::is_none")]
    pub holder: Option<String>,
    /// A permissioned domain's ID (`DomainID`: 256 bits, hex). Optional.
    #[serde(rename = "DomainID", default, skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<Hash256>,
    /// Metadata, hex (XLS-89 JSON) (`MPTokenMetadata`: bytes, hex). Optional.
    #[serde(
        rename = "MPTokenMetadata",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mp_token_metadata: Option<String>,
    /// The fee on secondary sales, in 1/100,000 (5000 = 5%); for tokens, see `TransferRate` (`TransferFee`: a number). Optional.
    #[serde(
        rename = "TransferFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transfer_fee: Option<u16>,
    /// `ImmutableFlags`: a number. Optional.
    #[serde(
        rename = "ImmutableFlags",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub immutable_flags: Option<u32>,
    /// `IssuerEncryptionKey`: bytes, hex. Optional.
    #[serde(
        rename = "IssuerEncryptionKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub issuer_encryption_key: Option<String>,
    /// `AuditorEncryptionKey`: bytes, hex. Optional.
    #[serde(
        rename = "AuditorEncryptionKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub auditor_encryption_key: Option<String>,
}

impl MPTokenIssuanceSet {
    /// `"MPTokenIssuanceSet"`.
    pub const TRANSACTION_TYPE: &'static str = "MPTokenIssuanceSet";

    /// A `MPTokenIssuanceSet` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, mp_token_issuance_id: Hash192) -> Self {
        Self {
            common: Common::new(account),
            mp_token_issuance_id,
            holder: None,
            domain_id: None,
            mp_token_metadata: None,
            transfer_fee: None,
            immutable_flags: None,
            issuer_encryption_key: None,
            auditor_encryption_key: None,
        }
    }

    /// With `Holder` set (see [`Self::holder`]).
    #[must_use]
    pub fn with_holder(mut self, value: impl Into<String>) -> Self {
        self.holder = Some(value.into());
        self
    }

    /// With `DomainID` set (see [`Self::domain_id`]).
    #[must_use]
    pub fn with_domain_id(mut self, value: impl Into<Hash256>) -> Self {
        self.domain_id = Some(value.into());
        self
    }

    /// With `MPTokenMetadata` set (see [`Self::mp_token_metadata`]).
    #[must_use]
    pub fn with_mp_token_metadata(mut self, value: impl Into<String>) -> Self {
        self.mp_token_metadata = Some(value.into());
        self
    }

    /// With `TransferFee` set (see [`Self::transfer_fee`]).
    #[must_use]
    pub fn with_transfer_fee(mut self, value: impl Into<u16>) -> Self {
        self.transfer_fee = Some(value.into());
        self
    }

    /// With `ImmutableFlags` set (see [`Self::immutable_flags`]).
    #[must_use]
    pub fn with_immutable_flags(mut self, value: impl Into<u32>) -> Self {
        self.immutable_flags = Some(value.into());
        self
    }

    /// With `IssuerEncryptionKey` set (see [`Self::issuer_encryption_key`]).
    #[must_use]
    pub fn with_issuer_encryption_key(mut self, value: impl Into<String>) -> Self {
        self.issuer_encryption_key = Some(value.into());
        self
    }

    /// With `AuditorEncryptionKey` set (see [`Self::auditor_encryption_key`]).
    #[must_use]
    pub fn with_auditor_encryption_key(mut self, value: impl Into<String>) -> Self {
        self.auditor_encryption_key = Some(value.into());
        self
    }
}

impl TxModel for MPTokenIssuanceSet {
    const TRANSACTION_TYPE: &'static str = "MPTokenIssuanceSet";
    type Flags = MPTokenIssuanceSetFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<MPTokenIssuanceSet> for Tx {
    fn from(tx: MPTokenIssuanceSet) -> Self {
        Tx::MPTokenIssuanceSet(tx)
    }
}

/// [`NFTokenAcceptOffer`](https://xrpl.org/docs/references/protocol/transactions/types/nftokenacceptoffer) transaction.
///
/// Required: none beyond the common fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NFTokenAcceptOffer {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The buy offer to accept (`NFTokenBuyOffer`: 256 bits, hex). Optional.
    #[serde(
        rename = "NFTokenBuyOffer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub nf_token_buy_offer: Option<Hash256>,
    /// The sell offer to accept (`NFTokenSellOffer`: 256 bits, hex). Optional.
    #[serde(
        rename = "NFTokenSellOffer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub nf_token_sell_offer: Option<Hash256>,
    /// A broker's cut when matching a buy and a sell offer (`NFTokenBrokerFee`: XRP in drops, a token, or an MPT). Optional.
    #[serde(
        rename = "NFTokenBrokerFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub nf_token_broker_fee: Option<Amount>,
}

impl NFTokenAcceptOffer {
    /// `"NFTokenAcceptOffer"`.
    pub const TRANSACTION_TYPE: &'static str = "NFTokenAcceptOffer";

    /// A `NFTokenAcceptOffer` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: Common::new(account),
            nf_token_buy_offer: None,
            nf_token_sell_offer: None,
            nf_token_broker_fee: None,
        }
    }

    /// With `NFTokenBuyOffer` set (see [`Self::nf_token_buy_offer`]).
    #[must_use]
    pub fn with_nf_token_buy_offer(mut self, value: impl Into<Hash256>) -> Self {
        self.nf_token_buy_offer = Some(value.into());
        self
    }

    /// With `NFTokenSellOffer` set (see [`Self::nf_token_sell_offer`]).
    #[must_use]
    pub fn with_nf_token_sell_offer(mut self, value: impl Into<Hash256>) -> Self {
        self.nf_token_sell_offer = Some(value.into());
        self
    }

    /// With `NFTokenBrokerFee` set (see [`Self::nf_token_broker_fee`]).
    #[must_use]
    pub fn with_nf_token_broker_fee(mut self, value: impl Into<Amount>) -> Self {
        self.nf_token_broker_fee = Some(value.into());
        self
    }
}

impl TxModel for NFTokenAcceptOffer {
    const TRANSACTION_TYPE: &'static str = "NFTokenAcceptOffer";
    type Flags = UniversalFlags;
    fn validate(&self) -> Result<(), CoreError> {
        super::validate::nf_token_accept_offer(self)
    }
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<NFTokenAcceptOffer> for Tx {
    fn from(tx: NFTokenAcceptOffer) -> Self {
        Tx::NFTokenAcceptOffer(tx)
    }
}

/// [`NFTokenBurn`](https://xrpl.org/docs/references/protocol/transactions/types/nftokenburn) transaction.
///
/// Required: `NFTokenID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NFTokenBurn {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// An NFT's ID (`NFTokenID`: 256 bits, hex). Required.
    #[serde(rename = "NFTokenID")]
    pub nf_token_id: Hash256,
    /// The account that owns the entry (`Owner`: a classic address). Optional.
    #[serde(rename = "Owner", default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
}

impl NFTokenBurn {
    /// `"NFTokenBurn"`.
    pub const TRANSACTION_TYPE: &'static str = "NFTokenBurn";

    /// A `NFTokenBurn` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, nf_token_id: Hash256) -> Self {
        Self {
            common: Common::new(account),
            nf_token_id,
            owner: None,
        }
    }

    /// With `Owner` set (see [`Self::owner`]).
    #[must_use]
    pub fn with_owner(mut self, value: impl Into<String>) -> Self {
        self.owner = Some(value.into());
        self
    }
}

impl TxModel for NFTokenBurn {
    const TRANSACTION_TYPE: &'static str = "NFTokenBurn";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<NFTokenBurn> for Tx {
    fn from(tx: NFTokenBurn) -> Self {
        Tx::NFTokenBurn(tx)
    }
}

/// [`NFTokenCancelOffer`](https://xrpl.org/docs/references/protocol/transactions/types/nftokencanceloffer) transaction.
///
/// Required: `NFTokenOffers`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NFTokenCancelOffer {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The offers to cancel (`NFTokenOffers`: a list of 256-bit IDs). Required.
    #[serde(rename = "NFTokenOffers")]
    pub nf_token_offers: Vec<Hash256>,
}

impl NFTokenCancelOffer {
    /// `"NFTokenCancelOffer"`.
    pub const TRANSACTION_TYPE: &'static str = "NFTokenCancelOffer";

    /// A `NFTokenCancelOffer` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, nf_token_offers: Vec<Hash256>) -> Self {
        Self {
            common: Common::new(account),
            nf_token_offers,
        }
    }
}

impl TxModel for NFTokenCancelOffer {
    const TRANSACTION_TYPE: &'static str = "NFTokenCancelOffer";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<NFTokenCancelOffer> for Tx {
    fn from(tx: NFTokenCancelOffer) -> Self {
        Tx::NFTokenCancelOffer(tx)
    }
}

flags_type!(
    /// The flags of a [`NFTokenCreateOffer`] (`Flags`).
    NFTokenCreateOfferFlags {
        /// `tfSellNFToken`.
        SELL_NFTOKEN = 0x00000001,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`NFTokenCreateOffer`](https://xrpl.org/docs/references/protocol/transactions/types/nftokencreateoffer) transaction.
///
/// Required: `NFTokenID`, `Amount`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NFTokenCreateOffer {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// An NFT's ID (`NFTokenID`: 256 bits, hex). Required.
    #[serde(rename = "NFTokenID")]
    pub nf_token_id: Hash256,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// The account that receives it (`Destination`: a classic address). Optional.
    #[serde(
        rename = "Destination",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination: Option<String>,
    /// The account that owns the entry (`Owner`: a classic address). Optional.
    #[serde(rename = "Owner", default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// When it expires (ledger time: seconds since 2000-01-01) (`Expiration`: a number). Optional.
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
}

impl NFTokenCreateOffer {
    /// `"NFTokenCreateOffer"`.
    pub const TRANSACTION_TYPE: &'static str = "NFTokenCreateOffer";

    /// A `NFTokenCreateOffer` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, nf_token_id: Hash256, amount: Amount) -> Self {
        Self {
            common: Common::new(account),
            nf_token_id,
            amount,
            destination: None,
            owner: None,
            expiration: None,
        }
    }

    /// With `Destination` set (see [`Self::destination`]).
    #[must_use]
    pub fn with_destination(mut self, value: impl Into<String>) -> Self {
        self.destination = Some(value.into());
        self
    }

    /// With `Owner` set (see [`Self::owner`]).
    #[must_use]
    pub fn with_owner(mut self, value: impl Into<String>) -> Self {
        self.owner = Some(value.into());
        self
    }

    /// With `Expiration` set (see [`Self::expiration`]).
    #[must_use]
    pub fn with_expiration(mut self, value: impl Into<u32>) -> Self {
        self.expiration = Some(value.into());
        self
    }
}

impl TxModel for NFTokenCreateOffer {
    const TRANSACTION_TYPE: &'static str = "NFTokenCreateOffer";
    type Flags = NFTokenCreateOfferFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<NFTokenCreateOffer> for Tx {
    fn from(tx: NFTokenCreateOffer) -> Self {
        Tx::NFTokenCreateOffer(tx)
    }
}

flags_type!(
    /// The flags of a [`NFTokenMint`] (`Flags`).
    NFTokenMintFlags {
        /// `tfBurnable`.
        BURNABLE = 0x00000001,
        /// `tfOnlyXRP`.
        ONLY_XRP = 0x00000002,
        /// `tfTransferable`.
        TRANSFERABLE = 0x00000008,
        /// `tfMutable`.
        MUTABLE = 0x00000010,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`NFTokenMint`](https://xrpl.org/docs/references/protocol/transactions/types/nftokenmint) transaction.
///
/// Required: `NFTokenTaxon`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NFTokenMint {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// A number the issuer uses to group its NFTs (`NFTokenTaxon`: a number). Required.
    #[serde(rename = "NFTokenTaxon")]
    pub nf_token_taxon: u32,
    /// The fee on secondary sales, in 1/100,000 (5000 = 5%); for tokens, see `TransferRate` (`TransferFee`: a number). Optional.
    #[serde(
        rename = "TransferFee",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub transfer_fee: Option<u16>,
    /// The issuer (`Issuer`: a classic address). Optional.
    #[serde(rename = "Issuer", default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    /// A URI, hex (`URI`: bytes, hex). Optional.
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "Amount", default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
    /// The account that receives it (`Destination`: a classic address). Optional.
    #[serde(
        rename = "Destination",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination: Option<String>,
    /// When it expires (ledger time: seconds since 2000-01-01) (`Expiration`: a number). Optional.
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
}

impl NFTokenMint {
    /// `"NFTokenMint"`.
    pub const TRANSACTION_TYPE: &'static str = "NFTokenMint";

    /// A `NFTokenMint` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, nf_token_taxon: u32) -> Self {
        Self {
            common: Common::new(account),
            nf_token_taxon,
            transfer_fee: None,
            issuer: None,
            uri: None,
            amount: None,
            destination: None,
            expiration: None,
        }
    }

    /// With `TransferFee` set (see [`Self::transfer_fee`]).
    #[must_use]
    pub fn with_transfer_fee(mut self, value: impl Into<u16>) -> Self {
        self.transfer_fee = Some(value.into());
        self
    }

    /// With `Issuer` set (see [`Self::issuer`]).
    #[must_use]
    pub fn with_issuer(mut self, value: impl Into<String>) -> Self {
        self.issuer = Some(value.into());
        self
    }

    /// With `URI` set (see [`Self::uri`]).
    #[must_use]
    pub fn with_uri(mut self, value: impl Into<String>) -> Self {
        self.uri = Some(value.into());
        self
    }

    /// With `Amount` set (see [`Self::amount`]).
    #[must_use]
    pub fn with_amount(mut self, value: impl Into<Amount>) -> Self {
        self.amount = Some(value.into());
        self
    }

    /// With `Destination` set (see [`Self::destination`]).
    #[must_use]
    pub fn with_destination(mut self, value: impl Into<String>) -> Self {
        self.destination = Some(value.into());
        self
    }

    /// With `Expiration` set (see [`Self::expiration`]).
    #[must_use]
    pub fn with_expiration(mut self, value: impl Into<u32>) -> Self {
        self.expiration = Some(value.into());
        self
    }
}

impl TxModel for NFTokenMint {
    const TRANSACTION_TYPE: &'static str = "NFTokenMint";
    type Flags = NFTokenMintFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<NFTokenMint> for Tx {
    fn from(tx: NFTokenMint) -> Self {
        Tx::NFTokenMint(tx)
    }
}

/// [`NFTokenModify`](https://xrpl.org/docs/references/protocol/transactions/types/nftokenmodify) transaction.
///
/// Required: `NFTokenID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct NFTokenModify {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// An NFT's ID (`NFTokenID`: 256 bits, hex). Required.
    #[serde(rename = "NFTokenID")]
    pub nf_token_id: Hash256,
    /// The account that owns the entry (`Owner`: a classic address). Optional.
    #[serde(rename = "Owner", default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// A URI, hex (`URI`: bytes, hex). Optional.
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

impl NFTokenModify {
    /// `"NFTokenModify"`.
    pub const TRANSACTION_TYPE: &'static str = "NFTokenModify";

    /// A `NFTokenModify` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, nf_token_id: Hash256) -> Self {
        Self {
            common: Common::new(account),
            nf_token_id,
            owner: None,
            uri: None,
        }
    }

    /// With `Owner` set (see [`Self::owner`]).
    #[must_use]
    pub fn with_owner(mut self, value: impl Into<String>) -> Self {
        self.owner = Some(value.into());
        self
    }

    /// With `URI` set (see [`Self::uri`]).
    #[must_use]
    pub fn with_uri(mut self, value: impl Into<String>) -> Self {
        self.uri = Some(value.into());
        self
    }
}

impl TxModel for NFTokenModify {
    const TRANSACTION_TYPE: &'static str = "NFTokenModify";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<NFTokenModify> for Tx {
    fn from(tx: NFTokenModify) -> Self {
        Tx::NFTokenModify(tx)
    }
}

/// [`OfferCancel`](https://xrpl.org/docs/references/protocol/transactions/types/offercancel) transaction.
///
/// Required: `OfferSequence`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OfferCancel {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The sequence of the transaction that created the entry (`OfferSequence`: a number). Required.
    #[serde(rename = "OfferSequence")]
    pub offer_sequence: u32,
}

impl OfferCancel {
    /// `"OfferCancel"`.
    pub const TRANSACTION_TYPE: &'static str = "OfferCancel";

    /// A `OfferCancel` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, offer_sequence: u32) -> Self {
        Self {
            common: Common::new(account),
            offer_sequence,
        }
    }
}

impl TxModel for OfferCancel {
    const TRANSACTION_TYPE: &'static str = "OfferCancel";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<OfferCancel> for Tx {
    fn from(tx: OfferCancel) -> Self {
        Tx::OfferCancel(tx)
    }
}

flags_type!(
    /// The flags of a [`OfferCreate`] (`Flags`).
    OfferCreateFlags {
        /// `tfPassive`.
        PASSIVE = 0x00010000,
        /// `tfImmediateOrCancel`.
        IMMEDIATE_OR_CANCEL = 0x00020000,
        /// `tfFillOrKill`.
        FILL_OR_KILL = 0x00040000,
        /// `tfSell`.
        SELL = 0x00080000,
        /// `tfHybrid`.
        HYBRID = 0x00100000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`OfferCreate`](https://xrpl.org/docs/references/protocol/transactions/types/offercreate) transaction.
///
/// Required: `TakerPays`, `TakerGets`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OfferCreate {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// What the offer wants in return (`TakerPays`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "TakerPays")]
    pub taker_pays: Amount,
    /// What the offer gives (`TakerGets`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "TakerGets")]
    pub taker_gets: Amount,
    /// When it expires (ledger time: seconds since 2000-01-01) (`Expiration`: a number). Optional.
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
    /// The sequence of the transaction that created the entry (`OfferSequence`: a number). Optional.
    #[serde(
        rename = "OfferSequence",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub offer_sequence: Option<u32>,
    /// A permissioned domain's ID (`DomainID`: 256 bits, hex). Optional.
    #[serde(rename = "DomainID", default, skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<Hash256>,
    /// What the owner holds of `TakerGets`' asset: `rippled` adds it in transaction
    /// streams and in `ledger` with `owner_funds: true`. Not a transaction field: the
    /// codec ignores it.
    #[serde(
        rename = "owner_funds",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub owner_funds: Option<String>,
}

impl OfferCreate {
    /// `"OfferCreate"`.
    pub const TRANSACTION_TYPE: &'static str = "OfferCreate";

    /// A `OfferCreate` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, taker_pays: Amount, taker_gets: Amount) -> Self {
        Self {
            common: Common::new(account),
            taker_pays,
            taker_gets,
            expiration: None,
            offer_sequence: None,
            domain_id: None,
            owner_funds: None,
        }
    }

    /// With `Expiration` set (see [`Self::expiration`]).
    #[must_use]
    pub fn with_expiration(mut self, value: impl Into<u32>) -> Self {
        self.expiration = Some(value.into());
        self
    }

    /// With `OfferSequence` set (see [`Self::offer_sequence`]).
    #[must_use]
    pub fn with_offer_sequence(mut self, value: impl Into<u32>) -> Self {
        self.offer_sequence = Some(value.into());
        self
    }

    /// With `DomainID` set (see [`Self::domain_id`]).
    #[must_use]
    pub fn with_domain_id(mut self, value: impl Into<Hash256>) -> Self {
        self.domain_id = Some(value.into());
        self
    }
}

impl TxModel for OfferCreate {
    const TRANSACTION_TYPE: &'static str = "OfferCreate";
    type Flags = OfferCreateFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<OfferCreate> for Tx {
    fn from(tx: OfferCreate) -> Self {
        Tx::OfferCreate(tx)
    }
}

/// [`OracleDelete`](https://xrpl.org/docs/references/protocol/transactions/types/oracledelete) transaction.
///
/// Required: `OracleDocumentID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OracleDelete {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The oracle's number among its owner's oracles (`OracleDocumentID`: a number). Required.
    #[serde(rename = "OracleDocumentID")]
    pub oracle_document_id: u32,
}

impl OracleDelete {
    /// `"OracleDelete"`.
    pub const TRANSACTION_TYPE: &'static str = "OracleDelete";

    /// A `OracleDelete` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, oracle_document_id: u32) -> Self {
        Self {
            common: Common::new(account),
            oracle_document_id,
        }
    }
}

impl TxModel for OracleDelete {
    const TRANSACTION_TYPE: &'static str = "OracleDelete";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<OracleDelete> for Tx {
    fn from(tx: OracleDelete) -> Self {
        Tx::OracleDelete(tx)
    }
}

/// [`OracleSet`](https://xrpl.org/docs/references/protocol/transactions/types/oracleset) transaction.
///
/// Required: `OracleDocumentID`, `LastUpdateTime`, `PriceDataSeries`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OracleSet {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The oracle's number among its owner's oracles (`OracleDocumentID`: a number). Required.
    #[serde(rename = "OracleDocumentID")]
    pub oracle_document_id: u32,
    /// The data provider, hex (`Provider`: bytes, hex). Optional.
    #[serde(rename = "Provider", default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// A URI, hex (`URI`: bytes, hex). Optional.
    #[serde(rename = "URI", default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// The kind of asset priced, hex (e.g. "currency") (`AssetClass`: bytes, hex). Optional.
    #[serde(
        rename = "AssetClass",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub asset_class: Option<String>,
    /// When the prices were last updated (Unix time) (`LastUpdateTime`: a number). Required.
    #[serde(rename = "LastUpdateTime")]
    pub last_update_time: u32,
    /// The prices (`PriceDataSeries`: a list). Required.
    #[serde(rename = "PriceDataSeries", with = "price_data_series")]
    pub price_data_series: Vec<PriceData>,
}

impl OracleSet {
    /// `"OracleSet"`.
    pub const TRANSACTION_TYPE: &'static str = "OracleSet";

    /// A `OracleSet` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        oracle_document_id: u32,
        last_update_time: u32,
        price_data_series: Vec<PriceData>,
    ) -> Self {
        Self {
            common: Common::new(account),
            oracle_document_id,
            provider: None,
            uri: None,
            asset_class: None,
            last_update_time,
            price_data_series,
        }
    }

    /// With `Provider` set (see [`Self::provider`]).
    #[must_use]
    pub fn with_provider(mut self, value: impl Into<String>) -> Self {
        self.provider = Some(value.into());
        self
    }

    /// With `URI` set (see [`Self::uri`]).
    #[must_use]
    pub fn with_uri(mut self, value: impl Into<String>) -> Self {
        self.uri = Some(value.into());
        self
    }

    /// With `AssetClass` set (see [`Self::asset_class`]).
    #[must_use]
    pub fn with_asset_class(mut self, value: impl Into<String>) -> Self {
        self.asset_class = Some(value.into());
        self
    }
}

impl TxModel for OracleSet {
    const TRANSACTION_TYPE: &'static str = "OracleSet";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<OracleSet> for Tx {
    fn from(tx: OracleSet) -> Self {
        Tx::OracleSet(tx)
    }
}

flags_type!(
    /// The flags of a [`Payment`] (`Flags`).
    PaymentFlags {
        /// `tfNoRippleDirect`.
        NO_RIPPLE_DIRECT = 0x00010000,
        /// `tfPartialPayment`.
        PARTIAL_PAYMENT = 0x00020000,
        /// `tfLimitQuality`.
        LIMIT_QUALITY = 0x00040000,
        /// `tfSponsorCreatedAccount`.
        SPONSOR_CREATED_ACCOUNT = 0x00080000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`Payment`](https://xrpl.org/docs/references/protocol/transactions/types/payment) transaction.
///
/// Required: `Destination`, `Amount`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Payment {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The account that receives it (`Destination`: a classic address). Required.
    #[serde(rename = "Destination")]
    pub destination: String,
    /// How much to send at most (`Amount`: XRP in drops, a token, or an MPT). Required.
    /// With `tfPartialPayment` the payment may deliver less: what arrived is the
    /// metadata's `delivered_amount` (`utils::delivered_amount`), never this field.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// The most the sender will spend, in any currency (`SendMax`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "SendMax", default, skip_serializing_if = "Option::is_none")]
    pub send_max: Option<Amount>,
    /// The paths a cross-currency payment may take (from `ripple_path_find`) (`Paths`: payment paths). Optional.
    #[serde(rename = "Paths", default, skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<Vec<PathStep>>>,
    /// An ID of the sender's choosing, such as an invoice number (`InvoiceID`: 256 bits, hex). Optional.
    #[serde(rename = "InvoiceID", default, skip_serializing_if = "Option::is_none")]
    pub invoice_id: Option<Hash256>,
    /// A tag the destination uses to tell its senders apart (`DestinationTag`: a number). Optional.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
    /// The least a partial payment must deliver (`DeliverMin`: XRP in drops, a token, or an MPT). Optional.
    #[serde(
        rename = "DeliverMin",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub deliver_min: Option<Amount>,
    /// Credentials to present (`CredentialIDs`: a list of 256-bit IDs). Optional.
    #[serde(
        rename = "CredentialIDs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub credential_ids: Option<Vec<Hash256>>,
    /// A permissioned domain's ID (`DomainID`: 256 bits, hex). Optional.
    #[serde(rename = "DomainID", default, skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<Hash256>,
    /// `rippled`'s API name for `Amount` in what it returns (API v1 writes both; v2 only
    /// this). Not a field of its own: the codec takes it as `Amount` (it must equal
    /// `Amount` when both are present), and a transaction read without `Amount` takes
    /// its `Amount` from here.
    #[serde(
        rename = "DeliverMax",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub deliver_max: Option<Amount>,
}

impl Payment {
    /// `"Payment"`.
    pub const TRANSACTION_TYPE: &'static str = "Payment";

    /// A `Payment` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, destination: impl Into<String>, amount: Amount) -> Self {
        Self {
            common: Common::new(account),
            destination: destination.into(),
            amount,
            send_max: None,
            paths: None,
            invoice_id: None,
            destination_tag: None,
            deliver_min: None,
            credential_ids: None,
            domain_id: None,
            deliver_max: None,
        }
    }

    /// With `SendMax` set (see [`Self::send_max`]).
    #[must_use]
    pub fn with_send_max(mut self, value: impl Into<Amount>) -> Self {
        self.send_max = Some(value.into());
        self
    }

    /// With `Paths` set (see [`Self::paths`]).
    #[must_use]
    pub fn with_paths(mut self, value: impl Into<Vec<Vec<PathStep>>>) -> Self {
        self.paths = Some(value.into());
        self
    }

    /// With `InvoiceID` set (see [`Self::invoice_id`]).
    #[must_use]
    pub fn with_invoice_id(mut self, value: impl Into<Hash256>) -> Self {
        self.invoice_id = Some(value.into());
        self
    }

    /// With `DestinationTag` set (see [`Self::destination_tag`]).
    #[must_use]
    pub fn with_destination_tag(mut self, value: impl Into<u32>) -> Self {
        self.destination_tag = Some(value.into());
        self
    }

    /// With `DeliverMin` set (see [`Self::deliver_min`]).
    #[must_use]
    pub fn with_deliver_min(mut self, value: impl Into<Amount>) -> Self {
        self.deliver_min = Some(value.into());
        self
    }

    /// With `CredentialIDs` set (see [`Self::credential_ids`]).
    #[must_use]
    pub fn with_credential_ids(mut self, value: impl Into<Vec<Hash256>>) -> Self {
        self.credential_ids = Some(value.into());
        self
    }

    /// With `DomainID` set (see [`Self::domain_id`]).
    #[must_use]
    pub fn with_domain_id(mut self, value: impl Into<Hash256>) -> Self {
        self.domain_id = Some(value.into());
        self
    }
}

impl TxModel for Payment {
    const TRANSACTION_TYPE: &'static str = "Payment";
    type Flags = PaymentFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<Payment> for Tx {
    fn from(tx: Payment) -> Self {
        Tx::Payment(tx)
    }
}

flags_type!(
    /// The flags of a [`PaymentChannelClaim`] (`Flags`).
    PaymentChannelClaimFlags {
        /// `tfRenew`.
        RENEW = 0x00010000,
        /// `tfClose`.
        CLOSE = 0x00020000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`PaymentChannelClaim`](https://xrpl.org/docs/references/protocol/transactions/types/paymentchannelclaim) transaction.
///
/// Required: `Channel`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PaymentChannelClaim {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The payment channel's ID (`Channel`: 256 bits, hex). Required.
    #[serde(rename = "Channel")]
    pub channel: Hash256,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "Amount", default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
    /// The balance (`Balance`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "Balance", default, skip_serializing_if = "Option::is_none")]
    pub balance: Option<Amount>,
    /// A signature, hex (`Signature`: bytes, hex). Optional.
    #[serde(rename = "Signature", default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// A public key, hex (`PublicKey`: bytes, hex). Optional.
    #[serde(rename = "PublicKey", default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    /// Credentials to present (`CredentialIDs`: a list of 256-bit IDs). Optional.
    #[serde(
        rename = "CredentialIDs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub credential_ids: Option<Vec<Hash256>>,
}

impl PaymentChannelClaim {
    /// `"PaymentChannelClaim"`.
    pub const TRANSACTION_TYPE: &'static str = "PaymentChannelClaim";

    /// A `PaymentChannelClaim` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, channel: Hash256) -> Self {
        Self {
            common: Common::new(account),
            channel,
            amount: None,
            balance: None,
            signature: None,
            public_key: None,
            credential_ids: None,
        }
    }

    /// With `Amount` set (see [`Self::amount`]).
    #[must_use]
    pub fn with_amount(mut self, value: impl Into<Amount>) -> Self {
        self.amount = Some(value.into());
        self
    }

    /// With `Balance` set (see [`Self::balance`]).
    #[must_use]
    pub fn with_balance(mut self, value: impl Into<Amount>) -> Self {
        self.balance = Some(value.into());
        self
    }

    /// With `Signature` set (see [`Self::signature`]).
    #[must_use]
    pub fn with_signature(mut self, value: impl Into<String>) -> Self {
        self.signature = Some(value.into());
        self
    }

    /// With `PublicKey` set (see [`Self::public_key`]).
    #[must_use]
    pub fn with_public_key(mut self, value: impl Into<String>) -> Self {
        self.public_key = Some(value.into());
        self
    }

    /// With `CredentialIDs` set (see [`Self::credential_ids`]).
    #[must_use]
    pub fn with_credential_ids(mut self, value: impl Into<Vec<Hash256>>) -> Self {
        self.credential_ids = Some(value.into());
        self
    }
}

impl TxModel for PaymentChannelClaim {
    const TRANSACTION_TYPE: &'static str = "PaymentChannelClaim";
    type Flags = PaymentChannelClaimFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<PaymentChannelClaim> for Tx {
    fn from(tx: PaymentChannelClaim) -> Self {
        Tx::PaymentChannelClaim(tx)
    }
}

/// [`PaymentChannelCreate`](https://xrpl.org/docs/references/protocol/transactions/types/paymentchannelcreate) transaction.
///
/// Required: `Destination`, `Amount`, `SettleDelay`, `PublicKey`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PaymentChannelCreate {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The account that receives it (`Destination`: a classic address). Required.
    #[serde(rename = "Destination")]
    pub destination: String,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// Seconds the source must wait before closing a channel with unclaimed XRP (`SettleDelay`: a number). Required.
    #[serde(rename = "SettleDelay")]
    pub settle_delay: u32,
    /// A public key, hex (`PublicKey`: bytes, hex). Required.
    #[serde(rename = "PublicKey")]
    pub public_key: String,
    /// The time after which it can be cancelled (ledger time) (`CancelAfter`: a number). Optional.
    #[serde(
        rename = "CancelAfter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cancel_after: Option<u32>,
    /// A tag the destination uses to tell its senders apart (`DestinationTag`: a number). Optional.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
}

impl PaymentChannelCreate {
    /// `"PaymentChannelCreate"`.
    pub const TRANSACTION_TYPE: &'static str = "PaymentChannelCreate";

    /// A `PaymentChannelCreate` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        destination: impl Into<String>,
        amount: Amount,
        settle_delay: u32,
        public_key: impl Into<String>,
    ) -> Self {
        Self {
            common: Common::new(account),
            destination: destination.into(),
            amount,
            settle_delay,
            public_key: public_key.into(),
            cancel_after: None,
            destination_tag: None,
        }
    }

    /// With `CancelAfter` set (see [`Self::cancel_after`]).
    #[must_use]
    pub fn with_cancel_after(mut self, value: impl Into<u32>) -> Self {
        self.cancel_after = Some(value.into());
        self
    }

    /// With `DestinationTag` set (see [`Self::destination_tag`]).
    #[must_use]
    pub fn with_destination_tag(mut self, value: impl Into<u32>) -> Self {
        self.destination_tag = Some(value.into());
        self
    }
}

impl TxModel for PaymentChannelCreate {
    const TRANSACTION_TYPE: &'static str = "PaymentChannelCreate";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<PaymentChannelCreate> for Tx {
    fn from(tx: PaymentChannelCreate) -> Self {
        Tx::PaymentChannelCreate(tx)
    }
}

/// [`PaymentChannelFund`](https://xrpl.org/docs/references/protocol/transactions/types/paymentchannelfund) transaction.
///
/// Required: `Channel`, `Amount`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PaymentChannelFund {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The payment channel's ID (`Channel`: 256 bits, hex). Required.
    #[serde(rename = "Channel")]
    pub channel: Hash256,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// When it expires (ledger time: seconds since 2000-01-01) (`Expiration`: a number). Optional.
    #[serde(
        rename = "Expiration",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration: Option<u32>,
}

impl PaymentChannelFund {
    /// `"PaymentChannelFund"`.
    pub const TRANSACTION_TYPE: &'static str = "PaymentChannelFund";

    /// A `PaymentChannelFund` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, channel: Hash256, amount: Amount) -> Self {
        Self {
            common: Common::new(account),
            channel,
            amount,
            expiration: None,
        }
    }

    /// With `Expiration` set (see [`Self::expiration`]).
    #[must_use]
    pub fn with_expiration(mut self, value: impl Into<u32>) -> Self {
        self.expiration = Some(value.into());
        self
    }
}

impl TxModel for PaymentChannelFund {
    const TRANSACTION_TYPE: &'static str = "PaymentChannelFund";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<PaymentChannelFund> for Tx {
    fn from(tx: PaymentChannelFund) -> Self {
        Tx::PaymentChannelFund(tx)
    }
}

/// [`PermissionedDomainDelete`](https://xrpl.org/docs/references/protocol/transactions/types/permissioneddomaindelete) transaction.
///
/// Required: `DomainID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PermissionedDomainDelete {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// A permissioned domain's ID (`DomainID`: 256 bits, hex). Required.
    #[serde(rename = "DomainID")]
    pub domain_id: Hash256,
}

impl PermissionedDomainDelete {
    /// `"PermissionedDomainDelete"`.
    pub const TRANSACTION_TYPE: &'static str = "PermissionedDomainDelete";

    /// A `PermissionedDomainDelete` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, domain_id: Hash256) -> Self {
        Self {
            common: Common::new(account),
            domain_id,
        }
    }
}

impl TxModel for PermissionedDomainDelete {
    const TRANSACTION_TYPE: &'static str = "PermissionedDomainDelete";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<PermissionedDomainDelete> for Tx {
    fn from(tx: PermissionedDomainDelete) -> Self {
        Tx::PermissionedDomainDelete(tx)
    }
}

/// [`PermissionedDomainSet`](https://xrpl.org/docs/references/protocol/transactions/types/permissioneddomainset) transaction.
///
/// Required: `AcceptedCredentials`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PermissionedDomainSet {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// A permissioned domain's ID (`DomainID`: 256 bits, hex). Optional.
    #[serde(rename = "DomainID", default, skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<Hash256>,
    /// The credentials the domain accepts (`AcceptedCredentials`: a list). Required.
    #[serde(rename = "AcceptedCredentials", with = "credentials")]
    pub accepted_credentials: Vec<CredentialSpec>,
}

impl PermissionedDomainSet {
    /// `"PermissionedDomainSet"`.
    pub const TRANSACTION_TYPE: &'static str = "PermissionedDomainSet";

    /// A `PermissionedDomainSet` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, accepted_credentials: Vec<CredentialSpec>) -> Self {
        Self {
            common: Common::new(account),
            domain_id: None,
            accepted_credentials,
        }
    }

    /// With `DomainID` set (see [`Self::domain_id`]).
    #[must_use]
    pub fn with_domain_id(mut self, value: impl Into<Hash256>) -> Self {
        self.domain_id = Some(value.into());
        self
    }
}

impl TxModel for PermissionedDomainSet {
    const TRANSACTION_TYPE: &'static str = "PermissionedDomainSet";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<PermissionedDomainSet> for Tx {
    fn from(tx: PermissionedDomainSet) -> Self {
        Tx::PermissionedDomainSet(tx)
    }
}

/// [`SetFee`](https://xrpl.org/docs/references/protocol/transactions/types/setfee) transaction.
///
/// Required: none beyond the common fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SetFee {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The ledger the pseudo-transaction applies to (`LedgerSequence`: a number). Optional.
    #[serde(
        rename = "LedgerSequence",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub ledger_sequence: Option<u32>,
    /// `BaseFee`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Optional.
    #[serde(rename = "BaseFee", default, skip_serializing_if = "Option::is_none")]
    pub base_fee: Option<String>,
    /// `ReferenceFeeUnits`: a number. Optional.
    #[serde(
        rename = "ReferenceFeeUnits",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reference_fee_units: Option<u32>,
    /// `ReserveBase`: a number. Optional.
    #[serde(
        rename = "ReserveBase",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reserve_base: Option<u32>,
    /// `ReserveIncrement`: a number. Optional.
    #[serde(
        rename = "ReserveIncrement",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reserve_increment: Option<u32>,
    /// The reference transaction cost, in drops (`BaseFeeDrops`: XRP in drops, a token, or an MPT). Optional.
    #[serde(
        rename = "BaseFeeDrops",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub base_fee_drops: Option<Amount>,
    /// The account reserve, in drops (`ReserveBaseDrops`: XRP in drops, a token, or an MPT). Optional.
    #[serde(
        rename = "ReserveBaseDrops",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reserve_base_drops: Option<Amount>,
    /// The reserve per owned entry, in drops (`ReserveIncrementDrops`: XRP in drops, a token, or an MPT). Optional.
    #[serde(
        rename = "ReserveIncrementDrops",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub reserve_increment_drops: Option<Amount>,
}

impl SetFee {
    /// `"SetFee"`.
    pub const TRANSACTION_TYPE: &'static str = "SetFee";

    /// A `SetFee` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: Common::new(account),
            ledger_sequence: None,
            base_fee: None,
            reference_fee_units: None,
            reserve_base: None,
            reserve_increment: None,
            base_fee_drops: None,
            reserve_base_drops: None,
            reserve_increment_drops: None,
        }
    }

    /// With `LedgerSequence` set (see [`Self::ledger_sequence`]).
    #[must_use]
    pub fn with_ledger_sequence(mut self, value: impl Into<u32>) -> Self {
        self.ledger_sequence = Some(value.into());
        self
    }

    /// With `BaseFee` set (see [`Self::base_fee`]).
    #[must_use]
    pub fn with_base_fee(mut self, value: impl Into<String>) -> Self {
        self.base_fee = Some(value.into());
        self
    }

    /// With `ReferenceFeeUnits` set (see [`Self::reference_fee_units`]).
    #[must_use]
    pub fn with_reference_fee_units(mut self, value: impl Into<u32>) -> Self {
        self.reference_fee_units = Some(value.into());
        self
    }

    /// With `ReserveBase` set (see [`Self::reserve_base`]).
    #[must_use]
    pub fn with_reserve_base(mut self, value: impl Into<u32>) -> Self {
        self.reserve_base = Some(value.into());
        self
    }

    /// With `ReserveIncrement` set (see [`Self::reserve_increment`]).
    #[must_use]
    pub fn with_reserve_increment(mut self, value: impl Into<u32>) -> Self {
        self.reserve_increment = Some(value.into());
        self
    }

    /// With `BaseFeeDrops` set (see [`Self::base_fee_drops`]).
    #[must_use]
    pub fn with_base_fee_drops(mut self, value: impl Into<Amount>) -> Self {
        self.base_fee_drops = Some(value.into());
        self
    }

    /// With `ReserveBaseDrops` set (see [`Self::reserve_base_drops`]).
    #[must_use]
    pub fn with_reserve_base_drops(mut self, value: impl Into<Amount>) -> Self {
        self.reserve_base_drops = Some(value.into());
        self
    }

    /// With `ReserveIncrementDrops` set (see [`Self::reserve_increment_drops`]).
    #[must_use]
    pub fn with_reserve_increment_drops(mut self, value: impl Into<Amount>) -> Self {
        self.reserve_increment_drops = Some(value.into());
        self
    }
}

impl TxModel for SetFee {
    const TRANSACTION_TYPE: &'static str = "SetFee";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<SetFee> for Tx {
    fn from(tx: SetFee) -> Self {
        Tx::SetFee(tx)
    }
}

/// [`SetRegularKey`](https://xrpl.org/docs/references/protocol/transactions/types/setregularkey) transaction.
///
/// Required: none beyond the common fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SetRegularKey {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The regular key pair's account; absent to remove it (`RegularKey`: a classic address). Optional.
    #[serde(
        rename = "RegularKey",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub regular_key: Option<String>,
}

impl SetRegularKey {
    /// `"SetRegularKey"`.
    pub const TRANSACTION_TYPE: &'static str = "SetRegularKey";

    /// A `SetRegularKey` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: Common::new(account),
            regular_key: None,
        }
    }

    /// With `RegularKey` set (see [`Self::regular_key`]).
    #[must_use]
    pub fn with_regular_key(mut self, value: impl Into<String>) -> Self {
        self.regular_key = Some(value.into());
        self
    }
}

impl TxModel for SetRegularKey {
    const TRANSACTION_TYPE: &'static str = "SetRegularKey";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<SetRegularKey> for Tx {
    fn from(tx: SetRegularKey) -> Self {
        Tx::SetRegularKey(tx)
    }
}

/// [`SignerListSet`](https://xrpl.org/docs/references/protocol/transactions/types/signerlistset) transaction.
///
/// Required: `SignerQuorum`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SignerListSet {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The total signer weight a multi-signature needs; 0 deletes the list (`SignerQuorum`: a number). Required.
    #[serde(rename = "SignerQuorum")]
    pub signer_quorum: u32,
    /// The signers and their weights (`SignerEntries`: a list). Optional.
    #[serde(
        rename = "SignerEntries",
        default,
        skip_serializing_if = "Option::is_none",
        with = "signer_entries_opt"
    )]
    pub signer_entries: Option<Vec<SignerEntry>>,
}

impl SignerListSet {
    /// `"SignerListSet"`.
    pub const TRANSACTION_TYPE: &'static str = "SignerListSet";

    /// A `SignerListSet` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, signer_quorum: u32) -> Self {
        Self {
            common: Common::new(account),
            signer_quorum,
            signer_entries: None,
        }
    }

    /// With `SignerEntries` set (see [`Self::signer_entries`]).
    #[must_use]
    pub fn with_signer_entries(mut self, value: impl Into<Vec<SignerEntry>>) -> Self {
        self.signer_entries = Some(value.into());
        self
    }
}

impl TxModel for SignerListSet {
    const TRANSACTION_TYPE: &'static str = "SignerListSet";
    type Flags = UniversalFlags;
    fn validate(&self) -> Result<(), CoreError> {
        super::validate::signer_list_set(self)
    }
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<SignerListSet> for Tx {
    fn from(tx: SignerListSet) -> Self {
        Tx::SignerListSet(tx)
    }
}

flags_type!(
    /// The flags of a [`SponsorshipSet`] (`Flags`).
    SponsorshipSetFlags {
        /// `tfSponsorshipSetRequireSignForFee`.
        SPONSORSHIP_SET_REQUIRE_SIGN_FOR_FEE = 0x00010000,
        /// `tfSponsorshipClearRequireSignForFee`.
        SPONSORSHIP_CLEAR_REQUIRE_SIGN_FOR_FEE = 0x00020000,
        /// `tfSponsorshipSetRequireSignForReserve`.
        SPONSORSHIP_SET_REQUIRE_SIGN_FOR_RESERVE = 0x00040000,
        /// `tfSponsorshipClearRequireSignForReserve`.
        SPONSORSHIP_CLEAR_REQUIRE_SIGN_FOR_RESERVE = 0x00080000,
        /// `tfDeleteObject`.
        DELETE_OBJECT = 0x00100000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`SponsorshipSet`](https://xrpl.org/docs/references/protocol/transactions/types/sponsorshipset) transaction.
///
/// Required: none beyond the common fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SponsorshipSet {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `CounterpartySponsor`: a classic address. Optional.
    #[serde(
        rename = "CounterpartySponsor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub counterparty_sponsor: Option<String>,
    /// `Sponsee`: a classic address. Optional.
    #[serde(rename = "Sponsee", default, skip_serializing_if = "Option::is_none")]
    pub sponsee: Option<String>,
    /// `FeeAmountDelta`: XRP in drops, a token, or an MPT. Optional.
    #[serde(
        rename = "FeeAmountDelta",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub fee_amount_delta: Option<Amount>,
    /// `MaxFee`: XRP in drops, a token, or an MPT. Optional.
    #[serde(rename = "MaxFee", default, skip_serializing_if = "Option::is_none")]
    pub max_fee: Option<Amount>,
    /// `RemainingOwnerCountDelta`: a signed number. Optional.
    #[serde(
        rename = "RemainingOwnerCountDelta",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub remaining_owner_count_delta: Option<i32>,
}

impl SponsorshipSet {
    /// `"SponsorshipSet"`.
    pub const TRANSACTION_TYPE: &'static str = "SponsorshipSet";

    /// A `SponsorshipSet` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: Common::new(account),
            counterparty_sponsor: None,
            sponsee: None,
            fee_amount_delta: None,
            max_fee: None,
            remaining_owner_count_delta: None,
        }
    }

    /// With `CounterpartySponsor` set (see [`Self::counterparty_sponsor`]).
    #[must_use]
    pub fn with_counterparty_sponsor(mut self, value: impl Into<String>) -> Self {
        self.counterparty_sponsor = Some(value.into());
        self
    }

    /// With `Sponsee` set (see [`Self::sponsee`]).
    #[must_use]
    pub fn with_sponsee(mut self, value: impl Into<String>) -> Self {
        self.sponsee = Some(value.into());
        self
    }

    /// With `FeeAmountDelta` set (see [`Self::fee_amount_delta`]).
    #[must_use]
    pub fn with_fee_amount_delta(mut self, value: impl Into<Amount>) -> Self {
        self.fee_amount_delta = Some(value.into());
        self
    }

    /// With `MaxFee` set (see [`Self::max_fee`]).
    #[must_use]
    pub fn with_max_fee(mut self, value: impl Into<Amount>) -> Self {
        self.max_fee = Some(value.into());
        self
    }

    /// With `RemainingOwnerCountDelta` set (see [`Self::remaining_owner_count_delta`]).
    #[must_use]
    pub fn with_remaining_owner_count_delta(mut self, value: impl Into<i32>) -> Self {
        self.remaining_owner_count_delta = Some(value.into());
        self
    }
}

impl TxModel for SponsorshipSet {
    const TRANSACTION_TYPE: &'static str = "SponsorshipSet";
    type Flags = SponsorshipSetFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<SponsorshipSet> for Tx {
    fn from(tx: SponsorshipSet) -> Self {
        Tx::SponsorshipSet(tx)
    }
}

flags_type!(
    /// The flags of a [`SponsorshipTransfer`] (`Flags`).
    SponsorshipTransferFlags {
        /// `tfSponsorshipEnd`.
        SPONSORSHIP_END = 0x00010000,
        /// `tfSponsorshipCreate`.
        SPONSORSHIP_CREATE = 0x00020000,
        /// `tfSponsorshipReassign`.
        SPONSORSHIP_REASSIGN = 0x00040000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`SponsorshipTransfer`](https://xrpl.org/docs/references/protocol/transactions/types/sponsorshiptransfer) transaction.
///
/// Required: none beyond the common fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SponsorshipTransfer {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `ObjectID`: 256 bits, hex. Optional.
    #[serde(rename = "ObjectID", default, skip_serializing_if = "Option::is_none")]
    pub object_id: Option<Hash256>,
    /// `Sponsee`: a classic address. Optional.
    #[serde(rename = "Sponsee", default, skip_serializing_if = "Option::is_none")]
    pub sponsee: Option<String>,
}

impl SponsorshipTransfer {
    /// `"SponsorshipTransfer"`.
    pub const TRANSACTION_TYPE: &'static str = "SponsorshipTransfer";

    /// A `SponsorshipTransfer` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: Common::new(account),
            object_id: None,
            sponsee: None,
        }
    }

    /// With `ObjectID` set (see [`Self::object_id`]).
    #[must_use]
    pub fn with_object_id(mut self, value: impl Into<Hash256>) -> Self {
        self.object_id = Some(value.into());
        self
    }

    /// With `Sponsee` set (see [`Self::sponsee`]).
    #[must_use]
    pub fn with_sponsee(mut self, value: impl Into<String>) -> Self {
        self.sponsee = Some(value.into());
        self
    }
}

impl TxModel for SponsorshipTransfer {
    const TRANSACTION_TYPE: &'static str = "SponsorshipTransfer";
    type Flags = SponsorshipTransferFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<SponsorshipTransfer> for Tx {
    fn from(tx: SponsorshipTransfer) -> Self {
        Tx::SponsorshipTransfer(tx)
    }
}

/// [`TicketCreate`](https://xrpl.org/docs/references/protocol/transactions/types/ticketcreate) transaction.
///
/// Required: `TicketCount`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TicketCreate {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// How many tickets to create (`TicketCount`: a number). Required.
    #[serde(rename = "TicketCount")]
    pub ticket_count: u32,
}

impl TicketCreate {
    /// `"TicketCreate"`.
    pub const TRANSACTION_TYPE: &'static str = "TicketCreate";

    /// A `TicketCreate` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, ticket_count: u32) -> Self {
        Self {
            common: Common::new(account),
            ticket_count,
        }
    }
}

impl TxModel for TicketCreate {
    const TRANSACTION_TYPE: &'static str = "TicketCreate";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<TicketCreate> for Tx {
    fn from(tx: TicketCreate) -> Self {
        Tx::TicketCreate(tx)
    }
}

flags_type!(
    /// The flags of a [`TrustSet`] (`Flags`).
    TrustSetFlags {
        /// `tfSetfAuth`.
        SETF_AUTH = 0x00010000,
        /// `tfSetNoRipple`.
        SET_NO_RIPPLE = 0x00020000,
        /// `tfClearNoRipple`.
        CLEAR_NO_RIPPLE = 0x00040000,
        /// `tfSetFreeze`.
        SET_FREEZE = 0x00100000,
        /// `tfClearFreeze`.
        CLEAR_FREEZE = 0x00200000,
        /// `tfSetDeepFreeze`.
        SET_DEEP_FREEZE = 0x00400000,
        /// `tfClearDeepFreeze`.
        CLEAR_DEEP_FREEZE = 0x00800000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`TrustSet`](https://xrpl.org/docs/references/protocol/transactions/types/trustset) transaction.
///
/// Required: none beyond the common fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TrustSet {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The most of the token the account will hold, and its issuer (`LimitAmount`: XRP in drops, a token, or an MPT). Optional.
    #[serde(
        rename = "LimitAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub limit_amount: Option<Amount>,
    /// The value of incoming balances, in billionths (0 or 1,000,000,000 = face value) (`QualityIn`: a number). Optional.
    #[serde(rename = "QualityIn", default, skip_serializing_if = "Option::is_none")]
    pub quality_in: Option<u32>,
    /// The value of outgoing balances, in billionths (`QualityOut`: a number). Optional.
    #[serde(
        rename = "QualityOut",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub quality_out: Option<u32>,
}

impl TrustSet {
    /// `"TrustSet"`.
    pub const TRANSACTION_TYPE: &'static str = "TrustSet";

    /// A `TrustSet` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>) -> Self {
        Self {
            common: Common::new(account),
            limit_amount: None,
            quality_in: None,
            quality_out: None,
        }
    }

    /// With `LimitAmount` set (see [`Self::limit_amount`]).
    #[must_use]
    pub fn with_limit_amount(mut self, value: impl Into<Amount>) -> Self {
        self.limit_amount = Some(value.into());
        self
    }

    /// With `QualityIn` set (see [`Self::quality_in`]).
    #[must_use]
    pub fn with_quality_in(mut self, value: impl Into<u32>) -> Self {
        self.quality_in = Some(value.into());
        self
    }

    /// With `QualityOut` set (see [`Self::quality_out`]).
    #[must_use]
    pub fn with_quality_out(mut self, value: impl Into<u32>) -> Self {
        self.quality_out = Some(value.into());
        self
    }
}

impl TxModel for TrustSet {
    const TRANSACTION_TYPE: &'static str = "TrustSet";
    type Flags = TrustSetFlags;
    fn validate(&self) -> Result<(), CoreError> {
        super::validate::trust_set(self)
    }
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<TrustSet> for Tx {
    fn from(tx: TrustSet) -> Self {
        Tx::TrustSet(tx)
    }
}

/// [`UNLModify`](https://xrpl.org/docs/references/protocol/transactions/types/unlmodify) transaction.
///
/// Required: `UNLModifyDisabling`, `LedgerSequence`, `UNLModifyValidator`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UNLModify {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `UNLModifyDisabling`: a number. Required.
    #[serde(rename = "UNLModifyDisabling")]
    pub unl_modify_disabling: u8,
    /// The ledger the pseudo-transaction applies to (`LedgerSequence`: a number). Required.
    #[serde(rename = "LedgerSequence")]
    pub ledger_sequence: u32,
    /// `UNLModifyValidator`: bytes, hex. Required.
    #[serde(rename = "UNLModifyValidator")]
    pub unl_modify_validator: String,
}

impl UNLModify {
    /// `"UNLModify"`.
    pub const TRANSACTION_TYPE: &'static str = "UNLModify";

    /// A `UNLModify` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        unl_modify_disabling: u8,
        ledger_sequence: u32,
        unl_modify_validator: impl Into<String>,
    ) -> Self {
        Self {
            common: Common::new(account),
            unl_modify_disabling,
            ledger_sequence,
            unl_modify_validator: unl_modify_validator.into(),
        }
    }
}

impl TxModel for UNLModify {
    const TRANSACTION_TYPE: &'static str = "UNLModify";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<UNLModify> for Tx {
    fn from(tx: UNLModify) -> Self {
        Tx::UNLModify(tx)
    }
}

/// [`VaultClawback`](https://xrpl.org/docs/references/protocol/transactions/types/vaultclawback) transaction.
///
/// Required: `VaultID`, `Holder`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VaultClawback {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `VaultID`: 256 bits, hex. Required.
    #[serde(rename = "VaultID")]
    pub vault_id: Hash256,
    /// The holder (`Holder`: a classic address). Required.
    #[serde(rename = "Holder")]
    pub holder: String,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Optional.
    #[serde(rename = "Amount", default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
}

impl VaultClawback {
    /// `"VaultClawback"`.
    pub const TRANSACTION_TYPE: &'static str = "VaultClawback";

    /// A `VaultClawback` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, vault_id: Hash256, holder: impl Into<String>) -> Self {
        Self {
            common: Common::new(account),
            vault_id,
            holder: holder.into(),
            amount: None,
        }
    }

    /// With `Amount` set (see [`Self::amount`]).
    #[must_use]
    pub fn with_amount(mut self, value: impl Into<Amount>) -> Self {
        self.amount = Some(value.into());
        self
    }
}

impl TxModel for VaultClawback {
    const TRANSACTION_TYPE: &'static str = "VaultClawback";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<VaultClawback> for Tx {
    fn from(tx: VaultClawback) -> Self {
        Tx::VaultClawback(tx)
    }
}

flags_type!(
    /// The flags of a [`VaultCreate`] (`Flags`).
    VaultCreateFlags {
        /// `tfVaultPrivate`.
        VAULT_PRIVATE = 0x00010000,
        /// `tfVaultShareNonTransferable`.
        VAULT_SHARE_NON_TRANSFERABLE = 0x00020000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`VaultCreate`](https://xrpl.org/docs/references/protocol/transactions/types/vaultcreate) transaction.
///
/// Required: `Asset`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VaultCreate {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// The pool's first asset (`Asset`: an asset: XRP, a token or an MPT). Required.
    #[serde(rename = "Asset")]
    pub asset: Asset,
    /// `AssetsMaximum`: a decimal number, as a string. Optional.
    #[serde(
        rename = "AssetsMaximum",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub assets_maximum: Option<String>,
    /// Metadata, hex (XLS-89 JSON) (`MPTokenMetadata`: bytes, hex). Optional.
    #[serde(
        rename = "MPTokenMetadata",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub mp_token_metadata: Option<String>,
    /// A permissioned domain's ID (`DomainID`: 256 bits, hex). Optional.
    #[serde(rename = "DomainID", default, skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<Hash256>,
    /// `WithdrawalPolicy`: a number. Optional.
    #[serde(
        rename = "WithdrawalPolicy",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub withdrawal_policy: Option<u8>,
    /// Data, hex (`Data`: bytes, hex). Optional.
    #[serde(rename = "Data", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// `Scale`: a number. Optional.
    #[serde(rename = "Scale", default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<u8>,
    /// `VaultKind`: a number. Optional.
    #[serde(rename = "VaultKind", default, skip_serializing_if = "Option::is_none")]
    pub vault_kind: Option<u8>,
    /// `SubscriptionDate`: a number. Optional.
    #[serde(
        rename = "SubscriptionDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub subscription_date: Option<u32>,
    /// `RedemptionDate`: a number. Optional.
    #[serde(
        rename = "RedemptionDate",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub redemption_date: Option<u32>,
}

impl VaultCreate {
    /// `"VaultCreate"`.
    pub const TRANSACTION_TYPE: &'static str = "VaultCreate";

    /// A `VaultCreate` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, asset: Asset) -> Self {
        Self {
            common: Common::new(account),
            asset,
            assets_maximum: None,
            mp_token_metadata: None,
            domain_id: None,
            withdrawal_policy: None,
            data: None,
            scale: None,
            vault_kind: None,
            subscription_date: None,
            redemption_date: None,
        }
    }

    /// With `AssetsMaximum` set (see [`Self::assets_maximum`]).
    #[must_use]
    pub fn with_assets_maximum(mut self, value: impl Into<String>) -> Self {
        self.assets_maximum = Some(value.into());
        self
    }

    /// With `MPTokenMetadata` set (see [`Self::mp_token_metadata`]).
    #[must_use]
    pub fn with_mp_token_metadata(mut self, value: impl Into<String>) -> Self {
        self.mp_token_metadata = Some(value.into());
        self
    }

    /// With `DomainID` set (see [`Self::domain_id`]).
    #[must_use]
    pub fn with_domain_id(mut self, value: impl Into<Hash256>) -> Self {
        self.domain_id = Some(value.into());
        self
    }

    /// With `WithdrawalPolicy` set (see [`Self::withdrawal_policy`]).
    #[must_use]
    pub fn with_withdrawal_policy(mut self, value: impl Into<u8>) -> Self {
        self.withdrawal_policy = Some(value.into());
        self
    }

    /// With `Data` set (see [`Self::data`]).
    #[must_use]
    pub fn with_data(mut self, value: impl Into<String>) -> Self {
        self.data = Some(value.into());
        self
    }

    /// With `Scale` set (see [`Self::scale`]).
    #[must_use]
    pub fn with_scale(mut self, value: impl Into<u8>) -> Self {
        self.scale = Some(value.into());
        self
    }

    /// With `VaultKind` set (see [`Self::vault_kind`]).
    #[must_use]
    pub fn with_vault_kind(mut self, value: impl Into<u8>) -> Self {
        self.vault_kind = Some(value.into());
        self
    }

    /// With `SubscriptionDate` set (see [`Self::subscription_date`]).
    #[must_use]
    pub fn with_subscription_date(mut self, value: impl Into<u32>) -> Self {
        self.subscription_date = Some(value.into());
        self
    }

    /// With `RedemptionDate` set (see [`Self::redemption_date`]).
    #[must_use]
    pub fn with_redemption_date(mut self, value: impl Into<u32>) -> Self {
        self.redemption_date = Some(value.into());
        self
    }
}

impl TxModel for VaultCreate {
    const TRANSACTION_TYPE: &'static str = "VaultCreate";
    type Flags = VaultCreateFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<VaultCreate> for Tx {
    fn from(tx: VaultCreate) -> Self {
        Tx::VaultCreate(tx)
    }
}

/// [`VaultDelete`](https://xrpl.org/docs/references/protocol/transactions/types/vaultdelete) transaction.
///
/// Required: `VaultID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VaultDelete {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `VaultID`: 256 bits, hex. Required.
    #[serde(rename = "VaultID")]
    pub vault_id: Hash256,
    /// `MemoData`: bytes, hex. Optional.
    #[serde(rename = "MemoData", default, skip_serializing_if = "Option::is_none")]
    pub memo_data: Option<String>,
}

impl VaultDelete {
    /// `"VaultDelete"`.
    pub const TRANSACTION_TYPE: &'static str = "VaultDelete";

    /// A `VaultDelete` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, vault_id: Hash256) -> Self {
        Self {
            common: Common::new(account),
            vault_id,
            memo_data: None,
        }
    }

    /// With `MemoData` set (see [`Self::memo_data`]).
    #[must_use]
    pub fn with_memo_data(mut self, value: impl Into<String>) -> Self {
        self.memo_data = Some(value.into());
        self
    }
}

impl TxModel for VaultDelete {
    const TRANSACTION_TYPE: &'static str = "VaultDelete";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<VaultDelete> for Tx {
    fn from(tx: VaultDelete) -> Self {
        Tx::VaultDelete(tx)
    }
}

/// [`VaultDeposit`](https://xrpl.org/docs/references/protocol/transactions/types/vaultdeposit) transaction.
///
/// Required: `VaultID`, `Amount`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VaultDeposit {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `VaultID`: 256 bits, hex. Required.
    #[serde(rename = "VaultID")]
    pub vault_id: Hash256,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
}

impl VaultDeposit {
    /// `"VaultDeposit"`.
    pub const TRANSACTION_TYPE: &'static str = "VaultDeposit";

    /// A `VaultDeposit` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, vault_id: Hash256, amount: Amount) -> Self {
        Self {
            common: Common::new(account),
            vault_id,
            amount,
        }
    }
}

impl TxModel for VaultDeposit {
    const TRANSACTION_TYPE: &'static str = "VaultDeposit";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<VaultDeposit> for Tx {
    fn from(tx: VaultDeposit) -> Self {
        Tx::VaultDeposit(tx)
    }
}

/// [`VaultSet`](https://xrpl.org/docs/references/protocol/transactions/types/vaultset) transaction.
///
/// Required: `VaultID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VaultSet {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `VaultID`: 256 bits, hex. Required.
    #[serde(rename = "VaultID")]
    pub vault_id: Hash256,
    /// `AssetsMaximum`: a decimal number, as a string. Optional.
    #[serde(
        rename = "AssetsMaximum",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub assets_maximum: Option<String>,
    /// A permissioned domain's ID (`DomainID`: 256 bits, hex). Optional.
    #[serde(rename = "DomainID", default, skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<Hash256>,
    /// Data, hex (`Data`: bytes, hex). Optional.
    #[serde(rename = "Data", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

impl VaultSet {
    /// `"VaultSet"`.
    pub const TRANSACTION_TYPE: &'static str = "VaultSet";

    /// A `VaultSet` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, vault_id: Hash256) -> Self {
        Self {
            common: Common::new(account),
            vault_id,
            assets_maximum: None,
            domain_id: None,
            data: None,
        }
    }

    /// With `AssetsMaximum` set (see [`Self::assets_maximum`]).
    #[must_use]
    pub fn with_assets_maximum(mut self, value: impl Into<String>) -> Self {
        self.assets_maximum = Some(value.into());
        self
    }

    /// With `DomainID` set (see [`Self::domain_id`]).
    #[must_use]
    pub fn with_domain_id(mut self, value: impl Into<Hash256>) -> Self {
        self.domain_id = Some(value.into());
        self
    }

    /// With `Data` set (see [`Self::data`]).
    #[must_use]
    pub fn with_data(mut self, value: impl Into<String>) -> Self {
        self.data = Some(value.into());
        self
    }
}

impl TxModel for VaultSet {
    const TRANSACTION_TYPE: &'static str = "VaultSet";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<VaultSet> for Tx {
    fn from(tx: VaultSet) -> Self {
        Tx::VaultSet(tx)
    }
}

/// [`VaultWithdraw`](https://xrpl.org/docs/references/protocol/transactions/types/vaultwithdraw) transaction.
///
/// Required: `VaultID`, `Amount`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VaultWithdraw {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `VaultID`: 256 bits, hex. Required.
    #[serde(rename = "VaultID")]
    pub vault_id: Hash256,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// The account that receives it (`Destination`: a classic address). Optional.
    #[serde(
        rename = "Destination",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination: Option<String>,
    /// A tag the destination uses to tell its senders apart (`DestinationTag`: a number). Optional.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
    /// Credentials to present (`CredentialIDs`: a list of 256-bit IDs). Optional.
    #[serde(
        rename = "CredentialIDs",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub credential_ids: Option<Vec<Hash256>>,
}

impl VaultWithdraw {
    /// `"VaultWithdraw"`.
    pub const TRANSACTION_TYPE: &'static str = "VaultWithdraw";

    /// A `VaultWithdraw` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, vault_id: Hash256, amount: Amount) -> Self {
        Self {
            common: Common::new(account),
            vault_id,
            amount,
            destination: None,
            destination_tag: None,
            credential_ids: None,
        }
    }

    /// With `Destination` set (see [`Self::destination`]).
    #[must_use]
    pub fn with_destination(mut self, value: impl Into<String>) -> Self {
        self.destination = Some(value.into());
        self
    }

    /// With `DestinationTag` set (see [`Self::destination_tag`]).
    #[must_use]
    pub fn with_destination_tag(mut self, value: impl Into<u32>) -> Self {
        self.destination_tag = Some(value.into());
        self
    }

    /// With `CredentialIDs` set (see [`Self::credential_ids`]).
    #[must_use]
    pub fn with_credential_ids(mut self, value: impl Into<Vec<Hash256>>) -> Self {
        self.credential_ids = Some(value.into());
        self
    }
}

impl TxModel for VaultWithdraw {
    const TRANSACTION_TYPE: &'static str = "VaultWithdraw";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<VaultWithdraw> for Tx {
    fn from(tx: VaultWithdraw) -> Self {
        Tx::VaultWithdraw(tx)
    }
}

/// [`XChainAccountCreateCommit`](https://xrpl.org/docs/references/protocol/transactions/types/xchainaccountcreatecommit) transaction.
///
/// Required: `XChainBridge`, `Destination`, `Amount`, `SignatureReward`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct XChainAccountCreateCommit {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `XChainBridge`: a cross-chain bridge. Required.
    #[serde(rename = "XChainBridge")]
    pub x_chain_bridge: XChainBridge,
    /// The account that receives it (`Destination`: a classic address). Required.
    #[serde(rename = "Destination")]
    pub destination: String,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// `SignatureReward`: XRP in drops, a token, or an MPT. Required.
    #[serde(rename = "SignatureReward")]
    pub signature_reward: Amount,
}

impl XChainAccountCreateCommit {
    /// `"XChainAccountCreateCommit"`.
    pub const TRANSACTION_TYPE: &'static str = "XChainAccountCreateCommit";

    /// A `XChainAccountCreateCommit` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        x_chain_bridge: XChainBridge,
        destination: impl Into<String>,
        amount: Amount,
        signature_reward: Amount,
    ) -> Self {
        Self {
            common: Common::new(account),
            x_chain_bridge,
            destination: destination.into(),
            amount,
            signature_reward,
        }
    }
}

impl TxModel for XChainAccountCreateCommit {
    const TRANSACTION_TYPE: &'static str = "XChainAccountCreateCommit";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<XChainAccountCreateCommit> for Tx {
    fn from(tx: XChainAccountCreateCommit) -> Self {
        Tx::XChainAccountCreateCommit(tx)
    }
}

/// [`XChainAddAccountCreateAttestation`](https://xrpl.org/docs/references/protocol/transactions/types/xchainaddaccountcreateattestation) transaction.
///
/// Required: `XChainBridge`, `AttestationSignerAccount`, `PublicKey`, `Signature`, `OtherChainSource`, `Amount`, `AttestationRewardAccount`, `WasLockingChainSend`, `XChainAccountCreateCount`, `Destination`, `SignatureReward`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct XChainAddAccountCreateAttestation {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `XChainBridge`: a cross-chain bridge. Required.
    #[serde(rename = "XChainBridge")]
    pub x_chain_bridge: XChainBridge,
    /// `AttestationSignerAccount`: a classic address. Required.
    #[serde(rename = "AttestationSignerAccount")]
    pub attestation_signer_account: String,
    /// A public key, hex (`PublicKey`: bytes, hex). Required.
    #[serde(rename = "PublicKey")]
    pub public_key: String,
    /// A signature, hex (`Signature`: bytes, hex). Required.
    #[serde(rename = "Signature")]
    pub signature: String,
    /// `OtherChainSource`: a classic address. Required.
    #[serde(rename = "OtherChainSource")]
    pub other_chain_source: String,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// `AttestationRewardAccount`: a classic address. Required.
    #[serde(rename = "AttestationRewardAccount")]
    pub attestation_reward_account: String,
    /// `WasLockingChainSend`: a number. Required.
    #[serde(rename = "WasLockingChainSend")]
    pub was_locking_chain_send: u8,
    /// `XChainAccountCreateCount`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "XChainAccountCreateCount")]
    pub x_chain_account_create_count: String,
    /// The account that receives it (`Destination`: a classic address). Required.
    #[serde(rename = "Destination")]
    pub destination: String,
    /// `SignatureReward`: XRP in drops, a token, or an MPT. Required.
    #[serde(rename = "SignatureReward")]
    pub signature_reward: Amount,
}

impl XChainAddAccountCreateAttestation {
    /// `"XChainAddAccountCreateAttestation"`.
    pub const TRANSACTION_TYPE: &'static str = "XChainAddAccountCreateAttestation";

    /// A `XChainAddAccountCreateAttestation` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        x_chain_bridge: XChainBridge,
        attestation_signer_account: impl Into<String>,
        public_key: impl Into<String>,
        signature: impl Into<String>,
        other_chain_source: impl Into<String>,
        amount: Amount,
        attestation_reward_account: impl Into<String>,
        was_locking_chain_send: u8,
        x_chain_account_create_count: impl Into<String>,
        destination: impl Into<String>,
        signature_reward: Amount,
    ) -> Self {
        Self {
            common: Common::new(account),
            x_chain_bridge,
            attestation_signer_account: attestation_signer_account.into(),
            public_key: public_key.into(),
            signature: signature.into(),
            other_chain_source: other_chain_source.into(),
            amount,
            attestation_reward_account: attestation_reward_account.into(),
            was_locking_chain_send,
            x_chain_account_create_count: x_chain_account_create_count.into(),
            destination: destination.into(),
            signature_reward,
        }
    }
}

impl TxModel for XChainAddAccountCreateAttestation {
    const TRANSACTION_TYPE: &'static str = "XChainAddAccountCreateAttestation";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<XChainAddAccountCreateAttestation> for Tx {
    fn from(tx: XChainAddAccountCreateAttestation) -> Self {
        Tx::XChainAddAccountCreateAttestation(tx)
    }
}

/// [`XChainAddClaimAttestation`](https://xrpl.org/docs/references/protocol/transactions/types/xchainaddclaimattestation) transaction.
///
/// Required: `XChainBridge`, `AttestationSignerAccount`, `PublicKey`, `Signature`, `OtherChainSource`, `Amount`, `AttestationRewardAccount`, `WasLockingChainSend`, `XChainClaimID`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct XChainAddClaimAttestation {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `XChainBridge`: a cross-chain bridge. Required.
    #[serde(rename = "XChainBridge")]
    pub x_chain_bridge: XChainBridge,
    /// `AttestationSignerAccount`: a classic address. Required.
    #[serde(rename = "AttestationSignerAccount")]
    pub attestation_signer_account: String,
    /// A public key, hex (`PublicKey`: bytes, hex). Required.
    #[serde(rename = "PublicKey")]
    pub public_key: String,
    /// A signature, hex (`Signature`: bytes, hex). Required.
    #[serde(rename = "Signature")]
    pub signature: String,
    /// `OtherChainSource`: a classic address. Required.
    #[serde(rename = "OtherChainSource")]
    pub other_chain_source: String,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// `AttestationRewardAccount`: a classic address. Required.
    #[serde(rename = "AttestationRewardAccount")]
    pub attestation_reward_account: String,
    /// `WasLockingChainSend`: a number. Required.
    #[serde(rename = "WasLockingChainSend")]
    pub was_locking_chain_send: u8,
    /// `XChainClaimID`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "XChainClaimID")]
    pub x_chain_claim_id: String,
    /// The account that receives it (`Destination`: a classic address). Optional.
    #[serde(
        rename = "Destination",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination: Option<String>,
}

impl XChainAddClaimAttestation {
    /// `"XChainAddClaimAttestation"`.
    pub const TRANSACTION_TYPE: &'static str = "XChainAddClaimAttestation";

    /// A `XChainAddClaimAttestation` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        x_chain_bridge: XChainBridge,
        attestation_signer_account: impl Into<String>,
        public_key: impl Into<String>,
        signature: impl Into<String>,
        other_chain_source: impl Into<String>,
        amount: Amount,
        attestation_reward_account: impl Into<String>,
        was_locking_chain_send: u8,
        x_chain_claim_id: impl Into<String>,
    ) -> Self {
        Self {
            common: Common::new(account),
            x_chain_bridge,
            attestation_signer_account: attestation_signer_account.into(),
            public_key: public_key.into(),
            signature: signature.into(),
            other_chain_source: other_chain_source.into(),
            amount,
            attestation_reward_account: attestation_reward_account.into(),
            was_locking_chain_send,
            x_chain_claim_id: x_chain_claim_id.into(),
            destination: None,
        }
    }

    /// With `Destination` set (see [`Self::destination`]).
    #[must_use]
    pub fn with_destination(mut self, value: impl Into<String>) -> Self {
        self.destination = Some(value.into());
        self
    }
}

impl TxModel for XChainAddClaimAttestation {
    const TRANSACTION_TYPE: &'static str = "XChainAddClaimAttestation";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<XChainAddClaimAttestation> for Tx {
    fn from(tx: XChainAddClaimAttestation) -> Self {
        Tx::XChainAddClaimAttestation(tx)
    }
}

/// [`XChainClaim`](https://xrpl.org/docs/references/protocol/transactions/types/xchainclaim) transaction.
///
/// Required: `XChainBridge`, `XChainClaimID`, `Destination`, `Amount`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct XChainClaim {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `XChainBridge`: a cross-chain bridge. Required.
    #[serde(rename = "XChainBridge")]
    pub x_chain_bridge: XChainBridge,
    /// `XChainClaimID`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "XChainClaimID")]
    pub x_chain_claim_id: String,
    /// The account that receives it (`Destination`: a classic address). Required.
    #[serde(rename = "Destination")]
    pub destination: String,
    /// A tag the destination uses to tell its senders apart (`DestinationTag`: a number). Optional.
    #[serde(
        rename = "DestinationTag",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destination_tag: Option<u32>,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
}

impl XChainClaim {
    /// `"XChainClaim"`.
    pub const TRANSACTION_TYPE: &'static str = "XChainClaim";

    /// A `XChainClaim` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        x_chain_bridge: XChainBridge,
        x_chain_claim_id: impl Into<String>,
        destination: impl Into<String>,
        amount: Amount,
    ) -> Self {
        Self {
            common: Common::new(account),
            x_chain_bridge,
            x_chain_claim_id: x_chain_claim_id.into(),
            destination: destination.into(),
            destination_tag: None,
            amount,
        }
    }

    /// With `DestinationTag` set (see [`Self::destination_tag`]).
    #[must_use]
    pub fn with_destination_tag(mut self, value: impl Into<u32>) -> Self {
        self.destination_tag = Some(value.into());
        self
    }
}

impl TxModel for XChainClaim {
    const TRANSACTION_TYPE: &'static str = "XChainClaim";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<XChainClaim> for Tx {
    fn from(tx: XChainClaim) -> Self {
        Tx::XChainClaim(tx)
    }
}

/// [`XChainCommit`](https://xrpl.org/docs/references/protocol/transactions/types/xchaincommit) transaction.
///
/// Required: `XChainBridge`, `XChainClaimID`, `Amount`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct XChainCommit {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `XChainBridge`: a cross-chain bridge. Required.
    #[serde(rename = "XChainBridge")]
    pub x_chain_bridge: XChainBridge,
    /// `XChainClaimID`: a 64-bit number as rippled writes it (hex; base 10 for MPT amounts). Required.
    #[serde(rename = "XChainClaimID")]
    pub x_chain_claim_id: String,
    /// How much (`Amount`: XRP in drops, a token, or an MPT). Required.
    #[serde(rename = "Amount")]
    pub amount: Amount,
    /// `OtherChainDestination`: a classic address. Optional.
    #[serde(
        rename = "OtherChainDestination",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub other_chain_destination: Option<String>,
}

impl XChainCommit {
    /// `"XChainCommit"`.
    pub const TRANSACTION_TYPE: &'static str = "XChainCommit";

    /// A `XChainCommit` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        x_chain_bridge: XChainBridge,
        x_chain_claim_id: impl Into<String>,
        amount: Amount,
    ) -> Self {
        Self {
            common: Common::new(account),
            x_chain_bridge,
            x_chain_claim_id: x_chain_claim_id.into(),
            amount,
            other_chain_destination: None,
        }
    }

    /// With `OtherChainDestination` set (see [`Self::other_chain_destination`]).
    #[must_use]
    pub fn with_other_chain_destination(mut self, value: impl Into<String>) -> Self {
        self.other_chain_destination = Some(value.into());
        self
    }
}

impl TxModel for XChainCommit {
    const TRANSACTION_TYPE: &'static str = "XChainCommit";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<XChainCommit> for Tx {
    fn from(tx: XChainCommit) -> Self {
        Tx::XChainCommit(tx)
    }
}

/// [`XChainCreateBridge`](https://xrpl.org/docs/references/protocol/transactions/types/xchaincreatebridge) transaction.
///
/// Required: `XChainBridge`, `SignatureReward`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct XChainCreateBridge {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `XChainBridge`: a cross-chain bridge. Required.
    #[serde(rename = "XChainBridge")]
    pub x_chain_bridge: XChainBridge,
    /// `SignatureReward`: XRP in drops, a token, or an MPT. Required.
    #[serde(rename = "SignatureReward")]
    pub signature_reward: Amount,
    /// `MinAccountCreateAmount`: XRP in drops, a token, or an MPT. Optional.
    #[serde(
        rename = "MinAccountCreateAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub min_account_create_amount: Option<Amount>,
}

impl XChainCreateBridge {
    /// `"XChainCreateBridge"`.
    pub const TRANSACTION_TYPE: &'static str = "XChainCreateBridge";

    /// A `XChainCreateBridge` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        x_chain_bridge: XChainBridge,
        signature_reward: Amount,
    ) -> Self {
        Self {
            common: Common::new(account),
            x_chain_bridge,
            signature_reward,
            min_account_create_amount: None,
        }
    }

    /// With `MinAccountCreateAmount` set (see [`Self::min_account_create_amount`]).
    #[must_use]
    pub fn with_min_account_create_amount(mut self, value: impl Into<Amount>) -> Self {
        self.min_account_create_amount = Some(value.into());
        self
    }
}

impl TxModel for XChainCreateBridge {
    const TRANSACTION_TYPE: &'static str = "XChainCreateBridge";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<XChainCreateBridge> for Tx {
    fn from(tx: XChainCreateBridge) -> Self {
        Tx::XChainCreateBridge(tx)
    }
}

/// [`XChainCreateClaimID`](https://xrpl.org/docs/references/protocol/transactions/types/xchaincreateclaimid) transaction.
///
/// Required: `XChainBridge`, `SignatureReward`, `OtherChainSource`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct XChainCreateClaimID {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `XChainBridge`: a cross-chain bridge. Required.
    #[serde(rename = "XChainBridge")]
    pub x_chain_bridge: XChainBridge,
    /// `SignatureReward`: XRP in drops, a token, or an MPT. Required.
    #[serde(rename = "SignatureReward")]
    pub signature_reward: Amount,
    /// `OtherChainSource`: a classic address. Required.
    #[serde(rename = "OtherChainSource")]
    pub other_chain_source: String,
}

impl XChainCreateClaimID {
    /// `"XChainCreateClaimID"`.
    pub const TRANSACTION_TYPE: &'static str = "XChainCreateClaimID";

    /// A `XChainCreateClaimID` from `account` with its required fields; everything else unset.
    pub fn new(
        account: impl Into<String>,
        x_chain_bridge: XChainBridge,
        signature_reward: Amount,
        other_chain_source: impl Into<String>,
    ) -> Self {
        Self {
            common: Common::new(account),
            x_chain_bridge,
            signature_reward,
            other_chain_source: other_chain_source.into(),
        }
    }
}

impl TxModel for XChainCreateClaimID {
    const TRANSACTION_TYPE: &'static str = "XChainCreateClaimID";
    type Flags = UniversalFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<XChainCreateClaimID> for Tx {
    fn from(tx: XChainCreateClaimID) -> Self {
        Tx::XChainCreateClaimID(tx)
    }
}

flags_type!(
    /// The flags of a [`XChainModifyBridge`] (`Flags`).
    XChainModifyBridgeFlags {
        /// `tfClearAccountCreateAmount`.
        CLEAR_ACCOUNT_CREATE_AMOUNT = 0x00010000,
        /// `tfFullyCanonicalSig` (any type): set on every transaction since the RequireFullyCanonicalSig amendment.
        FULLY_CANONICAL_SIG = 0x80000000,
        /// `tfInnerBatchTxn` (any type): marks a transaction inside a `Batch` (unsigned, zero fee).
        INNER_BATCH_TXN = 0x40000000,
    }
);

/// [`XChainModifyBridge`](https://xrpl.org/docs/references/protocol/transactions/types/xchainmodifybridge) transaction.
///
/// Required: `XChainBridge`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct XChainModifyBridge {
    /// The fields every transaction has (`Account`, `Fee`, `Sequence`, `Flags`, `Memos`, …).
    #[serde(flatten)]
    pub common: Common,
    /// `XChainBridge`: a cross-chain bridge. Required.
    #[serde(rename = "XChainBridge")]
    pub x_chain_bridge: XChainBridge,
    /// `SignatureReward`: XRP in drops, a token, or an MPT. Optional.
    #[serde(
        rename = "SignatureReward",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub signature_reward: Option<Amount>,
    /// `MinAccountCreateAmount`: XRP in drops, a token, or an MPT. Optional.
    #[serde(
        rename = "MinAccountCreateAmount",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub min_account_create_amount: Option<Amount>,
}

impl XChainModifyBridge {
    /// `"XChainModifyBridge"`.
    pub const TRANSACTION_TYPE: &'static str = "XChainModifyBridge";

    /// A `XChainModifyBridge` from `account` with its required fields; everything else unset.
    pub fn new(account: impl Into<String>, x_chain_bridge: XChainBridge) -> Self {
        Self {
            common: Common::new(account),
            x_chain_bridge,
            signature_reward: None,
            min_account_create_amount: None,
        }
    }

    /// With `SignatureReward` set (see [`Self::signature_reward`]).
    #[must_use]
    pub fn with_signature_reward(mut self, value: impl Into<Amount>) -> Self {
        self.signature_reward = Some(value.into());
        self
    }

    /// With `MinAccountCreateAmount` set (see [`Self::min_account_create_amount`]).
    #[must_use]
    pub fn with_min_account_create_amount(mut self, value: impl Into<Amount>) -> Self {
        self.min_account_create_amount = Some(value.into());
        self
    }
}

impl TxModel for XChainModifyBridge {
    const TRANSACTION_TYPE: &'static str = "XChainModifyBridge";
    type Flags = XChainModifyBridgeFlags;
    fn common(&self) -> &Common {
        &self.common
    }
    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<XChainModifyBridge> for Tx {
    fn from(tx: XChainModifyBridge) -> Self {
        Tx::XChainModifyBridge(tx)
    }
}

/// Every transaction type rippled 3.4.1 defines, by `TransactionType`.
pub const NAMES: &[&str] = &[
    "AMMBid",
    "AMMClawback",
    "AMMCreate",
    "AMMDelete",
    "AMMDeposit",
    "AMMVote",
    "AMMWithdraw",
    "AccountDelete",
    "AccountSet",
    "Batch",
    "CheckCancel",
    "CheckCash",
    "CheckCreate",
    "Clawback",
    "ConfidentialMPTClawback",
    "ConfidentialMPTConvert",
    "ConfidentialMPTConvertBack",
    "ConfidentialMPTMergeInbox",
    "ConfidentialMPTSend",
    "CredentialAccept",
    "CredentialCreate",
    "CredentialDelete",
    "DIDDelete",
    "DIDSet",
    "DelegateSet",
    "DepositPreauth",
    "EnableAmendment",
    "EscrowCancel",
    "EscrowCreate",
    "EscrowFinish",
    "LedgerStateFix",
    "LoanBrokerCoverClawback",
    "LoanBrokerCoverDeposit",
    "LoanBrokerCoverWithdraw",
    "LoanBrokerDelete",
    "LoanBrokerSet",
    "LoanDelete",
    "LoanManage",
    "LoanPay",
    "LoanSet",
    "MPTokenAuthorize",
    "MPTokenIssuanceCreate",
    "MPTokenIssuanceDestroy",
    "MPTokenIssuanceSet",
    "NFTokenAcceptOffer",
    "NFTokenBurn",
    "NFTokenCancelOffer",
    "NFTokenCreateOffer",
    "NFTokenMint",
    "NFTokenModify",
    "OfferCancel",
    "OfferCreate",
    "OracleDelete",
    "OracleSet",
    "Payment",
    "PaymentChannelClaim",
    "PaymentChannelCreate",
    "PaymentChannelFund",
    "PermissionedDomainDelete",
    "PermissionedDomainSet",
    "SetFee",
    "SetRegularKey",
    "SignerListSet",
    "SponsorshipSet",
    "SponsorshipTransfer",
    "TicketCreate",
    "TrustSet",
    "UNLModify",
    "VaultClawback",
    "VaultCreate",
    "VaultDelete",
    "VaultDeposit",
    "VaultSet",
    "VaultWithdraw",
    "XChainAccountCreateCommit",
    "XChainAddAccountCreateAttestation",
    "XChainAddClaimAttestation",
    "XChainClaim",
    "XChainCommit",
    "XChainCreateBridge",
    "XChainCreateClaimID",
    "XChainModifyBridge",
];

macro_rules! each_transaction_type {
    ($m:ident) => {
        $m!(
            AMMBid,
            AMMClawback,
            AMMCreate,
            AMMDelete,
            AMMDeposit,
            AMMVote,
            AMMWithdraw,
            AccountDelete,
            AccountSet,
            Batch,
            CheckCancel,
            CheckCash,
            CheckCreate,
            Clawback,
            ConfidentialMPTClawback,
            ConfidentialMPTConvert,
            ConfidentialMPTConvertBack,
            ConfidentialMPTMergeInbox,
            ConfidentialMPTSend,
            CredentialAccept,
            CredentialCreate,
            CredentialDelete,
            DIDDelete,
            DIDSet,
            DelegateSet,
            DepositPreauth,
            EnableAmendment,
            EscrowCancel,
            EscrowCreate,
            EscrowFinish,
            LedgerStateFix,
            LoanBrokerCoverClawback,
            LoanBrokerCoverDeposit,
            LoanBrokerCoverWithdraw,
            LoanBrokerDelete,
            LoanBrokerSet,
            LoanDelete,
            LoanManage,
            LoanPay,
            LoanSet,
            MPTokenAuthorize,
            MPTokenIssuanceCreate,
            MPTokenIssuanceDestroy,
            MPTokenIssuanceSet,
            NFTokenAcceptOffer,
            NFTokenBurn,
            NFTokenCancelOffer,
            NFTokenCreateOffer,
            NFTokenMint,
            NFTokenModify,
            OfferCancel,
            OfferCreate,
            OracleDelete,
            OracleSet,
            Payment,
            PaymentChannelClaim,
            PaymentChannelCreate,
            PaymentChannelFund,
            PermissionedDomainDelete,
            PermissionedDomainSet,
            SetFee,
            SetRegularKey,
            SignerListSet,
            SponsorshipSet,
            SponsorshipTransfer,
            TicketCreate,
            TrustSet,
            UNLModify,
            VaultClawback,
            VaultCreate,
            VaultDelete,
            VaultDeposit,
            VaultSet,
            VaultWithdraw,
            XChainAccountCreateCommit,
            XChainAddAccountCreateAttestation,
            XChainAddClaimAttestation,
            XChainClaim,
            XChainCommit,
            XChainCreateBridge,
            XChainCreateClaimID,
            XChainModifyBridge
        );
    };
}
pub(super) use each_transaction_type;
