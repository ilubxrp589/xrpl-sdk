//! Cookbook: an automated market maker (AMM).
//!
//! A liquidity provider creates an XRP/USD pool; a trader buys USD from it
//! with an ordinary offer (the ledger routes it through the pool); `amm_info`
//! shows the pool before and after.
//!
//! Runs on Testnet with faucet wallets: `cargo run --example amm`
#![allow(
    clippy::indexing_slicing,
    reason = "reading serde_json values with [] gives Null, never a panic"
)]

use xrpl_sdk::models::{
    AMMCreate, AccountSet, AccountSetFlag, OfferCreate, OfferCreateFlags, Payment, TrustSet,
};
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let net = FaucetNetwork::Testnet;
    let client = XrplHttpClient::new(net.rpc_url())?;
    println!("Funding three Testnet wallets…");
    let issuer = generate_faucet_wallet(net).await?;
    let provider = generate_faucet_wallet(net).await?;
    let trader = generate_faucet_wallet(net).await?;
    let usd = |value: &str| Amount::token(value, "USD", issuer.classic_address());

    // The issuer's token must ripple (DefaultRipple) to sit in a pool.
    let mut settings = AccountSet::new(issuer.classic_address());
    settings.set_flag = Some(u32::from(AccountSetFlag::DefaultRipple));
    send(&client, &issuer, settings.to_json()?).await?;
    for holder in [&provider, &trader] {
        let mut line = TrustSet::new(holder.classic_address());
        line.limit_amount = Some(usd("10000")?);
        send(&client, holder, line.to_json()?).await?;
    }
    send(
        &client,
        &issuer,
        Payment::new(
            issuer.classic_address(),
            provider.classic_address(),
            usd("200")?,
        )
        .to_json()?,
    )
    .await?;

    // 1. The pool: 40 XRP and 100 USD, 0.5% trading fee (500 / 100,000).
    //    AMMCreate costs one owner reserve instead of the usual fee;
    //    autofill charges it.
    let tx = AMMCreate::new(
        provider.classic_address(),
        Amount::Xrp(40_000_000),
        usd("100")?,
        500,
    )
    .to_json()?;
    send(&client, &provider, tx).await?;

    let (xrp, token) = (Asset::xrp(), Asset::token("USD", issuer.classic_address())?);
    let before = client.amm_info(&xrp, &token).await?.amm;
    println!(
        "  pool: {} and {}, fee {}",
        show(&before.amount),
        show(&before.amount2),
        before.trading_fee.unwrap_or(0)
    );

    // 2. The trader buys 10 USD for at most 5 XRP; the pool fills it.
    let mut buy = OfferCreate::new(trader.classic_address(), usd("10")?, Amount::Xrp(5_000_000));
    buy.common.flags = Some(u32::from(OfferCreateFlags::IMMEDIATE_OR_CANCEL));
    let trade = send(&client, &trader, buy.to_json()?).await?;
    for account in utils::get_balance_changes(trade.meta.as_ref().ok_or("no metadata")?)? {
        if account.account == trader.classic_address() {
            let changes: Vec<String> = account
                .balances
                .iter()
                .map(|b| format!("{} {}", b.value, b.currency))
                .collect();
            println!("  trader: {}", changes.join(", "));
        }
    }
    let after = client.amm_info(&xrp, &token).await?.amm;
    println!(
        "  pool now: {} and {}",
        show(&after.amount),
        show(&after.amount2)
    );
    Ok(())
}

fn show(amount: &Option<Amount>) -> String {
    amount.as_ref().map(Amount::to_string).unwrap_or_default()
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
