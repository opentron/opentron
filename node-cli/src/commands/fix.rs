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

    if let Some(val) = matches.value_of("height") {
        println!("original block height => {}", db.get_block_height());
        let new_height = val.parse().expect("height number");
        db.force_update_block_height(new_height)?;
        println!("force update block height => {}", new_height);
    }

    if let Some(val) = matches.value_of("fork") {
        let block_number = val.parse().expect("height number");
        db.handle_chain_fork_at(block_number)?;
    }

    Ok(())
}
