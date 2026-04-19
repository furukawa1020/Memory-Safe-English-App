use std::net::{IpAddr, SocketAddr};

use axum::{body::Body, extract::connect_info::ConnectInfo, http::HeaderMap, http::Request};

pub fn resolve_client_ip(request: &Request<Body>, trusted_proxy_ips: &[IpAddr]) -> String {
    let peer_ip = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|connect_info| connect_info.0.ip());
    resolve_client_ip_from_parts(request.headers(), peer_ip, trusted_proxy_ips)
}

pub fn resolve_client_ip_from_parts(
    headers: &HeaderMap,
    peer_ip: Option<IpAddr>,
    trusted_proxy_ips: &[IpAddr],
) -> String {
    if let Some(peer_ip) = peer_ip {
        if trusted_proxy_ips.contains(&peer_ip) {
            if let Some(forwarded_ip) = forwarded_client_ip(headers) {
                return forwarded_ip;
            }
            if let Some(real_ip) = real_ip_header(headers) {
                return real_ip;
            }
        }

        return peer_ip.to_string();
    }

    "unknown".to_string()
}

fn forwarded_client_ip(headers: &HeaderMap) -> Option<String> {
    let forwarded_for = headers.get("x-forwarded-for")?;
    let raw = forwarded_for.to_str().ok()?;
    let first = raw.split(',').next()?.trim();
    normalize_ip(first)
}

fn real_ip_header(headers: &HeaderMap) -> Option<String> {
    let real_ip = headers.get("x-real-ip")?;
    let raw = real_ip.to_str().ok()?.trim();
    normalize_ip(raw)
}

fn normalize_ip(raw: &str) -> Option<String> {
    raw.parse::<IpAddr>().ok().map(|ip| ip.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    #[test]
    fn uses_forwarded_for_when_peer_is_trusted() {
        let mut request = Request::builder()
            .uri("/auth/login")
            .header("x-forwarded-for", "203.0.113.10, 10.0.0.2")
            .body(Body::empty())
            .unwrap();
        request.extensions_mut().insert(ConnectInfo(SocketAddr::from((
            [127, 0, 0, 1],
            3000,
        ))));

        let trusted = vec!["127.0.0.1".parse::<IpAddr>().unwrap()];
        assert_eq!(resolve_client_ip(&request, &trusted), "203.0.113.10");
    }

    #[test]
    fn ignores_forwarded_for_when_peer_is_not_trusted() {
        let mut request = Request::builder()
            .uri("/auth/login")
            .header("x-forwarded-for", "203.0.113.10, 10.0.0.2")
            .body(Body::empty())
            .unwrap();
        request.extensions_mut().insert(ConnectInfo(SocketAddr::from((
            [10, 0, 0, 8],
            3000,
        ))));

        let trusted = vec!["127.0.0.1".parse::<IpAddr>().unwrap()];
        assert_eq!(resolve_client_ip(&request, &trusted), "10.0.0.8");
    }

    #[test]
    fn returns_unknown_without_peer_context() {
        let headers = HeaderMap::new();
        assert_eq!(
            resolve_client_ip_from_parts(&headers, None, &[]),
            "unknown"
        );
    }
}
