//! Cookbook: trade on the decentralized exchange.
//!
//! A maker offers 50 USD for 25 XRP (0.5 XRP per USD); the order book shows
//! it; a taker buys 20 USD from it; the maker cancels what is left.
//!
//! Runs on Testnet with faucet wallets: `cargo run --example dex_order`
#![allow(
    clippy::indexing_slicing,
    reason = "reading serde_json values with [] gives Null, never a panic"
)]

use xrpl_sdk::models::{OfferCancel, OfferCreate, OfferCreateFlags, Payment, TrustSet};
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let net = FaucetNetwork::Testnet;
    let client = XrplHttpClient::new(net.rpc_url())?;
    println!("Funding three Testnet wallets…");
    let issuer = generate_faucet_wallet(net).await?;
    let maker = generate_faucet_wallet(net).await?;
    let taker = generate_faucet_wallet(net).await?;
    let usd = |value: &str| Amount::token(value, "USD", issuer.classic_address());

    // The maker and the taker trust the issuer's USD; the maker gets 100 USD.
    for holder in [&maker, &taker] {
        let mut line = TrustSet::new(holder.classic_address());
        line.limit_amount = Some(usd("1000")?);
        send(&client, holder, line.to_json()?).await?;
    }
    send(
        &client,
        &issuer,
        Payment::new(
            issuer.classic_address(),
            maker.classic_address(),
            usd("100")?,
        )
        .to_json()?,
    )
    .await?;

    // 1. The maker's order: the taker would pay 25 XRP and get 50 USD.
    let order = OfferCreate::new(maker.classic_address(), Amount::Xrp(25_000_000), usd("50")?);
    send(&client, &maker, order.to_json()?).await?;

    // 2. The order book: offers selling USD for XRP, best first.
    let book = client
        .book_offers(
            &Asset::token("USD", issuer.classic_address())?, // the offers give USD
            &Asset::xrp(),                                   // and want XRP
        )
        .await?;
    for offer in &book.offers {
        println!(
            "  book: {} sells {} for {} (quality {})",
            offer.account,
            offer.taker_gets,
            offer.taker_pays,
            offer.quality.as_deref().unwrap_or("?")
        );
    }

    // 3. The taker buys 20 USD for 10 XRP, now or not at all.
    let mut buy = OfferCreate::new(taker.classic_address(), usd("20")?, Amount::Xrp(10_000_000));
    buy.common.flags = Some(u32::from(OfferCreateFlags::IMMEDIATE_OR_CANCEL));
    let trade = send(&client, &taker, buy.to_json()?).await?;
    for account in utils::get_balance_changes(trade.meta.as_ref().ok_or("no metadata")?)? {
        let changes: Vec<String> = account
            .balances
            .iter()
            .map(|b| format!("{} {}", b.value, b.currency))
            .collect();
        println!("  {}: {}", account.account, changes.join(", "));
    }

    // 4. The maker cancels the 30 USD left.
    let open = client
        .account_offers(maker.classic_address(), &LedgerIndex::Validated)
        .await?;
    println!(
        "  the maker's offer still sells {}",
        open.offers[0].taker_gets
    );
    send(
        &client,
        &maker,
        OfferCancel::new(maker.classic_address(), open.offers[0].seq).to_json()?,
    )
    .await?;
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
