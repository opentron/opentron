use chain_db::CheckResult;
use clap::ArgMatches;

use log::info;

use crate::context::AppContext;

pub async fn main(ctx: AppContext, matches: &ArgMatches<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let ref db = ctx.chain_db;

    db.await_background_jobs();

    match matches.value_of("WHAT") {
        Some("compact") => {
            println!("compact db ...");
            let ret = db.compact_db();
            println!("compact => {:?}", ret);
        }
        Some("merkle_tree") => {
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
