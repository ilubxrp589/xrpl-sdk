#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

pub mod address;
#[cfg(feature = "std")]
pub mod codec;
pub mod crypto;
pub mod error;
pub mod transaction;
pub mod types;
pub mod utils;
#[cfg(feature = "wasm")]
pub mod wasm_bindgen_exports;

pub use error::CoreError;
pub use types::*;
