use http::{HeaderMap, HeaderName, HeaderValue};

pub fn apply_standard_headers(
    headers: &mut HeaderMap,
    request_id: &HeaderValue,
    cache_state: &'static str,
) {
    headers.insert(HeaderName::from_static("x-request-id"), request_id.clone());
    headers.insert(
        HeaderName::from_static("x-proxy-cache"),
        HeaderValue::from_static(cache_state),
    );
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_proxy_headers() {
        let mut headers = HeaderMap::new();
        let request_id = HeaderValue::from_static("request-123");

        apply_standard_headers(&mut headers, &request_id, "miss");

        assert_eq!(headers.get("x-request-id").unwrap(), "request-123");
        assert_eq!(headers.get("x-proxy-cache").unwrap(), "miss");
        assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
    }
}
