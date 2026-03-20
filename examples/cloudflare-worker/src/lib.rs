//! Example Cloudflare Worker using xrpl-core WASM for transaction decoding.
//! Deploy with: wrangler deploy

use worker::*;

#[event(fetch)]
async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    let path = url.path();

    match path {
        "/decode" => handle_decode(req).await,
        "/validate" => handle_validate(req).await,
        "/encode" => handle_encode(req).await,
        _ => Response::ok(
            "XRPL Rust SDK — Cloudflare Worker\n\n\
             Endpoints:\n  \
             POST /decode  — decode hex blob\n  \
             POST /validate — validate address\n  \
             POST /encode  — encode transaction JSON",
        ),
    }
}

async fn handle_decode(mut req: Request) -> Result<Response> {
    #[derive(serde::Deserialize)]
    struct Body {
        blob: String,
    }

    let body: Body = req.json().await?;
    let bytes =
        hex::decode(&body.blob).map_err(|e| worker::Error::RustError(format!("invalid hex: {e}")))?;
    let decoded = xrpl_core::codec::decode_transaction_binary(&bytes)
        .map_err(|e| worker::Error::RustError(e.to_string()))?;
    Response::from_json(&decoded)
}

async fn handle_validate(mut req: Request) -> Result<Response> {
    #[derive(serde::Deserialize)]
    struct Body {
        address: String,
    }
    #[derive(serde::Serialize)]
    struct Out {
        valid: bool,
        account_id: Option<String>,
    }

    let body: Body = req.json().await?;
    let valid = xrpl_core::address::decode_account_id(&body.address).is_ok();
    let account_id = if valid {
        xrpl_core::address::decode_account_id(&body.address)
            .ok()
            .map(|b| hex::encode_upper(b))
    } else {
        None
    };
    Response::from_json(&Out { valid, account_id })
}

async fn handle_encode(mut req: Request) -> Result<Response> {
    let tx: serde_json::Value = req.json().await?;
    let bytes = xrpl_core::codec::encode_transaction_json(&tx, false)
        .map_err(|e| worker::Error::RustError(e.to_string()))?;
    #[derive(serde::Serialize)]
    struct Out {
        blob: String,
    }
    Response::from_json(&Out {
        blob: hex::encode_upper(bytes),
    })
}
