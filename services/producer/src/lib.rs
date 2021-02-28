//! The block producer.

use chrono::Utc;
use context::AppContext;
use futures::future::FutureExt;
use keys::{Address, KeyPair};
use log::{info, warn};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;

// DposTask.java

pub async fn producer_task(
    ctx: Arc<AppContext>,
    mut termination_signal: broadcast::Receiver<()>,
) -> Result<(), Box<dyn Error>> {
    let config = &ctx.config.producer;

    if !config.enable {
        return Ok(());
    }

    let keypairs = load_keypairs_from_config(config);
    if keypairs.is_empty() {
        warn!("empty producer keypairs");
        return Ok(());
    }
    info!(
        "📦block producer enabled, with {} keys: {}",
        keypairs.len(),
        keypairs.keys().map(|k| k.to_string()).collect::<Vec<_>>().join(",")
    );

    // true except first block and first producer
    let mut sync_check_required = false;

    let mut manager = ctx.manager.write().unwrap();

    loop {
        if sync_check_required {
            tokio::time::sleep(Duration::from_secs(1)).await;
            info!("dposSlot.getTime(1)  {}", manager.get_slot_timestamp(1));
            info!("current {}", Utc::now().timestamp_millis());
            // if first slot timestamp is greater than current, skip sync check
            sync_check_required = manager.get_slot_timestamp(1) < Utc::now().timestamp_millis()
        } else {
            let d = constants::BLOCK_PRODUCING_INTERVAL -
                Utc::now().timestamp_millis() % constants::BLOCK_PRODUCING_INTERVAL;

            // produceBlock
            select! {
                _ = sleep(Duration::from_millis(d as u64)) => {
                    info!("produce block ... dummy");
                }
                _ = termination_signal.recv().fuse() => {
                    warn!("block producer closed");
                    break;
                }
            }
        }
    }

    Ok(())
}

fn load_keypairs_from_config(config: &config::ProducerConfig) -> HashMap<Address, KeyPair> {
    let mut keypairs: HashMap<Address, KeyPair> = Default::default();
    for key in &config.keypair {
        let keypair = key
            .private_key
            .parse()
            .and_then(KeyPair::from_private)
            .expect("invalid private key");
        let addr: Address = key.address.parse().expect("Invalid address");
        if keypair.address() != addr {
            warn!("inconsistent address and private key found, might be a multisig");
        }
        keypairs.insert(addr, keypair);
    }
    if let Some(ref keystore_path) = config.keystore {
        info!("load keystore file from {}", keystore_path);
        unimplemented!()
    }
    keypairs
}
