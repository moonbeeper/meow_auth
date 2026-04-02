use std::sync::{Arc, atomic::AtomicUsize};

use tokio::sync::Notify;
use tokio_util::sync::{CancellationToken, WaitForCancellationFuture};

// TODO: Maybe add a method that forces futures to return Poll::Ready when the Watcher is stopped.

pub struct WatcherChild(Arc<WatcherInner>);

impl WatcherChild {
    pub fn cancelled(&self) -> WaitForCancellationFuture<'_> {
        self.0.token.cancelled()
    }

    pub fn token(&self) -> CancellationToken {
        self.0.token.clone()
    }
}

impl Drop for WatcherChild {
    fn drop(&mut self) {
        let counter = self
            .0
            .counter
            .fetch_sub(1, std::sync::atomic::Ordering::AcqRel);

        tracing::info!(
            "watched services decremented from {} to {}",
            counter,
            counter - 1
        );

        if counter == 1 {
            self.0.is_last.notify_waiters();
        }
    }
}

#[derive(Debug)]
struct WatcherInner {
    counter: AtomicUsize,
    token: CancellationToken,
    is_last: Notify,
}

impl WatcherInner {
    pub fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
            token: CancellationToken::new(),
            is_last: Notify::new(),
        }
    }

    pub fn child(self: &Arc<Self>) -> WatcherChild {
        WatcherChild(Arc::clone(self))
    }
}

#[derive(Debug)]
pub struct Watcher(Arc<WatcherInner>);

impl Default for Watcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Watcher {
    pub fn new() -> Self {
        Self(Arc::new(WatcherInner::new()))
    }

    pub fn child(&self) -> WatcherChild {
        let counter = self
            .0
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        tracing::info!(
            "watched services incremented from {} to {}",
            counter,
            counter + 1
        );

        self.0.child()
    }

    pub fn stop(&self) {
        self.0.token.cancel();
    }

    pub async fn wait(&self) {
        let notify = self.0.is_last.notified();
        tracing::info!("waiting for watched tasks to finish");
        notify.await;
    }
}
