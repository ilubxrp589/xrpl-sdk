/// XRPL transaction type discriminants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum TransactionType {
    /// The `Payment` transaction type.
    Payment = 0,
    /// The `EscrowCreate` transaction type.
    EscrowCreate = 1,
    /// The `EscrowFinish` transaction type.
    EscrowFinish = 2,
    /// The `AccountSet` transaction type.
    AccountSet = 3,
    /// The `EscrowCancel` transaction type.
    EscrowCancel = 4,
    /// The `SetRegularKey` transaction type.
    SetRegularKey = 5,
    /// The `OfferCreate` transaction type.
    OfferCreate = 7,
    /// The `OfferCancel` transaction type.
    OfferCancel = 8,
    /// The `CheckCreate` transaction type.
    CheckCreate = 16,
    /// The `CheckCash` transaction type.
    CheckCash = 17,
    /// The `CheckCancel` transaction type.
    CheckCancel = 18,
    /// The `DepositPreauth` transaction type.
    DepositPreauth = 19,
    /// The `TrustSet` transaction type.
    TrustSet = 20,
    /// The `AccountDelete` transaction type.
    AccountDelete = 21,
    /// The `NFTokenMint` transaction type.
    NFTokenMint = 25,
    /// The `NFTokenBurn` transaction type.
    NFTokenBurn = 26,
    /// The `NFTokenCreateOffer` transaction type.
    NFTokenCreateOffer = 27,
    /// The `NFTokenCancelOffer` transaction type.
    NFTokenCancelOffer = 28,
    /// The `NFTokenAcceptOffer` transaction type.
    NFTokenAcceptOffer = 29,
    /// The `AMMCreate` transaction type.
    AMMCreate = 35,
    /// The `AMMDeposit` transaction type.
    AMMDeposit = 36,
    /// The `AMMWithdraw` transaction type.
    AMMWithdraw = 37,
    /// The `AMMVote` transaction type.
    AMMVote = 38,
    /// The `AMMBid` transaction type.
    AMMBid = 39,
    /// The `AMMDelete` transaction type.
    AMMDelete = 40,
}

impl TransactionType {
    /// The `TransactionType` code.
    pub fn code(self) -> u16 {
        self as u16
    }

    /// The `TransactionType` name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Payment => "Payment",
            Self::EscrowCreate => "EscrowCreate",
            Self::EscrowFinish => "EscrowFinish",
            Self::AccountSet => "AccountSet",
            Self::EscrowCancel => "EscrowCancel",
            Self::SetRegularKey => "SetRegularKey",
            Self::OfferCreate => "OfferCreate",
            Self::OfferCancel => "OfferCancel",
            Self::CheckCreate => "CheckCreate",
            Self::CheckCash => "CheckCash",
            Self::CheckCancel => "CheckCancel",
            Self::DepositPreauth => "DepositPreauth",
            Self::TrustSet => "TrustSet",
            Self::AccountDelete => "AccountDelete",
            Self::NFTokenMint => "NFTokenMint",
            Self::NFTokenBurn => "NFTokenBurn",
            Self::NFTokenCreateOffer => "NFTokenCreateOffer",
            Self::NFTokenCancelOffer => "NFTokenCancelOffer",
            Self::NFTokenAcceptOffer => "NFTokenAcceptOffer",
            Self::AMMCreate => "AMMCreate",
            Self::AMMDeposit => "AMMDeposit",
            Self::AMMWithdraw => "AMMWithdraw",
            Self::AMMVote => "AMMVote",
            Self::AMMBid => "AMMBid",
            Self::AMMDelete => "AMMDelete",
        }
    }
}
