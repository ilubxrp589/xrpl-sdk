//! Cookbook: issue a token.
//!
//! An issuer creates USD; Alice and Bob trust it; the issuer sends Alice 100
//! USD; Alice pays Bob 25, which ripples through the issuer.
//!
//! Runs on Testnet with faucet wallets, nothing to set up:
//! `cargo run --example issue_token`
#![allow(
    clippy::indexing_slicing,
    reason = "reading serde_json values with [] gives Null, never a panic"
)]

use serde_json::json;
use xrpl_sdk::models::{AccountSet, AccountSetFlag, Payment, TrustSet};
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let net = FaucetNetwork::Testnet;
    let client = XrplHttpClient::new(net.rpc_url())?;
    println!("Funding three Testnet wallets…");
    let issuer = generate_faucet_wallet(net).await?;
    let alice = generate_faucet_wallet(net).await?;
    let bob = generate_faucet_wallet(net).await?;
    let usd = |value: &str| Amount::token(value, "USD", issuer.classic_address());

    // 1. The issuer lets its token ripple between holders (DefaultRipple).
    let mut settings = AccountSet::new(issuer.classic_address());
    settings.set_flag = Some(u32::from(AccountSetFlag::DefaultRipple));
    send(&client, &issuer, settings.to_json()?).await?;

    // 2. Alice and Bob each trust the issuer for up to 1,000 USD.
    for holder in [&alice, &bob] {
        let mut line = TrustSet::new(holder.classic_address());
        line.limit_amount = Some(usd("1000")?);
        send(&client, holder, line.to_json()?).await?;
    }

    // 3. The issuer issues 100 USD to Alice.
    let issue = Payment::new(
        issuer.classic_address(),
        alice.classic_address(),
        usd("100")?,
    );
    send(&client, &issuer, issue.to_json()?).await?;

    // 4. Alice pays Bob 25 USD.
    let pay = Payment::new(alice.classic_address(), bob.classic_address(), usd("25")?);
    let result = send(&client, &alice, pay.to_json()?).await?;
    println!("\nWhat Alice's payment changed:");
    for account in utils::get_balance_changes(result.meta.as_ref().unwrap_or(&json!({})))? {
        for b in &account.balances {
            println!("  {}  {:>8} {}", account.account, b.value, b.currency);
        }
    }

    println!("\nTrust lines now:");
    for (name, holder) in [("Alice", &alice), ("Bob", &bob)] {
        for line in client
            .account_lines_all(holder.classic_address(), LedgerIndex::Validated)
            .await?
        {
            println!(
                "  {name}: {} {} (limit {})",
                line.balance, line.currency, line.limit
            );
        }
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
