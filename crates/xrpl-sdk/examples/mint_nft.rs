//! Mint an NFT on testnet with a typed transaction and autofill.
//!
//! Usage: cargo run --example mint_nft

use xrpl_sdk::models::{NFTokenMint, NFTokenMintFlags};
use xrpl_sdk::{Client, ToTxJson, TxModel, Wallet, XrplHttpClient};

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

    // The typed transaction: required fields in `new`, the rest with `with_…`.
    let nft_tx = NFTokenMint::new(wallet.classic_address(), 0)
        .with_uri("68747470733A2F2F6578616D706C652E636F6D2F6E66742E6A736F6E")
        .with_flags(NFTokenMintFlags::TRANSFERABLE);

    // As JSON for autofill (checked with `validate` first).
    let mut tx_json = nft_tx.to_tx_json()?;

    client.autofill(&mut tx_json).await?;
    let signed = wallet.sign(&tx_json)?;
    println!("Signed blob length: {} chars", signed.tx_blob.len());

    let result = client.submit(&signed.tx_blob).await?;
    println!("Engine result: {}", result.engine_result);
    println!("Message: {}", result.engine_result_message);

    Ok(())
}
