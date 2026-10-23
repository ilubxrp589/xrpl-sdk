//! The offline commands of `hxrpl`, run as a user would.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use std::process::Command;

fn hxrpl(args: &[&str], env: &[(&str, &str)]) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_hxrpl"))
        .args(args)
        .env_remove("XRPL_SEED")
        .env_remove("XRPL_NETWORK")
        .envs(env.iter().copied())
        .output()
        .unwrap();
    (
        out.status.success(),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

const SIGNED: &str = "120000220000000024000000016140000000000F424068400000000000000C7321ED01FA53FA5A7E77798F882ECE20B1ABC00BB358A9E55A202D0D0676BD0CE37A637440A7C427E6D84B1DB144466DF589D88C2F4CFC530743015EF5E5C31587FDA5334EB5A7E9C418DC17702A56860C28EEC2475098106330504543CEF0965D27EF570C8114D28B177E48D9A8D057E70F7E464B498367281B988314B5F762798A53D543A014CAF8B297CFF8F2F937E8";

#[test]
fn hashes_decodes_and_encodes_offline() {
    let (ok, out, _) = hxrpl(&["tx", "hash", SIGNED], &[]);
    assert!(ok);
    assert_eq!(
        out.trim(),
        "05DD129B3CBA984E7DEB24FA2522C9DA9AAF4A518F1A01C78F2BD35B46298BF3"
    );
    let (ok, json, _) = hxrpl(&["tx", "decode", SIGNED], &[]);
    assert!(ok);
    assert!(json.contains("\"TransactionType\": \"Payment\""));
    let (ok, blob, _) = hxrpl(&["tx", "encode", &json], &[]);
    assert!(ok);
    assert_eq!(blob.trim(), SIGNED);
}

#[test]
fn signs_with_the_seed_from_the_environment_only() {
    let tx = r#"{"TransactionType":"Payment","Account":"rLUEXYuLiQptky37CqLcm9USQpPiz5rkpD","Destination":"rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh","Amount":"1000000","Fee":"12","Sequence":1,"Flags":0}"#;
    let (ok, out, _) = hxrpl(
        &["tx", "sign", tx],
        &[("XRPL_SEED", "sEdSKaCy2JT7JaM7v95H9SxkhP9wS2r")],
    );
    assert!(ok);
    // rippled 3.4.1 signs the same transaction to the same hash.
    assert!(
        out.contains("05DD129B3CBA984E7DEB24FA2522C9DA9AAF4A518F1A01C78F2BD35B46298BF3"),
        "{out}"
    );
    let (ok, _, err) = hxrpl(&["tx", "sign", tx], &[]);
    assert!(!ok && err.contains("XRPL_SEED"), "{err}");
}

#[test]
fn computes_ids_and_converts() {
    let (_, out, _) = hxrpl(
        &["id", "account", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh"],
        &[],
    );
    assert_eq!(
        out.trim(),
        "2B6AC232AA4C4BE41BF49D2459FA4A0347E1B543A4C92FCEE0821C0201E2E9A8"
    );
    let (_, out, _) = hxrpl(
        &[
            "id",
            "trust-line",
            "rB5TihdPbKgMrkFqrqUC3yLdE8hhv4BdeY",
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "USD",
        ],
        &[],
    );
    assert_eq!(
        out.trim(),
        "C683B5BB928F025F1E860D9D69D6C554C2202DE0D45877ADB3077DA4CB9E125C"
    );
    assert_eq!(
        hxrpl(&["convert", "xrp-to-drops", "1.5"], &[]).1.trim(),
        "1500000"
    );
    assert_eq!(
        hxrpl(&["convert", "drops-to-xrp", "12"], &[]).1.trim(),
        "0.000012"
    );
    assert_eq!(
        hxrpl(&["convert", "time", "0"], &[]).1.trim(),
        "2000-01-01T00:00:00Z"
    );
    assert_eq!(
        hxrpl(&["convert", "time", "2000-01-01T00:00:01Z"], &[])
            .1
            .trim(),
        "1"
    );
    assert_eq!(
        hxrpl(&["convert", "hex", "example.com"], &[]).1.trim(),
        "6578616D706C652E636F6D"
    );
    let (_, nft, _) = hxrpl(
        &[
            "convert",
            "nft-id",
            "000B0539C35B55AA096BA6D87A6E6C965A6534150DC56E5E12C5D09E0000000C",
        ],
        &[],
    );
    assert!(
        nft.contains("rJoxBSzpXhPtAuqFmqxQtGKjA13jUJWthE") && nft.contains("\"taxon\": 1337"),
        "{nft}"
    );
}

#[test]
fn wallets_and_mainnet_guards() {
    let (ok, out, _) = hxrpl(
        &["wallet", "show"],
        &[("XRPL_SEED", "sp5fghtJtpUorTwvof1NpDXAzNwf5")],
    );
    assert!(
        ok && out.contains("rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1"),
        "{out}"
    );
    let (ok, out, _) = hxrpl(&["wallet", "new"], &[]);
    assert!(ok && out.contains("\"seed\": \"sEd"), "{out}");
    let (ok, _, err) = hxrpl(&["--network", "mainnet", "wallet", "faucet"], &[]);
    assert!(!ok && err.contains("no faucet"), "{err}");
    let (ok, _, err) = hxrpl(&["--network", "mainnet", "tx", "submit", SIGNED], &[]);
    assert!(!ok && err.contains("--yes"), "{err}");
}

#[test]
fn long_currency_codes_become_hex() {
    // RLUSD's trust line ID is the same whether the code is given as text or as hex.
    let text = hxrpl(
        &[
            "id",
            "trust-line",
            "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De",
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "RLUSD",
        ],
        &[],
    );
    let hex = hxrpl(
        &[
            "id",
            "trust-line",
            "rMxCKbEDwqr76QuheSUMdEGf4B9xJ8m5De",
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "524C555344000000000000000000000000000000",
        ],
        &[],
    );
    assert!(text.0 && hex.0);
    assert_eq!(text.1, hex.1);
}

/// A one-request JSON-RPC server answering `server_info` with `network_id`.
fn fake_server(network_id: u32) -> String {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let body = format!(
                r#"{{"result":{{"info":{{"network_id":{network_id}}},"status":"success"}}}}"#
            );
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    url
}

/// --url pointing at a mainnet server needs --yes even though --network
/// says Testnet (the default): the server is asked which network it is on.
#[test]
fn the_server_decides_whether_yes_is_needed() {
    let url = fake_server(0);
    let (ok, _, err) = hxrpl(&["--url", &url, "tx", "submit", SIGNED], &[]);
    assert!(
        !ok && err.contains("mainnet") && err.contains("--yes"),
        "{err}"
    );
    let url = fake_server(21337);
    let (ok, _, err) = hxrpl(
        &["--url", &url, "raw", "submit", r#"{"tx_blob":"00"}"#],
        &[],
    );
    assert!(
        !ok && err.contains("21337") && err.contains("--yes"),
        "{err}"
    );
    let url = fake_server(0);
    let (ok, _, err) = hxrpl(
        &[
            "--url",
            &url,
            "tx",
            "send-xrp",
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
            "1",
            "--seed-file",
            "/nonexistent",
        ],
        &[],
    );
    assert!(!ok, "{err}");
}

#[test]
fn a_new_seed_can_go_to_a_private_file() {
    let dir = std::env::temp_dir().join(format!("hxrpl-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("seed");
    let _ = std::fs::remove_file(&path);
    let p = path.to_str().unwrap();
    let (ok, out, _) = hxrpl(&["wallet", "new", "--seed-out", p], &[]);
    assert!(ok && !out.contains("\"seed\""), "{out}");
    let seed = std::fs::read_to_string(&path).unwrap();
    assert!(seed.starts_with("sEd"), "{seed}");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    // Never overwritten.
    let (ok, _, err) = hxrpl(&["wallet", "new", "--seed-out", p], &[]);
    assert!(!ok && err.contains("never overwritten"), "{err}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), seed);
    // Reading a seed file others can read is warned about.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let (ok, _, err) = hxrpl(&["wallet", "show", "--seed-file", p], &[]);
        assert!(ok && err.contains("chmod 600"), "{err}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn watch_refuses_a_url_that_is_not_websocket() {
    let (ok, _, err) = hxrpl(&["--url", "https://example.com", "watch", "ledger"], &[]);
    assert!(!ok && err.contains("WebSocket"), "{err}");
}
