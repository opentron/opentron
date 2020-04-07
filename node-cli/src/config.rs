use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const CONFIG_FILE: &str = "./conf.toml";

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ChainConfig {
    pub genesis: String,
    pub p2p_version: i32,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct StorageConfig {
    pub data_dir: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct DiscoveryProtoConfig {
    pub enable: bool,
    pub endpoint: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ChannelProtoConfig {
    pub endpoint: String,
    pub advertised_endpoint: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ProtocolConfig {
    pub seed_nodes: Vec<String>,
    pub discovery: DiscoveryProtoConfig,
    pub channel: ChannelProtoConfig,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub chain: ChainConfig,
    pub storage: StorageConfig,
    pub protocol: ProtocolConfig,
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore]
    fn test_config_parse() {
        let content = fs::read_to_string(CONFIG_FILE).unwrap();
        let config: Config = toml::from_str(&content).unwrap();
        println!("yep => {:?}", config);
    }
}
