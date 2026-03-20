//! Subscribe to ledger close events via WebSocket.
//!
//! Connects to testnet and prints each ledger close as it happens (~3-5s interval).
//!
//! Usage: cargo run --example subscribe_ledger

use xrpl_sdk::{WsEvent, XrplWsClient};

const TESTNET_WS: &str = "wss://s.altnet.rippletest.net:51233";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to {TESTNET_WS}...");
    let client = XrplWsClient::connect(TESTNET_WS).await?;
    println!("Connected!");

    // Subscribe to ledger stream
    let mut rx = client.subscribe_ledger().await?;
    println!("Subscribed to ledger events. Waiting for ledger closes...\n");

    let mut count = 0;
    while let Ok(event) = rx.recv().await {
        match event {
            WsEvent::Ledger(le) => {
                println!(
                    "Ledger #{} closed | hash: {} | txns: {} | fee: {} drops",
                    le.ledger_index,
                    le.ledger_hash.as_deref().unwrap_or("?"),
                    le.txn_count.unwrap_or(0),
                    le.fee_base.unwrap_or(0),
                );
                count += 1;
                if count >= 5 {
                    println!("\nReceived 5 ledger events, exiting.");
                    break;
                }
            }
            WsEvent::Reconnected => {
                println!("-- Reconnected --");
            }
            _ => {}
        }
    }

    Ok(())
}
