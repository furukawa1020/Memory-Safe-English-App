use std::time::Duration;

use mse_proxy::{cache::CacheStore, config::Config, gc, routes, state::AppState};
use reqwest::Client;
use tokio::{net::TcpListener, signal};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mse_proxy=info".into()),
        )
        .init();

    let config = Config::from_env()?;
    let cache = CacheStore::new(config.cache_ttl, config.cache_max_entries);
    let gc_handle = gc::spawn_gc_task(cache.clone(), config.gc_interval);

    let http_client = Client::builder()
        .timeout(config.upstream_timeout)
        .connect_timeout(Duration::from_secs(3))
        .build()?;

    let app_state = AppState {
        config: config.clone(),
        http_client,
        cache,
    };
    let app = routes::build_router(app_state);

    let listener = TcpListener::bind(config.http_addr).await?;
    info!(address = %config.http_addr, "proxy listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    gc::shutdown_gc_task(gc_handle).await;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        let mut signal =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
        signal.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
