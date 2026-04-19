use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use http::{HeaderMap, HeaderValue};

const REQUEST_ID_HEADER: &str = "x-request-id";
static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn resolve_request_id(headers: &HeaderMap) -> HeaderValue {
    if let Some(existing) = headers.get(REQUEST_ID_HEADER) {
        return existing.clone();
    }

    HeaderValue::from_str(&generate_request_id())
        .unwrap_or_else(|_| HeaderValue::from_static("proxy-request-id"))
}

fn generate_request_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    let counter = REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("proxy-{millis:x}-{counter:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuses_existing_request_id() {
        let mut headers = HeaderMap::new();
        headers.insert(
            REQUEST_ID_HEADER,
            HeaderValue::from_static("existing-request-id"),
        );

        let value = resolve_request_id(&headers);
        assert_eq!(value.to_str().unwrap(), "existing-request-id");
    }

    #[test]
    fn generates_request_id_when_missing() {
        let headers = HeaderMap::new();
        let value = resolve_request_id(&headers);
        assert!(value.to_str().unwrap().starts_with("proxy-"));
    }
}
