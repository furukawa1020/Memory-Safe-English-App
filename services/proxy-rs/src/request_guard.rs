use axum::http::{HeaderMap, Method, StatusCode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardUpstream {
    Api,
    Worker,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuardRejection {
    pub status: StatusCode,
    pub event: &'static str,
    pub message: &'static str,
}

pub fn validate_request(
    upstream: GuardUpstream,
    method: &Method,
    path_and_query: &str,
    headers: &HeaderMap,
) -> Option<GuardRejection> {
    if matches!(*method, Method::TRACE | Method::CONNECT) {
        return Some(GuardRejection {
            status: StatusCode::METHOD_NOT_ALLOWED,
            event: "proxy_method_rejected",
            message: "unsupported http method",
        });
    }

    if has_ambiguous_or_dangerous_path(path_and_query) {
        return Some(GuardRejection {
            status: StatusCode::BAD_REQUEST,
            event: "proxy_path_rejected",
            message: "invalid request path",
        });
    }

    let normalized = normalized_path(path_and_query);
    match upstream {
        GuardUpstream::Worker => validate_worker_request(method, normalized, headers),
        GuardUpstream::Api => validate_api_alias_request(method, normalized, headers),
    }
}

fn validate_worker_request(
    method: &Method,
    normalized_path: &str,
    headers: &HeaderMap,
) -> Option<GuardRejection> {
    let allowed = matches!(
        normalized_path,
        "/worker/analyze/chunks" | "/worker/analyze/skeleton"
    );
    if !allowed {
        return Some(GuardRejection {
            status: StatusCode::NOT_FOUND,
            event: "worker_route_rejected",
            message: "worker route is not exposed by the proxy",
        });
    }

    if *method != Method::POST {
        return Some(GuardRejection {
            status: StatusCode::METHOD_NOT_ALLOWED,
            event: "worker_method_rejected",
            message: "worker analysis routes only accept POST",
        });
    }

    require_json_content_type(headers, "worker_content_type_rejected")
}

fn validate_api_alias_request(
    method: &Method,
    normalized_path: &str,
    headers: &HeaderMap,
) -> Option<GuardRejection> {
    if normalized_path.starts_with("/auth/") {
        if *method != Method::POST {
            return Some(GuardRejection {
                status: StatusCode::METHOD_NOT_ALLOWED,
                event: "auth_method_rejected",
                message: "auth routes only accept POST",
            });
        }
        return require_json_content_type(headers, "auth_content_type_rejected");
    }

    if normalized_path.starts_with("/analysis/") || normalized_path.starts_with("/sessions/") {
        if *method != Method::POST {
            return Some(GuardRejection {
                status: StatusCode::METHOD_NOT_ALLOWED,
                event: "api_method_rejected",
                message: "this route only accepts POST",
            });
        }
        return require_json_content_type(headers, "api_content_type_rejected");
    }

    if normalized_path == "/me" {
        if *method == Method::PATCH {
            return require_json_content_type(headers, "api_content_type_rejected");
        }
        if !matches!(*method, Method::GET | Method::PATCH) {
            return Some(GuardRejection {
                status: StatusCode::METHOD_NOT_ALLOWED,
                event: "me_method_rejected",
                message: "/me only accepts GET or PATCH",
            });
        }
    }

    if normalized_path == "/contents" && *method == Method::POST {
        return require_json_content_type(headers, "api_content_type_rejected");
    }

    if normalized_path.starts_with("/contents/") && *method == Method::PATCH {
        return require_json_content_type(headers, "api_content_type_rejected");
    }

    None
}

fn require_json_content_type(
    headers: &HeaderMap,
    event: &'static str,
) -> Option<GuardRejection> {
    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim().to_ascii_lowercase());

    let is_json = content_type
        .as_deref()
        .map(|value| value.starts_with("application/json"))
        .unwrap_or(false);

    if is_json {
        None
    } else {
        Some(GuardRejection {
            status: StatusCode::UNSUPPORTED_MEDIA_TYPE,
            event,
            message: "content-type must be application/json",
        })
    }
}

fn normalized_path(path_and_query: &str) -> &str {
    path_and_query.split('?').next().unwrap_or(path_and_query)
}

fn has_ambiguous_or_dangerous_path(path_and_query: &str) -> bool {
    let path = normalized_path(path_and_query);
    if path.is_empty() || !path.starts_with('/') {
        return true;
    }
    if path.contains('\\') || path.contains("//") || path.contains("/./") || path.ends_with("/.") {
        return true;
    }

    let lowered = path.to_ascii_lowercase();
    if lowered.contains("/../")
        || lowered.ends_with("/..")
        || lowered.contains("%2e")
        || lowered.contains("%2f")
        || lowered.contains("%5c")
    {
        return true;
    }

    path.chars().any(|ch| ch.is_control())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{header::CONTENT_TYPE, HeaderValue};

    #[test]
    fn rejects_trace_and_connect_methods() {
        let headers = HeaderMap::new();
        let trace = validate_request(GuardUpstream::Api, &Method::TRACE, "/auth/login", &headers);
        let connect =
            validate_request(GuardUpstream::Worker, &Method::CONNECT, "/worker/analyze/chunks", &headers);

        assert_eq!(trace.unwrap().status, StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(connect.unwrap().status, StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn rejects_non_json_auth_request() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));

        let rejection = validate_request(GuardUpstream::Api, &Method::POST, "/auth/login", &headers)
            .expect("expected rejection");

        assert_eq!(rejection.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[test]
    fn rejects_non_post_worker_request() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let rejection = validate_request(
            GuardUpstream::Worker,
            &Method::GET,
            "/worker/analyze/chunks",
            &headers,
        )
        .expect("expected rejection");

        assert_eq!(rejection.status, StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn rejects_unknown_worker_route() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let rejection = validate_request(
            GuardUpstream::Worker,
            &Method::POST,
            "/worker/analyze/reader-plan",
            &headers,
        )
        .expect("expected rejection");

        assert_eq!(rejection.status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn rejects_ambiguous_or_traversal_like_paths() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        for path in [
            "/auth//login",
            "/auth/../login",
            "/auth/%2e%2e/login",
            "/auth\\login",
        ] {
            let rejection = validate_request(GuardUpstream::Api, &Method::POST, path, &headers)
                .expect("expected rejection");
            assert_eq!(rejection.status, StatusCode::BAD_REQUEST);
        }
    }
}
