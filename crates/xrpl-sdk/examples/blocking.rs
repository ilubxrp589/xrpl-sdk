//! Cookbook: no async — the blocking client.
//!
//! A plain `fn main`, no `#[tokio::main]`: a funded Testnet wallet, a payment,
//! and a balance, each a simple function call.
//!
//! Runs on Testnet with a faucet wallet: `cargo run --example blocking`

use serde_json::json;
use xrpl_sdk::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let net = FaucetNetwork::Testnet;
    let client = BlockingClient::new(net.rpc_url())?;

    // Async helpers run on the client's own runtime with block_on.
    println!("Funding a Testnet wallet…");
    let wallet = client.block_on(generate_faucet_wallet(net))?;
    let info = client.account_info(wallet.classic_address(), &LedgerIndex::Validated)?;
    println!(
        "  {} holds {} XRP",
        wallet.classic_address(),
        utils::drops_to_xrp(info.account_data.balance.parse()?)
    );

    let mut tx = json!({
        "TransactionType": "Payment",
        "Account": wallet.classic_address(),
        "Destination": "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        "Amount": utils::xrp_to_drops("2.5")?.to_string(),
    });
    client.autofill(&mut tx)?;
    let blob = wallet.sign_and_encode(&tx)?;
    let last = tx
        .get("LastLedgerSequence")
        .and_then(|v| v.as_u64())
        .ok_or("no LastLedgerSequence")? as u32;
    let result = client.submit_and_wait(&blob, last)?;
    println!(
        "  Payment {}: {}",
        result.hash().unwrap_or("?"),
        result.transaction_result().unwrap_or("?")
    );

    let info = client.account_info(wallet.classic_address(), &LedgerIndex::Validated)?;
    println!(
        "  balance now {} XRP",
        utils::drops_to_xrp(info.account_data.balance.parse()?)
    );
    Ok(())
}
