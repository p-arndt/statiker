use crate::config::Config;
use clap::Parser;

/// Static file server with proxy support
#[derive(Parser, Debug)]
#[command(name = "statiker")]
#[command(version)]
#[command(about = "A simple, efficient static file hosting server written in Rust", long_about = None)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, env = "CONFIG", default_value = "statiker.yaml")]
    pub config: String,
}

/// Print configuration summary
pub fn print_config(cfg: &Config) {
    println!("=== Configuration ===");
    println!("Server: {}:{}", cfg.server.host, cfg.server.port);
    println!("Root: {}", cfg.server.root.display());
    println!("Index: {}", cfg.server.index);
    println!("Auto-index: {}", cfg.server.auto_index);

    if cfg.tls.enabled {
        println!("TLS: enabled");
    }

    if cfg.routing.is_empty() {
        println!("Routes: default (serve static at /)");
    } else {
        println!("Routes: {}", cfg.routing.len());
        for route in &cfg.routing {
            if let Some(serve) = &route.serve {
                println!("  - {} -> serve: {}", route.path, serve);
            }
            if route.proxy.is_some() {
                println!("  - {} -> proxy", route.path);
            }
        }
    }

    if cfg.spa.enabled {
        println!("SPA: enabled (fallback: {})", cfg.spa.fallback);
    }

    if cfg.compression.enable {
        let mut methods = Vec::new();
        if cfg.compression.gzip {
            methods.push("gzip");
        }
        if cfg.compression.br {
            methods.push("brotli");
        }
        let methods_str = methods.join(", ");
        println!("Compression: enabled ({})", methods_str);
    }

    if cfg.security.cors.enabled {
        println!("CORS: enabled");
    }

    if cfg.security.rate_limit.enabled {
        println!("Rate limit: {} req/min", cfg.security.rate_limit.requests_per_min);
    }

    if !cfg.security.headers.is_empty() {
        println!("Security headers: {} configured", cfg.security.headers.len());
    }

    if cfg.assets.cache.enabled {
        println!("Asset cache: enabled (max-age: {}s)", cfg.assets.cache.max_age.as_secs());
    }

    println!("Log level: {}", cfg.obs.level);
    println!("====================");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_config() {
        let cfg = Config::default();
        // Just verify it doesn't panic
        print_config(&cfg);
    }

    #[test]
    fn test_cli_parse_default() {
        // Test default value
        let cli = Cli::parse_from(vec!["statiker"]);
        assert_eq!(cli.config, "statiker.yaml");
    }

    #[test]
    fn test_cli_parse_long_flag() {
        // Test long flag
        let cli = Cli::parse_from(vec!["statiker", "--config", "custom.yaml"]);
        assert_eq!(cli.config, "custom.yaml");
    }

    #[test]
    fn test_cli_parse_short_flag() {
        // Test short flag
        let cli = Cli::parse_from(vec!["statiker", "-c", "short.yaml"]);
        assert_eq!(cli.config, "short.yaml");
    }
}
