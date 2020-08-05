use std::path::Path;

use clap::ArgMatches;
use log::info;
use chain_db::{ChainDB, CheckResult};
use config::Config;

pub async fn main<P: AsRef<Path>>(config_path: P, matches: &ArgMatches<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load_from_file(config_path)?;
    info!("config file loaded");
    let db = ChainDB::new(&config.storage.data_dir);
    info!("db opened");

    db.await_background_jobs();

    match matches.value_of("WHAT") {
        Some("compact") => {
            println!("compact db ...");
            let ret = db.compact_db();
            println!("compact => {:?}", ret);
        }
        Some("merkle_tree") => {
            /*
            let patch = config
                .merkle_tree_patch
                .map(|patch| {
                    patch
                        .iter()
                        .map(|patch| {
                            (
                                H256::from_slice(&hex::decode(&patch.txn).unwrap()),
                                H256::from_slice(&hex::decode(&patch.tree_node_hash).unwrap()),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            */
            db.verify_merkle_tree(&Default::default())?;
        }
        Some("parent_hash") => {
            while let CheckResult::ForkAt(pos) = db.verify_parent_hashes()? {
                db.handle_chain_fork_at(pos, /* dry_run */ false)?;
            }
        }
        _ => (),
    }

    info!("block height = {}", db.get_block_height());

    Ok(())
}
