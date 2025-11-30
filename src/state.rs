use crate::config::Config;
use axum::body::Body;
use governor::{
    clock::DefaultClock, middleware::NoOpMiddleware, state::keyed::DashMapStateStore, RateLimiter,
};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use once_cell::sync::Lazy;
use std::{net::IpAddr, path::PathBuf, sync::Arc};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<Config>,
    pub root: PathBuf,
    pub limiter: Option<Arc<IpLimiterInner>>,
}

pub type IpLimiterInner = RateLimiter<IpAddr, DashMapStateStore<IpAddr>, DefaultClock, NoOpMiddleware>;

/// Shared hyper client (HTTP/1 + TLS). HTTP/2 optional — skipped here.
pub static HTTP_CLIENT: Lazy<Client<hyper_rustls::HttpsConnector<HttpConnector>, Body>> =
    Lazy::new(|| {
        let https = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .build();
        Client::builder(TokioExecutor::new()).build(https)
    });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_app_state_clone() {
        let cfg = Arc::new(Config::default());
        let state = AppState {
            cfg: cfg.clone(),
            root: PathBuf::from("."),
            limiter: None,
        };
        let cloned = state.clone();
        assert_eq!(state.root, cloned.root);
        // Verify Arc is shared
        assert!(Arc::ptr_eq(&state.cfg, &cloned.cfg));
    }

    #[test]
    fn test_app_state_with_limiter() {
        let cfg = Arc::new(Config::default());
        let limiter = Some(Arc::new(
            RateLimiter::keyed(governor::Quota::per_minute(
                std::num::NonZeroU32::new(60).unwrap(),
            )),
        ));
        let state = AppState {
            cfg,
            root: PathBuf::from("/tmp"),
            limiter: limiter.clone(),
        };
        assert!(state.limiter.is_some());
        assert_eq!(state.root, PathBuf::from("/tmp"));
    }
}

