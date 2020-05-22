use clap::ArgMatches;
use std::path::Path;

use crate::config::Config;
use crate::db::ChainDB;

pub async fn main<P: AsRef<Path>>(config_path: P, matches: &ArgMatches<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load_from_file(config_path)?;
    println!("opening db ...");
    let db = ChainDB::new(&config.storage.data_dir);
    println!("db opened");

    db.await_background_jobs();

    match matches.value_of("WHAT") {
        Some("compact") => {
            println!("compact db ...");
            let ret = db.compact_db();
            println!("compact => {:?}", ret);
        }
        Some("merkle_tree") => {
            db.verify_merkle_tree()?;
        }
        Some("parent_hash") => {
            db.verify_parent_hashes()?;
        }
        _ => (),
    }

    println!("height = {}", db.get_block_height());

    Ok(())
}
