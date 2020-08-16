use clap::ArgMatches;
use log::info;
use std::path::Path;

use crate::context::AppContext;
use crate::manager::Manager;
use chrono::Utc;

pub async fn main<P: AsRef<Path>>(config_path: P, _matches: &ArgMatches<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = AppContext::from_config(config_path)?;

    let mut db_manager = Manager::new(&ctx.config, &ctx.genesis_config);

    let ref_block_hashes = ctx
        .chain_db
        .ref_block_hashes_of_block_num(db_manager.latest_block_number());
    /*for (i, v) in ref_block_hashes.iter().enumerate() {
        println!("{} {:x}=> {:?}", i, i, v);
    }*/
    db_manager.init_ref_blocks(ref_block_hashes);

    let mut start_time = Utc::now().timestamp_millis();
    let mut n_blocks = 0;

    let start_block = db_manager.latest_block_number() as u64 + 1;

    // 741457, first AssetIssueContract
    for i in start_block.. {
        let blk = ctx.chain_db.get_block_by_number(i)?;

        db_manager.push_block(&blk)?;

        n_blocks += 1;

        if n_blocks % 1_000 == 0 {
            let now = Utc::now().timestamp_millis();
            info!("speed => {}blocks/s", 1_000 * 1_000 / (now - start_time));
            start_time = Utc::now().timestamp_millis();
        }
    }

    Ok(())
}
