pub mod client;
pub mod error;
pub mod http;
pub mod requests;
pub mod types;

#[cfg(not(target_arch = "wasm32"))]
pub mod ws;

#[cfg(target_arch = "wasm32")]
pub mod ws_wasm;

pub use client::Client;
pub use error::ClientError;
pub use http::XrplHttpClient;
pub use requests::{Paginated, Request};

#[cfg(not(target_arch = "wasm32"))]
pub use ws::XrplWsClient;

#[cfg(target_arch = "wasm32")]
pub use ws_wasm::XrplWsClient;
