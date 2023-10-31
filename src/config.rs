use anyhow::{Context, Result};
use tokio::{fs::File, io::AsyncReadExt};

use serde::Deserialize;

/// Reads configs from the config file
/// where is "\<cuzi executable directory\>/config.json".
pub async fn read_config_file() -> Result<Config> {
    let mut file = File::open("config.json").await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    let config: Config = serde_json::from_str(&contents).context("Invalid config config.json")?;

    return Ok(config);
}

/// Represents the global configuration of Cuzi.
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Port number will Cuzi works on it.
    pub port: i32,

    /// Proxy definitions.
    pub proxies: Vec<ProxyConfig>,
}

/// Represents a proxy configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct ProxyConfig {
    /// Target path such as "/blog/new-article" or "/api/auth".
    pub path: String,

    /// Target domain which could be an IP address or domain name.
    pub target: String,

    /// Target port number. Mostly 80 or 443 for webpage proxies.
    pub target_port: i32,

    /// Does this proxy will be over TLS?
    pub use_tls: Option<bool>,
}
