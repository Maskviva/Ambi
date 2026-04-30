// src/runtime.rs

//! The platform-agnostic asynchronous runtime abstraction layer.
//!
//! This module bridges standard `tokio` capabilities on native platforms
//! and adapts them for `wasm32` execution inside browsers. It provides a
//! unified API for spawning tasks, handling timeouts, and enforcing thread safety.

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

// ============================================================================
// Native Implementation (Linux, Windows, macOS)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
mod native {
    // Directly re-export Tokio's native implementations
    pub use tokio::spawn;
    pub use tokio::task::spawn_blocking;
    pub use tokio::time::{sleep, timeout};

    /// A marker trait that enforces thread safety (`Send + Sync`) on native platforms.
    /// It gracefully degrades to nothing in single-threaded WASM environments.
    pub trait SendSync: Send + Sync {}

    impl<T: ?Sized + Send + Sync> SendSync for T {}
}

// ============================================================================
// WebAssembly Implementation (Browser Polyfills)
// ============================================================================

#[cfg(target_arch = "wasm32")]
mod wasm {
    use futures::future::{select, Either};
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use std::time::Duration;

    // --- Dummy Error Types ---

    /// A polyfill for Tokio's `JoinError` when executing blocking tasks in WASM.
    #[derive(Debug)]
    pub struct JoinError;

    impl std::fmt::Display for JoinError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "wasm spawn_blocking error")
        }
    }

    impl std::error::Error for JoinError {}

    /// A polyfill for Tokio's `Elapsed` error when a timeout triggers in WASM.
    #[derive(Debug)]
    pub struct Elapsed;

    impl std::fmt::Display for Elapsed {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "deadline has elapsed")
        }
    }

    impl std::error::Error for Elapsed {}

    // --- Core Runtime Functions ---

    /// Suspends execution for the specified duration using browser timers.
    #[inline]
    pub async fn sleep(duration: Duration) {
        gloo_timers::future::sleep(duration).await;
    }

    /// Simulates `tokio::task::spawn_blocking` in a single-threaded WASM environment.
    /// It bypasses the thread pool and executes the closure directly.
    #[inline]
    pub async fn spawn_blocking<F, R>(f: F) -> Result<R, JoinError>
    where
        F: FnOnce() -> R + 'static,
        R: 'static,
    {
        Ok(f())
    }

    /// Simulates `tokio::time::timeout` by racing the provided future against a browser timer.
    pub async fn timeout<T>(duration: Duration, future: T) -> Result<T::Output, Elapsed>
    where
        T: Future,
    {
        let timer = Box::pin(sleep(duration));
        let fut = Box::pin(future);

        match select(fut, timer).await {
            Either::Left((res, _)) => Ok(res),
            Either::Right((_, _)) => Err(Elapsed),
        }
    }

    // --- Task Spawning ---

    /// A mock handle mimicking `tokio::task::JoinHandle`.
    pub struct JoinHandle {
        _private: (),
    }

    impl Future for JoinHandle {
        type Output = Result<(), JoinError>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            // The task was spawned to the JS event loop, so we immediately resolve the handle.
            Poll::Ready(Ok(()))
        }
    }

    /// Spawns an asynchronous task onto the JavaScript event loop.
    pub fn spawn<F>(future: F) -> JoinHandle
    where
        F: Future<Output = ()> + 'static,
    {
        wasm_bindgen_futures::spawn_local(future);
        JoinHandle { _private: () }
    }

    // --- Cross-Platform Traits ---

    /// A marker trait that enforces thread safety on native platforms,
    /// but safely degrades to nothing in single-threaded WASM environments.
    pub trait SendSync {}

    impl<T: ?Sized> SendSync for T {}
}
