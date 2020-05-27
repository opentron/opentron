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
    pub enable: bool,
    pub enable_passive: bool,
    pub enable_active: bool,
    pub endpoint: String,
    pub advertised_endpoint: String,
    pub active_nodes: Vec<String>,
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
pub struct GraphQLConfig {
    pub enable: bool,
    pub endpoint: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct MerkleTreePatch {
    pub txn: String,
    pub tree_node_hash: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub chain: ChainConfig,
    pub storage: StorageConfig,
    pub protocol: ProtocolConfig,
    pub graphql: GraphQLConfig,
    pub merkle_tree_patch: Option<Vec<MerkleTreePatch>>,
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
