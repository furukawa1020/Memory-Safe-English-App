use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, Response, StatusCode, Uri},
};
use bytes::Bytes;
use reqwest::Method;
use sha2::{Digest, Sha256};

use crate::{
    cache::CachedResponse, request_id::resolve_request_id,
    response_headers::{apply_standard_headers, apply_upstream_header},
    state::AppState,
};

static HOP_BY_HOP_HEADERS: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

pub async fn proxy_to_api(State(state): State<AppState>, request: Request<Body>) -> Response<Body> {
    forward(state, request, Upstream::Api).await
}

pub async fn proxy_to_worker(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Response<Body> {
    forward(state, request, Upstream::Worker).await
}

async fn forward(state: AppState, request: Request<Body>, upstream: Upstream) -> Response<Body> {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    let request_id = resolve_request_id(&parts.headers);
    let path_and_query = parts
        .uri
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or("/");

    let body_bytes = match to_bytes(body, state.config.max_request_body_bytes).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return error_response(
                StatusCode::PAYLOAD_TOO_LARGE,
                "request body too large",
                &request_id,
            )
        }
    };

    let maybe_cache_key = cache_key(&upstream, &method, path_and_query, &body_bytes);
    if let Some(key) = maybe_cache_key.as_ref() {
        if let Some(cached) = state.cache.get(key).await {
            return build_cached_response(cached, &request_id);
        }
    }

    let upstream_url = match upstream_url(&state, &upstream, path_and_query) {
        Ok(url) => url,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid upstream request path",
                &request_id,
            )
        }
    };

    let request_headers = sanitize_request_headers(&parts.headers, &request_id);
    let reqwest_method = Method::from_bytes(method.as_str().as_bytes()).unwrap_or(Method::GET);
    let upstream_response = match state
        .http_client
        .request(reqwest_method, upstream_url)
        .headers(request_headers)
        .body(body_bytes.clone())
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return error_response(
                StatusCode::BAD_GATEWAY,
                "upstream request failed",
                &request_id,
            )
        }
    };

    let status = StatusCode::from_u16(upstream_response.status().as_u16())
        .unwrap_or(StatusCode::BAD_GATEWAY);
    let headers = sanitize_response_headers(upstream_response.headers(), &request_id);
    let response_body = match upstream_response.bytes().await {
        Ok(bytes) => bytes,
        Err(_) => {
            return error_response(
                StatusCode::BAD_GATEWAY,
                "failed to read upstream response",
                &request_id,
            )
        }
    };

    if let Some(key) = maybe_cache_key {
        if status.is_success() {
            state
                .cache
                .insert(
                    key,
                    CachedResponse {
                        status,
                        headers: headers.clone(),
                        body: response_body.clone(),
                    },
                )
                .await;
        }
    }

    build_response(status, headers, response_body)
}

fn upstream_url(
    state: &AppState,
    upstream: &Upstream,
    path_and_query: &str,
) -> Result<String, http::uri::InvalidUri> {
    let uri: Uri = path_and_query.parse()?;
    let suffix = uri
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or("/");
    let without_prefix = suffix.trim_start_matches(upstream.route_prefix());
    let rewritten = if without_prefix.starts_with('/') {
        without_prefix.to_string()
    } else {
        format!("/{without_prefix}")
    };

    let base = match upstream {
        Upstream::Api => &state.config.api_base_url,
        Upstream::Worker => &state.config.worker_base_url,
    };

    Ok(format!("{base}{rewritten}"))
}

fn cache_key(
    upstream: &Upstream,
    method: &http::Method,
    path_and_query: &str,
    body: &Bytes,
) -> Option<String> {
    if *upstream != Upstream::Worker || method != http::Method::POST {
        return None;
    }

    let normalized = path_and_query.split('?').next().unwrap_or(path_and_query);
    if normalized != "/worker/analyze/chunks" && normalized != "/worker/analyze/skeleton" {
        return None;
    }

    let mut hasher = Sha256::new();
    hasher.update(body);
    let digest = format!("{:x}", hasher.finalize());
    Some(format!("{}:{normalized}:{digest}", method.as_str()))
}

fn sanitize_request_headers(
    headers: &HeaderMap,
    request_id: &HeaderValue,
) -> reqwest::header::HeaderMap {
    let mut sanitized = reqwest::header::HeaderMap::new();
    for (name, value) in headers {
        if should_skip_header(name.as_str()) || name.as_str().eq_ignore_ascii_case("host") {
            continue;
        }
        sanitized.insert(name.clone(), value.clone());
    }
    sanitized.insert(HeaderName::from_static("x-request-id"), request_id.clone());
    sanitized
}

fn sanitize_response_headers(
    headers: &reqwest::header::HeaderMap,
    request_id: &HeaderValue,
) -> HeaderMap {
    let mut sanitized = HeaderMap::new();
    for (name, value) in headers {
        if should_skip_header(name.as_str()) {
            continue;
        }
        sanitized.insert(name.clone(), value.clone());
    }
    apply_standard_headers(&mut sanitized, request_id, "miss");
    sanitized
}

fn build_cached_response(mut response: CachedResponse, request_id: &HeaderValue) -> Response<Body> {
    apply_standard_headers(&mut response.headers, request_id, "hit");
    build_response(response.status, response.headers, response.body)
}

fn build_response(status: StatusCode, headers: HeaderMap, body: Bytes) -> Response<Body> {
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    response
}

fn error_response(
    status: StatusCode,
    message: &'static str,
    request_id: &HeaderValue,
) -> Response<Body> {
    let payload = serde_json::json!({ "error": message }).to_string();
    let mut response = Response::new(Body::from(payload));
    *response.status_mut() = status;
    let headers = response.headers_mut();
    headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    apply_standard_headers(headers, request_id, "miss");
    response
}

fn should_skip_header(header_name: &str) -> bool {
    HOP_BY_HOP_HEADERS
        .iter()
        .any(|value| value.eq_ignore_ascii_case(header_name))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Upstream {
    Api,
    Worker,
}

impl Upstream {
    fn route_prefix(&self) -> &'static str {
        match self {
            Self::Api => "/api",
            Self::Worker => "/worker",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_worker_analysis_posts_are_cacheable() {
        let body = Bytes::from_static(br#"{"text":"hello"}"#);
        assert!(cache_key(
            &Upstream::Worker,
            &http::Method::POST,
            "/worker/analyze/chunks",
            &body
        )
        .is_some());
        assert!(cache_key(
            &Upstream::Worker,
            &http::Method::POST,
            "/worker/analyze/skeleton",
            &body
        )
        .is_some());
        assert!(cache_key(
            &Upstream::Worker,
            &http::Method::GET,
            "/worker/analyze/chunks",
            &body
        )
        .is_none());
        assert!(cache_key(&Upstream::Api, &http::Method::POST, "/api/contents", &body).is_none());
    }
}
