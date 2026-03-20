//! Check account balance and reserve calculation.
//!
//! Fetches account_info and server_info, then calculates
//! the available (spendable) balance after reserves.
//!
//! Usage: cargo run --example check_balance

use xrpl_sdk::{LedgerIndex, XrplHttpClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("XRPL_NODE_URL")
        .unwrap_or_else(|_| "https://s.altnet.rippletest.net:51234".into());
    let account = std::env::var("XRPL_ACCOUNT")
        .unwrap_or_else(|_| "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".into());

    let client = XrplHttpClient::new(&url)?;

    let info = client
        .account_info(&account, &LedgerIndex::Validated)
        .await?;
    let server = client.server_info().await?;

    let balance_drops: u64 = info.account_data.balance.parse()?;
    let balance_xrp = balance_drops as f64 / 1_000_000.0;

    println!("Account:      {}", info.account_data.account);
    println!(
        "Balance:      {} XRP ({} drops)",
        balance_xrp, balance_drops
    );
    println!("Sequence:     {}", info.account_data.sequence);
    println!("Owner count:  {}", info.account_data.owner_count);

    if let Some(vl) = &server.info.validated_ledger {
        let reserve_base = vl.reserve_base_xrp.unwrap_or(10.0);
        let reserve_inc = vl.reserve_inc_xrp.unwrap_or(2.0);

        let available = xrpl_core::utils::reserve::available_balance_drops(
            balance_drops,
            reserve_base,
            reserve_inc,
            info.account_data.owner_count,
        );

        println!("\nReserve base: {} XRP", reserve_base);
        println!("Reserve inc:  {} XRP per object", reserve_inc);
        println!(
            "Total reserve: {} XRP",
            xrpl_core::utils::reserve::total_reserve_drops(
                reserve_base,
                reserve_inc,
                info.account_data.owner_count
            ) as f64
                / 1_000_000.0
        );
        println!("Available:    {} XRP", available as f64 / 1_000_000.0);
    }

    Ok(())
}
