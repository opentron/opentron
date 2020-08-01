use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Deserialize, Serialize, Debug)]
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
    // forbid-transfer-to-contract = false
    /// Default energy price is 100 SUN/unit. While in Mainnet/Testnet, it's 10 SUN/unit.
    #[serde(default = "default_energy_fee")]
    pub energy_fee: i64,
}

fn default_maintenance_interval() -> i64 {
    // Args.java: 6h
    21600_000
}

fn default_energy_fee() -> i64 {
    100
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct ChainConfig {
    pub genesis: String,
    /// Default p2p version is 0.
    #[serde(default = "Default::default")]
    pub p2p_version: i32,
    #[serde(default = "default_proposal_expiration_duration")]
    pub proposal_expiration_duration: String,
    pub parameter: ChainParameterConfig,
}

fn default_proposal_expiration_duration() -> String {
    "259200000".into()
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub struct StorageConfig {
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    // TODO: impl a different engine
    #[serde(default = "Default::default")]
    pub engine: String,
}

fn default_data_dir() -> String {
    "./data".into()
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
    #[serde(default = "default_sync_batch_size")]
    pub sync_batch_size: usize,
}

fn default_sync_batch_size() -> usize {
    200
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
#[serde(deny_unknown_fields)]
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
