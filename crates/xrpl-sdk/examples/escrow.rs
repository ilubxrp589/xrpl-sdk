//! Cookbook: lock XRP in a time-based escrow, then release it.
//!
//! Alice escrows 10 XRP for Bob that can be released 15 seconds from now
//! (and cancelled after an hour if it never is); once the time passes, Bob
//! finishes it and receives the XRP.
//!
//! Runs on Testnet with faucet wallets: `cargo run --example escrow`
#![allow(
    clippy::indexing_slicing,
    reason = "reading serde_json values with [] gives Null, never a panic"
)]

use std::time::{Duration, SystemTime};
use xrpl_sdk::models::{EscrowCreate, EscrowFinish};
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let net = FaucetNetwork::Testnet;
    let client = XrplHttpClient::new(net.rpc_url())?;
    println!("Funding two Testnet wallets…");
    let alice = generate_faucet_wallet(net).await?;
    let bob = generate_faucet_wallet(net).await?;

    // Escrow times are ledger time: seconds since 2000-01-01.
    let now = utils::time::system_time_to_ripple_time(SystemTime::now())?;
    let mut escrow = EscrowCreate::new(
        alice.classic_address(),
        bob.classic_address(),
        Amount::Xrp(10_000_000),
    );
    escrow.finish_after = Some(now + 15);
    escrow.cancel_after = Some(now + 3600);
    let created = send(&client, &alice, escrow.to_json()?).await?;
    let sequence = created
        .tx_fields
        .get("Sequence")
        .and_then(|s| s.as_u64())
        .and_then(|s| u32::try_from(s).ok())
        .ok_or("no Sequence")?;
    println!(
        "  escrow {} releases at {}",
        keylet::escrow(alice.classic_address(), sequence)?,
        utils::ripple_time_to_iso8601(now + 15)
    );

    // Finishing before FinishAfter fails (tecNO_PERMISSION), so wait it out.
    println!("  waiting 20 seconds…");
    tokio::time::sleep(Duration::from_secs(20)).await;

    let finish = EscrowFinish::new(bob.classic_address(), alice.classic_address(), sequence);
    let finished = send(&client, &bob, finish.to_json()?).await?;
    for account in utils::get_balance_changes(finished.meta.as_ref().ok_or("no metadata")?)? {
        if account.account == bob.classic_address() {
            println!(
                "  Bob received {} XRP (10, less the fee he paid to finish it)",
                account.balances[0].value
            );
        }
    }
    Ok(())
}

/// Autofill, sign, submit and wait; stop on anything but tesSUCCESS.
async fn send(
    client: &XrplHttpClient,
    wallet: &Wallet,
    tx: serde_json::Value,
) -> Result<TxResult, Box<dyn std::error::Error>> {
    let result = sign_and_submit(client, &tx, wallet).await?;
    let outcome = result.transaction_result().unwrap_or("unknown").to_string();
    println!(
        "  {:<24} {outcome}",
        tx["TransactionType"].as_str().unwrap_or("?")
    );
    if outcome != "tesSUCCESS" {
        return Err(outcome.into());
    }
    Ok(result)
}
