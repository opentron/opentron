use super::protocol::{ChannelMessage, ChannelMessageCodec};
use chain::IndexedBlock;
use chrono::Utc;
use futures::channel::oneshot;
use futures::future::FutureExt;
use futures::join;
use futures::select;
use futures::sink::{Sink, SinkExt};
use futures::stream::Stream;
use log::{debug, error, info, warn};
use primitives::H256;
use proto2::channel::{
    BlockInventory, ChainInventory, HandshakeDisconnect, HandshakeHello, Inventory, ReasonCode as DisconnectReasonCode,
};
use proto2::common::{BlockId, Endpoint};
use slog::{o, slog_info};
use slog_scope_futures::FutureExt as SlogFutureExt;
use std::error::Error;
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio::time::Duration;
use tokio::time::{delay_for, timeout};

use crate::context::AppContext;
use crate::util::block_hash_to_number;

pub async fn channel_server<F>(ctx: Arc<AppContext>, signal: F) -> Result<(), Box<dyn Error>>
where
    F: Future<Output = ()> + Unpin,
{
    let config = &ctx.config.protocol.channel;

    if !config.enable {
        warn!("channel service disabled");
        return Ok(());
    }

    let incomming_service = {
        let ctx = ctx.clone();
        let logger = slog_scope::logger().new(o!("direction" => "incomming"));
        passive_channel_service(ctx, signal).with_logger(logger)
    };

    let outgoing_wervice = {
        let ctx = ctx.clone();
        let logger = slog_scope::logger().new(o!("direction" => "outgoing"));
        active_channel_service(ctx).with_logger(logger)
    };

    let _ = join!(incomming_service, outgoing_wervice);

    Ok(())
}

async fn passive_channel_service<F>(ctx: Arc<AppContext>, signal: F) -> Result<(), Box<dyn Error>>
where
    F: Future<Output = ()> + Unpin,
{
    let config = &ctx.config.protocol.channel;
    if !config.enable_passive {
        warn!("passive channel service disabled");
        return Ok(());
    }

    let listening_addr = &ctx.config.protocol.channel.endpoint;

    // passive connections
    let mut listener = TcpListener::bind(listening_addr).await?;
    let server = {
        let ctx = ctx.clone();
        async move {
            info!("listening on grpc://{}", listener.local_addr().unwrap());
            let mut incoming = listener.incoming();
            let mut signal = signal.fuse();
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
                    _ = signal => {
                        warn!("incoming connection service closed");
                        break;
                    }
                }
            }
        }
    };
    server.await;
    Ok(())
}

async fn active_channel_service(ctx: Arc<AppContext>) -> Result<(), Box<dyn Error>> {
    let config = &ctx.config.protocol.channel;
    if !config.enable_active {
        warn!("active channel service disabled");
        return Ok(());
    }

    let active_service = {
        let ctx = ctx.clone();
        let active_nodes = ctx.config.protocol.channel.active_nodes.clone();
        tokio::spawn(async move {
            for peer_addr in active_nodes.into_iter().cycle() {
                if !ctx.running.load(Ordering::Relaxed) {
                    warn!("active connection service closed");
                    break;
                }
                ctx.db.await_background_jobs();
                if !ctx.running.load(Ordering::Relaxed) {
                    warn!("active connection service closed");
                    break;
                }
                info!("active connection to {}", peer_addr);
                let ctx = ctx.clone();
                if let Ok(conn) = timeout(Duration::from_secs(10), TcpStream::connect(&peer_addr)).await {
                    match conn {
                        Ok(sock) => {
                            let _ = handshake_handler(ctx, sock).await;
                        }
                        Err(e) => {
                            warn!("connect {} failed: {}", peer_addr, e);
                        }
                    }
                } else {
                    warn!("connect timeout");
                }
            }
        })
    };
    active_service.await?;
    Ok(())
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
            node_id: ctx.node_id.clone(),
        })
        .unwrap_or_else(|_| Endpoint {
            address: ctx.outbound_ip.clone(),
            port: channel_conf
                .endpoint
                .parse::<SocketAddr>()
                .map(|addr| addr.port())
                .unwrap_or(18888) as _,
            node_id: ctx.node_id.clone(),
        });

    let block_height = ctx.db.get_block_height();
    let head_block_id = ctx
        .db
        .get_block_by_number(block_height as u64)
        .map(|blk| blk.block_id())
        .ok();

    info!("my head block id {}", head_block_id.as_ref().unwrap());

    let _solid_block_id = if block_height > 27 {
        ctx.db
            .get_block_by_number(block_height as u64 - 20)
            .map(|blk| blk.block_id())
            .ok()
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

                if version != p2p_version {
                    writer
                        .send(ChannelMessage::disconnect_with_reason(
                            DisconnectReasonCode::IncompatibleVersion,
                        ))
                        .await?;
                    warn!("p2p version mismatch version={}, disconnect", version);
                    return Ok(());
                }
                if peer_genesis_block_id != ctx.genesis_block_id {
                    writer
                        .send(ChannelMessage::disconnect_with_reason(
                            DisconnectReasonCode::IncompatibleChain,
                        ))
                        .await?;
                    warn!("genesis block mismatch, disconnect");
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
    const BATCH: usize = 500;

    let mut done = {
        let mut peers = ctx.peers.write().unwrap();
        let (tx, rx) = oneshot::channel::<()>();
        peers.push(tx);
        rx.fuse()
    };

    let highest_block = ctx.db.get_block_by_number(ctx.db.get_block_height() as u64).ok();
    let highest_block_id = highest_block
        .as_ref()
        .map(|blk| blk.block_id())
        .unwrap_or(ctx.genesis_block_id.clone().unwrap());
    /*
    let highest_block_id = BlockId {
        number: 19822000,
        hash: hex::decode("00000000012e75b0c3dcb528f9bc31a43f7098d97b59f618387b958b4180bf8d").unwrap(),
    };
   */

    let mut last_block_number = highest_block_id.number;
    let mut last_block_number_in_this_batch = 0_i64;
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
                        last_block_number = last_blk_id.number;

                        let tail = if syncing_block_ids.len() >= BATCH {
                            syncing_block_ids.split_off(BATCH)
                        } else {
                            vec![]
                        };
                        if syncing_block_ids.is_empty() {
                            warn!("syning finished");
                            // remore: peer.setNeedSyncFromUs = false
                            *ctx.syncing.write().unwrap() = false;
                        } else {
                            last_block_number_in_this_batch = block_hash_to_number(syncing_block_ids.last().unwrap());
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
                            if block.number() == last_block_number {
                                ctx.db.report_status();
                                info!("sync next bulk of blocks from {}", block.number());
                                let inv = BlockInventory {
                                    ids: vec![block.block_id()],
                                    ..Default::default()
                                };
                                writer.send(ChannelMessage::SyncBlockchain(inv)).await?;
                            } else if block.number() == last_block_number_in_this_batch {
                                info!("sync next bulk of blocks from {} batch={}", block.number(), BATCH);
                                let tail = if syncing_block_ids.len() >= BATCH {
                                    syncing_block_ids.split_off(BATCH)
                                } else {
                                    vec![]
                                };

                                if !syncing_block_ids.is_empty() {
                                    last_block_number_in_this_batch = block_hash_to_number(syncing_block_ids.last().unwrap());

                                    let block_inv = Inventory {
                                        r#type: 1, // BLOCK
                                        ids: syncing_block_ids,
                                    };
                                    writer.send(ChannelMessage::FetchBlockInventory(block_inv)).await?;
                                    syncing_block_ids = tail;
                                }
                            }
                        }
                    }
                    Ok(ChannelMessage::SyncBlockchain(blk_inv)) => {
                        const SYNC_FETCH_BATCH_NUM: i64 = 2000;
                        let BlockInventory { mut ids, .. } = blk_inv;
                        info!("sync blockchain: {:?}", ids.iter().map(|blk_id| blk_id.number).collect::<Vec<_>>());
                        let unfork_id = ids.iter()
                            .rev()
                            .find(|blk_id| ctx.db.has_block_id(&H256::from_slice(&blk_id.hash)));

                        match unfork_id {
                            None => {
                                warn!("can not find an unfork id");
                                writer.send(
                                    ChannelMessage::disconnect_with_reason(DisconnectReasonCode::SyncFail))
                                .await?;
                                return Ok(());
                            }
                            Some(unfork_id) => {
                                info!("unfork id => {}", unfork_id);
                                let block_height = ctx.db.get_block_height();
                                let max_block_num = block_height.min(unfork_id.number + SYNC_FETCH_BATCH_NUM);
                                let reply_ids:Vec<BlockId> =
                                    ctx.db.block_hashes_from(
                                        &unfork_id.hash, (max_block_num - unfork_id.number) as usize + 1)
                                    .into_iter()
                                    .map(|block_hash| BlockId::from(block_hash))
                                    .collect();
                                let remain_num = block_height - reply_ids.last().unwrap().number;
                                info!("reply with remain_num = {}", remain_num);
                                info!("reply with ids = {}", reply_ids.len());
                                let chain_inv = ChainInventory {
                                    ids: reply_ids,
                                    remain_num: remain_num,
                                };
                                writer.send(ChannelMessage::BlockchainInventory(chain_inv)).await?
                            }
                        }
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
