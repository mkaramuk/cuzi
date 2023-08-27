use anyhow::{Context, Result};
use tokio::{fs::File, io::AsyncReadExt};

use serde::Deserialize;

pub async fn read_config_file() -> Result<Config> {
    let mut file = File::open("config.json").await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    let config: Config = serde_json::from_str(&contents).context("Invalid config config.json")?;

    return Ok(config);
}

// The config may contain fields that will be readed pretty regularly,
// so its best to clone it when needed
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub port: i32,
    pub proxies: Vec<ProxyConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ProxyConfig {
    pub path: String,
    pub target: String,
    pub target_port: i32,
}
