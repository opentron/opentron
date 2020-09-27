use chrono::Utc;
use log::info;

use crate::context::AppContext;

pub async fn main(ctx: AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = ctx.manager.write().unwrap();

    let mut start_time = Utc::now().timestamp_millis();
    let mut n_blocks = 0;

    let start_block = manager.latest_block_number() as u64 + 1;

    for i in start_block.. {
        let blk = ctx.chain_db.get_block_by_number(i)?;

        manager.push_block(&blk)?;

        n_blocks += 1;

        if n_blocks % 1_000 == 0 {
            let now = Utc::now().timestamp_millis();
            info!("speed => {}blocks/s", 1_000 * 1_000 / (now - start_time));
            start_time = Utc::now().timestamp_millis();
        }
    }

    Ok(())
}
