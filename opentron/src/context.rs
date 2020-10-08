use std::collections::HashSet;
use std::error::Error;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::RwLock;

use chain_db::ChainDB;
use config::genesis::GenesisConfig;
use config::Config;
use futures::channel::oneshot;
use log::info;
use primitive_types::H256;
use proto2::common::BlockId;

use crate::manager::Manager;
use crate::util::get_my_ip;

pub struct AppContext {
    pub outbound_ip: String,
    pub node_id: Vec<u8>,
    pub genesis_block_id: Option<BlockId>,
    pub config: Config,
    pub genesis_config: GenesisConfig,
    pub chain_db: ChainDB,
    pub running: AtomicBool,
    pub syncing: AtomicBool,
    pub num_active_connections: AtomicU32,
    pub num_passive_connections: AtomicU32,
    pub recent_blk_ids: RwLock<HashSet<H256>>,
    pub peers: RwLock<Vec<oneshot::Sender<()>>>,
    pub manager: RwLock<Manager>,
}

impl AppContext {
    pub fn from_config<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let config = Config::load_from_file(&path)?;

        let genesis_path = path.as_ref().parent().unwrap().join(&config.chain.genesis);

        let genesis_config = GenesisConfig::load_from_file(&genesis_path)?;
        let genesis_blk = genesis_config.to_indexed_block()?;

        let chain_db = ChainDB::new(&config.storage.data_dir);
        if !chain_db.has_block(&genesis_blk) {
            if let Ok(_) = chain_db.get_genesis_block() {
                panic!("genesis block config is inconsistent with chain-db");
            }
            chain_db.insert_block(&genesis_blk)?;
            info!("inserted genesis block to chain-db");
        }
        chain_db.report_status();

        let outbound_ip = get_my_ip().unwrap_or("127.0.0.1".into());
        info!("outbound ip address: {}", outbound_ip);

        let genesis_block_id = BlockId {
            number: 0,
            hash: genesis_blk.header.hash.as_ref().to_owned(),
        };

        let node_id = chain_db.get_node_id();
        info!("node id => {}", hex::encode(&node_id));
        info!("p2p version => {}", config.chain.p2p_version);
        info!("genesis block id => {}", hex::encode(&genesis_block_id.hash));
        info!("chain-db loaded");

        let mut db_manager = Manager::new(&config, &genesis_config);
        let ref_block_hashes = chain_db.ref_block_hashes_of_block_num(db_manager.latest_block_number());
        db_manager.init_ref_blocks(ref_block_hashes);

        Ok(AppContext {
            chain_db,
            config,
            genesis_config,
            node_id,
            outbound_ip,
            genesis_block_id: Some(genesis_block_id),
            running: AtomicBool::new(true),
            syncing: AtomicBool::new(false),
            num_active_connections: AtomicU32::new(0),
            num_passive_connections: AtomicU32::new(0),
            recent_blk_ids: RwLock::new(HashSet::new()),
            peers: RwLock::default(),
            manager: RwLock::new(db_manager),
        })
    }
}
