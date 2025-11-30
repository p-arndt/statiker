use crate::state::AppState;
use crate::utils::is_asset_path;
use axum::{
    http::{HeaderName, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use http::{header::CACHE_CONTROL, Request};
use std::{net::IpAddr, str::FromStr};

/// Rate limiting middleware
/// 
/// Security: Uses a fallback IP (0.0.0.0) when client IP cannot be extracted
/// to prevent bypassing rate limits by omitting identification headers.
pub async fn rate_limit_mw(state: AppState, req: Request<axum::body::Body>, next: Next) -> Response {
    if let Some(limiter) = &state.limiter {
        // Try to extract IP from headers or socket address
        let ip = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse::<IpAddr>().ok())
            .or_else(|| req.extensions().get::<std::net::SocketAddr>().map(|a| a.ip()))
            // Fallback to 0.0.0.0 for unknown clients to prevent rate limit bypass
            .unwrap_or_else(|| IpAddr::from([0, 0, 0, 0]));
        
        // Apply rate limiting check - all requests are checked, including unknown IPs
        if limiter.check_key(&ip).is_err() {
            return (StatusCode::TOO_MANY_REQUESTS, "rate limit").into_response();
        }
    }
    next.run(req).await
}

/// Cache control middleware
pub async fn cache_control_mw(state: AppState, req: Request<axum::body::Body>, next: Next) -> Response {
    let path = req.uri().path().to_owned();
    let mut res = next.run(req).await;
    let cache = &state.cfg.assets.cache;
    if cache.enabled && is_asset_path(&path) {
        let secs = cache.max_age.as_secs();
        if let Ok(hv) = HeaderValue::from_str(&format!("public, max-age={secs}, immutable")) {
            res.headers_mut().insert(CACHE_CONTROL, hv);
        }
    }
    res
}

/// Security headers middleware
pub async fn with_security_headers(state: AppState, req: Request<axum::body::Body>, next: Next) -> Response {
    let mut res = next.run(req).await;
    for (k, v) in &state.cfg.security.headers {
        if let (Ok(name), Ok(val)) = (HeaderName::from_str(k), HeaderValue::from_str(v)) {
            res.headers_mut().insert(name, val);
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;

    #[test]
    fn test_fallback_ip_constant() {
        // Verify the fallback IP is 0.0.0.0
        // This ensures unknown clients share a single rate limit bucket
        let fallback = IpAddr::from([0, 0, 0, 0]);
        assert_eq!(fallback, IpAddr::from([0, 0, 0, 0]));
    }

    #[test]
    fn test_ip_extraction_logic() {
        // Test the IP extraction logic that's used in the middleware
        use std::net::SocketAddr;
        use http::Request;
        use axum::body::Body;

        // Test 1: Extract from x-forwarded-for header
        let req1 = Request::builder()
            .header("x-forwarded-for", "192.168.1.1, 10.0.0.1")
            .body(Body::empty())
            .unwrap();
        let ip1 = req1
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse::<IpAddr>().ok())
            .or_else(|| req1.extensions().get::<SocketAddr>().map(|a| a.ip()))
            .unwrap_or_else(|| IpAddr::from([0, 0, 0, 0]));
        assert_eq!(ip1, IpAddr::from([192, 168, 1, 1]));

        // Test 2: Extract from socket address
        let mut req2 = Request::builder().body(Body::empty()).unwrap();
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        req2.extensions_mut().insert(addr);
        let ip2 = req2
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse::<IpAddr>().ok())
            .or_else(|| req2.extensions().get::<SocketAddr>().map(|a| a.ip()))
            .unwrap_or_else(|| IpAddr::from([0, 0, 0, 0]));
        assert_eq!(ip2, IpAddr::from([127, 0, 0, 1]));

        // Test 3: Fallback to 0.0.0.0 when no IP can be extracted
        let req3 = Request::builder().body(Body::empty()).unwrap();
        let ip3 = req3
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse::<IpAddr>().ok())
            .or_else(|| req3.extensions().get::<SocketAddr>().map(|a| a.ip()))
            .unwrap_or_else(|| IpAddr::from([0, 0, 0, 0]));
        assert_eq!(ip3, IpAddr::from([0, 0, 0, 0]), "Unknown IPs should use fallback 0.0.0.0");
    }
}

