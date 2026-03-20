//! Place an OfferCreate on the XRPL DEX (testnet).
//!
//! This example creates a limit order to buy USD with XRP.
//!
//! Usage: cargo run --example place_offer

use xrpl_sdk::{LedgerIndex, Wallet, XrplHttpClient};

const TESTNET_URL: &str = "https://s.altnet.rippletest.net:51234";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wallet = Wallet::generate();
    println!("Wallet address: {}", wallet.classic_address());
    println!("Fund via: https://faucet.altnet.rippletest.net/accounts\n");
    println!("Press Enter after funding...");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let client = XrplHttpClient::new(TESTNET_URL)?;
    let info = client
        .account_info(wallet.classic_address(), &LedgerIndex::Current)
        .await?;
    let fee = client.fee().await?;
    let ledger_idx = client.ledger_current().await?;

    // OfferCreate: offer to pay 10 XRP to get 1 USD (from a test issuer)
    let tx = serde_json::json!({
        "TransactionType": "OfferCreate",
        "Account": wallet.classic_address(),
        "TakerPays": {
            "currency": "USD",
            "issuer": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
            "value": "1"
        },
        "TakerGets": "10000000",  // 10 XRP
        "Fee": fee.drops.base_fee,
        "Sequence": info.account_data.sequence,
        "LastLedgerSequence": ledger_idx + 4,
        "Flags": 0
    });

    let blob = wallet.sign_and_encode(&tx)?;
    println!("Submitting OfferCreate...");
    let result = client.submit(&blob).await?;
    println!(
        "Result: {} — {}",
        result.engine_result, result.engine_result_message
    );

    Ok(())
}
