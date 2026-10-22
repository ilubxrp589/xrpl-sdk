//! Cookbook: mint an NFT, sell it, and earn a royalty on the resale.
//!
//! An artist mints a transferable NFT with a 5% transfer fee and sells it to
//! a buyer for 10 XRP; the buyer resells it to a collector for 20 XRP, and
//! the artist receives 5% of that sale automatically.
//!
//! Runs on Testnet with faucet wallets: `cargo run --example nft_marketplace`
#![allow(
    clippy::indexing_slicing,
    reason = "reading serde_json values with [] gives Null, never a panic"
)]

use xrpl_sdk::models::{NFTokenAcceptOffer, NFTokenCreateOffer, NFTokenMint};
use xrpl_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let net = FaucetNetwork::Testnet;
    let client = XrplHttpClient::new(net.rpc_url())?;
    println!("Funding three Testnet wallets…");
    let artist = generate_faucet_wallet(net).await?;
    let buyer = generate_faucet_wallet(net).await?;
    let collector = generate_faucet_wallet(net).await?;

    // 1. Mint: transferable, 5% (5000 / 100,000) on every resale.
    let mut mint = NFTokenMint::new(artist.classic_address(), 1);
    mint.common.flags = Some(NFTokenMint::TF_TRANSFERABLE);
    mint.transfer_fee = Some(5000);
    mint.uri = Some(utils::str_to_hex("ipfs://bafy.../artwork.json"));
    let minted = send(&client, &artist, mint.to_json()?).await?;
    let nft_id = utils::get_nftoken_id(minted.meta.as_ref().ok_or("no metadata")?)?
        .ok_or("no NFT minted")?;
    let parts = utils::parse_nftoken_id(&nft_id)?;
    println!(
        "  NFT {nft_id}\n    issuer {}, taxon {}, transfer fee {}%, serial {}",
        parts.issuer,
        parts.taxon,
        parts.transfer_fee / 1000,
        parts.sequence
    );

    // 2. The artist offers it for 10 XRP; the buyer accepts.
    sell_and_buy(&client, &artist, &buyer, &nft_id, 10_000_000).await?;

    // 3. The buyer resells it to the collector for 20 XRP: the ledger pays the artist 5%.
    let sale = sell_and_buy(&client, &buyer, &collector, &nft_id, 20_000_000).await?;
    for account in utils::get_balance_changes(sale.meta.as_ref().ok_or("no metadata")?)? {
        if account.account == artist.classic_address() {
            println!(
                "\n  The artist's royalty on the resale: {} XRP",
                account.balances[0].value
            );
        }
    }
    let owned = client
        .account_nfts_all(collector.classic_address(), &LedgerIndex::Validated)
        .await?;
    println!(
        "  The collector now holds {} NFT(s): {}",
        owned.len(),
        owned[0].nf_token_id
    );
    Ok(())
}

/// `seller` offers the NFT for `drops`; `buyer` accepts the offer.
async fn sell_and_buy(
    client: &XrplHttpClient,
    seller: &Wallet,
    buyer: &Wallet,
    nft_id: &str,
    drops: u64,
) -> Result<TxResult, Box<dyn std::error::Error>> {
    let mut offer = NFTokenCreateOffer::new(
        seller.classic_address(),
        Hash256::from_hex(nft_id)?,
        Amount::Xrp(drops),
    );
    offer.common.flags = Some(NFTokenCreateOffer::TF_SELL_NFTOKEN);
    send(client, seller, offer.to_json()?).await?;
    // The offer's ID, from the NFT's sell offers.
    let offers = client
        .request(&requests::NftSellOffersRequest::new(nft_id))
        .await?;
    let mut accept = NFTokenAcceptOffer::new(buyer.classic_address());
    accept.nf_token_sell_offer = Some(Hash256::from_hex(&offers.offers[0].nft_offer_index)?);
    send(client, buyer, accept.to_json()?).await
}

/// Autofill, sign, submit and wait; stop on anything but tesSUCCESS.
async fn send(
    client: &XrplHttpClient,
    wallet: &Wallet,
    mut tx: serde_json::Value,
) -> Result<TxResult, Box<dyn std::error::Error>> {
    let result = submit_transaction(client, &mut tx, wallet).await?;
    let outcome = result.transaction_result().unwrap_or("unknown").to_string();
    println!(
        "  {:<24} {outcome}",
        tx["TransactionType"].as_str().unwrap_or("?")
    );
    if outcome != "tesSUCCESS" {
        return Err(outcome.into());
    }
    Ok(result)
}
