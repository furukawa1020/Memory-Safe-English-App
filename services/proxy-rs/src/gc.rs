use std::time::Duration;

use tokio::task::JoinHandle;
use tracing::{debug, error, info};

use crate::cache::CacheStore;

pub fn spawn_gc_task(cache: CacheStore, interval: Duration) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);

        loop {
            ticker.tick().await;
            let result = cache.sweep_expired().await;
            if result.expired_removed == 0 && result.overflow_removed == 0 {
                debug!(
                    remaining = result.remaining,
                    "cache gc sweep completed without removals"
                );
                continue;
            }

            info!(
                expired_removed = result.expired_removed,
                overflow_removed = result.overflow_removed,
                remaining = result.remaining,
                "cache gc sweep removed entries"
            );
        }
    })
}

pub async fn shutdown_gc_task(handle: JoinHandle<()>) {
    handle.abort();
    if let Err(err) = handle.await {
        if !err.is_cancelled() {
            error!(error = %err, "gc task exited unexpectedly");
        }
    }
}
