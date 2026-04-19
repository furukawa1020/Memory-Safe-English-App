use axum::body::Body;
use http::{HeaderValue, Response};

use crate::response_headers::apply_standard_headers;

pub fn with_standard_headers(
    mut response: Response<Body>,
    request_id: &HeaderValue,
    cache_state: &'static str,
) -> Response<Body> {
    apply_standard_headers(response.headers_mut(), request_id, cache_state);
    response
}
