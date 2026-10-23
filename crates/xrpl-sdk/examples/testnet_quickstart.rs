//! A funded Testnet wallet, a payment, and what it did to each balance.
//!
//! The faucet only learns the new wallet's address; the seed is printed so
//! the wallet can be reused with `Wallet::from_seed`.
//!
//! Usage: cargo run --example testnet_quickstart

use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let network = FaucetNetwork::Testnet;
    println!("Funding a new wallet from the {network:?} faucet…");
    let wallet = generate_faucet_wallet(network).await?;
    println!("Address: {}", wallet.classic_address());
    println!(
        "Seed:    {} (test network only)",
        wallet.seed().unwrap_or_default()
    );

    let client = XrplHttpClient::new(network.rpc_url())?;
    let mut tx = serde_json::json!({
        "TransactionType": "Payment",
        "Account": wallet.classic_address(),
        "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        "Amount": utils::xrp_to_drops("1.5")?.to_string(),
    });
    client.autofill(&mut tx).await?;
    let blob = wallet.sign_and_encode(&tx)?;
    println!("Hash:    {}", hash_signed_tx_blob(&blob)?);

    let result = client.submit_and_wait(&blob).await?;
    if let Some(meta) = &result.meta {
        let result = meta.get("TransactionResult").and_then(|v| v.as_str());
        println!("Result:  {}", result.unwrap_or("?"));
        for account in utils::get_balance_changes(meta)? {
            for b in &account.balances {
                println!("  {} {} {}", account.account, b.value, b.currency);
            }
        }
    }
    Ok(())
}
