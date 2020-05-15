extern crate node_cli;

use chrono::Utc;
use futures::sink::{Sink, SinkExt};
use futures::stream::Stream;
// use futures::stream::StreamExt;
use chain::IndexedBlock;
use node_cli::channel::{ChannelMessage, ChannelMessageCodec};
use node_cli::config::Config;
use node_cli::db::ChainDB;
use node_cli::genesis::GenesisConfig;
use node_cli::util::get_my_ip;
use proto2::chain::Block;
use proto2::channel::{
    BlockInventory, ChainInventory, HandshakeDisconnect, HandshakeHello, Inventory, ReasonCode as DisconnectReasonCode,
    ReasonCode, Transactions,
};
use proto2::common::{BlockId, Endpoint};
// use slog::{debug, error, info, o, warn, Drain};
use futures::channel::oneshot;
use futures::select;
use log::{debug, error, info, warn};
use primitives::H256;
use slog::{o, slog_info, Drain};
use slog_scope_futures::FutureExt;
use std::collections::HashSet;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio::time::timeout;
use tokio::time::Duration;

// use tokio_util::codec::{Framed, FramedRead, FramedWrite};
// const P2P_VERSION: i32 = 11111;

const MY_IP: &str = "0.0.0.0";

pub enum PeerStatus {
    HandshakeFinished,
    BeforeSyncing,
    Syncing,
    BackingUp,
}

pub enum Direction {
    Inbound,
    Outbound,
}

pub struct PeerConneContext {
    peer_addr: SocketAddr,
    done: oneshot::Receiver<()>,
    syncing: RwLock<bool>,
}

pub struct AppContext {
    inbound_ip: Option<SocketAddr>,
    genesis_block_id: Option<BlockId>,
    config: Config,
    db: ChainDB,
    running: Arc<AtomicBool>,
    recent_blk_ids: RwLock<HashSet<H256>>,
    syncing: RwLock<bool>,
}

impl AppContext {
    pub fn from_config<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let config = Config::load_from_file(path)?;

        let genesis_config = GenesisConfig::load_from_file(&config.chain.genesis)?;
        let genesis_blk = genesis_config.to_indexed_block()?;

        let db = ChainDB::new(&config.storage.data_dir);

        if !db.has_block(&genesis_blk) {
            if let Some(_) = db.get_genesis_block() {
                panic!("genesis block is inconsistent with db");
            }
            info!("insert genesis block to db");
            db.insert_block(&genesis_blk)?;
        } else {
            info!("genesis block already in db");
        }
        db.report_status();

        let genesis_block_id = BlockId {
            number: 0,
            hash: genesis_blk.header.hash.as_ref().to_owned(),
        };

        info!("genesis block id => {:?}", hex::encode(&genesis_block_id.hash));
        Ok(AppContext {
            inbound_ip: None,
            genesis_block_id: Some(genesis_block_id),
            config: config,
            db,
            running: Arc::new(AtomicBool::new(true)),
            recent_blk_ids: RwLock::new(HashSet::new()),
            syncing: RwLock::new(true),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    // let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = slog::LevelFilter(drain, slog::Level::Info).fuse();

    let logger = slog::Logger::root(drain, o!());

    let _scope_guard = slog_scope::set_global_logger(logger);
    let _log_guard = slog_stdlog::init().unwrap();

    let ctx = AppContext::from_config("./conf.toml")?;
    let ctx = Arc::new(ctx);

    // FIX gap
    // ctx.db.force_update_block_height(19722237);

    {
        let ctx = ctx.clone();
        ctrlc::set_handler(move || {
            eprintln!("\nCtrl-C pressed...");
            ctx.running.store(false, Ordering::SeqCst);
            ctx.db.report_status();
            unsafe { ctx.db.close() }
            std::process::exit(-1);
        })
        .expect("Error setting Ctrl-C handler");
    }

    let my_addr = &ctx.config.protocol.channel.endpoint;

    // passive connections
    let mut listener = TcpListener::bind(my_addr).await?;

    info!("server start listening at {}", listener.local_addr().unwrap());

    let server = {
        let ctx = ctx.clone();
        async move {
            let mut incoming = listener.incoming();
            while let Some(conn) = incoming.next().await {
                match conn {
                    Err(e) => error!("accept failed = {:?}", e),
                    Ok(sock) => match sock.peer_addr() {
                        Ok(peer_addr) => {
                            let logger = slog_scope::logger().new(o!(
                                "peer_addr" => peer_addr,
                                "connection" => "passive",
                            ));
                            let ctx = ctx.clone();
                            tokio::spawn(async move {
                                let _ = outgoing_handshake_handler(ctx, sock).with_logger(logger).await;
                            });
                        }
                        Err(e) => error!("accept failed = {:?}", e),
                    },
                }
            }
        }
    };

    // active coonections
    let passive_nodes = ctx.config.protocol.channel.passive_nodes.clone();

    tokio::spawn(async move {
        passive_nodes.into_iter().take(1).for_each(|peer_addr| {
            info!("active connection to {}", peer_addr);
            let ctx = ctx.clone();
            tokio::spawn(async move {
                match TcpStream::connect(&peer_addr).await {
                    Ok(sock) => {
                        let logger = slog_scope::logger().new(o!(
                        "peer_addr" => sock.peer_addr().unwrap(),
                        "connection" => "active",
                        ));
                        let _ = outgoing_handshake_handler(ctx, sock).with_logger(logger).await;
                    }
                    Err(e) => {
                        warn!("connect {} failed: {}", peer_addr, e);
                    }
                }
            });
        });
    });

    // Fix: account state root First appares in 8222293
    /*
    for num in 8222293..8230846 {
        if let Some(blk) = ctx.db.get_block_by_number(num) {
            println!("delete {} => {:?}", num, ctx.db.delete_block(&blk));
        } else {
            println!("done");
            return Ok(());
        }
    }
    */

    /*
    for peer_addr in passive_nodes.into_iter().cycle() {
        // ctx.db.await_background_jobs();

        info!("active connection to {}", peer_addr);
        let ctx = ctx.clone();
        match TcpStream::connect(&peer_addr).await {
            Ok(sock) => {
                let logger = slog_scope::logger().new(o!(
                    "peer_addr" => sock.peer_addr().unwrap(),
                    "connection" => "sync",
                ));
                let _ = incoming_handshake_handler(ctx, sock).with_logger(logger).await;
            }
            Err(e) => {
                warn!("connect {} failed: {}", peer_addr, e);
            }
        }
    }
    */

    server.await;
    Ok(())
}



async fn outgoing_handshake_handler(ctx: Arc<AppContext>, mut sock: TcpStream) -> Result<(), Box<dyn Error>> {
    info!("connected");

    let (reader, writer) = sock.split();

    let mut reader = ChannelMessageCodec::new_read(reader);
    let mut writer = ChannelMessageCodec::new_write(writer);

    let p2p_version = ctx.config.chain.p2p_version;

    let block_height = ctx.db.get_block_height();
    let head_block_id = ctx
        .db
        .get_block_by_number(block_height as u64)
        .map(|blk| blk.block_id());

    info!("my head block id {}", head_block_id.as_ref().unwrap());

    let _solid_block_id = if block_height > 27 {
        ctx.db
            .get_block_by_number(block_height as u64 - 20)
            .map(|blk| blk.block_id())
    } else {
        ctx.genesis_block_id.clone()
    };

    let hello = HandshakeHello {
        from: Some(Endpoint {
            address: MY_IP.into(),
            port: 18888,
            node_id: b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAF0".to_vec(),
        }),
        version: p2p_version,
        timestamp: Utc::now().timestamp_millis(),
        genesis_block_id: ctx.genesis_block_id.clone(),
        head_block_id: head_block_id.clone(),
        solid_block_id: ctx.genesis_block_id.clone(), // solid_block_id.clone(),
        ..Default::default()
    };

    writer.send(hello.into()).await?;

    while let Ok(payload) = timeout(Duration::from_secs(10), reader.next()).await {
        if payload.is_none() {
            warn!("empty payload");
            break;
        }

        match payload.unwrap() {
            Ok(ChannelMessage::HandshakeHello(HandshakeHello {
                version,
                genesis_block_id: peer_genesis_block_id,
                head_block_id: peer_head_block_id,
                solid_block_id: peer_solid_block_id,
                ..
            })) => {
                slog_info!(slog_scope::logger(), "handshake";
                    "version" => version,
                    "genesis_block" => hex::encode(&peer_genesis_block_id.as_ref().unwrap().hash),
                    "head_block" => head_block_id.as_ref().unwrap().number,
                );

                assert_eq!(version, p2p_version);
                if peer_genesis_block_id != ctx.genesis_block_id {
                    writer
                        .send(ChannelMessage::HandshakeDisconnect(HandshakeDisconnect {
                            reason: ReasonCode::IncompatibleChain as i32,
                        }))
                        .await?;
                    warn!("incompatible chain, disconnect");
                    return Ok(());
                }

                info!("handshake finished");
                let logger = slog_scope::logger().new(o!(
                    "protocol" => "channel"
                ));
                // let ret = channel_handler(ctx, reader, writer).with_logger(logger).await;
                let ret = sync_channel_handler(ctx, reader, writer).with_logger(logger).await;
                info!("channel finished, return={}", format!("{:?}", ret));
                return Ok(());
            }
            Ok(ChannelMessage::HandshakeDisconnect(HandshakeDisconnect { reason })) => {
                warn!(
                    "disconnect, reason={}",
                    DisconnectReasonCode::from_i32(reason).unwrap().to_string()
                );
                return Ok(());
            }
            Err(e) => {
                error!("error: {:?}", e);
                return Ok(());
            }
            Ok(message) => {
                error!("unhandled message {:?}", &message);
                return Ok(());
            }
        }
    }

    warn!("disconnect");

    Ok(())
}


async fn sync_channel_handler(
    ctx: Arc<AppContext>,
    reader: impl Stream<Item = Result<ChannelMessage, io::Error>> + Unpin,
    mut writer: impl Sink<ChannelMessage, Error = io::Error> + Unpin,
) -> Result<(), Box<dyn Error>> {
    /*
    let highest_block = ctx.db.highest_block(0);
    let highest_block_id = highest_block
        .as_ref()
        .map(|blk| blk.block_id())
        .unwrap_or(ctx.genesis_block_id.clone());
        */
    /*
    let highest_block_id = BlockId {
        number: 19700000,
        hash: hex::decode("00000000012c9920aff706bae73f632b72eb74deb557d44feed35dd48470fcf9").unwrap(),
    };*/
    // 19700001); //
    let highest_block = ctx.db.get_block_by_number(ctx.db.get_block_height() as u64);
    let highest_block_id = highest_block
        .as_ref()
        .map(|blk| blk.block_id())
        .unwrap_or(ctx.genesis_block_id.clone().unwrap());

    let start_block_number = highest_block_id.number;

    info!("sync block from {}", highest_block_id);

    // info!("delete => {:?}", ctx.db.delete_block(&highest_block.unwrap()));
    // return Ok(());

    if *ctx.syncing.read().unwrap() {
        let inv = BlockInventory {
            ids: vec![highest_block_id],
            ..Default::default()
        };

        writer.send(ChannelMessage::SyncBlockchain(inv)).await?;
    }

    let mut syncing_block_ids = vec![];
    let mut last_block_id = start_block_number;

    let mut reader = reader.timeout(Duration::from_secs(20));

    while let Some(payload) = reader.next().await {
        debug!("receive message, payload={}", format!("{:?}", payload));
        if payload.is_err() {
            error!("timeout");
            return Ok(());
        }
        match payload.unwrap() {
            Ok(ChannelMessage::Ping) => {
                info!("<= ping");
                writer.send(ChannelMessage::Pong).await?;
            }
            Ok(ChannelMessage::Pong) => {
                info!("<= pong");
            }
            Ok(ChannelMessage::TransactionInventory(_)) => {}
            Ok(ChannelMessage::BlockInventory(inv)) => {
                if *ctx.syncing.read().unwrap() {
                    continue;
                }
                let Inventory { ids, r#type } = inv;
                let ids: Vec<_> = ids
                    .into_iter()
                    .filter(|blk_id| {
                        info!("block inventory, blk_id={}", hex::encode(&blk_id));
                        if ctx.recent_blk_ids.read().unwrap().contains(&H256::from_slice(blk_id)) {
                            warn!("block in recent blocks, skip fetch");
                            false
                        } else {
                            true
                        }
                    })
                    .collect();
                if !ids.is_empty() {
                    writer
                        .send(ChannelMessage::FetchBlockInventory(Inventory { ids, r#type }))
                        .await?;
                }
            }

            Ok(ChannelMessage::BlockchainInventory(mut chain_inv)) => {
                info!("remains = {}", chain_inv.remain_num);
                info!("id[+0] = {}", chain_inv.ids[0]);
                syncing_block_ids = chain_inv.ids.iter().skip(1).map(|blk_id| blk_id.hash.clone()).collect();
                let last_blk_id = chain_inv.ids.pop().unwrap();
                info!("id[-1] = {}", last_blk_id);
                last_block_id = last_blk_id.number;

                // let inv = BlockInventory {
                // ids: vec![last_blk_id],
                // ..Default::default()
                // };
                // writer.send(ChannelMessage::SyncBlockchain(inv)).await?;
                let tail = if syncing_block_ids.len() >= 1000 {
                    syncing_block_ids.split_off(1000)
                } else {
                    vec![]
                };

                if syncing_block_ids.len() == 0 {
                    warn!("syning finished");
                    // remore: peer.setNeedSyncFromUs = false
                    *ctx.syncing.write().unwrap() = false;
                }

                let block_inv = Inventory {
                    r#type: 1, // BLOCK
                    ids: syncing_block_ids,
                };
                syncing_block_ids = tail;
                writer.send(ChannelMessage::FetchBlockInventory(block_inv)).await?;
            }

            Ok(ChannelMessage::Block(block)) => {
                if *ctx.syncing.read().unwrap() {
                    if block.number() % 100 == 0 {
                        info!("syncing {}", block.to_string());
                    }
                } else {
                    info!("receive {}", block.to_string());
                }

                let block = IndexedBlock::from_raw(block);
                if ctx.recent_blk_ids.read().unwrap().contains(&block.header.hash) {
                    warn!("block in recent blocks");
                    continue;
                }

                ctx.recent_blk_ids.write().unwrap().insert(block.header.hash);
                if !ctx.db.has_block(&block) {
                    ctx.db.insert_block(&block)?;
                    ctx.db.update_block_height(block.number());
                } else {
                    warn!("block exists in db");
                }

                if *ctx.syncing.read().unwrap() {
                    if block.number() == last_block_id {
                        /*
                        let highest_block = ctx.db.highest_block(start_block_number).unwrap();
                        let highest_block_id = BlockId {
                            number: highest_block.number() as _,
                            hash: highest_block.hash().as_bytes().to_vec(),
                        };*/

                        ctx.db.report_status();
                        info!("sync next bulk of blocks from {}", block.number());
                        let inv = BlockInventory {
                            ids: vec![block.block_id()],
                            ..Default::default()
                        };
                        writer.send(ChannelMessage::SyncBlockchain(inv)).await?;
                    } else if block.number() == last_block_id - 1000 {
                        info!("sync next bulk of blocks from {}", block.number());
                        if !syncing_block_ids.is_empty() {
                            let block_inv = Inventory {
                                r#type: 1, // BLOCK
                                ids: syncing_block_ids,
                            };
                            syncing_block_ids = vec![];
                            writer.send(ChannelMessage::FetchBlockInventory(block_inv)).await?;
                        }
                    }
                }
            }
            Ok(ChannelMessage::HandshakeDisconnect(HandshakeDisconnect { reason })) => {
                warn!(
                    "disconnect, reason={}",
                    DisconnectReasonCode::from_i32(reason).unwrap().to_string()
                );
                return Ok(());
            }
            Ok(ChannelMessage::SyncBlockchain(blk_inv)) => {
                info!("sync blockchain: {:?}", blk_inv);
                let BlockInventory { ids, .. } = blk_inv;
                let chain_inv = ChainInventory {
                    ids: ids,
                    remain_num: 20,
                };
                writer.send(ChannelMessage::BlockchainInventory(chain_inv)).await?
            }
            Err(e) => {
                error!("error disconnect, {:?}", e);
                return Err(e).map_err(From::from);
            }
            Ok(msg) => {
                error!("unhandled message => {:?}", msg);
                return Ok(());
            }
        }
    }
    Ok(())
}
