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
        wallet
            .seed()
            .as_deref()
            .map(String::as_str)
            .unwrap_or_default()
    );

    let client = XrplHttpClient::new(network.rpc_url())?;
    let payment = models::Payment::new(
        wallet.classic_address(),
        "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        Amount::xrp(utils::xrp_to_drops("1.5")?)?,
    );
    // Step by step, to show the hash before it is sent; `sign_and_submit`
    // does all of this in one call.
    let mut tx = payment.to_tx_json()?;
    client.autofill(&mut tx).await?;
    let signed = wallet.sign(&tx)?;
    println!("Hash:    {}", signed.hash);

    let result = client.submit_and_wait(&signed.tx_blob).await?;
    if let Some(meta) = &result.meta {
        println!("Result:  {}", meta.transaction_result);
        for account in utils::get_balance_changes(meta)? {
            for b in &account.balances {
                println!("  {} {} {}", account.account, b.value, b.currency);
            }
        }
    }
    Ok(())
}
