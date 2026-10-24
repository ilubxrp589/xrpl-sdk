//! The checks `rippled` makes before it applies a transaction (its
//! `preflight`) that need nothing from the ledger, for the types that have
//! them: what [`TxModel::validate`](super::TxModel::validate) runs. A
//! transaction that fails one would be refused (`tem…`); checking here says
//! why before anything is signed or sent.

use super::*;
use crate::flags::Flags;

/// The largest AMM trading fee: 1000 (1%, in 1/100,000).
const MAX_TRADING_FEE: u16 = 1000;

fn invalid(tx: &str, message: impl std::fmt::Display) -> CoreError {
    CoreError::ValidationError(format!("{tx}: {message}"))
}

/// Whether two amounts are of the same asset (XRP, a token by currency and
/// issuer, or an MPT by issuance).
fn same_asset(a: &Amount, b: &Amount) -> bool {
    match (a, b) {
        (Amount::Xrp(_), Amount::Xrp(_)) => true,
        (
            Amount::Iou {
                currency: c1,
                issuer: i1,
                ..
            },
            Amount::Iou {
                currency: c2,
                issuer: i2,
                ..
            },
        ) => c1 == c2 && i1 == i2,
        (Amount::Mpt { issuance_id: a, .. }, Amount::Mpt { issuance_id: b, .. }) => a == b,
        _ => false,
    }
}

/// `field` (hex) is at most `max_bytes` bytes.
fn check_hex(
    tx: &str,
    value: Option<&str>,
    field: &str,
    max_bytes: usize,
) -> Result<(), CoreError> {
    match value.map(hex::decode) {
        Some(Ok(bytes)) if bytes.len() > max_bytes => Err(invalid(
            tx,
            format!(
                "{field} is at most {max_bytes} bytes ({} hex digits)",
                2 * max_bytes
            ),
        )),
        Some(Err(_)) => Err(invalid(tx, format!("{field} must be hex"))),
        _ => Ok(()),
    }
}

pub(super) fn account_set(tx: &AccountSet) -> Result<(), CoreError> {
    let bad = |m: &str| invalid("AccountSet", m);
    if tx.set_flag.is_some() && tx.set_flag == tx.clear_flag {
        return Err(bad("SetFlag and ClearFlag are the same flag"));
    }
    if tx
        .tick_size
        .is_some_and(|t| t != 0 && !(3..=15).contains(&t))
    {
        return Err(bad("TickSize is 0 (clear) or 3 to 15"));
    }
    if tx
        .transfer_rate
        .is_some_and(|r| r != 0 && !(1_000_000_000..=2_000_000_000).contains(&r))
    {
        return Err(bad(
            "TransferRate is 0 (none) or 1,000,000,000 to 2,000,000,000 (0% to 100%)",
        ));
    }
    check_hex("AccountSet", tx.domain.as_deref(), "Domain", 256)?;
    check_hex("AccountSet", tx.message_key.as_deref(), "MessageKey", 33)
}

pub(super) fn amm_create(tx: &AMMCreate) -> Result<(), CoreError> {
    if tx.trading_fee > MAX_TRADING_FEE {
        return Err(invalid("AMMCreate", "TradingFee is at most 1000 (1%)"));
    }
    if same_asset(&tx.amount, &tx.amount2) {
        return Err(invalid(
            "AMMCreate",
            "the two amounts must be different assets",
        ));
    }
    Ok(())
}

/// The one mode flag among `modes` set in `flags`.
fn one_mode<F: Flags>(tx: &str, flags: F, modes: F) -> Result<F, CoreError> {
    let mode = flags.bits() & modes.bits();
    if mode.count_ones() == 1 {
        Ok(F::from_bits(mode))
    } else {
        Err(invalid(
            tx,
            "exactly one mode flag (tfLPToken, tfSingleAsset, tfTwoAsset, …)",
        ))
    }
}

pub(super) fn amm_deposit(tx: &AMMDeposit) -> Result<(), CoreError> {
    type D = AMMDepositFlags;
    let modes = D::LP_TOKEN
        | D::SINGLE_ASSET
        | D::TWO_ASSET
        | D::ONE_ASSET_LP_TOKEN
        | D::LIMIT_LP_TOKEN
        | D::TWO_ASSET_IF_EMPTY;
    let mode = one_mode("AMMDeposit", tx.flags(), modes)?;
    let (a, a2, lp, ep) = (
        tx.amount.is_some(),
        tx.amount2.is_some(),
        tx.lp_token_out.is_some(),
        tx.e_price.is_some(),
    );
    let fits = match mode {
        D::LP_TOKEN => lp && a == a2 && !ep,
        D::SINGLE_ASSET => a && !a2 && !lp && !ep,
        D::TWO_ASSET | D::TWO_ASSET_IF_EMPTY => a && a2 && !lp && !ep,
        D::ONE_ASSET_LP_TOKEN => a && lp && !a2 && !ep,
        D::LIMIT_LP_TOKEN => a && ep && !a2 && !lp,
        _ => false,
    };
    if !fits {
        return Err(invalid(
            "AMMDeposit",
            "the amounts set do not fit the deposit mode flag",
        ));
    }
    if tx.trading_fee.is_some() && mode != D::TWO_ASSET_IF_EMPTY {
        return Err(invalid(
            "AMMDeposit",
            "TradingFee goes only with tfTwoAssetIfEmpty",
        ));
    }
    if tx.trading_fee.is_some_and(|f| f > MAX_TRADING_FEE) {
        return Err(invalid("AMMDeposit", "TradingFee is at most 1000 (1%)"));
    }
    Ok(())
}

pub(super) fn amm_withdraw(tx: &AMMWithdraw) -> Result<(), CoreError> {
    type W = AMMWithdrawFlags;
    let modes = W::LP_TOKEN
        | W::WITHDRAW_ALL
        | W::ONE_ASSET_WITHDRAW_ALL
        | W::SINGLE_ASSET
        | W::TWO_ASSET
        | W::ONE_ASSET_LP_TOKEN
        | W::LIMIT_LP_TOKEN;
    let mode = one_mode("AMMWithdraw", tx.flags(), modes)?;
    let (a, a2, lp, ep) = (
        tx.amount.is_some(),
        tx.amount2.is_some(),
        tx.lp_token_in.is_some(),
        tx.e_price.is_some(),
    );
    let fits = match mode {
        W::LP_TOKEN => lp && a == a2 && !ep,
        W::WITHDRAW_ALL => !a && !a2 && !lp && !ep,
        W::ONE_ASSET_WITHDRAW_ALL | W::SINGLE_ASSET => a && !a2 && !lp && !ep,
        W::TWO_ASSET => a && a2 && !lp && !ep,
        W::ONE_ASSET_LP_TOKEN => a && lp && !a2 && !ep,
        W::LIMIT_LP_TOKEN => a && ep && !a2 && !lp,
        _ => false,
    };
    if fits {
        Ok(())
    } else {
        Err(invalid(
            "AMMWithdraw",
            "the amounts set do not fit the withdrawal mode flag",
        ))
    }
}

pub(super) fn amm_vote(tx: &AMMVote) -> Result<(), CoreError> {
    if tx.trading_fee > MAX_TRADING_FEE {
        return Err(invalid("AMMVote", "TradingFee is at most 1000 (1%)"));
    }
    Ok(())
}

pub(super) fn amm_bid(tx: &AMMBid) -> Result<(), CoreError> {
    if tx.auth_accounts.as_ref().is_some_and(|a| a.len() > 4) {
        return Err(invalid("AMMBid", "at most 4 AuthAccounts"));
    }
    Ok(())
}

pub(super) fn clawback(tx: &Clawback) -> Result<(), CoreError> {
    let bad = |m: &str| invalid("Clawback", m);
    let holder = match &tx.amount {
        Amount::Xrp(_) => {
            return Err(bad(
                "XRP cannot be clawed back: Amount is a token or an MPT",
            ))
        }
        Amount::Iou { value, issuer, .. } => {
            if value.is_zero() || value.is_negative {
                return Err(bad("the amount must be positive"));
            }
            if tx.holder.is_some() {
                return Err(bad(
                    "a token clawback names the holder in Amount's issuer, not Holder",
                ));
            }
            issuer.to_address()
        }
        Amount::Mpt { value, .. } => {
            if *value <= 0 {
                return Err(bad("the amount must be positive"));
            }
            let holder = tx
                .holder
                .as_deref()
                .ok_or_else(|| bad("an MPT clawback names the holder in Holder"))?;
            crate::address::decode_account_id(holder)?;
            holder.to_string()
        }
    };
    if holder == tx.common.account {
        return Err(bad("the holder cannot be the issuer itself"));
    }
    Ok(())
}

pub(super) fn delegate_set(tx: &DelegateSet) -> Result<(), CoreError> {
    let bad = |m: &str| invalid("DelegateSet", m);
    if tx.authorize == tx.common.account {
        return Err(bad("an account cannot delegate to itself"));
    }
    crate::address::decode_account_id(&tx.authorize)?;
    if tx.permissions.len() > 10 {
        return Err(bad("at most 10 permissions"));
    }
    for (i, p) in tx.permissions.iter().enumerate() {
        if tx.permissions.iter().skip(i + 1).any(|q| q == p) {
            return Err(bad("a permission is listed twice"));
        }
    }
    Ok(())
}

pub(super) fn signer_list_set(tx: &SignerListSet) -> Result<(), CoreError> {
    let bad = |m: String| invalid("SignerListSet", m);
    let entries = tx.signer_entries.as_deref().unwrap_or_default();
    if tx.signer_quorum == 0 {
        // A quorum of 0 deletes the list, which then has no entries.
        return if entries.is_empty() {
            Ok(())
        } else {
            Err(bad(
                "a SignerQuorum of 0 deletes the list: leave SignerEntries out".into(),
            ))
        };
    }
    if entries.is_empty() || entries.len() > 32 {
        return Err(bad(format!("1 to 32 SignerEntries, got {}", entries.len())));
    }
    let mut seen = Vec::with_capacity(entries.len());
    let mut total: u64 = 0;
    for entry in entries {
        let id = crate::address::decode_account_id(&entry.account)?;
        if entry.account == tx.common.account {
            return Err(bad("the account cannot be on its own signer list".into()));
        }
        if seen.contains(&id) {
            return Err(bad(format!("{} is listed twice", entry.account)));
        }
        seen.push(id);
        if entry.signer_weight == 0 {
            return Err(bad(format!(
                "{}'s SignerWeight is 1 to 65535",
                entry.account
            )));
        }
        total += u64::from(entry.signer_weight);
    }
    if total < u64::from(tx.signer_quorum) {
        return Err(bad(format!(
            "the weights add up to {total}, less than the quorum {}",
            tx.signer_quorum
        )));
    }
    Ok(())
}

pub(super) fn trust_set(tx: &TrustSet) -> Result<(), CoreError> {
    if matches!(tx.limit_amount, Some(Amount::Xrp(_) | Amount::Mpt { .. })) {
        return Err(invalid(
            "TrustSet",
            "LimitAmount is a token amount (currency, issuer and value)",
        ));
    }
    Ok(())
}

pub(super) fn nf_token_accept_offer(tx: &NFTokenAcceptOffer) -> Result<(), CoreError> {
    if tx.nf_token_buy_offer.is_none() && tx.nf_token_sell_offer.is_none() {
        return Err(invalid(
            "NFTokenAcceptOffer",
            "at least one of NFTokenBuyOffer and NFTokenSellOffer is required",
        ));
    }
    Ok(())
}

/// `Batch` (`Batch::preflight`, and `STTx`, which refuses more than eight
/// inner transactions when it reads one): exactly one mode
/// (`ALL_OR_NOTHING`, `ONLY_ONE`, `UNTIL_FAILURE` or `INDEPENDENT`:
/// `temINVALID_FLAG`); 2 to 8 inner transactions (`temARRAY_EMPTY`), none of
/// them a Batch (`temINVALID_INNER_BATCH`), none with both a `Sequence` and
/// a `TicketSequence` (`temSEQ_AND_TICKET`), each passing its own checks.
/// What autofill fills in (an inner transaction's sequence, zero fee, empty
/// signing key and `tfInnerBatchTxn` flag) is not checked here, since
/// `sign_and_submit` checks a transaction before it fills it in.
pub(super) fn batch(tx: &Batch) -> Result<(), CoreError> {
    use crate::transaction::batch::{MAX_INNER_TRANSACTIONS, MIN_INNER_TRANSACTIONS};
    let modes = BatchFlags::ALL_OR_NOTHING
        | BatchFlags::ONLY_ONE
        | BatchFlags::UNTIL_FAILURE
        | BatchFlags::INDEPENDENT;
    if (tx.flags().bits() & modes.bits()).count_ones() != 1 {
        return Err(invalid(
            "Batch",
            "exactly one mode flag (ALL_OR_NOTHING, ONLY_ONE, UNTIL_FAILURE or INDEPENDENT)",
        ));
    }
    let count = tx.raw_transactions.len();
    if !(MIN_INNER_TRANSACTIONS..=MAX_INNER_TRANSACTIONS).contains(&count) {
        return Err(invalid(
            "Batch",
            format!(
                "holds {MIN_INNER_TRANSACTIONS} to {MAX_INNER_TRANSACTIONS} inner transactions, not {count}"
            ),
        ));
    }
    for inner in &tx.raw_transactions {
        if matches!(inner, Tx::Batch(_)) {
            return Err(invalid("Batch", "an inner transaction cannot be a Batch"));
        }
        if let Some(common) = inner.common() {
            if common.ticket_sequence.is_some() && common.sequence.is_some_and(|s| s != 0) {
                return Err(invalid(
                    "Batch",
                    format!(
                        "an inner {} has both a Sequence and a TicketSequence",
                        inner.transaction_type()
                    ),
                ));
            }
        }
        inner.validate()?;
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    const A: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
    const B: &str = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";
    const C: &str = "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1";

    fn usd(value: &str, issuer: &str) -> Amount {
        Amount::token(value, "USD", issuer).unwrap()
    }

    fn refused(r: Result<(), CoreError>, why: &str) {
        let err = r.expect_err(why);
        assert!(err.to_string().contains(why), "{err} (expected {why:?})");
    }

    #[test]
    fn account_set() {
        assert!(AccountSet::new(A).with_tick_size(5u8).validate().is_ok());
        refused(
            AccountSet::new(A).with_tick_size(2u8).validate(),
            "TickSize",
        );
        refused(
            AccountSet::new(A)
                .with_transfer_rate(999_999_999u32)
                .validate(),
            "TransferRate",
        );
        assert!(AccountSet::new(A)
            .with_transfer_rate(0u32)
            .validate()
            .is_ok());
        refused(
            AccountSet::new(A)
                .with_set_flag(8u32)
                .with_clear_flag(8u32)
                .validate(),
            "same flag",
        );
        refused(AccountSet::new(A).with_domain("ZZ").validate(), "hex");
        refused(
            AccountSet::new(A).with_domain("00".repeat(257)).validate(),
            "256 bytes",
        );
    }

    #[test]
    fn amm() {
        let (xrp, token) = (Asset::xrp(), Asset::token("USD", B).unwrap());
        assert!(AMMCreate::new(A, Amount::Xrp(10), usd("1", B), 500)
            .validate()
            .is_ok());
        refused(
            AMMCreate::new(A, Amount::Xrp(10), usd("1", B), 1001).validate(),
            "TradingFee",
        );
        refused(
            AMMCreate::new(A, usd("2", B), usd("1", B), 5).validate(),
            "different assets",
        );
        // Same currency, another issuer: a different asset.
        assert!(AMMCreate::new(A, usd("2", C), usd("1", B), 5)
            .validate()
            .is_ok());

        let deposit = || AMMDeposit::new(A, xrp.clone(), token.clone());
        assert!(deposit()
            .with_amount(Amount::Xrp(5))
            .with_flags(AMMDepositFlags::SINGLE_ASSET)
            .validate()
            .is_ok());
        refused(
            deposit().with_amount(Amount::Xrp(5)).validate(),
            "exactly one mode flag",
        );
        refused(
            deposit()
                .with_amount(Amount::Xrp(5))
                .with_flags(AMMDepositFlags::TWO_ASSET)
                .validate(),
            "do not fit",
        );
        refused(
            deposit()
                .with_amount(Amount::Xrp(5))
                .with_trading_fee(10u16)
                .with_flags(AMMDepositFlags::SINGLE_ASSET)
                .validate(),
            "tfTwoAssetIfEmpty",
        );

        let withdraw = || AMMWithdraw::new(A, xrp.clone(), token.clone());
        assert!(withdraw()
            .with_flags(AMMWithdrawFlags::WITHDRAW_ALL)
            .validate()
            .is_ok());
        refused(
            withdraw()
                .with_amount(Amount::Xrp(5))
                .with_flags(AMMWithdrawFlags::WITHDRAW_ALL)
                .validate(),
            "do not fit",
        );
        refused(
            AMMVote::new(A, xrp.clone(), token.clone(), 1500).validate(),
            "TradingFee",
        );
        let five = vec![AuthAccount::new(B); 5];
        refused(
            AMMBid::new(A, xrp, token)
                .with_auth_accounts(five)
                .validate(),
            "at most 4",
        );
    }

    #[test]
    fn clawback() {
        assert!(Clawback::new(A, usd("5", B)).validate().is_ok());
        refused(
            Clawback::new(A, Amount::Xrp(5)).validate(),
            "XRP cannot be clawed back",
        );
        refused(Clawback::new(A, usd("0", B)).validate(), "positive");
        refused(Clawback::new(A, usd("5", A)).validate(), "issuer itself");
        refused(
            Clawback::new(A, usd("5", B)).with_holder(C).validate(),
            "not Holder",
        );
        let mpt = Amount::mpt(5, "000004C463C52827307480341125DA0577DEFC38405B0E3E").unwrap();
        refused(
            Clawback::new(A, mpt.clone()).validate(),
            "names the holder in Holder",
        );
        assert!(Clawback::new(A, mpt).with_holder(B).validate().is_ok());
    }

    #[test]
    fn delegate_set() {
        let p = |v: &str| Permission::new(v);
        assert!(DelegateSet::new(A, B, vec![p("Payment")])
            .validate()
            .is_ok());
        refused(
            DelegateSet::new(A, A, vec![p("Payment")]).validate(),
            "itself",
        );
        refused(
            DelegateSet::new(A, B, vec![p("Payment"), p("Payment")]).validate(),
            "twice",
        );
        refused(
            DelegateSet::new(A, B, vec![p("Payment"); 11]).validate(),
            "at most 10",
        );
    }

    #[test]
    fn signer_list_set() {
        let entries = |e: &[(&str, u16)]| {
            e.iter()
                .map(|(a, w)| SignerEntry::new(*a, *w))
                .collect::<Vec<_>>()
        };
        assert!(SignerListSet::new(A, 2)
            .with_signer_entries(entries(&[(B, 1), (C, 1)]))
            .validate()
            .is_ok());
        assert!(SignerListSet::new(A, 0).validate().is_ok());
        refused(
            SignerListSet::new(A, 0)
                .with_signer_entries(entries(&[(B, 1)]))
                .validate(),
            "deletes the list",
        );
        refused(
            SignerListSet::new(A, 3)
                .with_signer_entries(entries(&[(B, 1), (C, 1)]))
                .validate(),
            "less than the quorum",
        );
        refused(
            SignerListSet::new(A, 1)
                .with_signer_entries(entries(&[(B, 1), (B, 1)]))
                .validate(),
            "twice",
        );
        refused(
            SignerListSet::new(A, 1)
                .with_signer_entries(entries(&[(A, 1)]))
                .validate(),
            "its own",
        );
        refused(
            SignerListSet::new(A, 1)
                .with_signer_entries(entries(&[(B, 0)]))
                .validate(),
            "SignerWeight",
        );
        refused(SignerListSet::new(A, 1).validate(), "1 to 32");
    }

    #[test]
    fn trust_set_and_nft_offers() {
        assert!(TrustSet::new(A)
            .with_limit_amount(usd("100", B))
            .validate()
            .is_ok());
        refused(
            TrustSet::new(A)
                .with_limit_amount(Amount::Xrp(5))
                .validate(),
            "token amount",
        );
        refused(NFTokenAcceptOffer::new(A).validate(), "at least one");
        assert!(NFTokenAcceptOffer::new(A)
            .with_nf_token_sell_offer(crate::types::Hash256::ZERO)
            .validate()
            .is_ok());
    }

    #[test]
    fn a_tx_runs_its_type_check() {
        let vote: Tx = AMMVote::new(A, Asset::xrp(), Asset::token("USD", B).unwrap(), 1500).into();
        refused(vote.validate(), "TradingFee");
        assert!(Tx::Other(serde_json::json!({"TransactionType": "Future"}))
            .validate()
            .is_ok());
    }

    #[test]
    fn batch() {
        let pay = |from: &str, to: &str| -> Tx {
            Payment::new(from, to, Amount::xrp(1_000_000).unwrap()).into()
        };
        let batch = |inner: Vec<Tx>| Batch::new(A, inner).with_flags(BatchFlags::ALL_OR_NOTHING);
        // Before autofill: no inner sequence, fee or flag yet, and that is fine.
        batch(vec![pay(A, B), pay(B, A)]).validate().unwrap();
        batch(vec![pay(A, B); 8]).validate().unwrap();
        refused(
            batch(vec![pay(A, B)]).validate(),
            "2 to 8 inner transactions, not 1",
        );
        refused(batch(vec![pay(A, B); 9]).validate(), "not 9");
        refused(
            Batch::new(A, vec![pay(A, B), pay(B, A)]).validate(),
            "exactly one mode flag",
        );
        refused(
            Batch::new(A, vec![pay(A, B), pay(B, A)])
                .with_flags(BatchFlags::ALL_OR_NOTHING | BatchFlags::INDEPENDENT)
                .validate(),
            "exactly one mode flag",
        );
        let nested: Tx = batch(vec![pay(A, B), pay(B, A)]).into();
        refused(
            batch(vec![pay(A, B), nested]).validate(),
            "cannot be a Batch",
        );
        let both: Tx = Payment::new(B, A, Amount::xrp(1).unwrap())
            .with_sequence(5)
            .with_ticket_sequence(9)
            .into();
        // with_ticket_sequence sets Sequence 0, as a ticketed transaction has.
        batch(vec![pay(A, B), both.clone()]).validate().unwrap();
        let mut both = both;
        both.common_mut().unwrap().sequence = Some(5);
        refused(
            batch(vec![pay(A, B), both]).validate(),
            "both a Sequence and a TicketSequence",
        );
        // Each inner transaction's own checks run too.
        let vote: Tx = AMMVote::new(A, Asset::xrp(), Asset::token("USD", B).unwrap(), 1500).into();
        refused(batch(vec![pay(A, B), vote]).validate(), "TradingFee");
    }
}
