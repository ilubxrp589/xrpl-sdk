//! Cookbook: write a check, and cash part of it.
//!
//! Alice writes Bob a check for up to 10 XRP (nothing moves yet); Bob cashes
//! 7 XRP of it whenever he likes.
//!
//! Runs on Testnet with faucet wallets: `cargo run --example checks`
#![allow(
    clippy::indexing_slicing,
    reason = "reading serde_json values with [] gives Null, never a panic"
)]

use xrpl_sdk::models::{CheckCash, CheckCreate};
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let net = FaucetNetwork::Testnet;
    let client = XrplHttpClient::new(net.rpc_url())?;
    println!("Funding two Testnet wallets…");
    let alice = generate_faucet_wallet(net).await?;
    let bob = generate_faucet_wallet(net).await?;

    // 1. Alice writes the check. Its ID follows from her address and the sequence used.
    let check = CheckCreate::new(
        alice.classic_address(),
        bob.classic_address(),
        Amount::Xrp(10_000_000),
    );
    let written = send(&client, &alice, check.to_json()?).await?;
    let sequence = written
        .tx_fields
        .get("Sequence")
        .and_then(|s| s.as_u64())
        .and_then(|s| u32::try_from(s).ok())
        .ok_or("no Sequence")?;
    let check_id = keylet::check(alice.classic_address(), sequence)?;
    println!("  check {check_id}");

    // The check is a ledger entry anyone can look up.
    let entry = client
        .ledger_entry(&check_id.to_string(), LedgerIndex::Validated)
        .await?;
    println!(
        "  on the ledger: {}",
        entry
            .node
            .map(|n| n.entry_type().to_string())
            .unwrap_or_default()
    );

    // 2. Bob cashes 7 XRP of it.
    let mut cash = CheckCash::new(bob.classic_address(), check_id);
    cash.amount = Some(Amount::Xrp(7_000_000));
    let cashed = send(&client, &bob, cash.to_json()?).await?;
    for account in utils::get_balance_changes(cashed.meta.as_ref().ok_or("no metadata")?)? {
        println!("  {}: {} XRP", account.account, account.balances[0].value);
    }
    Ok(())
}

/// Autofill, sign, submit and wait; stop on anything but tesSUCCESS.
async fn send(
    client: &XrplHttpClient,
    wallet: &Wallet,
    mut tx: serde_json::Value,
) -> Result<TxResult, Box<dyn std::error::Error>> {
    let result = submit_transaction(client, &mut tx, wallet).await?;
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
