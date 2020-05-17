extern crate node_cli;

use node_cli::config::Config;
use node_cli::db::ChainDB;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load_from_file("./conf.toml")?;
    let db = ChainDB::new(&config.storage.data_dir);

    db.verify_merkle_tree()?;

    Ok(())
}
