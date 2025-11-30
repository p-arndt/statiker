use crate::config::Proxy;
use crate::state::HTTP_CLIENT;
use axum::{
    body::Body,
    extract::Path,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::any,
};
use bytes::Bytes;
use futures_util::TryStreamExt;
use http::Request;
use http_body_util::BodyStream;
use std::{net::SocketAddr, str::FromStr, sync::Arc, time::Duration};

/// Proxy state for a route
#[derive(Clone)]
pub struct ProxyState {
    pub target: String,
    pub timeout: Duration,
    pub add_headers: Vec<(HeaderName, String)>,
}

impl ProxyState {
    pub fn new(p: Proxy) -> Self {
        let add_headers = p
            .add_headers
            .into_iter()
            .filter_map(|(k, v)| HeaderName::from_str(&k).ok().map(|n| (n, v)))
            .collect::<Vec<_>>();
        Self {
            target: p.url.trim_end_matches('/').to_string(),
            timeout: if p.timeout == Duration::ZERO {
                Duration::from_secs(5)
            } else {
                p.timeout
            },
            add_headers,
        }
    }
}

/// Create a proxy route handler
pub fn make_proxy_route(base: &str, p: Proxy) -> (String, axum::routing::MethodRouter) {
    let ps = Arc::new(ProxyState::new(p));
    let route_path = format!("{}*tail", base.trim_end_matches('*'));
    let handler = {
        let ps = ps.clone();
        any(move |Path(tail): Path<String>, req: Request<Body>| {
            let ps = ps.clone();
            async move { proxy_forward(ps, tail, req).await }
        })
    };
    (route_path, handler)
}

/// Forward a request to the upstream proxy
pub async fn proxy_forward(pstate: Arc<ProxyState>, tail: String, mut req: Request<Body>) -> Response {
    let mut upstream = format!("{}/{}", pstate.target, tail);
    if let Some(q) = req.uri().query() {
        upstream.push('?');
        upstream.push_str(q);
    }
    let Ok(uri) = Uri::from_str(&upstream) else {
        return StatusCode::BAD_GATEWAY.into_response();
    };

    *req.uri_mut() = uri;

    // Add configured headers (supports {client_ip})
    let client_ip = client_ip(&req).unwrap_or_else(|| "unknown".into());
    for (k, v) in &pstate.add_headers {
        let vv = v.replace("{client_ip}", &client_ip);
        if let Ok(hv) = HeaderValue::from_str(&vv) {
            req.headers_mut().insert(k.clone(), hv);
        }
    }

    // Remove hop-by-hop headers
    strip_hop_by_hop(req.headers_mut());

    match tokio::time::timeout(pstate.timeout, HTTP_CLIENT.request(req)).await {
        Ok(Ok(upstream_res)) => {
            // Copy status/headers; stream body through using BodyStream
            let mut builder = Response::builder()
                .status(upstream_res.status())
                .version(upstream_res.version());
            let mut headers = upstream_res.headers().clone();
            strip_hop_by_hop(&mut headers);
            *builder.headers_mut().unwrap() = headers;

            let incoming = upstream_res.into_body(); // hyper::body::Incoming
            let stream = BodyStream::new(incoming)
                .map_ok(|frame| frame.into_data().unwrap_or_else(|_| Bytes::new())); // -> Bytes
            let body = Body::from_stream(stream); // axum::body::Body
            builder.body(body).unwrap()
        }
        _ => StatusCode::BAD_GATEWAY.into_response(),
    }
}

/// Remove hop-by-hop headers from a header map
pub fn strip_hop_by_hop(headers: &mut HeaderMap) {
    for h in [
        "connection",
        "proxy-connection",
        "keep-alive",
        "transfer-encoding",
        "upgrade",
        "te",
        "trailers",
    ] {
        if let Ok(n) = HeaderName::from_str(h) {
            headers.remove(n);
        }
    }
}

/// Extract client IP from request
pub fn client_ip<B>(req: &Request<B>) -> Option<String> {
    if let Some(v) = req.headers().get("x-forwarded-for") {
        return v.to_str().ok().map(|s| s.to_string());
    }
    req.extensions()
        .get::<SocketAddr>()
        .map(|a| a.ip().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Proxy;
    use http::Request;
    use std::collections::HashMap;
    use std::net::SocketAddr;

    #[test]
    fn test_proxy_state_new() {
        let proxy = Proxy {
            url: "https://example.com/".to_string(),
            timeout: Duration::from_secs(10),
            add_headers: HashMap::new(),
        };
        let state = ProxyState::new(proxy);
        assert_eq!(state.target, "https://example.com");
        assert_eq!(state.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_proxy_state_default_timeout() {
        let proxy = Proxy {
            url: "https://example.com".to_string(),
            timeout: Duration::ZERO,
            add_headers: HashMap::new(),
        };
        let state = ProxyState::new(proxy);
        assert_eq!(state.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_proxy_state_trim_url() {
        let proxy = Proxy {
            url: "https://example.com/".to_string(),
            timeout: Duration::from_secs(5),
            add_headers: HashMap::new(),
        };
        let state = ProxyState::new(proxy);
        assert_eq!(state.target, "https://example.com");
    }

    #[test]
    fn test_proxy_state_add_headers() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom-Header".to_string(), "value".to_string());
        let proxy = Proxy {
            url: "https://example.com".to_string(),
            timeout: Duration::from_secs(5),
            add_headers: headers,
        };
        let state = ProxyState::new(proxy);
        assert_eq!(state.add_headers.len(), 1);
    }

    #[test]
    fn test_strip_hop_by_hop() {
        let mut headers = HeaderMap::new();
        headers.insert("connection", HeaderValue::from_static("keep-alive"));
        headers.insert("proxy-connection", HeaderValue::from_static("close"));
        headers.insert("content-type", HeaderValue::from_static("text/html"));
        headers.insert("transfer-encoding", HeaderValue::from_static("chunked"));
        strip_hop_by_hop(&mut headers);
        assert!(!headers.contains_key("connection"));
        assert!(!headers.contains_key("proxy-connection"));
        assert!(!headers.contains_key("transfer-encoding"));
        assert!(headers.contains_key("content-type"));
    }

    #[test]
    fn test_client_ip_from_x_forwarded_for() {
        let req = Request::builder()
            .header("x-forwarded-for", "192.168.1.1")
            .body(())
            .unwrap();
        let ip = client_ip(&req);
        assert_eq!(ip, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_client_ip_from_x_forwarded_for_multiple() {
        // Note: client_ip returns the full header value, not just the first IP
        // The actual parsing happens in rate_limit_mw middleware
        let req = Request::builder()
            .header("x-forwarded-for", "192.168.1.1, 10.0.0.1")
            .body(())
            .unwrap();
        let ip = client_ip(&req);
        assert_eq!(ip, Some("192.168.1.1, 10.0.0.1".to_string()));
    }

    #[test]
    fn test_client_ip_from_extensions() {
        let mut req = Request::builder().body(()).unwrap();
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        req.extensions_mut().insert(addr);
        let ip = client_ip(&req);
        assert_eq!(ip, Some("127.0.0.1".to_string()));
    }

    #[test]
    fn test_client_ip_none() {
        let req = Request::builder().body(()).unwrap();
        let ip = client_ip(&req);
        assert_eq!(ip, None);
    }
}

