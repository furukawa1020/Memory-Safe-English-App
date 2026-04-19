use tracing::warn;

pub fn log_event(
    event: &'static str,
    request_id: &str,
    client_ip: &str,
    path: &str,
    detail: &'static str,
) {
    warn!(
        event = event,
        request_id = request_id,
        client_ip = client_ip,
        path = path,
        detail = detail,
        "security event"
    );
}
