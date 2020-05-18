use futures::channel::oneshot;
use log::info;
use primitives::H256;
use proto2::common::BlockId;
use std::collections::HashSet;
use std::error::Error;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use crate::config::Config;
use crate::db::ChainDB;
use crate::genesis::GenesisConfig;

pub struct AppContext {
    pub outbound_ip: String,
    pub genesis_block_id: Option<BlockId>,
    pub config: Config,
    pub db: ChainDB,
    pub running: Arc<AtomicBool>,
    pub recent_blk_ids: RwLock<HashSet<H256>>,
    pub syncing: RwLock<bool>,
    pub peers: RwLock<Vec<oneshot::Sender<()>>>,
}

impl AppContext {
    pub fn from_config<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let config = Config::load_from_file(path)?;

        let genesis_config = GenesisConfig::load_from_file(&config.chain.genesis)?;
        let genesis_blk = genesis_config.to_indexed_block()?;

        let db = ChainDB::new(&config.storage.data_dir);

        if !db.has_block(&genesis_blk) {
            if let Some(_) = db.get_genesis_block() {
                panic!("genesis block is inconsistent with db");
            }
            info!("insert genesis block to db");
            db.insert_block(&genesis_blk)?;
        } else {
            info!("genesis block check passed");
        }
        db.report_status();

        let genesis_block_id = BlockId {
            number: 0,
            hash: genesis_blk.header.hash.as_ref().to_owned(),
        };

        info!("chain version => {}", config.chain.p2p_version);
        info!("genesis block id => {}", hex::encode(&genesis_block_id.hash));
        Ok(AppContext {
            db,
            config,
            outbound_ip: String::new(),
            genesis_block_id: Some(genesis_block_id),
            running: Arc::new(AtomicBool::new(true)),
            recent_blk_ids: RwLock::new(HashSet::new()),
            syncing: RwLock::new(true),
            peers: RwLock::default(),
        })
    }
}
