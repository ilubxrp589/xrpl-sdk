//! Check account balance and reserve calculation.
//!
//! Fetches account_info and server_info, then calculates
//! the available (spendable) balance after reserves.
//!
//! Usage: cargo run --example check_balance

use xrpl_sdk::{utils, Client, LedgerIndex, XrplHttpClient};

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
    let balance_xrp = utils::drops_to_xrp(balance_drops);

    println!("Account:      {}", info.account_data.account);
    println!(
        "Balance:      {} XRP ({} drops)",
        balance_xrp, balance_drops
    );
    println!("Sequence:     {}", info.account_data.sequence);
    println!("Owner count:  {}", info.account_data.owner_count);

    if let Some(vl) = &server.info.validated_ledger {
        use xrpl_core::utils::reserve;
        // server_info gives the reserves in XRP; convert them exactly.
        let base =
            reserve::reserve_xrp_to_drops(vl.reserve_base_xrp.ok_or("no reserve_base_xrp")?)?;
        let inc = reserve::reserve_xrp_to_drops(vl.reserve_inc_xrp.ok_or("no reserve_inc_xrp")?)?;
        let owners = info.account_data.owner_count;
        let total = reserve::total_reserve_drops(base, inc, owners);
        let available = reserve::available_balance_drops(balance_drops, base, inc, owners);

        println!("\nReserve base: {} XRP", utils::drops_to_xrp(base));
        println!("Reserve inc:  {} XRP per object", utils::drops_to_xrp(inc));
        println!("Total reserve: {} XRP", utils::drops_to_xrp(total));
        println!("Available:    {} XRP", utils::drops_to_xrp(available));
    }

    Ok(())
}
