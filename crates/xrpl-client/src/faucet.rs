//! Test-network faucets: fund an account with test XRP on Testnet or Devnet.
//! The faucet only ever learns the address; keys stay with the caller.

use crate::client::{sleep_secs, Client};
use crate::error::ClientError;
use crate::http::XrplHttpClient;
use crate::types::LedgerIndex;
use serde_json::{json, Value};

/// A public test network with a faucet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaucetNetwork {
    /// The XRP Ledger Testnet (altnet).
    Testnet,
    /// The XRP Ledger Devnet, where new amendments arrive first.
    Devnet,
}

impl FaucetNetwork {
    /// The faucet's account-funding endpoint.
    pub fn faucet_url(self) -> &'static str {
        match self {
            FaucetNetwork::Testnet => "https://faucet.altnet.rippletest.net/accounts",
            FaucetNetwork::Devnet => "https://faucet.devnet.rippletest.net/accounts",
        }
    }

    /// A public JSON-RPC server on the network.
    pub fn rpc_url(self) -> &'static str {
        match self {
            FaucetNetwork::Testnet => "https://s.altnet.rippletest.net:51234",
            FaucetNetwork::Devnet => "https://s.devnet.rippletest.net:51234",
        }
    }

    /// A public WebSocket server on the network.
    pub fn ws_url(self) -> &'static str {
        match self {
            FaucetNetwork::Testnet => "wss://s.altnet.rippletest.net:51233",
            FaucetNetwork::Devnet => "wss://s.devnet.rippletest.net:51233",
        }
    }
}

/// What the faucet sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaucetFunding {
    /// The funded classic address.
    pub address: String,
    /// The test XRP sent, as the faucet reported it (`"100"`).
    pub amount_xrp: String,
    /// The funding Payment's hash.
    pub transaction_hash: Option<String>,
}

/// Ask `network`'s faucet to send test XRP to `address`. Returns once the
/// faucet has submitted the payment, not once it is validated (see
/// [`fund_and_wait`]).
pub async fn fund(network: FaucetNetwork, address: &str) -> Result<FaucetFunding, ClientError> {
    let response = reqwest::Client::new()
        .post(network.faucet_url())
        .json(&json!({ "destination": address, "userAgent": concat!("halcyon-xrpl-client/", env!("CARGO_PKG_VERSION")) }))
        .send()
        .await?;
    let status = response.status();
    let body: Value = response.json().await?;
    if !status.is_success() {
        return Err(ClientError::UnexpectedResponse(format!(
            "faucet answered {status}: {body}"
        )));
    }
    let funded = body
        .pointer("/account/classicAddress")
        .or_else(|| body.pointer("/account/address"))
        .and_then(Value::as_str)
        .unwrap_or(address);
    Ok(FaucetFunding {
        address: funded.to_string(),
        amount_xrp: body
            .get("amount")
            .map(|a| a.to_string())
            .unwrap_or_default(),
        transaction_hash: body
            .get("transactionHash")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

/// [`fund`], then wait (up to about 40 seconds) until a validated ledger on the
/// network shows the account.
pub async fn fund_and_wait(
    network: FaucetNetwork,
    address: &str,
) -> Result<FaucetFunding, ClientError> {
    let funding = fund(network, address).await?;
    let client = XrplHttpClient::new(network.rpc_url())?;
    for _ in 0..20 {
        sleep_secs(2).await;
        if client
            .account_info(address, &LedgerIndex::Validated)
            .await
            .is_ok()
        {
            return Ok(funding);
        }
    }
    Err(ClientError::UnexpectedResponse(format!(
        "{address} did not appear in a validated ledger after the faucet funded it"
    )))
}
