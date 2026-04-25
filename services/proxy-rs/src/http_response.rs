use axum::body::Body;
use http::{HeaderValue, Response};

use crate::{
    config::RuntimeEnvironment,
    response_headers::{apply_standard_headers, HeaderPolicy},
};

pub fn with_standard_headers(
    mut response: Response<Body>,
    request_id: &HeaderValue,
    cache_state: &'static str,
    runtime_environment: &RuntimeEnvironment,
    policy: HeaderPolicy,
) -> Response<Body> {
    apply_standard_headers(
        response.headers_mut(),
        request_id,
        cache_state,
        runtime_environment,
        policy,
    );
    response
}
