use std::path::Path;

use crate::config::Config;
use crate::db::ChainDB;

pub async fn main<P: AsRef<Path>>(config_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load_from_file(config_path)?;
    let db = ChainDB::new(&config.storage.data_dir);
    println!("db opened");

    println!("compact db ...");
    let ret = db.compact_db();
    println!("compact => {:?}", ret);

    db.await_background_jobs();

    db.verify_parent_hashes()?;
    db.verify_merkle_tree()?;

    println!("height = {}", db.get_block_height());

    Ok(())
}
