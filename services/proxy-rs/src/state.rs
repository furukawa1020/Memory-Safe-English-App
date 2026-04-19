use reqwest::Client;

use crate::{cache::CacheStore, config::Config};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub http_client: Client,
    pub cache: CacheStore,
}
