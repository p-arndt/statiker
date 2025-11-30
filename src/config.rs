use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, time::Duration};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub server: Server,
    #[serde(default)]
    pub tls: Tls,
    #[serde(default)]
    pub routing: Vec<Route>,
    #[serde(default)]
    pub spa: Spa,
    #[serde(default)]
    pub assets: Assets,
    #[serde(default)]
    pub compression: Compression,
    #[serde(default)]
    pub security: Security,
    #[serde(default)]
    pub obs: Obs,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
    pub root: PathBuf,
    pub index: String,
    #[serde(default)]
    pub auto_index: bool,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: 8080,
            root: PathBuf::from("."),
            index: "index.html".into(),
            auto_index: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tls {
    pub enabled: bool,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

impl Default for Tls {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: PathBuf::new(),
            key_path: PathBuf::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Route {
    pub path: String,
    #[serde(default)]
    pub serve: Option<String>,
    #[serde(default)]
    pub proxy: Option<Proxy>,
}

impl Default for Route {
    fn default() -> Self {
        Self {
            path: "/".into(),
            serve: None,
            proxy: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Proxy {
    pub url: String,
    #[serde(default, with = "humantime_serde")]
    pub timeout: Duration,
    #[serde(default)]
    pub add_headers: HashMap<String, String>,
}

impl Default for Proxy {
    fn default() -> Self {
        Self {
            url: String::new(),
            timeout: Duration::ZERO,
            add_headers: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Spa {
    pub enabled: bool,
    pub fallback: String,
}

impl Default for Spa {
    fn default() -> Self {
        Self {
            enabled: false,
            fallback: "index.html".into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Assets {
    #[serde(default)]
    pub cache: Cache,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Cache {
    pub enabled: bool,
    #[serde(default, with = "humantime_serde")]
    pub max_age: Duration,
    #[serde(default)]
    pub etag: bool, // NOTE: not computed in this MVP (toggle ignored if false)
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            enabled: false,
            max_age: Duration::from_secs(3600),
            etag: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Compression {
    pub enable: bool,
    pub gzip: bool,
    pub br: bool,
}

impl Default for Compression {
    fn default() -> Self {
        Self {
            enable: false,
            gzip: true,
            br: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Security {
    #[serde(default)]
    pub cors: Cors,
    #[serde(default)]
    pub rate_limit: RateLimit,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

impl Default for Security {
    fn default() -> Self {
        Self {
            cors: Cors::default(),
            rate_limit: RateLimit::default(),
            headers: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Cors {
    pub enabled: bool,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    #[serde(default)]
    pub allowed_methods: Vec<String>,
}

impl Default for Cors {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_origins: Vec::new(),
            allowed_methods: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimit {
    pub enabled: bool,
    pub requests_per_min: u32,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_min: 60,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Obs {
    pub level: String, // "info", "debug", ...
}

impl Default for Obs {
    fn default() -> Self {
        Self {
            level: "info".into(),
        }
    }
}
