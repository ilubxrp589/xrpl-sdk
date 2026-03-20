//! Mint an NFT on testnet using typed transaction builders.
//!
//! Demonstrates using NFTokenMint builder with autofill.
//!
//! Usage: cargo run --example mint_nft

use xrpl_sdk::transactions::{NFTokenMint, Transaction};
use xrpl_sdk::{autofill_and_sign, Wallet, XrplHttpClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("XRPL_NODE_URL")
        .unwrap_or_else(|_| "https://s.altnet.rippletest.net:51234".into());

    let wallet = Wallet::generate();
    println!("Minter address: {}", wallet.classic_address());
    println!("Fund via: https://faucet.altnet.rippletest.net/accounts");
    println!("Press Enter after funding...");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let client = XrplHttpClient::new(&url)?;

    // Build NFTokenMint with typed builder
    let nft_tx = NFTokenMint::builder(wallet.classic_address())
        .nf_token_taxon(0)
        .uri("68747470733A2F2F6578616D706C652E636F6D2F6E66742E6A736F6E")
        .flags(8) // tfTransferable
        .build()?;

    println!("Transaction type: {}", nft_tx.transaction_type());

    // Convert to JSON for autofill
    let mut tx_json = nft_tx.to_json();

    let blob = autofill_and_sign(&client, &mut tx_json, &wallet).await?;
    println!("Signed blob length: {} chars", blob.len());

    let result = client.submit(&blob).await?;
    println!("Engine result: {}", result.engine_result);
    println!("Message: {}", result.engine_result_message);

    Ok(())
}
