//! Cookbook: Multi-Purpose Tokens (MPTs).
//!
//! An issuer creates an MPT with two decimal places and a cap of 10,000.00;
//! a holder opts in; the issuer sends the holder 123.45.
//!
//! Runs on Testnet with faucet wallets: `cargo run --example mpt`
#![allow(
    clippy::indexing_slicing,
    reason = "reading serde_json values with [] gives Null, never a panic"
)]

use xrpl_sdk::models::{
    MPTokenAuthorize, MPTokenIssuanceCreate, MPTokenIssuanceCreateFlags, Payment,
};
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let net = FaucetNetwork::Testnet;
    let client = XrplHttpClient::new(net.rpc_url())?;
    println!("Funding two Testnet wallets…");
    let issuer = generate_faucet_wallet(net).await?;
    let holder = generate_faucet_wallet(net).await?;

    // 1. The issuance: 2 decimal places, at most 1,000,000 units (10,000.00), transferable.
    let mut create = MPTokenIssuanceCreate::new(issuer.classic_address());
    create.asset_scale = Some(2);
    create.maximum_amount = Some("1000000".into());
    create.common.flags = Some(u32::from(MPTokenIssuanceCreateFlags::MPT_CAN_TRANSFER));
    create.mp_token_metadata = Some(utils::str_to_hex(
        r#"{"ticker":"HAL","name":"Halcyon test token"}"#,
    ));
    let created = send(&client, &issuer, create.to_json()?).await?;
    let sequence = created
        .tx_fields
        .get("Sequence")
        .and_then(|s| s.as_u64())
        .and_then(|s| u32::try_from(s).ok())
        .ok_or("no Sequence")?;
    let id = keylet::mpt_issuance_id(sequence, issuer.classic_address())?;
    println!("  MPTokenIssuanceID {id}");

    // 2. The holder opts in to the token.
    send(
        &client,
        &holder,
        MPTokenAuthorize::new(holder.classic_address(), id).to_json()?,
    )
    .await?;

    // 3. The issuer sends 12,345 units: 123.45 at two decimal places.
    let pay = Payment::new(
        issuer.classic_address(),
        holder.classic_address(),
        Amount::mpt(12_345, &id.to_string())?,
    );
    send(&client, &issuer, pay.to_json()?).await?;

    // The holder's MPToken entry, typed.
    let owned = client
        .account_objects_all(
            holder.classic_address(),
            Some("mptoken"),
            LedgerIndex::Validated,
        )
        .await?;
    for entry in owned {
        println!("  holder's MPToken: {}", entry.to_value()?["MPTAmount"]);
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
