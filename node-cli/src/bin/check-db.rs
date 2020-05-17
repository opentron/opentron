extern crate node_cli;

use node_cli::config::Config;
use node_cli::db::ChainDB;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load_from_file("./conf.toml")?;
    let db = ChainDB::new(&config.storage.data_dir);
    println!("db opened");

    println!("compact db ...");
    let ret = db.compact_db();
    println!("compact => {:?}", ret);

    db.await_background_jobs();

    db.verify_parent_hashes()?;
    db.verify_merkle_tree()?;

    Ok(())
}
