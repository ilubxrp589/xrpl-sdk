//! [`MaybeSend`] and [`MaybeSync`]: `Send` and `Sync` on native targets,
//! nothing on `wasm32`, where JavaScript values are not `Send`. Futures this
//! crate and `halcyon-xrpl-client` hand out are `MaybeSend`, so on a native
//! target they can move between threads (`tokio::spawn`).

/// `Send` on native targets, so a future can move between threads
/// (`tokio::spawn`); nothing on `wasm32`, where JavaScript values are not
/// `Send`. Implemented for every type that qualifies.
///
/// ```
/// use xrpl_core::MaybeSend;
///
/// /// Runs `future` to completion on another thread, which needs `Send`:
/// /// on a native target, a `MaybeSend` future is `Send`.
/// fn on_another_thread<F>(future: F) -> F::Output
/// where
///     F: core::future::Future + MaybeSend,
///     F::Output: MaybeSend,
/// {
///     std::thread::scope(|s| s.spawn(|| block_on(future)).join().unwrap())
/// }
///
/// assert_eq!(on_another_thread(async { 6 * 7 }), 42);
/// # fn block_on<F: core::future::Future>(f: F) -> F::Output {
/// #     use core::task::{Context, Poll, Waker};
/// #     let mut f = core::pin::pin!(f);
/// #     loop {
/// #         if let Poll::Ready(v) = f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
/// #             return v;
/// #         }
/// #     }
/// # }
/// ```
#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeSend: Send {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send + ?Sized> MaybeSend for T {}
/// `Send` on native targets, so a future can move between threads
/// (`tokio::spawn`); nothing on `wasm32`, where JavaScript values are not
/// `Send`. Implemented for every type that qualifies.
///
/// ```
/// use xrpl_core::MaybeSend;
///
/// /// Runs `future` to completion on another thread, which needs `Send`:
/// /// on a native target, a `MaybeSend` future is `Send`.
/// fn on_another_thread<F>(future: F) -> F::Output
/// where
///     F: core::future::Future + MaybeSend,
///     F::Output: MaybeSend,
/// {
///     std::thread::scope(|s| s.spawn(|| block_on(future)).join().unwrap())
/// }
///
/// assert_eq!(on_another_thread(async { 6 * 7 }), 42);
/// # fn block_on<F: core::future::Future>(f: F) -> F::Output {
/// #     use core::task::{Context, Poll, Waker};
/// #     let mut f = core::pin::pin!(f);
/// #     loop {
/// #         if let Poll::Ready(v) = f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
/// #             return v;
/// #         }
/// #     }
/// # }
/// ```
#[cfg(target_arch = "wasm32")]
pub trait MaybeSend {}
#[cfg(target_arch = "wasm32")]
impl<T: ?Sized> MaybeSend for T {}

/// `Sync` on native targets; nothing on `wasm32` (see [`MaybeSend`]).
///
/// ```
/// use xrpl_core::crypto::{Keypair, Seed, SigningKey};
/// use xrpl_core::MaybeSync;
///
/// /// Reads the key from another thread: `MaybeSync` is `Sync` on a native
/// /// target, so a shared reference may cross.
/// fn public_key_len<K: SigningKey + MaybeSync>(key: &K) -> usize {
///     std::thread::scope(|s| s.spawn(|| key.public_key().len()).join().unwrap())
/// }
///
/// let keys = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// assert_eq!(public_key_len(&keys), 33);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeSync: Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Sync + ?Sized> MaybeSync for T {}
/// `Sync` on native targets; nothing on `wasm32` (see [`MaybeSend`]).
///
/// ```
/// use xrpl_core::crypto::{Keypair, Seed, SigningKey};
/// use xrpl_core::MaybeSync;
///
/// /// Reads the key from another thread: `MaybeSync` is `Sync` on a native
/// /// target, so a shared reference may cross.
/// fn public_key_len<K: SigningKey + MaybeSync>(key: &K) -> usize {
///     std::thread::scope(|s| s.spawn(|| key.public_key().len()).join().unwrap())
/// }
///
/// let keys = Keypair::from_seed(&Seed::from_base58("sp5fghtJtpUorTwvof1NpDXAzNwf5")?)?;
/// assert_eq!(public_key_len(&keys), 33);
/// # Ok::<(), xrpl_core::CoreError>(())
/// ```
#[cfg(target_arch = "wasm32")]
pub trait MaybeSync {}
#[cfg(target_arch = "wasm32")]
impl<T: ?Sized> MaybeSync for T {}
