//! Cookbook: a Batch (XLS-56): several transactions, all or nothing.
//!
//! Alice pays Bob 5 XRP and Bob pays Alice 2 XRP in one Batch: both happen or
//! neither does. Bob approves the batch with his own signature; Alice signs
//! and submits it. Each inner transaction lands on the ledger with its own ID.
//!
//! Runs on Testnet with faucet wallets: `cargo run --example batch`
#![allow(
    clippy::indexing_slicing,
    reason = "reading serde_json values with [] gives Null, never a panic"
)]

use xrpl_sdk::models::{Batch, BatchFlags, Payment};
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let net = FaucetNetwork::Testnet;
    let client = XrplHttpClient::new(net.rpc_url())?;
    println!("Funding two Testnet wallets…");
    let alice = generate_faucet_wallet(net).await?;
    let bob = generate_faucet_wallet(net).await?;

    // 1. The batch, from typed transactions.
    let pay_bob: Tx = Payment::new(
        alice.classic_address(),
        bob.classic_address(),
        Amount::Xrp(5_000_000),
    )
    .into();
    let pay_alice: Tx = Payment::new(
        bob.classic_address(),
        alice.classic_address(),
        Amount::Xrp(2_000_000),
    )
    .into();
    let mut batch_tx = Batch::new(alice.classic_address(), vec![pay_bob, pay_alice]);
    batch_tx.common.flags = Some(u32::from(BatchFlags::ALL_OR_NOTHING));
    let mut tx = batch_tx.to_json()?;

    // 2. autofill numbers the inner transactions and prices the whole batch.
    client.autofill(&mut tx).await?;
    println!(
        "  batch fee: {} drops for 2 transactions and 1 co-signer",
        tx["Fee"].as_str().unwrap_or("?")
    );

    // 3. Bob approves (his payment is inside); then Alice signs the batch.
    let approval = bob.sign_batch(&tx)?;
    batch::add_batch_signers(&mut tx, vec![approval])?;
    let blob = alice.sign_and_encode(&tx)?;
    let result = client.submit_and_wait(&blob).await?;
    println!(
        "  Batch                    {}",
        result.transaction_result().unwrap_or("?")
    );

    // 4. Each inner transaction is on the ledger under the ID computed for it.
    for id in batch::inner_transaction_ids(&tx)? {
        let inner = client.tx(&id.to_string()).await?;
        println!(
            "  inner {id}: {}",
            inner.transaction_result().unwrap_or("?")
        );
    }
    Ok(())
}
