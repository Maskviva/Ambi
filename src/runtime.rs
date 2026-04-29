// src/runtime.rs

//! The platform-agnostic asynchronous runtime abstraction layer.
//! This module bridges standard `tokio` capabilities on native platforms
//! and adapts them for `wasm32` execution inside browsers.

// Native Implementation (Linux, Windows, macOS)

#[cfg(not(target_arch = "wasm32"))]
pub use tokio::spawn;

#[cfg(not(target_arch = "wasm32"))]
pub use tokio::task::spawn_blocking;

#[cfg(not(target_arch = "wasm32"))]
pub use tokio::time::{sleep, timeout};

// WebAssembly Implementation (Browser Polyfills)

#[cfg(target_arch = "wasm32")]
use std::future::Future;
#[cfg(target_arch = "wasm32")]
use std::time::Duration;

#[cfg(target_arch = "wasm32")]
#[inline]
pub async fn sleep(duration: Duration) {
    gloo_timers::future::sleep(duration).await;
}

// WASM Dummy Errors for Seamless Integration

#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct JoinError;

#[cfg(target_arch = "wasm32")]
impl std::fmt::Display for JoinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "wasm spawn_blocking error")
    }
}
#[cfg(target_arch = "wasm32")]
impl std::error::Error for JoinError {}

#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct Elapsed;

#[cfg(target_arch = "wasm32")]
impl std::fmt::Display for Elapsed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "deadline has elapsed")
    }
}
#[cfg(target_arch = "wasm32")]
impl std::error::Error for Elapsed {}

// WASM Fallback Implementations

#[cfg(target_arch = "wasm32")]
#[inline]
pub async fn spawn_blocking<F, R>(f: F) -> Result<R, JoinError>
where
    F: FnOnce() -> R + 'static,
    R:  'static,
{
    // WASM is inherently single-threaded. We bypass the thread pool and execute directly.
    Ok(f())
}

#[cfg(target_arch = "wasm32")]
pub async fn timeout<T>(duration: Duration, future: T) -> Result<T::Output, Elapsed>
where
    T: Future,
{
    use futures::future::{select, Either};

    let timer = Box::pin(sleep(duration));
    let fut = Box::pin(future);

    // Races the actual future against a browser timeout
    match select(fut, timer).await {
        Either::Left((res, _)) => Ok(res),
        Either::Right((_, _)) => Err(Elapsed),
    }
}

#[cfg(target_arch = "wasm32")]
pub struct JoinHandle {
    _private: (),
}

#[cfg(target_arch = "wasm32")]
impl std::future::Future for JoinHandle {
    type Output = Result<(), JoinError>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::task::Poll::Ready(Ok(()))
    }
}

#[cfg(target_arch = "wasm32")]
pub fn spawn<F>(future: F) -> JoinHandle
where
    F: Future<Output=()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
    JoinHandle { _private: () }
}

// Cross-Platform Send/Sync Marker

/// Send  but gracefully degrades to nothing in single-threaded WASM environments.
#[cfg(not(target_arch = "wasm32"))]
pub trait SendSync: Send + Sync {}

#[cfg(not(target_arch = "wasm32"))]
impl<T: ?Sized + Send + Sync> SendSync for T {}

#[cfg(target_arch = "wasm32")]
pub trait SendSync {}

#[cfg(target_arch = "wasm32")]
impl<T: ?Sized> SendSync for T {}
