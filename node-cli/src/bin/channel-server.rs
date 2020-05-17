#![recursion_limit = "1024"]

extern crate node_cli;

use chrono::Utc;
use futures::future::FutureExt;
use futures::sink::{Sink, SinkExt};
use futures::stream::Stream;
// use futures::stream::StreamExt;
use chain::IndexedBlock;
use futures::channel::oneshot;
use futures::select;
use log::{debug, error, info, warn};
use node_cli::channel::{ChannelMessage, ChannelMessageCodec};
use node_cli::config::Config;
use node_cli::db::ChainDB;
use node_cli::genesis::GenesisConfig;
use node_cli::util::get_my_ip;
use primitives::H256;
use proto2::channel::{
    BlockInventory, ChainInventory, HandshakeDisconnect, HandshakeHello, Inventory, ReasonCode as DisconnectReasonCode,
    ReasonCode,
};
use proto2::common::{BlockId, Endpoint};
use slog::{o, slog_info, Drain};
use slog_scope_futures::FutureExt as SlogFutureExt;
use std::collections::HashSet;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
// use tokio::sync::oneshot;
use tokio::time::Duration;
use tokio::time::{delay_for, timeout};
// use slog::{debug, error, info, o, warn, Drain};

// use tokio_util::codec::{Framed, FramedRead, FramedWrite};
// const P2P_VERSION: i32 = 11111;

const NODE_ID: &[u8] = b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAF0";

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

/*
pub struct PeerConnectionContext {
    peer_addr: SocketAddr,
    done: oneshot::Receiver<()>,
    syncing: RwLock<bool>,
}
*/

pub struct AppContext {
    outbound_ip: String,
    genesis_block_id: Option<BlockId>,
    config: Config,
    db: ChainDB,
    running: Arc<AtomicBool>,
    recent_blk_ids: RwLock<HashSet<H256>>,
    syncing: RwLock<bool>,
    peers: RwLock<Vec<oneshot::Sender<()>>>,
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
            info!("genesis block check passed");
        }
        db.report_status();

        let genesis_block_id = BlockId {
            number: 0,
            hash: genesis_blk.header.hash.as_ref().to_owned(),
        };

        info!("version => {}", config.chain.p2p_version);
        info!("genesis block id => {}", hex::encode(&genesis_block_id.hash));
        Ok(AppContext {
            db,
            config,
            outbound_ip: String::new(),
            genesis_block_id: Some(genesis_block_id),
            running: Arc::new(AtomicBool::new(true)),
            recent_blk_ids: RwLock::new(HashSet::new()),
            syncing: RwLock::new(true),
            peers: RwLock::default(),
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use tokio::runtime::Builder;

    // ! init loggers
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    // let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = slog::LevelFilter(drain, slog::Level::Info).fuse();

    let logger = slog::Logger::root(drain, o!());

    let _scope_guard = slog_scope::set_global_logger(logger);
    let _log_guard = slog_stdlog::init().unwrap();

    // ! original #[tokio::main] runner
    let fut = tokio_main();
    let mut rt = Builder::new().basic_scheduler().enable_all().build()?;
    rt.block_on(fut)
}

// NOTE: #[tokio::main] conflicts with slog_scope, cause data race in global static resource release.
async fn tokio_main() -> Result<(), Box<dyn Error>> {
    let mut ctx = AppContext::from_config("./conf.toml")?;
    ctx.outbound_ip = get_my_ip().await?;
    info!("outbound ip address: {}", ctx.outbound_ip);
    let ctx = Arc::new(ctx);

    // Fix: account state root First appares in 8222293

    /*
    for num in 1102553..1135973 {
        if let Some(blk) = ctx.db.get_block_by_number(num) {
            println!("delete {} => {:?}", num, ctx.db.delete_block(&blk));
        } else {
            println!("done");
            return Ok(());
        }
    }
    */

    // FIX gap
    // ctx.db.get_block_by_number(2999)
    // ctx.db.force_update_block_height(2998);

    let my_addr = &ctx.config.protocol.channel.endpoint;

    // passive connections
    let mut listener = TcpListener::bind(my_addr).await?;
    info!("server start listening at {}", listener.local_addr().unwrap());
    let (server_tx, rx) = oneshot::channel::<()>();
    let server = {
        let ctx = ctx.clone();
        async move {
            let mut incoming = listener.incoming();
            let mut rx_fut = rx.fuse();
            loop {
                let mut incoming = incoming.next().fuse();
                select! {
                    conn = incoming => {
                        match conn {
                            Some(Ok(sock)) => {
                                let ctx = ctx.clone();
                                tokio::spawn(async move {
                                    let _ = handshake_handler(ctx, sock).await;
                                });
                            },
                            Some(Err(e)) => error!("accept failed = {:?}", e),
                            None => {
                                info!("listener done");
                                break;
                            }
                        }
                    },
                    _ = rx_fut => {
                        warn!("passive connection service closed");
                        break;
                    }
                }
            }
        }
    };

    let (termination_tx, termination_done) = oneshot::channel::<()>();
    let termination_handler = {
        let ctx = ctx.clone();
        move || {
            let _ = server_tx.send(());
            while let Some(done) = ctx.peers.write().unwrap().pop() {
                let _ = done.send(());
            }
            ctx.running.store(false, Ordering::SeqCst);
            ctx.db.report_status();
            unsafe {
                ctx.db.prepare_close();
            }
            let _ = termination_tx.send(());
        }
    };

    let f = Mutex::new(Some(termination_handler));
    ctrlc::set_handler(move || {
        eprintln!("\nCtrl-C pressed...");
        if let Ok(mut guard) = f.lock() {
            let f = guard.take().expect("f can only be taken once");
            f();
        }
    })
    .expect("Error setting Ctrl-C handler");

    // active coonections
    let active_nodes = ctx.config.protocol.channel.active_nodes.clone();
    let active_service = tokio::spawn(async move {
        for peer_addr in active_nodes.into_iter().cycle() {
            if !ctx.running.load(Ordering::Relaxed) {
                warn!("active connection service closed");
                break;
            }
            info!("active connection to {}", peer_addr);
            let ctx = ctx.clone();
            match TcpStream::connect(&peer_addr).await {
                Ok(sock) => {
                    let _ = handshake_handler(ctx, sock).await;
                }
                Err(e) => {
                    warn!("connect {} failed: {}", peer_addr, e);
                }
            }
        }
    });

    server.await;
    let _ = active_service.await;

    Ok(termination_done.await?)
}

async fn handshake_handler(ctx: Arc<AppContext>, sock: TcpStream) -> Result<(), Box<dyn Error>> {
    let peer_addr = sock.peer_addr()?;
    let logger = slog_scope::logger().new(o!(
        "peer_addr" => peer_addr,
    ));
    inner_handshake_handler(ctx, sock).with_logger(logger).await
}

async fn inner_handshake_handler(ctx: Arc<AppContext>, mut sock: TcpStream) -> Result<(), Box<dyn Error>> {
    let (reader, writer) = sock.split();

    let mut reader = ChannelMessageCodec::new_read(reader);
    let mut writer = ChannelMessageCodec::new_write(writer);

    let p2p_version = ctx.config.chain.p2p_version;

    let channel_conf = &ctx.config.protocol.channel;
    let advertised_endpoint = channel_conf
        .advertised_endpoint
        .parse::<SocketAddr>()
        .map(|addr| Endpoint {
            address: addr.ip().to_string(),
            port: addr.port() as _,
            node_id: NODE_ID.to_vec(),
        })
        .unwrap_or_else(|_| Endpoint {
            address: ctx.outbound_ip.clone(),
            port: channel_conf
                .endpoint
                .parse::<SocketAddr>()
                .map(|addr| addr.port())
                .unwrap_or(18888) as _,
            node_id: NODE_ID.to_vec(),
        });

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
        from: Some(advertised_endpoint),
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
                solid_block_id: _peer_solid_block_id,
                ..
            })) => {
                slog_info!(slog_scope::logger(), "handshake";
                    "version" => version,
                    "genesis_block" => hex::encode(&peer_genesis_block_id.as_ref().unwrap().hash),
                    "head_block" => peer_head_block_id.as_ref().unwrap().number,
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
    mut reader: impl Stream<Item = Result<ChannelMessage, io::Error>> + Unpin,
    mut writer: impl Sink<ChannelMessage, Error = io::Error> + Unpin,
) -> Result<(), Box<dyn Error>> {
    let mut done = {
        let mut peers = ctx.peers.write().unwrap();
        let (tx, rx) = oneshot::channel::<()>();
        peers.push(tx);
        rx.fuse()
    };

    let highest_block = ctx.db.get_block_by_number(ctx.db.get_block_height() as u64);
    let highest_block_id = highest_block
        .as_ref()
        .map(|blk| blk.block_id())
        .unwrap_or(ctx.genesis_block_id.clone().unwrap());

    let mut last_block_id = highest_block_id.number;

    if *ctx.syncing.read().unwrap() {
        info!("sync block from {}", highest_block_id);
        let inv = BlockInventory {
            ids: vec![highest_block_id],
            ..Default::default()
        };

        writer.send(ChannelMessage::SyncBlockchain(inv)).await?;
    }

    let mut syncing_block_ids: Vec<Vec<u8>> = vec![];

    loop {
        let mut next_packet = reader.next().fuse();
        let mut timeout = delay_for(Duration::from_secs(20)).fuse();
        select! {
            _ = timeout => {
                warn!("timeout");
                break
            }
            _ = done => {
                warn!("close channel connection");
                break;
            }
            payload = next_packet => {
                if payload.is_none() {
                    warn!("connection closed");
                    break;
                }
                let payload = payload.unwrap();
                debug!("receive message, payload={}", format!("{:?}", payload));
                match payload {
                    Err(e) => {
                        error!("error disconnect, {:?}", e);
                        return Err(e).map_err(From::from);
                    },
                    Ok(ChannelMessage::HandshakeDisconnect(HandshakeDisconnect { reason })) => {
                        warn!(
                            "disconnect, reason={}",
                            DisconnectReasonCode::from_i32(reason).unwrap().to_string()
                        );
                        return Ok(());
                    },
                    Ok(ChannelMessage::Ping) => {
                        info!("ping");
                        writer.send(ChannelMessage::Pong).await?;
                    },
                    Ok(ChannelMessage::Pong) => {
                        info!("pong");
                    },
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
                        // || block.number() == 2999
                        if !ctx.db.has_block(&block)  {
                            ctx.db.insert_block(&block)?;
                            ctx.db.update_block_height(block.number());
                        } else {
                            warn!("block exists in db");
                        }
                        if *ctx.syncing.read().unwrap() {
                            if block.number() == last_block_id {
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
                    Ok(ChannelMessage::SyncBlockchain(blk_inv)) => {
                        info!("peer wants to sync blockchain: {:?}", blk_inv);
                        let BlockInventory { mut ids, .. } = blk_inv;
                        let last_block_id = ids[0].clone();
                        const SYNC_FETCH_BATCH_NUM: usize = 2000;
                        let block_ids: Vec<BlockId> = ctx
                            .db
                            .block_hashes_from(&last_block_id.hash, SYNC_FETCH_BATCH_NUM+1)
                            .into_iter()
                            .map(|block_hash| BlockId::from(block_hash))
                            .collect();
                        let remain_num = if block_ids.len() < SYNC_FETCH_BATCH_NUM {
                            block_ids.last().unwrap().number - last_block_id.number
                        } else {
                            ctx.db.get_block_height() - block_ids.last().unwrap().number
                        };
                        info!("block ids {}", block_ids.len());
                        info!("remain num {}", remain_num);
                        let chain_inv = ChainInventory {
                            ids: block_ids,
                            remain_num: remain_num,
                        };
                        // writer.send(ChannelMessage::BlockchainInventory(chain_inv)).await?
                    }
                    Ok(msg) => {
                        error!("unhandled message {:?}", msg);
                        return Ok(());
                    },
                }
            }
        }
    }

    Ok(())
}
