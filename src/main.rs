mod cli;
mod config;
mod handlers;
mod middleware;
mod proxy;
mod router;
mod server;
mod state;
mod utils;

use anyhow::{Context, Result};
use axum::middleware::{from_fn, Next};
use governor::RateLimiter;
use std::{net::SocketAddr, num::NonZeroU32, sync::Arc};
use tokio::fs;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{info, warn, Level};
use tracing_subscriber::{fmt, EnvFilter};

use crate::cli::{print_config, Cli};
use clap::Parser;
use crate::config::Config;
use crate::middleware::{cache_control_mw, rate_limit_mw, with_security_headers};
use crate::router::{build_compression, build_cors, build_router};
use crate::server::validate_tls;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber first, before any logging calls
    // Use environment variable or default to "info" level
    // The config file's log level will be respected for subsequent messages,
    // but early messages (like config loading) will use this initial level
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).compact().init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Load config (clap handles env var automatically)
    let config_path = cli.config;

    // Try read config file; if missing, use defaults
    let cfg: Config = match fs::read_to_string(&config_path).await {
        Ok(text) => match serde_yaml::from_str(&text) {
            Ok(parsed) => {
                info!("Loaded configuration from {}", &config_path);
                parsed
            }
            Err(e) => {
                // If parsing fails, fail fast — config file present but invalid.
                return Err(
                    anyhow::anyhow!("parsing YAML {}: {e}", &config_path).context("parsing YAML")
                );
            }
        },
        Err(e) => {
            warn!(
                "Could not read config file '{}': {}. Falling back to built-in defaults.",
                config_path, e
            );
            Config::default()
        }
    };

    // Print configuration
    print_config(&cfg);

    // Validate TLS configuration if enabled
    validate_tls(&cfg).await?;

    // State
    let limiter = if cfg.security.rate_limit.enabled {
        let rpm = cfg.security.rate_limit.requests_per_min.max(1);
        let quota = governor::Quota::per_minute(NonZeroU32::new(rpm).unwrap());
        Some(Arc::new(RateLimiter::keyed(quota)))
    } else {
        None
    };

    let state = AppState {
        root: cfg.server.root.clone(),
        cfg: Arc::new(cfg),
        limiter,
    };

    // Router
    let trace = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    let mut app = build_router(&state)?;

    // Middlewares (capture state with closures)
    let rl_state = state.clone();
    app = app.layer(from_fn(move |req, next: Next| {
        rate_limit_mw(rl_state.clone(), req, next)
    }));

    if let Some(cors) = build_cors(&state.cfg) {
        app = app.layer(cors);
    }
    if let Some(comp) = build_compression(&state.cfg) {
        app = app.layer(comp);
    }

    let cc_state = state.clone();
    app = app.layer(from_fn(move |req, next: Next| {
        cache_control_mw(cc_state.clone(), req, next)
    }));

    let sh_state = state.clone();
    app = app.layer(from_fn(move |req, next: Next| {
        with_security_headers(sh_state.clone(), req, next)
    }));

    app = app.layer(trace);

    // Bind and serve (TLS or plain)
    let addr: SocketAddr = format!("{}:{}", state.cfg.server.host, state.cfg.server.port)
        .parse()
        .context("invalid host/port")?;

    if state.cfg.tls.enabled {
        let tls = crate::server::load_tls_config(&state.cfg).await?;

        info!("listening https://{addr}");

        axum_server::bind_rustls(addr, tls)
            .serve(app.into_make_service())
            .await
            .context("failed to start TLS server")?;
    } else {
        info!("listening http://{addr}");
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .context("failed to bind TCP listener")?;
        axum::serve(listener, app)
            .await
            .context("failed to start HTTP server")?;
    }

    Ok(())
}
