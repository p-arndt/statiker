use crate::config::{Config, Route};
use crate::handlers::serve_static;
use crate::proxy::make_proxy_route;
use crate::state::AppState;
use anyhow::Result;
use axum::{
    body::Body,
    extract::Path,
    http::{HeaderValue, Method},
    routing::any,
    Router,
};
use http::Request;
use std::path::{Path as StdPath, PathBuf};
use std::str::FromStr;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeFile,
};
use tracing::{info, warn};

/// Safely resolve a path within the root directory, preventing path traversal
/// 
/// Returns an error if the path contains `..` components or would escape the root.
/// This function validates path components without requiring the file to exist.
fn resolve_path_within_root(root: &PathBuf, rel_path: &str) -> Result<PathBuf> {
    // Security: disallow path traversal attempts like ".."
    if rel_path.split('/').any(|p| p == "..") {
        return Err(anyhow::anyhow!("Path traversal detected in fallback path"));
    }

    // Build path safely, only allowing normal components
    let mut resolved = root.clone();
    if !rel_path.is_empty() {
        for component in StdPath::new(rel_path).components() {
            match component {
                std::path::Component::Normal(os_str) => {
                    resolved.push(os_str);
                }
                std::path::Component::CurDir => {
                    // Allow "." (current directory) - it's safe
                    continue;
                }
                _ => {
                    // Reject "..", "RootDir", "Prefix", etc.
                    return Err(anyhow::anyhow!("Invalid path component in fallback"));
                }
            }
        }
    }

    // Additional security: Ensure the resolved path is still within the root directory
    // Use canonicalize if paths exist to verify the resolved path stays within root
    if let (Ok(root_canonical), Ok(resolved_canonical)) = (
        root.canonicalize(),
        resolved.canonicalize(),
    ) {
        // If both paths exist, verify resolved is within root
        if !resolved_canonical.starts_with(&root_canonical) {
            return Err(anyhow::anyhow!("Resolved path escapes root directory"));
        }
    }
    // Note: If paths don't exist yet, we rely on the earlier validation:
    // - Line 28: String-based filtering of ".." components
    // - Lines 36-48: Component-based validation that only allows Normal and CurDir
    //   (ParentDir/.. is already rejected by the string check and would be caught here too)
    // The component count check was removed as it's ineffective: since resolved starts
    // equal to root and only adds components (or skips "."), it can never have fewer components.

    Ok(resolved)
}

/// Mount a static file route
pub fn mount_static_route(router: Router, state: &AppState, path: &str) -> Router {
    let st = state.clone();

    if path == "/" {
        // For root path, register both "/" and "/*tail"
        let handler_root = {
            let st = st.clone();
            any(move |req: Request<Body>| {
                let st = st.clone();
                async move { serve_static(st, String::new(), req).await }
            })
        };
        let mut router = router.route("/", handler_root);

        let handler_tail = {
            let st = st.clone();
            any(move |Path(tail): Path<String>, req: Request<Body>| {
                let st = st.clone();
                async move { serve_static(st, tail, req).await }
            })
        };
        router = router.route("/*tail", handler_tail);
        router
    } else {
        // For non-root paths, use nested routing
        let base = path.trim_end_matches('/');
        let handler = {
            let st = st.clone();
            any(move |Path(tail): Path<String>, req: Request<Body>| {
                let st = st.clone();
                async move { serve_static(st, tail, req).await }
            })
        };
        router.route(&format!("{}/*tail", base), handler)
    }
}

/// Build the application router
pub fn build_router(state: &AppState) -> Result<Router> {
    let mut router = Router::new();
    let mut has_routes = false;

    for Route { path, serve, proxy } in &state.cfg.routing {
        // Routes should be mutually exclusive: either serve static files OR proxy, not both
        if serve.as_deref() == Some("static") {
            if proxy.is_some() {
                warn!("Route '{}' has both 'serve: static' and 'proxy' configured. 'proxy' will be ignored. Routes should be mutually exclusive.", path);
            }
            // Create handlers for static files and directories
            info!("Mounting static route: {}", path);
            router = mount_static_route(router, state, path);
            has_routes = true;
        } else if let Some(p) = proxy.clone() {
            let (route_path, handler) = make_proxy_route(path, p);
            router = router.route(&route_path, handler);
            has_routes = true;
        }
    }

    // Default: if no routes configured, serve static files at root
    if !has_routes {
        info!("No routes configured, defaulting to serve static files at /");
        router = mount_static_route(router, state, "/");
    }

    if state.cfg.spa.enabled {
        // Security: Safely resolve SPA fallback path within root directory
        let fallback = state.cfg.spa.fallback.trim_start_matches('/');
        let index_file = match resolve_path_within_root(&state.root, fallback) {
            Ok(path) => path,
            Err(_) => {
                // If path traversal detected, use default index.html
                warn!("SPA fallback path '{}' contains path traversal, using default index.html", fallback);
                state.root.join("index.html")
            }
        };
        router = router.fallback_service(ServeFile::new(index_file));
    }

    Ok(router)
}

/// Build compression layer
pub fn build_compression(cfg: &Config) -> Option<tower_http::compression::CompressionLayer> {
    if cfg.compression.enable && (cfg.compression.gzip || cfg.compression.br) {
        Some(tower_http::compression::CompressionLayer::new())
    } else {
        None
    }
}

/// Build CORS layer
pub fn build_cors(cfg: &Config) -> Option<CorsLayer> {
    if !cfg.security.cors.enabled {
        return None;
    }

    // Origins
    let origins = if cfg.security.cors.allowed_origins.is_empty() {
        tower_http::cors::AllowOrigin::any()
    } else {
        let list = cfg
            .security
            .cors
            .allowed_origins
            .iter()
            .filter_map(|s| HeaderValue::from_str(s).ok())
            .collect::<Vec<_>>();
        tower_http::cors::AllowOrigin::list(list)
    };

    // Methods
    let methods = if cfg.security.cors.allowed_methods.is_empty() {
        vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ]
    } else {
        cfg.security
            .cors
            .allowed_methods
            .iter()
            .filter_map(|m| Method::from_str(m).ok())
            .collect()
    };

    Some(
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(methods)
            .allow_headers(Any)
            .expose_headers(Any),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    #[test]
    fn test_resolve_path_within_root_valid() {
        let root = PathBuf::from("/tmp/test");
        
        // Valid relative path
        let result = resolve_path_within_root(&root, "index.html");
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert!(resolved.ends_with("index.html"));
        assert!(resolved.to_string_lossy().contains("test"));
    }

    #[test]
    fn test_resolve_path_within_root_empty() {
        let root = PathBuf::from("/tmp/test");
        
        // Empty path should return root
        let result = resolve_path_within_root(&root, "");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), root);
    }

    #[test]
    fn test_resolve_path_within_root_nested() {
        let root = PathBuf::from("/tmp/test");
        
        // Nested path
        let result = resolve_path_within_root(&root, "app/index.html");
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert!(resolved.to_string_lossy().contains("app"));
        assert!(resolved.to_string_lossy().contains("index.html"));
    }

    #[test]
    fn test_resolve_path_within_root_rejects_traversal() {
        let root = PathBuf::from("/tmp/test");
        
        // Path traversal should be rejected
        assert!(resolve_path_within_root(&root, "../etc/passwd").is_err());
        assert!(resolve_path_within_root(&root, "..").is_err());
        assert!(resolve_path_within_root(&root, "app/../../etc/passwd").is_err());
        assert!(resolve_path_within_root(&root, "../../etc/passwd").is_err());
    }

    #[test]
    fn test_resolve_path_within_root_with_current_dir() {
        let root = PathBuf::from("/tmp/test");
        
        // Current directory component should be allowed
        let result = resolve_path_within_root(&root, "./index.html");
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert!(resolved.ends_with("index.html"));
    }

    #[test]
    fn test_build_compression_disabled() {
        let cfg = Config::default();
        assert!(build_compression(&cfg).is_none());
    }

    #[test]
    fn test_build_compression_enabled() {
        let mut cfg = Config::default();
        cfg.compression.enable = true;
        cfg.compression.gzip = true;
        assert!(build_compression(&cfg).is_some());
    }

    #[test]
    fn test_build_compression_enabled_but_no_methods() {
        let mut cfg = Config::default();
        cfg.compression.enable = true;
        cfg.compression.gzip = false;
        cfg.compression.br = false;
        assert!(build_compression(&cfg).is_none());
    }

    #[test]
    fn test_build_cors_disabled() {
        let cfg = Config::default();
        assert!(build_cors(&cfg).is_none());
    }

    #[test]
    fn test_build_cors_enabled_no_origins() {
        let mut cfg = Config::default();
        cfg.security.cors.enabled = true;
        assert!(build_cors(&cfg).is_some());
    }

    #[test]
    fn test_build_cors_enabled_with_origins() {
        let mut cfg = Config::default();
        cfg.security.cors.enabled = true;
        cfg.security.cors.allowed_origins.push("https://example.com".to_string());
        assert!(build_cors(&cfg).is_some());
    }

    #[test]
    fn test_build_cors_enabled_with_methods() {
        let mut cfg = Config::default();
        cfg.security.cors.enabled = true;
        cfg.security.cors.allowed_methods.push("GET".to_string());
        cfg.security.cors.allowed_methods.push("POST".to_string());
        assert!(build_cors(&cfg).is_some());
    }
}

