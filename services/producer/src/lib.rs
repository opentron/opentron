//! The block producer.

use context::AppContext;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::broadcast;

// DposTask.java

pub async fn producer_task(ctx: Arc<AppContext>, signal: broadcast::Receiver<()>) -> Result<(), Box<dyn Error>> {
    let config = &ctx.config.producer;

    if !config.enable {
        return Ok(());
    }

    Ok(())
}
