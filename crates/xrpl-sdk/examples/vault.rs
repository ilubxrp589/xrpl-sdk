//! Cookbook: a single-asset vault (XLS-65, the `SingleAssetVault` amendment).
//!
//! Alice creates a vault that holds XRP. Bob deposits 25 XRP and receives
//! vault shares, a multi-purpose token (MPT) the vault issues, in return.
//! He withdraws 10 XRP by asset amount, then redeems all his remaining
//! shares, which empties the vault, and Alice deletes it.
//!
//! Runs on Devnet with faucet wallets (vaults are not on Testnet or Mainnet
//! yet): `cargo run --example vault`
#![allow(
    clippy::indexing_slicing,
    reason = "reading serde_json values with [] gives Null, never a panic"
)]

use serde_json::{json, Value};
use xrpl_sdk::models::{VaultCreate, VaultDelete, VaultDeposit, VaultWithdraw};
use xrpl_sdk::requests::VaultInfoRequest;
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let net = FaucetNetwork::Devnet;
    let client = XrplHttpClient::new(net.rpc_url())?;
    println!("Funding two Devnet wallets…");
    let alice = generate_faucet_wallet(net).await?; // the vault's owner
    let bob = generate_faucet_wallet(net).await?; // a depositor

    // 1. Alice creates the vault. Its ID follows from her address and the
    //    Sequence of the VaultCreate.
    let created = send(
        &client,
        &alice,
        VaultCreate::new(alice.classic_address(), Asset::Xrp),
    )
    .await?;
    let sequence = created
        .transaction()?
        .common()
        .and_then(|c| c.sequence)
        .ok_or("no Sequence")?;
    let vault_id = keylet::vault(alice.classic_address(), sequence)?;
    println!("  vault {vault_id}");

    // 2. Bob deposits 25 XRP and receives shares for it.
    let deposit = VaultDeposit::new(bob.classic_address(), vault_id, Amount::Xrp(25_000_000));
    send(&client, &bob, deposit).await?;
    let info = client
        .request(&VaultInfoRequest::new(vault_id.to_string()))
        .await?;
    let share_id = info.vault["ShareMPTID"]
        .as_str()
        .ok_or("no ShareMPTID")?
        .to_string();
    println!(
        "  the vault holds {} drops; Bob holds {} shares ({share_id})",
        info.vault["AssetsTotal"].as_str().unwrap_or("?"),
        shares_of(&client, &bob, &share_id).await?
    );

    // 3. He withdraws 10 XRP, stated in the vault's asset: the vault burns
    //    the shares that are worth it.
    let withdraw = VaultWithdraw::new(bob.classic_address(), vault_id, Amount::Xrp(10_000_000));
    send(&client, &bob, withdraw).await?;
    let left = shares_of(&client, &bob, &share_id).await?;
    println!("  Bob has {left} shares left");

    // 4. Then he redeems every share he has left, stated in shares: the vault
    //    pays out what they are worth, and is empty.
    let redeem = VaultWithdraw::new(
        bob.classic_address(),
        vault_id,
        Amount::mpt(left, &share_id)?,
    );
    send(&client, &bob, redeem).await?;

    // 5. An empty vault can be deleted by its owner.
    send(
        &client,
        &alice,
        VaultDelete::new(alice.classic_address(), vault_id),
    )
    .await?;
    Ok(())
}

/// How many of the vault's shares (an MPT) `wallet` holds.
async fn shares_of(
    client: &XrplHttpClient,
    wallet: &Wallet,
    share_id: &str,
) -> Result<i64, Box<dyn std::error::Error>> {
    let token = client
        .raw_request(
            "ledger_entry",
            json!({"mptoken": {"mpt_issuance_id": share_id, "account": wallet.classic_address()},
                   "ledger_index": "validated"}),
        )
        .await?;
    Ok(token["node"]["MPTAmount"].as_str().unwrap_or("0").parse()?)
}

/// Autofill, sign, submit and wait; stop on anything but tesSUCCESS.
async fn send(
    client: &XrplHttpClient,
    wallet: &Wallet,
    tx: impl TxModel,
) -> Result<TxResult, Box<dyn std::error::Error>> {
    let json: Value = tx.to_json()?;
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
