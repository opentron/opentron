use clap::ArgMatches;
use log::info;
use std::path::Path;

// use crate::config::Config;
// use crate::db::ChainDB;
use state::db::StateDB;
use crate::context::AppContext;

pub async fn main<P: AsRef<Path>>(config_path: P, _matches: &ArgMatches<'_>) -> Result<(), Box<dyn std::error::Error>> {
    // let config = Config::load_from_file(config_path)?;
    let ctx = AppContext::from_config(config_path)?;
    info!("config file loaded");
    // info!("{:#?}", ctx.config);
    let mut state_db = StateDB::new(&ctx.config.storage.state_data_dir);
    info!("state db opened");
    // info!("genesis config => {:#?}", ctx.genesis_config);

    state_db.init_genesis(&ctx.genesis_config, &ctx.config.chain)?;

    for i in 1..10 {
        let blk = ctx.db.get_block_by_number(i)?;

       // state_db.apply_block(&blk);
    }

    Ok(())
}
