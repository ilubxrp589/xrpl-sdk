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
///
/// # Errors
///
/// [`ClientError::HttpStatus`] when the faucet refuses (429 when it limits
/// its rate); [`ClientError::Http`] when it cannot be reached;
/// [`ClientError::Json`] for an answer that is not JSON; and
/// [`ClientError::UnexpectedResponse`] when it reports funding another
/// address, or its answer is over 1 MiB or not UTF-8.
pub async fn fund(network: FaucetNetwork, address: &str) -> Result<FaucetFunding, ClientError> {
    use crate::http::{http_error, read_capped};
    #[cfg(not(target_arch = "wasm32"))]
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(http_error)?;
    #[cfg(target_arch = "wasm32")]
    let client = reqwest::Client::new();
    let response = client
        .post(network.faucet_url())
        .json(&json!({ "destination": address, "userAgent": concat!("halcyon-xrpl-client/", env!("CARGO_PKG_VERSION")) }))
        .send()
        .await
        .map_err(http_error)?;
    let status = response.status();
    let retry_after = crate::http::retry_after(response.headers());
    let text = read_capped(response, 1024 * 1024).await?;
    if !status.is_success() {
        // A faucet that limits its rate answers 429, often with Retry-After.
        return Err(ClientError::HttpStatus {
            status: status.as_u16(),
            retry_after,
            excerpt: text.chars().take(200).collect(),
        });
    }
    let body: Value = serde_json::from_str(&text)?;
    let funded = body
        .pointer("/account/classicAddress")
        .or_else(|| body.pointer("/account/address"))
        .and_then(Value::as_str)
        .unwrap_or(address);
    if funded != address {
        return Err(ClientError::UnexpectedResponse(format!(
            "the faucet funded {funded}, not {address}"
        )));
    }
    Ok(FaucetFunding {
        address: funded.to_string(),
        amount_xrp: match body.get("amount") {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            _ => String::new(),
        },
        transaction_hash: body
            .get("transactionHash")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

/// The account's XRP balance in a validated ledger, in drops; 0 for an
/// account that does not exist yet.
async fn validated_balance(client: &XrplHttpClient, address: &str) -> Result<u64, ClientError> {
    match client.account_info(address, &LedgerIndex::Validated).await {
        Ok(info) => info.account_data.balance.parse().map_err(|_| {
            ClientError::UnexpectedResponse("an XRP balance that is not drops".into())
        }),
        Err(ClientError::RpcError { ref code, .. }) if code == "actNotFound" => Ok(0),
        Err(e) => Err(e),
    }
}

/// [`fund`], then wait (up to about 40 seconds) until a validated ledger on
/// the network shows the account's balance grown by the faucet's payment
/// (an account that already existed is funded too, not just found).
///
/// # Errors
///
/// The errors of [`fund`]; a [`ClientError`] when the network's server
/// cannot be asked for the balance before funding; and
/// [`ClientError::UnexpectedResponse`] when no validated ledger shows the
/// balance grown within about 40 seconds.
pub async fn fund_and_wait(
    network: FaucetNetwork,
    address: &str,
) -> Result<FaucetFunding, ClientError> {
    let client = XrplHttpClient::new(network.rpc_url())?;
    let before = validated_balance(&client, address).await?;
    let funding = fund(network, address).await?;
    for _ in 0..20 {
        sleep_secs(2).await;
        if validated_balance(&client, address)
            .await
            .is_ok_and(|now| now > before)
        {
            return Ok(funding);
        }
    }
    Err(ClientError::UnexpectedResponse(format!(
        "{address}'s balance did not grow in a validated ledger after the faucet funded it"
    )))
}
