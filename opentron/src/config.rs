use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ChainParameterConfig {
    pub maintenance_interval: i64,
    pub allow_creation_of_contracts: bool,
    pub allow_multisig: bool,
    pub allow_adaptive_energy: bool,
    pub allow_delegate_resource: bool,
    pub allow_duplicate_asset_names: bool,
    pub allow_tvm_transfer_trc10_upgrade: bool,
    pub allow_tvm_constantinople_upgrade: bool,
    pub allow_tvm_solidity_059_upgrade: bool,
    pub allow_shielded_trc20_transaction: bool,
    // forbid-transfer-to-contract = false
    pub energy_fee: Option<i64>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ChainConfig {
    pub genesis: String,
    pub p2p_version: i32,
    pub proposal_expiration_duration: String,
    pub parameter: ChainParameterConfig,
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
    pub max_active_connections: u32,
    pub sync_batch_size: usize,
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
pub struct Config {
    pub chain: ChainConfig,
    pub storage: StorageConfig,
    pub protocol: ProtocolConfig,
    pub graphql: GraphQLConfig,
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}
