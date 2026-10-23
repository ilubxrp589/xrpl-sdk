//! Cookbook: multi-signing.
//!
//! Alice puts Bob and Carol on her signer list with a quorum of 2. A
//! transaction from Alice's account then needs both their signatures, made
//! separately and combined; it is submitted with `submit_multisigned`.
//!
//! Runs on Testnet with faucet wallets: `cargo run --example multisign`
#![allow(
    clippy::indexing_slicing,
    reason = "reading serde_json values with [] gives Null, never a panic"
)]

use serde_json::json;
use xrpl_sdk::models::{AccountSet, SignerEntry, SignerListSet};
use xrpl_sdk::wallet::collect_signers;
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let net = FaucetNetwork::Testnet;
    let client = XrplHttpClient::new(net.rpc_url())?;
    println!("Funding three Testnet wallets…");
    let alice = generate_faucet_wallet(net).await?;
    let bob = generate_faucet_wallet(net).await?;
    let carol = generate_faucet_wallet(net).await?;

    // 1. Alice's signer list: Bob and Carol, one vote each, two needed.
    let list = SignerListSet::new(alice.classic_address(), 2).with_signer_entries(
        [&bob, &carol]
            .iter()
            .map(|w| SignerEntry::new(w.classic_address(), 1))
            .collect::<Vec<_>>(),
    );
    send(&client, &alice, list.to_json()?).await?;

    // 2. A transaction from Alice's account, signed by Bob and Carol.
    let domain =
        AccountSet::new(alice.classic_address()).with_domain(utils::str_to_hex("example.com"));
    let mut tx = domain.to_json()?;
    client.autofill(&mut tx).await?;
    let fee: u64 = tx["Fee"].as_str().ok_or("no Fee")?.parse()?;
    tx["Fee"] = json!((fee * 3).to_string()); // (1 + 2 signers) × the single-signature fee
    tx["SigningPubKey"] = json!(""); // multi-signed: no single key
    let signatures = vec![
        bob.sign_for_multisigning(&tx)?,
        carol.sign_for_multisigning(&tx)?,
    ];
    collect_signers(&mut tx, signatures)?; // in the account-ID order rippled requires

    // 3. Submit and wait.
    let submitted = client.submit_multisigned(&tx).await?;
    println!("  submit_multisigned        {}", submitted.engine_result);
    let blob = hex::encode_upper(encode_transaction_json(&tx)?);
    let hash = hash_signed_tx_blob(&blob)?.to_string();
    for _ in 0..15 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        if let Ok(done) = client.tx(&hash).await {
            if done.validated == Some(true) {
                println!(
                    "  validated: {} ({} signers)",
                    done.transaction_result().unwrap_or("?"),
                    tx["Signers"].as_array().map(Vec::len).unwrap_or(0)
                );
                return Ok(());
            }
        }
    }
    Err("not validated in time".into())
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
