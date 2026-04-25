use http::{HeaderMap, HeaderName, HeaderValue};

use crate::config::RuntimeEnvironment;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeaderPolicy {
    Default,
    Sensitive,
}

pub fn apply_standard_headers(
    headers: &mut HeaderMap,
    request_id: &HeaderValue,
    cache_state: &'static str,
    runtime_environment: &RuntimeEnvironment,
    policy: HeaderPolicy,
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
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("accelerometer=(), camera=(), geolocation=(), gyroscope=(), microphone=(), payment=(), usb=()"),
    );
    headers.insert(
        HeaderName::from_static("cross-origin-resource-policy"),
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        HeaderName::from_static("x-permitted-cross-domain-policies"),
        HeaderValue::from_static("none"),
    );
    if *runtime_environment == RuntimeEnvironment::Production {
        headers.insert(
            HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        );
    }
    if matches!(policy, HeaderPolicy::Sensitive) {
        headers.insert(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
        );
        headers.insert(
            HeaderName::from_static("pragma"),
            HeaderValue::from_static("no-cache"),
        );
        headers.insert(
            HeaderName::from_static("expires"),
            HeaderValue::from_static("0"),
        );
    }
}

pub fn apply_upstream_header(headers: &mut HeaderMap, upstream_name: &'static str) {
    headers.insert(
        HeaderName::from_static("x-proxy-upstream"),
        HeaderValue::from_static(upstream_name),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_proxy_headers() {
        let mut headers = HeaderMap::new();
        let request_id = HeaderValue::from_static("request-123");

        apply_standard_headers(
            &mut headers,
            &request_id,
            "miss",
            &RuntimeEnvironment::Development,
            HeaderPolicy::Default,
        );

        assert_eq!(headers.get("x-request-id").unwrap(), "request-123");
        assert_eq!(headers.get("x-proxy-cache").unwrap(), "miss");
        assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
        assert_eq!(
            headers.get("cross-origin-resource-policy").unwrap(),
            "same-origin"
        );
        assert_eq!(headers.get("x-permitted-cross-domain-policies").unwrap(), "none");
        assert!(headers.get("cache-control").is_none());
    }

    #[test]
    fn applies_upstream_header() {
        let mut headers = HeaderMap::new();

        apply_upstream_header(&mut headers, "api");

        assert_eq!(headers.get("x-proxy-upstream").unwrap(), "api");
    }

    #[test]
    fn sensitive_headers_disable_caching_and_enable_hsts_in_production() {
        let mut headers = HeaderMap::new();
        let request_id = HeaderValue::from_static("request-123");

        apply_standard_headers(
            &mut headers,
            &request_id,
            "miss",
            &RuntimeEnvironment::Production,
            HeaderPolicy::Sensitive,
        );

        assert_eq!(
            headers.get("cache-control").unwrap(),
            "no-store, no-cache, must-revalidate, private"
        );
        assert_eq!(headers.get("pragma").unwrap(), "no-cache");
        assert_eq!(headers.get("expires").unwrap(), "0");
        assert_eq!(
            headers.get("strict-transport-security").unwrap(),
            "max-age=31536000; includeSubDomains"
        );
    }
}
