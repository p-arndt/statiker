use crate::state::AppState;
use axum::{
    body::Body,
    http::{header::CONTENT_LENGTH, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
};
use http::Request;
use mime_guess;
use std::path::PathBuf;

/// Serve static files with auto-index support
pub async fn serve_static(state: AppState, tail: String, req: Request<Body>) -> Response {
    // Only allow GET and HEAD for static files
    match *req.method() {
        Method::GET | Method::HEAD => {}
        _ => {
            return StatusCode::METHOD_NOT_ALLOWED.into_response();
        }
    }

    // Security: disallow path traversal attempts like ".."
    if tail.split('/').any(|p| p == "..") {
        return StatusCode::FORBIDDEN.into_response();
    }

    // Compute normalized path relative to root
    let rel = tail.trim_start_matches('/');
    let mut fs_path = state.root.clone();
    if !rel.is_empty() {
        // Safely join path, preventing directory traversal
        for component in std::path::Path::new(rel).components() {
            match component {
                std::path::Component::Normal(os_str) => {
                    fs_path.push(os_str);
                }
                _ => {
                    return StatusCode::FORBIDDEN.into_response();
                }
            }
        }
    }

    // If path exists and is a file -> serve it
    match tokio::fs::metadata(&fs_path).await {
        Ok(meta) if meta.is_file() => {
            let file_size = meta.len();
            match tokio::fs::read(&fs_path).await {
                Ok(bytes) => {
                    let mime = mime_guess::from_path(&fs_path).first_or_octet_stream();
                    let mut builder = Response::builder().status(StatusCode::OK);
                    if let Ok(hv) = HeaderValue::from_str(&mime.to_string()) {
                        builder = builder.header("content-type", hv);
                    }
                    // Set Content-Length header for both GET and HEAD (required by HTTP spec)
                    if let Ok(cl_hv) = HeaderValue::from_str(&file_size.to_string()) {
                        builder = builder.header(CONTENT_LENGTH, cl_hv);
                    }
                    // For HEAD, return empty body but with Content-Length header
                    if req.method() == Method::HEAD {
                        builder.body(Body::empty()).unwrap()
                    } else {
                        builder.body(Body::from(bytes)).unwrap()
                    }
                }
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        // If it's a directory or doesn't exist, handle accordingly
        Ok(meta) if meta.is_dir() => {
            // try index file first
            let index_name = &state.cfg.server.index;
            let index_path = fs_path.join(index_name);
            // Get metadata to check if file exists and get its size
            if let Ok(index_meta) = tokio::fs::metadata(&index_path).await {
                if index_meta.is_file() {
                    let file_size = index_meta.len();
                    match tokio::fs::read(&index_path).await {
                        Ok(bytes) => {
                            let mime = mime_guess::from_path(&index_path).first_or_octet_stream();
                            let mut builder = Response::builder().status(StatusCode::OK);
                            if let Ok(hv) = HeaderValue::from_str(&mime.to_string()) {
                                builder = builder.header("content-type", hv);
                            }
                            // Set Content-Length header for both GET and HEAD (required by HTTP spec)
                            if let Ok(cl_hv) = HeaderValue::from_str(&file_size.to_string()) {
                                builder = builder.header(CONTENT_LENGTH, cl_hv);
                            }
                            // For HEAD, return empty body but with Content-Length header
                            if req.method() == Method::HEAD {
                                builder.body(Body::empty()).unwrap()
                            } else {
                                builder.body(Body::from(bytes)).unwrap()
                            }
                        }
                        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                    }
                } else {
                    // Index path exists but is not a file, fall through to auto-index or 404
                    if state.cfg.server.auto_index {
                        match render_directory_listing(&fs_path, rel).await {
                            Ok(html) => {
                                let html_len = html.len();
                                let mut builder = Response::builder().status(StatusCode::OK);
                                builder = builder.header("content-type", "text/html; charset=utf-8");
                                // Set Content-Length header for both GET and HEAD
                                if let Ok(cl_hv) = HeaderValue::from_str(&html_len.to_string()) {
                                    builder = builder.header(CONTENT_LENGTH, cl_hv);
                                }
                                if req.method() == Method::HEAD {
                                    builder.body(Body::empty()).unwrap()
                                } else {
                                    builder.body(Body::from(html)).unwrap()
                                }
                            }
                            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                        }
                    } else {
                        StatusCode::NOT_FOUND.into_response()
                    }
                }
            } else if state.cfg.server.auto_index {
                // Index file doesn't exist, try auto-index
                match render_directory_listing(&fs_path, rel).await {
                    Ok(html) => {
                        let html_len = html.len();
                        let mut builder = Response::builder().status(StatusCode::OK);
                        builder = builder.header("content-type", "text/html; charset=utf-8");
                        // Set Content-Length header for both GET and HEAD
                        if let Ok(cl_hv) = HeaderValue::from_str(&html_len.to_string()) {
                            builder = builder.header(CONTENT_LENGTH, cl_hv);
                        }
                        if req.method() == Method::HEAD {
                            builder.body(Body::empty()).unwrap()
                        } else {
                            builder.body(Body::from(html)).unwrap()
                        }
                    }
                    Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                }
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        }
        // Path doesn't exist
        _ => {
            // If SPA is enabled, the router may fallback to SPA index. But here return 404.
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

/// Render HTML directory listing
pub async fn render_directory_listing(dir: &PathBuf, rel_path: &str) -> std::io::Result<String> {
    let mut entries = tokio::fs::read_dir(dir).await?;
    let mut items: Vec<(String, bool)> = Vec::new(); // (name, is_dir)
    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let meta = entry.metadata().await?;
        items.push((file_name, meta.is_dir()));
    }
    // sort: directories first, then files, both alphabetically
    items.sort_by(|a, b| match (a.1, b.1) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
    });

    // Build simple HTML
    let title = if rel_path.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", rel_path)
    };
    let mut html = String::new();
    html.push_str("<!doctype html>\n<html><head><meta charset=\"utf-8\"><title>Index of ");
    html.push_str(&html_escape::encode_text(&title));
    html.push_str("</title><style>body { font-family: monospace; margin: 20px; } h1 { color: #333; } ul { list-style: none; padding: 0; } li { padding: 5px 0; } a { color: #0066cc; text-decoration: none; } a:hover { text-decoration: underline; } hr { margin-top: 20px; border: none; border-top: 1px solid #ccc; }</style></head><body><h1>Index of ");
    html.push_str(&html_escape::encode_text(&title));
    html.push_str("</h1><ul>");

    // parent link if not root
    if !rel_path.is_empty() {
        let parent = {
            let mut p = std::path::Path::new(rel_path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            if p == "." {
                p = "".into();
            }
            format!("/{}", p)
        };
        html.push_str(&format!(
            "<li><a href=\"{}\">..</a></li>",
            html_escape::encode_double_quoted_attribute(&parent)
        ));
    }

    for (name, is_dir) in items {
        // Construct URL path
        let mut url = String::new();
        if rel_path.is_empty() {
            url.push('/');
            url.push_str(&name);
        } else {
            url.push('/');
            url.push_str(rel_path.trim_end_matches('/'));
            url.push('/');
            url.push_str(&name);
        }
        if is_dir {
            url.push('/');
        }
        // Escape for safety
        let esc_url = html_escape::encode_double_quoted_attribute(&url);
        let esc_name = html_escape::encode_text(&name);
        html.push_str(&format!(
            "<li><a href=\"{}\">{}</a></li>",
            esc_url, esc_name
        ));
    }

    html.push_str("</ul><hr><address>statiker</address></body></html>");
    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use axum::http::{header::CONTENT_LENGTH, Method};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_serve_static_path_traversal() {
        let state = AppState {
            cfg: Arc::new(Config::default()),
            root: std::path::PathBuf::from("."),
            limiter: None,
        };
        let req = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(Body::empty())
            .unwrap();
        let res = serve_static(state, "../etc/passwd".to_string(), req).await;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_serve_static_method_not_allowed() {
        let state = AppState {
            cfg: Arc::new(Config::default()),
            root: std::path::PathBuf::from("."),
            limiter: None,
        };
        let req = Request::builder()
            .method(Method::POST)
            .uri("/")
            .body(Body::empty())
            .unwrap();
        let res = serve_static(state, "".to_string(), req).await;
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_head_request_has_content_length() {
        // Test with Cargo.toml which should exist in the project root
        let state = AppState {
            cfg: Arc::new(Config::default()),
            root: std::path::PathBuf::from("."),
            limiter: None,
        };
        let req = Request::builder()
            .method(Method::HEAD)
            .uri("/Cargo.toml")
            .body(Body::empty())
            .unwrap();
        let res = serve_static(state, "Cargo.toml".to_string(), req).await;

        // If file exists, verify Content-Length header is set
        if res.status() == StatusCode::OK {
            assert!(res.headers().contains_key(CONTENT_LENGTH), "HEAD response should include Content-Length header");
            // Verify body is empty for HEAD request
            let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
            assert_eq!(body_bytes.len(), 0, "HEAD response should have empty body");
        }
    }

    #[tokio::test]
    async fn test_get_request_has_content_length() {
        // Test with Cargo.toml which should exist in the project root
        let state = AppState {
            cfg: Arc::new(Config::default()),
            root: std::path::PathBuf::from("."),
            limiter: None,
        };
        let req = Request::builder()
            .method(Method::GET)
            .uri("/Cargo.toml")
            .body(Body::empty())
            .unwrap();
        let res = serve_static(state, "Cargo.toml".to_string(), req).await;

        // If file exists, verify Content-Length header is set
        if res.status() == StatusCode::OK {
            assert!(res.headers().contains_key(CONTENT_LENGTH), "GET response should include Content-Length header");
            let cl_value = res.headers().get(CONTENT_LENGTH).unwrap().to_str().unwrap();
            let cl_num: usize = cl_value.parse().unwrap();
            // Verify body contains the file content and matches Content-Length
            let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
            assert_eq!(body_bytes.len(), cl_num, "Body length should match Content-Length header");
            assert!(body_bytes.len() > 0, "GET response should have non-empty body");
        }
    }
}

