//! End to end on the XRPL Testnet (`--features live-tests`): two faucet
//! wallets submit one transaction of each kind that creates a ledger entry,
//! and every entry's ID must be the one `keylet` computes. Along the way the
//! SDK's own transaction hash, NFTokenID, balance changes and payment
//! channel claim signature are checked against what the network did.
#![cfg(feature = "live-tests")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use serde_json::{json, Value};
use xrpl_sdk::utils::{self, nftoken::parse_nftoken_id, str_to_hex};
use xrpl_sdk::{
    generate_faucet_wallet, hash_signed_tx_blob, keylet, Client, ClientError, FaucetNetwork,
    LedgerIndex, Wallet, XrplHttpClient,
};

/// Autofill, sign, submit and wait. `None` when the network has not enabled
/// the transaction's amendment (temDISABLED); panics on any other failure.
async fn submit(client: &XrplHttpClient, wallet: &Wallet, mut tx: Value) -> Option<Value> {
    client.autofill(&mut tx).await.unwrap();
    let blob = wallet.sign_and_encode(&tx).unwrap();
    let our_hash = hash_signed_tx_blob(&blob).unwrap().to_string();
    let last = tx["LastLedgerSequence"].as_u64().unwrap() as u32;
    let result = match client.submit_and_wait(&blob, last).await {
        Ok(r) => r,
        Err(ClientError::TransactionFailed { engine_result, .. })
            if engine_result == "temDISABLED" =>
        {
            println!(
                "  {} skipped: not enabled on testnet",
                tx["TransactionType"]
            );
            return None;
        }
        Err(e) => panic!("{}: {e}", tx["TransactionType"]),
    };
    let meta = result.meta.clone().unwrap();
    assert_eq!(
        meta["TransactionResult"], "tesSUCCESS",
        "{}",
        tx["TransactionType"]
    );
    assert_eq!(
        result.tx_fields["hash"],
        json!(our_hash),
        "hash_signed_tx_blob"
    );
    let mut out = tx;
    out["meta"] = meta;
    Some(out)
}

/// The ID of the one entry of `entry_type` the transaction created.
fn created(tx: &Value, entry_type: &str) -> String {
    let ids: Vec<&str> = tx["meta"]["AffectedNodes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|n| n.get("CreatedNode"))
        .filter(|n| n["LedgerEntryType"] == entry_type)
        .map(|n| n["LedgerIndex"].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 1, "{entry_type} entries created: {ids:?}");
    ids[0].to_string()
}

fn check(checked: &mut Vec<String>, name: &str, got: String, want: String) {
    assert_eq!(got, want, "{name}");
    checked.push(name.to_string());
}

fn seq(tx: &Value) -> u32 {
    tx["Sequence"].as_u64().unwrap() as u32
}

#[tokio::test]
async fn keylets_and_helpers_match_testnet() {
    let net = FaucetNetwork::Testnet;
    let (alice, bob) = (
        generate_faucet_wallet(net).await.unwrap(),
        generate_faucet_wallet(net).await.unwrap(),
    );
    let (a, b) = (alice.address.as_str(), bob.address.as_str());
    let client = XrplHttpClient::new(net.rpc_url()).unwrap();
    let now = utils::time::system_time_to_ripple_time(std::time::SystemTime::now()).unwrap();
    let mut checked = Vec::new();

    // A Payment: what get_balance_changes reports.
    let pay = submit(
        &client,
        &alice,
        json!({"TransactionType": "Payment", "Account": a, "Destination": b, "Amount": "1000000"}),
    )
    .await
    .unwrap();
    let changes = utils::get_balance_changes(&pay["meta"]).unwrap();
    let fee = utils::drops_to_xrp(pay["Fee"].as_str().unwrap().parse().unwrap());
    let of = |acct: &str| {
        changes.iter().find(|c| c.account == acct).unwrap().balances[0]
            .value
            .clone()
    };
    assert_eq!(of(b), "1");
    assert_eq!(
        of(a),
        format!(
            "-{}",
            utils::xrp::drops_to_xrp(1_000_000 + utils::xrp_to_drops(&fee).unwrap())
        )
    );
    checked.push("get_balance_changes".into());

    // ledger_entry by the computed ID returns Alice's AccountRoot.
    let entry = client
        .ledger_entry(
            &keylet::account(a).unwrap().to_string(),
            &LedgerIndex::Validated,
        )
        .await
        .unwrap();
    assert_eq!(entry.node.to_value().unwrap()["Account"], a);
    checked.push("account".into());

    if let Some(tx) = submit(
        &client,
        &alice,
        json!({"TransactionType": "TicketCreate", "Account": a, "TicketCount": 1}),
    )
    .await
    {
        check(
            &mut checked,
            "ticket",
            keylet::ticket(a, seq(&tx) + 1).unwrap().to_string(),
            created(&tx, "Ticket"),
        );
        check(
            &mut checked,
            "owner_dir",
            keylet::owner_dir(a).unwrap().to_string(),
            created(&tx, "DirectoryNode"),
        );
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "CheckCreate", "Account": a, "Destination": b, "SendMax": "1000000"})).await {
        check(&mut checked, "check", keylet::check(a, seq(&tx)).unwrap().to_string(), created(&tx, "Check"));
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "EscrowCreate", "Account": a, "Destination": b, "Amount": "1000000", "FinishAfter": now + 3600})).await {
        check(&mut checked, "escrow", keylet::escrow(a, seq(&tx)).unwrap().to_string(), created(&tx, "Escrow"));
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "PaymentChannelCreate", "Account": a, "Destination": b, "Amount": "2000000", "SettleDelay": 60, "PublicKey": alice.public_key_hex()})).await {
        let channel = created(&tx, "PayChannel");
        check(&mut checked, "payment_channel", keylet::payment_channel(a, b, seq(&tx)).unwrap().to_string(), channel.clone());
        // Bob redeems 0.5 XRP with Alice's claim: the network accepts our signature.
        let signature = alice.sign_payment_channel_claim(&channel, 500_000).unwrap();
        submit(&client, &bob, json!({"TransactionType": "PaymentChannelClaim", "Account": b, "Channel": channel, "Balance": "500000", "Amount": "500000", "Signature": signature, "PublicKey": alice.public_key_hex()})).await.unwrap();
        checked.push("payment channel claim accepted".into());
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "NFTokenMint", "Account": a, "NFTokenTaxon": 7, "Flags": 8, "TransferFee": 500})).await {
        let id = utils::get_nftoken_id(&tx["meta"]).unwrap().unwrap();
        let mut meta = tx["meta"].clone();
        meta.as_object_mut().unwrap().remove("nftoken_id");
        check(&mut checked, "get_nftoken_id (page diff)", utils::get_nftoken_id(&meta).unwrap().unwrap(), id.clone());
        let p = parse_nftoken_id(&id).unwrap();
        assert_eq!((p.issuer.as_str(), p.taxon, p.flags, p.transfer_fee), (a, 7, 8, 500));
        if let Some(offer) = submit(&client, &alice, json!({"TransactionType": "NFTokenCreateOffer", "Account": a, "NFTokenID": id, "Amount": "1", "Flags": 1})).await {
            check(&mut checked, "nftoken_offer", keylet::nftoken_offer(a, seq(&offer)).unwrap().to_string(), created(&offer, "NFTokenOffer"));
        }
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "DIDSet", "Account": a, "URI": str_to_hex("https://example.com/did")})).await {
        check(&mut checked, "did", keylet::did(a).unwrap().to_string(), created(&tx, "DID"));
    }
    let unix_now = utils::ripple_time_to_unix(now);
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "OracleSet", "Account": a, "OracleDocumentID": 1, "Provider": str_to_hex("halcyon"), "AssetClass": str_to_hex("currency"), "LastUpdateTime": unix_now, "PriceDataSeries": [{"PriceData": {"BaseAsset": "XRP", "QuoteAsset": "USD", "AssetPrice": 74, "Scale": 2}}]})).await {
        check(&mut checked, "oracle", keylet::oracle(a, 1).unwrap().to_string(), created(&tx, "Oracle"));
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "MPTokenIssuanceCreate", "Account": a, "AssetScale": 2, "MaximumAmount": "100000", "Flags": 0x20})).await {
        let id = keylet::mpt_issuance_id(seq(&tx), a).unwrap().to_string();
        if let Some(rippled_id) = tx["meta"].get("mpt_issuance_id").and_then(Value::as_str) {
            check(&mut checked, "mpt_issuance_id", id.clone(), rippled_id.to_string());
        }
        check(&mut checked, "mpt_issuance", keylet::mpt_issuance(&id).unwrap().to_string(), created(&tx, "MPTokenIssuance"));
        if let Some(auth) = submit(&client, &bob, json!({"TransactionType": "MPTokenAuthorize", "Account": b, "MPTokenIssuanceID": id})).await {
            check(&mut checked, "mptoken", keylet::mptoken(&id, b).unwrap().to_string(), created(&auth, "MPToken"));
        }
    }
    let credential_type = str_to_hex("KYC");
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "CredentialCreate", "Account": a, "Subject": b, "CredentialType": credential_type})).await {
        check(&mut checked, "credential", keylet::credential(b, a, &credential_type).unwrap().to_string(), created(&tx, "Credential"));
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "PermissionedDomainSet", "Account": a, "AcceptedCredentials": [{"Credential": {"Issuer": a, "CredentialType": credential_type}}]})).await {
        check(&mut checked, "permissioned_domain", keylet::permissioned_domain(a, seq(&tx)).unwrap().to_string(), created(&tx, "PermissionedDomain"));
    }
    if let Some(tx) = submit(
        &client,
        &alice,
        json!({"TransactionType": "DepositPreauth", "Account": a, "Authorize": b}),
    )
    .await
    {
        check(
            &mut checked,
            "deposit_preauth",
            keylet::deposit_preauth(a, b).unwrap().to_string(),
            created(&tx, "DepositPreauth"),
        );
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "DelegateSet", "Account": a, "Authorize": b, "Permissions": [{"Permission": {"PermissionValue": "Payment"}}]})).await {
        check(&mut checked, "delegate", keylet::delegate(a, b).unwrap().to_string(), created(&tx, "Delegate"));
    }
    if let Some(tx) = submit(&client, &alice, json!({"TransactionType": "TrustSet", "Account": a, "LimitAmount": {"currency": "USD", "issuer": b, "value": "100"}})).await {
        check(&mut checked, "trust_line", keylet::trust_line(a, b, "USD").unwrap().to_string(), created(&tx, "RippleState"));
    }
    println!("checked on testnet: {}", checked.join(", "));
    assert!(checked.len() >= 12, "only {checked:?}");
}
