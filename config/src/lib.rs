//! Config parser.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub use genesis::GenesisConfig;

pub mod genesis;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct ChainParameterConfig {
    #[serde(default = "default_maintenance_interval")]
    pub maintenance_interval: i64,
    #[serde(default = "Default::default")]
    pub allow_multisig: bool,
    #[serde(default = "Default::default")]
    pub allow_adaptive_energy: bool,
    #[serde(default = "Default::default")]
    pub allow_delegate_resource: bool,
    #[serde(default = "Default::default")]
    pub allow_duplicate_asset_names: bool,
    #[serde(default = "Default::default")]
    pub allow_tvm: bool,
    #[serde(default = "Default::default")]
    pub allow_tvm_transfer_trc10_upgrade: bool,
    #[serde(default = "Default::default")]
    pub allow_tvm_constantinople_upgrade: bool,
    #[serde(default = "Default::default")]
    pub allow_tvm_solidity_059_upgrade: bool,
    #[serde(default = "Default::default")]
    pub allow_tvm_shielded_upgrade: bool,
    #[serde(default = "Default::default")]
    pub allow_tvm_istanbul_upgrade: bool,
    // forbid-transfer-to-contract = false
    /// Default energy price is 100 SUN/unit. While in Mainnet/Testnet, it's 140 SUN/unit.
    #[serde(default = "default_energy_fee")]
    pub energy_fee: i64,
    #[serde(default = "default_bandwidth_fee")]
    pub bandwidth_fee: i64,
}

fn default_maintenance_interval() -> i64 {
    // Args.java: 6h
    21600_000
}

fn default_energy_fee() -> i64 {
    100
}

fn default_bandwidth_fee() -> i64 {
    100
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct ChainConfig {
    pub genesis: String,
    /// Default p2p version is 0.
    #[serde(default = "Default::default")]
    pub p2p_version: i32,
    #[serde(default = "default_proposal_expiration_duration")]
    pub proposal_expiration_duration: i64,
    pub parameter: ChainParameterConfig,
}

fn default_proposal_expiration_duration() -> i64 {
    259200_000
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct StorageConfig {
    /// Path to ChainDB.
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    // TODO: impl a different engine
    #[serde(default = "Default::default")]
    pub engine: String,
    /// Path to StateDB.
    #[serde(default = "default_state_data_dir")]
    pub state_data_dir: String,
    #[serde(default = "default_state_cache_dir")]
    pub state_cache_dir: String,
}

fn default_data_dir() -> String {
    "./data/chaindb".into()
}

fn default_state_data_dir() -> String {
    "./data/statedb".into()
}

fn default_state_cache_dir() -> String {
    "./data/cache".into()
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct DiscoveryProtoConfig {
    pub enable: bool,
    pub endpoint: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ChannelProtoConfig {
    pub enable: bool,
    pub enable_passive: bool,
    pub enable_active: bool,
    pub endpoint: String,
    pub advertised_endpoint: String,
    pub active_nodes: Vec<String>,
    pub max_active_connections: u32,
    #[serde(default = "default_sync_batch_size")]
    pub sync_batch_size: usize,
}

fn default_sync_batch_size() -> usize {
    200
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProtocolConfig {
    pub seed_nodes: Vec<String>,
    pub discovery: DiscoveryProtoConfig,
    pub channel: ChannelProtoConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct GraphQLConfig {
    pub enable: bool,
    pub endpoint: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct ProducerKey {
    pub address: String,
    pub private_key: String,
}

/// Config for block producer.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct ProducerConfig {
    pub enable: bool,
    /// Key store file path
    pub keystore: Option<String>,
    // Key paris in config file
    pub keypair: Vec<ProducerKey>,
}

impl Default for ProducerConfig {
    fn default() -> Self {
        ProducerConfig {
            enable: false,
            keystore: None,
            keypair: vec![],
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub chain: ChainConfig,
    pub storage: StorageConfig,
    pub protocol: ProtocolConfig,
    pub graphql: GraphQLConfig,
    #[serde(default = "Default::default")]
    pub producer: ProducerConfig,
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn load_from_str(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(toml::from_str(content)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_mainnet_config() {
        assert!(Config::load_from_str(include_str!("../../etc/conf.toml")).is_ok());
    }
}
