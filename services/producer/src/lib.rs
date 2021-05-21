//! The block producer.
//!
//! The java-tron's block producer is not optimized.
//! It sleeps and wait for next slot, so it can only use 750ms(default) to generate a block,
//! Making the 3s block producing interval meaningless.

use chrono::Utc;
use futures::future::FutureExt;
use indexmap::IndexMap;
use log::{debug, info, trace, warn};
use types::H256;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast;
use tokio::time::sleep;

use chain::IndexedTransaction;
use context::AppContext;
use keys::{Address, KeyPair};

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
        keypairs.keys().map(|k| k.to_string()).collect::<Vec<_>>().join(", ")
    );

    // true except first block and first producer
    let mut sync_check_required = false;
    let mut mempool: IndexMap<H256, IndexedTransaction> = IndexMap::new();
    let mut incoming_transaction_rx = ctx.incoming_transaction_tx.subscribe();

    loop {
        if sync_check_required {
            let manager = ctx.manager.read().unwrap();

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
                Ok(txn) = incoming_transaction_rx.recv() => {
                    if mempool.contains_key(&txn.hash) {
                        warn!("got duplicated transaction => {:?}", txn.hash);
                    } else {
                        info!("new txn => {:?}", txn.hash);
                        mempool.insert(txn.hash, txn);
                    }
                }
                _ = sleep(Duration::from_millis(d as u64)) => {
                    // produceBlock
                    let mut manager = ctx.manager.write().unwrap();

                    let slot = manager.get_slot(Utc::now().timestamp_millis() + 50);
                    if slot == 0 {
                        // NOT_TIME_YET
                        debug!("💤not time yet, skip slots in maintenance");
                        continue;
                    }
                    let block_timestamp = manager.get_slot_timestamp(slot);
                    let block_number = manager.latest_block_number() + 1;

                    match block_number {
                        1 => {
                            info!("👀generating block #1 without sync check");
                            // FIXME: should choose one from genesis config
                            let (witness_address, keypair) = keypairs.iter().next().unwrap();
                            let new_block =
                                manager.generate_empty_block(block_timestamp, witness_address, keypair).unwrap();
                            ctx.chain_db.insert_block(&new_block).expect("TODO: handle insert_block error");
                            ctx.chain_db.update_block_height(new_block.number());
                            info!("=> {:?}", new_block.hash());
                            info!("=> produce {:?}", manager.push_generated_block(&new_block));
                            info!("block pushed");
                        }
                        _ if block_number > 1 => {
                            let witness_address = manager.get_scheduled_witness(slot);
                            if let Some(keypair) = keypairs.get(&witness_address) {
                                info!("👀producing block #{} slot={} timestamp={} with {}", block_number, slot, block_timestamp, witness_address);

                                let deadline = block_timestamp + constants::BLOCK_PRODUCING_INTERVAL / 2 *
                                    constants::BLOCK_PRODUCE_TIMEOUT_PERCENT / 100;

                                trace!("deadline {}", deadline);
                                let new_block = manager.generate_and_push_block(mempool.values(), block_number, block_timestamp, deadline, &witness_address, keypair).unwrap();
                                ctx.chain_db.insert_block(&new_block).expect("TODO: handle insert_block error");
                                ctx.chain_db.update_block_height(new_block.number());
                                // info!("block #{} => {:?} txns={}", new_block.number(), new_block.hash(), new_block.transactions.len());

                                for txn in new_block.transactions.iter() {
                                    mempool.remove(&txn.hash);
                                }

                            } else {
                                info!("💤not my turn, pass");
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
