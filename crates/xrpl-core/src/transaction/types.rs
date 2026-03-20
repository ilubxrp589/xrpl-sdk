/// XRPL transaction type discriminants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum TransactionType {
    Payment = 0,
    EscrowCreate = 1,
    EscrowFinish = 2,
    AccountSet = 3,
    EscrowCancel = 4,
    SetRegularKey = 5,
    OfferCreate = 7,
    OfferCancel = 8,
    CheckCreate = 16,
    CheckCash = 17,
    CheckCancel = 18,
    DepositPreauth = 19,
    TrustSet = 20,
    AccountDelete = 21,
    NFTokenMint = 25,
    NFTokenBurn = 26,
    NFTokenCreateOffer = 27,
    NFTokenCancelOffer = 28,
    NFTokenAcceptOffer = 29,
    AMMCreate = 35,
    AMMDeposit = 36,
    AMMWithdraw = 37,
    AMMVote = 38,
    AMMBid = 39,
    AMMDelete = 40,
}

impl TransactionType {
    pub fn code(self) -> u16 {
        self as u16
    }

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
