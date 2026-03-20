pub mod error;
pub mod http;
pub mod types;

#[cfg(not(target_arch = "wasm32"))]
pub mod ws;

#[cfg(target_arch = "wasm32")]
pub mod ws_wasm;

pub use error::ClientError;
pub use http::XrplHttpClient;

#[cfg(not(target_arch = "wasm32"))]
pub use ws::XrplWsClient;

#[cfg(target_arch = "wasm32")]
pub use ws_wasm::XrplWsClient;
