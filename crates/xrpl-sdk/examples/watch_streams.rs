//! Cookbook: watch the live mainnet over WebSocket.
//!
//! Subscribes to ledger closes, each ledger's order-book trading and
//! validations, and prints what arrives for three ledgers. Read-only: nothing
//! is signed or sent.
//!
//! `cargo run --example watch_streams` (set `XRPL_WS_URL` for another server;
//! public servers differ in which streams they offer, so one that is refused
//! is reported and skipped).

use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("XRPL_WS_URL").unwrap_or_else(|_| "wss://s1.ripple.com".into());
    let ws = XrplWsClient::connect(&url).await?;
    let mut events = ws.subscribe_ledger().await?;
    if let Err(e) = ws.subscribe_book_changes().await {
        println!("(book changes not offered here: {e})");
    }
    if let Err(e) = ws.subscribe_validations().await {
        println!("(validations not offered here: {e})");
    }
    println!("Watching {url} for three ledgers…");

    let (mut ledgers, mut validations) = (0, 0);
    while ledgers < 3 {
        match events.recv().await? {
            WsEvent::Ledger(l) => {
                ledgers += 1;
                println!(
                    "ledger {} closed: {} transactions, {validations} validations heard since the last",
                    l.ledger_index,
                    l.txn_count.unwrap_or(0)
                );
                validations = 0;
            }
            WsEvent::BookChanges(b) => {
                for change in b.changes.iter().take(3) {
                    println!(
                        "  {} / {}: close {}, volume {}",
                        change.currency_a, change.currency_b, change.close, change.volume_a
                    );
                }
            }
            WsEvent::Validation(_) => validations += 1,
            WsEvent::Reconnected => println!("(reconnected; subscriptions restored)"),
            _ => {}
        }
    }
    Ok(())
}
