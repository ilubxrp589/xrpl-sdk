//! Cookbook: pay through a payment channel.
//!
//! Alice puts 10 XRP in a channel to Bob. She then pays him off-ledger by
//! signing claims (no transaction, no fee, instant); Bob checks each claim
//! himself and redeems the latest one on the ledger when he chooses.
//!
//! Runs on Testnet with faucet wallets: `cargo run --example payment_channel`
#![allow(
    clippy::indexing_slicing,
    reason = "reading serde_json values with [] gives Null, never a panic"
)]

use xrpl_sdk::models::{PaymentChannelClaim, PaymentChannelCreate};
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let net = FaucetNetwork::Testnet;
    let client = XrplHttpClient::new(net.rpc_url())?;
    println!("Funding two Testnet wallets…");
    let alice = generate_faucet_wallet(net).await?;
    let bob = generate_faucet_wallet(net).await?;

    // 1. Open the channel: 10 XRP, closable by Alice only after a day's notice.
    let open = PaymentChannelCreate::new(
        alice.classic_address(),
        bob.classic_address(),
        Amount::Xrp(10_000_000),
        86_400,
        alice.public_key_hex(),
    );
    let opened = send(&client, &alice, open.to_json()?).await?;
    let sequence = opened
        .tx_fields
        .get("Sequence")
        .and_then(|s| s.as_u64())
        .and_then(|s| u32::try_from(s).ok())
        .ok_or("no Sequence")?;
    let channel =
        keylet::payment_channel(alice.classic_address(), bob.classic_address(), sequence)?
            .to_string();
    println!("  channel {channel}");

    // 2. Off-ledger: Alice signs claims for running totals of 1, then 3 XRP.
    let mut latest = String::new();
    for total in [1_000_000u64, 3_000_000] {
        let claim = alice.sign_payment_channel_claim(&channel, total)?;
        // Bob verifies each claim against the channel's key before relying on it.
        assert!(verify_payment_channel_claim(
            &channel,
            total,
            &claim,
            &alice.public_key_hex()
        )?);
        println!(
            "  claim for {} XRP signed and verified off-ledger",
            utils::drops_to_xrp(total)
        );
        latest = claim;
    }

    // 3. Bob redeems the latest claim (3 XRP) on the ledger.
    let mut redeem = PaymentChannelClaim::new(bob.classic_address(), Hash256::from_hex(&channel)?);
    redeem.balance = Some(Amount::Xrp(3_000_000));
    redeem.amount = Some(Amount::Xrp(3_000_000));
    redeem.signature = Some(latest);
    redeem.public_key = Some(alice.public_key_hex());
    send(&client, &bob, redeem.to_json()?).await?;

    let channels = client
        .account_channels(alice.classic_address(), None, LedgerIndex::Validated)
        .await?;
    let c = &channels.channels[0];
    println!(
        "  the channel holds {} XRP and has paid out {} XRP",
        utils::drops_to_xrp(c.amount.parse()?),
        utils::drops_to_xrp(c.balance.parse()?)
    );
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
