use clap::ArgMatches;
use log::info;

use crate::context::AppContext;

pub async fn main(ctx: AppContext, matches: &ArgMatches<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let ref db = ctx.chain_db;

    db.await_background_jobs();

    if let Some(val) = matches.value_of("height") {
        info!("original block height => {}", db.get_block_height());
        let new_height = val.parse().expect("height number");
        db.force_update_block_height(new_height)?;
        info!("force update block height => {}", new_height);
    }

    if let Some(val) = matches.value_of("fork") {
        let block_number = val.parse().expect("height number");
        db.handle_chain_fork_at(block_number, /* dry_run */ false)?;
    }

    Ok(())
}
