use std::net::SocketAddr;

use axum::{body::Body, extract::connect_info::ConnectInfo, http::Request};

pub fn resolve_client_ip(request: &Request<Body>) -> String {
    if let Some(forwarded_for) = request.headers().get("x-forwarded-for") {
        if let Ok(raw) = forwarded_for.to_str() {
            if let Some(value) = raw.split(',').next() {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }

    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(raw) = real_ip.to_str() {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    if let Some(connect_info) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
        return connect_info.0.ip().to_string();
    }

    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    #[test]
    fn uses_forwarded_for_when_available() {
        let request = Request::builder()
            .uri("/auth/login")
            .header("x-forwarded-for", "203.0.113.10, 10.0.0.2")
            .body(Body::empty())
            .unwrap();

        assert_eq!(resolve_client_ip(&request), "203.0.113.10");
    }
}
