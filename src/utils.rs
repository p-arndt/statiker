/// Check if a path is an asset file based on extension
pub fn is_asset_path(p: &str) -> bool {
    const EXTS: &[&str] = &[
        "css", "js", "mjs", "map", "png", "jpg", "jpeg", "gif", "webp", "svg", "ico", "ttf", "otf",
        "woff", "woff2", "mp4", "webm", "mp3",
    ];
    p.rsplit('.')
        .next()
        .map(|e| EXTS.contains(&e))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_asset_path_css() {
        assert!(is_asset_path("style.css"));
        assert!(is_asset_path("app.min.css"));
    }

    #[test]
    fn test_is_asset_path_js() {
        assert!(is_asset_path("script.js"));
        assert!(is_asset_path("app.mjs"));
        assert!(is_asset_path("bundle.js"));
    }

    #[test]
    fn test_is_asset_path_images() {
        assert!(is_asset_path("image.png"));
        assert!(is_asset_path("photo.jpg"));
        assert!(is_asset_path("picture.jpeg"));
        assert!(is_asset_path("icon.gif"));
        assert!(is_asset_path("logo.webp"));
        assert!(is_asset_path("vector.svg"));
        assert!(is_asset_path("favicon.ico"));
    }

    #[test]
    fn test_is_asset_path_fonts() {
        assert!(is_asset_path("font.ttf"));
        assert!(is_asset_path("font.otf"));
        assert!(is_asset_path("font.woff"));
        assert!(is_asset_path("font.woff2"));
    }

    #[test]
    fn test_is_asset_path_media() {
        assert!(is_asset_path("video.mp4"));
        assert!(is_asset_path("video.webm"));
        assert!(is_asset_path("audio.mp3"));
    }

    #[test]
    fn test_is_asset_path_not_assets() {
        assert!(!is_asset_path("file.txt"));
        assert!(!is_asset_path("README"));
        assert!(!is_asset_path(""));
        assert!(!is_asset_path("document.pdf"));
        assert!(!is_asset_path("data.json"));
        assert!(!is_asset_path("statiker.yaml"));
    }

    #[test]
    fn test_is_asset_path_with_path() {
        assert!(is_asset_path("/assets/style.css"));
        assert!(is_asset_path("static/js/app.js"));
        assert!(is_asset_path("public/images/logo.png"));
        assert!(!is_asset_path("/path/to/file.txt"));
        assert!(!is_asset_path("docs/README.md"));
    }

    #[test]
    fn test_is_asset_path_case_sensitive() {
        // Extension matching is case-sensitive - lowercase only
        assert!(!is_asset_path("file.CSS")); // Uppercase doesn't match
        assert!(!is_asset_path("file.JS")); // Uppercase doesn't match
        assert!(is_asset_path("file.css")); // Lowercase works
        assert!(is_asset_path("file.js")); // Lowercase works
    }
}

