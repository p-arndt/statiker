use crate::config::Config;
use anyhow::{Context, Result};
use axum_server::tls_rustls::RustlsConfig;
use tokio::io::AsyncReadExt;
use tracing::info;

/// Validate TLS configuration and files
pub async fn validate_tls(cfg: &Config) -> Result<()> {
    if !cfg.tls.enabled {
        return Ok(());
    }

    // Ensure cert & key paths are provided and files exist
    if cfg.tls.cert_path.as_os_str().is_empty() || cfg.tls.key_path.as_os_str().is_empty() {
        return Err(anyhow::anyhow!(
            "TLS enabled but cert_path or key_path is empty. Provide paths in config or disable TLS."
        ));
    }

    // Check files exist and are readable
    let cert_ok = tokio::fs::metadata(&cfg.tls.cert_path)
        .await
        .map(|m| m.is_file())
        .unwrap_or(false);
    let key_ok = tokio::fs::metadata(&cfg.tls.key_path)
        .await
        .map(|m| m.is_file())
        .unwrap_or(false);
    if !cert_ok || !key_ok {
        return Err(anyhow::anyhow!(
            "TLS enabled but cert or key file not found or not a regular file. cert_path='{:?}', key_path='{:?}'",
            &cfg.tls.cert_path,
            &cfg.tls.key_path
        ));
    }

    // Optional: quick attempt to read the files to ensure they're accessible
    let mut buf = Vec::new();
    let mut cert_f = tokio::fs::File::open(&cfg.tls.cert_path)
        .await
        .context("opening TLS cert file")?;
    cert_f
        .read_to_end(&mut buf)
        .await
        .context("reading TLS cert file")?;
    buf.clear();
    let mut key_f = tokio::fs::File::open(&cfg.tls.key_path)
        .await
        .context("opening TLS key file")?;
    key_f
        .read_to_end(&mut buf)
        .await
        .context("reading TLS key file")?;
    info!("TLS enabled and cert/key validated.");

    Ok(())
}

/// Load TLS configuration
pub async fn load_tls_config(cfg: &Config) -> Result<RustlsConfig> {
    RustlsConfig::from_pem_file(
        cfg.tls.cert_path.clone(),
        cfg.tls.key_path.clone(),
    )
    .await
    .context("loading TLS cert/key")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_validate_tls_disabled() {
        let cfg = Config::default();
        assert!(validate_tls(&cfg).await.is_ok());
    }

    #[tokio::test]
    async fn test_validate_tls_enabled_no_paths() {
        let mut cfg = Config::default();
        cfg.tls.enabled = true;
        let result = validate_tls(&cfg).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[tokio::test]
    async fn test_validate_tls_enabled_empty_cert_path() {
        let mut cfg = Config::default();
        cfg.tls.enabled = true;
        cfg.tls.key_path = std::path::PathBuf::from("key.pem");
        let result = validate_tls(&cfg).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_tls_enabled_empty_key_path() {
        let mut cfg = Config::default();
        cfg.tls.enabled = true;
        cfg.tls.cert_path = std::path::PathBuf::from("cert.pem");
        let result = validate_tls(&cfg).await;
        assert!(result.is_err());
    }
}

