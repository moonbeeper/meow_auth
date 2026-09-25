use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use tokio::sync::Notify;
use tokio_util::sync::{CancellationToken, WaitForCancellationFuture};

// TODO: Maybe add a method that forces futures to return Poll::Ready when the Watcher is stopped.

pub struct WatcherChild {
    inner: Arc<WatcherInner>,
    name: &'static str,
}

impl WatcherChild {
    /// returns a future that resolves when the watcher has been stopped.
    ///
    /// its actually just a wrapper around the `CancellationToken`
    pub fn cancelled(&self) -> WaitForCancellationFuture<'_> {
        self.inner.token.cancelled()
    }

    /// returns the actual `CancellationToken` used by the watcher
    pub fn token(&self) -> CancellationToken {
        self.inner.token.clone()
    }
}

impl Drop for WatcherChild {
    fn drop(&mut self) {
        let past_counter = self.inner.counter.fetch_sub(1, Ordering::AcqRel);
        let actual_counter = past_counter - 1;

        tracing::info!(
            "service '{}' stopped. watched services decremented from {past_counter} to {actual_counter}",
            self.name
        );

        if past_counter == 1 {
            self.inner.is_last.notify_waiters();
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

    pub fn child(self: &Arc<Self>, name: &'static str) -> WatcherChild {
        WatcherChild {
            inner: self.clone(),
            name,
        }
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

    /// spawn a service that will be "watched" by the Mr. Watcher.
    ///
    /// you'll be given a `WatcherChild` that you can use to check if the watcher has been stopped (by using
    /// its wrapped `CancellationToken` with `WatcherChild::cancelled()` or just by getting the actual token
    /// with `WatcherChild::token()`).
    pub fn spawn_service<F, O>(&self, name: &'static str, fut: F)
    where
        F: FnOnce(WatcherChild) -> O,
        O: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        let child = self.child(name);
        let fut = fut(child);

        tokio::spawn(async move {
            let inner = tokio::spawn(fut);
            match inner.await {
                Ok(Ok(())) => tracing::info!("{name} exited normally"),
                Ok(Err(e)) => tracing::error!("{name} exited with an error: {e}"),
                Err(e) => tracing::error!("{name} panicked: {e}"),
            }
        });
    }

    /// returns a `WatcherChild` that can be used to check if the watcher has been stopped
    pub fn child(&self, name: &'static str) -> WatcherChild {
        let initial_counter = self.0.counter.fetch_add(1, Ordering::Relaxed);
        let actual_counter = initial_counter + 1;

        tracing::info!(
            "added service '{name}'. watched services incremented from {initial_counter} to {actual_counter}",
        );

        self.0.child(name)
    }

    /// used to signal the watcher to stop all watched services, but it does not wait for them to finish.
    /// use `wait()` right after this to wait for them to finish
    pub fn stop(&self) {
        self.0.token.cancel();
    }

    /// waits for all watched services to finish. Usually called after `stop()` (usually.... hehe)
    pub async fn wait(&self) {
        tracing::info!("waiting for watched tasks to finish");
        let notify = self.0.is_last.notified();

        if self.0.counter.load(Ordering::Relaxed) == 0 {
            tracing::info!("no watched tasks to wait for");
            return;
        }

        notify.await;
    }
}
