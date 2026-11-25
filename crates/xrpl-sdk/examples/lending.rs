//! Cookbook: a loan from a vault (XLS-66, the `LendingProtocol` amendment),
//! from start to finish.
//!
//! Alice runs a lending business: she creates an XRP vault, deposits 50 XRP
//! into it and sets up a loan broker that lends from it. Carol borrows 10 XRP
//! for two payment periods. A loan needs both parties' signatures in one
//! transaction: Carol sends the `LoanSet`, and Alice, as the broker's owner,
//! approves it with a `CounterpartySignature`. Carol then repays each
//! installment, the paid-off loan is deleted, and Alice winds everything down
//! and takes her XRP, with the interest, back out of the vault.
//!
//! Since `LendingProtocolV1_1`, a new loan broker can be created only on a
//! **closed-ended** vault, which lives through three phases set by two dates:
//!
//! | phase | when | deposits | loans | withdrawals |
//! |---|---|---|---|---|
//! | subscription | until `SubscriptionDate` | yes | no (`tecTOO_SOON`) | yes |
//! | investment | until `RedemptionDate` | no | yes, ending 60 s before it | no |
//! | redemption | after `RedemptionDate` | no | no (`tecEXPIRED`) | yes |
//!
//! The investment phase lasts at least 180 seconds. This example uses a
//! short one, so it waits about seven minutes in all, following the ledger's
//! clock.
//!
//! Runs on Devnet with faucet wallets (lending is not on Testnet or Mainnet
//! yet): `cargo run --example lending`
#![allow(
    clippy::indexing_slicing,
    reason = "reading serde_json values with [] gives Null, never a panic"
)]

use serde_json::{json, Value};
use xrpl_sdk::models::{
    LoanBrokerDelete, LoanBrokerSet, LoanDelete, LoanPay, LoanSet, VaultCreate, VaultDelete,
    VaultDeposit, VaultWithdraw,
};
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let net = FaucetNetwork::Devnet;
    let client = XrplHttpClient::new(net.rpc_url())?;
    println!("Funding two Devnet wallets…");
    let alice = generate_faucet_wallet(net).await?; // the lender
    let carol = generate_faucet_wallet(net).await?; // the borrower

    // 1. Alice's vault holds the XRP that will be lent, and the broker lends
    //    it. Deposits close after 90 seconds; loans may run for 330 more, with
    //    time to spare: the loan must end 60 seconds before redemption.
    let now = ledger_time(&client).await?;
    let subscription_ends = now + 90;
    let redemption_starts = subscription_ends + 330;
    let mut vault = VaultCreate::new(alice.classic_address(), Asset::Xrp);
    vault.vault_kind = Some(1); // closed-ended (0, the default, is open-ended)
    vault.subscription_date = Some(subscription_ends);
    vault.redemption_date = Some(redemption_starts);
    let created = send(&client, &alice, vault).await?;
    let vault_id = keylet::vault(alice.classic_address(), sequence_of(&created)?)?;
    let deposit = VaultDeposit::new(alice.classic_address(), vault_id, Amount::Xrp(50_000_000));
    send(&client, &alice, deposit).await?;
    let broker = send(
        &client,
        &alice,
        LoanBrokerSet::new(alice.classic_address(), vault_id),
    )
    .await?;
    let broker_id = keylet::loan_broker(alice.classic_address(), sequence_of(&broker)?)?;
    println!("  vault  {vault_id}\n  broker {broker_id}");
    println!("  waiting for the investment phase, when loans open…");
    wait_until_after(&client, subscription_ends).await?;

    // 2. Carol asks for 10 XRP at 10% a year, in two installments a minute apart.
    let mut request = LoanSet::new(carol.classic_address(), broker_id, "10000000");
    request.interest_rate = Some(10_000); // 1/10 basis points: 10_000 is 10%
    request.payment_total = Some(2);
    request.payment_interval = Some(60); // seconds, the shortest allowed
    let mut tx = request.to_json()?;

    // autofill sets Sequence, LastLedgerSequence and a Fee that pays for
    // Alice's signature too. Alice signs every one of those fields, and
    // Carol's SigningPubKey, so they are all set before she does.
    client.autofill(&mut tx).await?;
    tx["SigningPubKey"] = json!(carol.public_key_hex());

    // 3. Alice approves the loan with her CounterpartySignature, under the
    //    signing rules of this network (fixCleanup3_4_0: SigningRules::Current).
    let rules = client.counterparty_signing_rules().await?;
    let approved = alice.sign_as_counterparty(&tx, rules)?;
    assert!(loan::verify_counterparty_signature(&approved, rules)?);

    // 4. Carol signs it as the sender, and submits it.
    let signed = carol.sign(&approved)?;
    let result = client.submit_and_wait(&signed.tx_blob).await?;
    println!(
        "  {:<24} {} ({})",
        "LoanSet",
        result.transaction_result().unwrap_or("?"),
        signed.hash
    );
    let loan_id = created_entry(&result, "Loan")?;
    let terms = entry(&client, &loan_id).await?;
    println!(
        "  loan {loan_id}\n    principal {} drops, {} payments of {} drops",
        text(&terms["PrincipalOutstanding"]),
        terms["PaymentRemaining"],
        text(&terms["PeriodicPayment"])
    );

    // 5. Carol repays each installment (paying ahead of the due date is
    //    allowed). An XRP payment is whole drops, so round the amount up: the
    //    ledger takes only what is due.
    loop {
        let loan = entry(&client, &loan_id).await?;
        if loan["PaymentRemaining"].as_u64().unwrap_or(0) == 0 {
            break;
        }
        let drops = drops_rounded_up(&loan["PeriodicPayment"])?
            + drops_rounded_up(&loan["LoanServiceFee"])?;
        send(
            &client,
            &carol,
            LoanPay::new(carol.classic_address(), loan_id, Amount::Xrp(drops)),
        )
        .await?;
    }

    // 6. A paid-off loan is deleted (by the borrower or the broker's owner),
    //    and then the broker and vault can be wound down.
    send(
        &client,
        &carol,
        LoanDelete::new(carol.classic_address(), loan_id),
    )
    .await?;
    send(
        &client,
        &alice,
        LoanBrokerDelete::new(alice.classic_address(), broker_id),
    )
    .await?;
    println!("  waiting for the redemption phase, when withdrawals reopen…");
    wait_until_after(&client, redemption_starts).await?;
    let vault = entry(&client, &vault_id).await?;
    println!(
        "  the vault holds {} drops: Alice's 50 XRP and Carol's interest",
        text(&vault["AssetsTotal"])
    );
    let shares = alice_shares(&client, &alice, &vault).await?;
    let withdraw = VaultWithdraw::new(alice.classic_address(), vault_id, shares);
    send(&client, &alice, withdraw).await?;
    send(
        &client,
        &alice,
        VaultDelete::new(alice.classic_address(), vault_id),
    )
    .await?;
    Ok(())
}

/// Every vault share Alice holds: withdrawing them all empties the vault.
async fn alice_shares(
    client: &XrplHttpClient,
    alice: &Wallet,
    vault: &Value,
) -> Result<Amount, Box<dyn std::error::Error>> {
    let share_id = vault["ShareMPTID"].as_str().ok_or("no ShareMPTID")?;
    let token = client
        .raw_request(
            "ledger_entry",
            json!({"mptoken": {"mpt_issuance_id": share_id, "account": alice.classic_address()},
                   "ledger_index": "validated"}),
        )
        .await?;
    let held: i64 = token["node"]["MPTAmount"].as_str().unwrap_or("0").parse()?;
    Ok(Amount::mpt(held, share_id)?)
}

/// A loan amount in drops, rounded up to a whole drop. The ledger writes
/// these numbers as decimal text (`"5000002.5"`), or as a mantissa and an
/// exponent (`"1000000000000000e-4"`) when they are very large or small; an
/// absent field is zero.
fn drops_rounded_up(value: &Value) -> Result<u64, Box<dyn std::error::Error>> {
    let text = match value {
        Value::Null => return Ok(0),
        Value::String(s) => s.as_str(),
        other => return Err(format!("not a number: {other}").into()),
    };
    let (mantissa, exponent) = match text.split_once(['e', 'E']) {
        Some((m, e)) => (m, e.parse::<i32>()?),
        None => (text, 0),
    };
    if mantissa.starts_with('-') {
        return Err(format!("a negative amount: {text}").into());
    }
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    // All the digits as one integer, and the power of ten that scales it.
    let digits: u128 = format!("{whole}{fraction}").parse()?;
    let scale = exponent - i32::try_from(fraction.len())?;
    let too_large = || format!("too large to pay in drops: {text}");
    let drops = if scale >= 0 {
        10u128
            .checked_pow(scale.unsigned_abs())
            .and_then(|p| digits.checked_mul(p))
            .ok_or_else(too_large)?
    } else {
        match 10u128.checked_pow(scale.unsigned_abs()) {
            Some(divisor) => digits.div_ceil(divisor),
            // Smaller than any u128 divisor: under one drop, so one if not zero.
            None => u128::from(digits != 0),
        }
    };
    Ok(u64::try_from(drops)?)
}

/// The close time of the latest validated ledger (seconds since 2000-01-01):
/// the clock the vault's dates are compared with.
async fn ledger_time(client: &XrplHttpClient) -> Result<u32, Box<dyn std::error::Error>> {
    let ledger = client.ledger(LedgerIndex::Validated).await?;
    let close_time = ledger
        .ledger
        .and_then(|l| l.close_time)
        .ok_or("no close_time")?;
    Ok(u32::try_from(close_time)?)
}

/// Wait until a validated ledger closes after `date`: a transaction applied
/// on top of it sees that time as past.
async fn wait_until_after(
    client: &XrplHttpClient,
    date: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    while ledger_time(client).await? <= date {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
    Ok(())
}

/// A string field's text, without JSON quotes.
fn text(value: &Value) -> &str {
    value.as_str().unwrap_or("?")
}

/// A ledger entry as JSON, from the latest validated ledger.
async fn entry(client: &XrplHttpClient, id: &Hash256) -> Result<Value, Box<dyn std::error::Error>> {
    let reply = client
        .raw_request(
            "ledger_entry",
            json!({"index": id.to_string(), "ledger_index": "validated"}),
        )
        .await?;
    Ok(reply["node"].clone())
}

/// The ID of the entry of `kind` the transaction created.
fn created_entry(result: &TxResult, kind: &str) -> Result<Hash256, Box<dyn std::error::Error>> {
    let meta = result.meta.as_ref().ok_or("no metadata")?;
    meta.affected_nodes
        .iter()
        .find(|n| matches!(n, AffectedNode::CreatedNode(_)) && n.ledger_entry_type() == kind)
        .map(|n| *n.ledger_index())
        .ok_or_else(|| format!("the transaction created no {kind}").into())
}

/// The Sequence the transaction used (vaults and brokers are numbered by it).
fn sequence_of(result: &TxResult) -> Result<u32, Box<dyn std::error::Error>> {
    Ok(result
        .transaction()?
        .common()
        .and_then(|c| c.sequence)
        .ok_or("no Sequence")?)
}

/// Autofill, sign, submit and wait; stop on anything but tesSUCCESS.
async fn send(
    client: &XrplHttpClient,
    wallet: &Wallet,
    tx: impl TxModel,
) -> Result<TxResult, Box<dyn std::error::Error>> {
    let json = tx.to_json()?;
    let result = sign_and_submit(client, &json, wallet).await?;
    let outcome = result.transaction_result().unwrap_or("unknown").to_string();
    println!(
        "  {:<24} {outcome}",
        json["TransactionType"].as_str().unwrap_or("?")
    );
    if outcome != "tesSUCCESS" {
        return Err(outcome.into());
    }
    Ok(result)
}
