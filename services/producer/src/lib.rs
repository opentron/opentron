//! The block producer.
//!
//! The java-tron's block producer is not optimized.
//! It sleeps and wait for next slot, so it can only use 750ms(default) to generate a block,
//! Making the 3s block producing interval meaningless.

use chrono::Utc;
use context::AppContext;
use futures::future::FutureExt;
use keys::{Address, KeyPair};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;

pub enum State {
    Ok,
    /// Same block number, different block id, multiple nodes using same producer key.
    DuplicatedWitness,
    SystemClockLagged,
    /// Not my turn
    NotMySlot,
    NotTimeYet,
    /// Participation:{} <  minParticipationRate:{}
    LowParticipationRate,
    ProduceFailed,
}

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
            info!("dposSlot.getTime(1) {}", manager.get_slot_timestamp(1));
            info!("current {}", Utc::now().timestamp_millis());
            // if first slot timestamp is greater than current, skip sync check
            sync_check_required = manager.get_slot_timestamp(1) < Utc::now().timestamp_millis()
        } else {
            // duration to next slot
            let d = constants::BLOCK_PRODUCING_INTERVAL -
                Utc::now().timestamp_millis() % constants::BLOCK_PRODUCING_INTERVAL;

            select! {
                _ = sleep(Duration::from_millis(d as u64)) => {
                    // produceBlock

                    let slot = manager.get_slot(Utc::now().timestamp_millis() + 50);
                    let block_timestamp = manager.get_slot_timestamp(slot);
                    let block_number = manager.latest_block_number() + 1;

                    debug!("produce block #{} slot={} timestamp={}", block_number, slot, block_timestamp);

                    match block_number {
                        1 => {
                            info!("👀generating block #1 without sync check");
                            let (witness_address, keypair) = keypairs.iter().next().unwrap();
                            let new_block =
                                manager.generate_empty_block(block_timestamp, witness_address, keypair).unwrap();
                            ctx.chain_db.insert_block(&new_block).expect("TODO: handle insert_block error");
                            info!("=> {:?}", new_block.hash());
                            info!("=> produce {:?}", manager.push_generated_block(&new_block));
                            info!("block pushed");
                        }
                        _ if block_number > 1 => {
                            let witness_address = manager.get_scheduled_witness(slot);
                            if let Some(keypair) = keypairs.get(&witness_address) {
                                info!("TODO: generate block #{} with {}", block_number, witness_address);

                                let deadline = block_timestamp + constants::BLOCK_PRODUCING_INTERVAL / 2 *
                                    constants::BLOCK_PRODUCE_TIMEOUT_PERCENT / 100;

                                // produce
                                debug!("deadline {}", deadline);
                                let new_block = manager.generate_empty_block(block_timestamp, &witness_address, keypair).unwrap();
                                ctx.chain_db.insert_block(&new_block).expect("TODO: handle insert_block error");
                                info!("=> {:?}", new_block.hash());
                                info!("=> produce {:?}", manager.push_generated_block(&new_block));


                            } else {
                                // Not my turn, pass
                            }
                        }
                        _ => unreachable!("block number is always >= 1")
                    }
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
