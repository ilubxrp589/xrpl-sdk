//! [`MaybeSend`] and [`MaybeSync`]: `Send` and `Sync` on native targets,
//! nothing on `wasm32`, where JavaScript values are not `Send`. Futures this
//! crate and `halcyon-xrpl-client` hand out are `MaybeSend`, so on a native
//! target they can move between threads (`tokio::spawn`).

/// `Send` on native targets, so a future can move between threads
/// (`tokio::spawn`); nothing on `wasm32`, where JavaScript values are not
/// `Send`. Implemented for every type that qualifies.
#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeSend: Send {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send + ?Sized> MaybeSend for T {}
/// `Send` on native targets, so a future can move between threads
/// (`tokio::spawn`); nothing on `wasm32`, where JavaScript values are not
/// `Send`. Implemented for every type that qualifies.
#[cfg(target_arch = "wasm32")]
pub trait MaybeSend {}
#[cfg(target_arch = "wasm32")]
impl<T: ?Sized> MaybeSend for T {}

/// `Sync` on native targets; nothing on `wasm32` (see [`MaybeSend`]).
#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeSync: Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Sync + ?Sized> MaybeSync for T {}
/// `Sync` on native targets; nothing on `wasm32` (see [`MaybeSend`]).
#[cfg(target_arch = "wasm32")]
pub trait MaybeSync {}
#[cfg(target_arch = "wasm32")]
impl<T: ?Sized> MaybeSync for T {}
