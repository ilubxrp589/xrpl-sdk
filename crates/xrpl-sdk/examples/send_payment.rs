//! Send an XRP payment on testnet using autofill.
//!
//! This example:
//! 1. Creates a wallet from a seed
//! 2. Connects to the XRPL testnet
//! 3. Uses autofill to set Fee, Sequence, and LastLedgerSequence
//! 4. Signs and submits the payment
//!
//! Usage: cargo run --example send_payment

use xrpl_sdk::{autofill_and_sign, Wallet, XrplHttpClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("XRPL_NODE_URL")
        .unwrap_or_else(|_| "https://s.altnet.rippletest.net:51234".into());

    let wallet = Wallet::generate();
    println!("Sender address: {}", wallet.classic_address());
    println!("Public key:     {}", wallet.public_key_hex());
    println!();
    println!("Fund this address via the testnet faucet:");
    println!("  https://faucet.altnet.rippletest.net/accounts");
    println!();
    println!("Press Enter after funding...");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let client = XrplHttpClient::new(&url)?;

    // Build payment — autofill handles Fee, Sequence, LastLedgerSequence
    let mut tx = serde_json::json!({
        "TransactionType": "Payment",
        "Account": wallet.classic_address(),
        "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        "Amount": "1000000"
    });

    println!("Autofilling and signing...");
    let blob = autofill_and_sign(&client, &mut tx, &wallet).await?;
    println!(
        "Signed blob: {}...{}",
        &blob[..40],
        &blob[blob.len() - 20..]
    );

    println!("Submitting to {}...", url);
    let result = client.submit(&blob).await?;
    println!("Engine result: {}", result.engine_result);
    println!("Message: {}", result.engine_result_message);

    Ok(())
}
