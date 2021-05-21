use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use byteorder::{ByteOrder, BE};
use chain::IndexedBlock;
use chrono::Utc;
use context::AppContext;
use futures::future::FutureExt;
use futures::join;
use futures::sink::{Sink, SinkExt};
use futures::stream::Stream;
use keys::b58encode_check;
use log::{debug, error, info, warn};
use primitive_types::H256;
use proto::channel::{
    BlockInventory, ChainInventory, HandshakeDisconnect, HandshakeHello, Inventory, ReasonCode as DisconnectReasonCode,
    Transactions,
};
use proto::common::{BlockId, Endpoint};
use slog::{o, slog_info, slog_warn};
use slog_scope_futures::FutureExt as SlogFutureExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio::time::{sleep, timeout};
use tokio_stream::StreamExt;

use crate::protocol::{ChannelMessage, ChannelMessageCodec};

pub async fn channel_server(ctx: Arc<AppContext>, signal: broadcast::Receiver<()>) -> Result<(), Box<dyn Error>> {
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

    let outgoing_service = {
        let ctx = ctx.clone();
        let logger = slog_scope::logger().new(o!("direction" => "outgoing"));
        active_channel_service(ctx).with_logger(logger)
    };

    let _ = join!(incomming_service, outgoing_service);

    Ok(())
}

async fn passive_channel_service(
    ctx: Arc<AppContext>,
    mut signal: broadcast::Receiver<()>,
) -> Result<(), Box<dyn Error>> {
    let config = &ctx.config.protocol.channel;
    if !config.enable_passive {
        warn!("passive channel service disabled");
        return Ok(());
    }

    let listening_addr = &ctx.config.protocol.channel.endpoint;
    let listener = TcpListener::bind(listening_addr).await?;
    let server = {
        let ctx = ctx.clone();
        async move {
            info!("listening on grpc://{}", listener.local_addr().unwrap());

            tokio::select! {
                _ = async {
                    loop {
                        let ctx = ctx.clone();
                        let (sock, peer_addr) = listener.accept().await?;
                        ctx.num_passive_connections.fetch_add(1, Ordering::SeqCst);
                        let logger = slog_scope::logger().new(o!(
                            "peer_addr" => peer_addr,
                        ));
                        tokio::spawn(async move {
                            let _ = handshake_handler(ctx.clone(), sock).with_logger(logger).await;
                            ctx.num_passive_connections.fetch_sub(1, Ordering::SeqCst);
                        });
                    }
                    // Help the rust type inferencer out
                    #[allow(unreachable_code)]
                    Ok::<_, io::Error>(())
                } => {}
                _ = signal.recv().fuse() => {
                    warn!("incoming connection service closed");
                },
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

    let max_active_connections = config.max_active_connections;

    let active_service = {
        let ctx = ctx.clone();
        let active_nodes = ctx.config.protocol.channel.active_nodes.clone();
        tokio::spawn(async move {
            for peer_addr in active_nodes.into_iter().cycle() {
                while ctx.num_active_connections.load(Ordering::SeqCst) >= max_active_connections {
                    sleep(Duration::from_secs(2)).await;
                }
                if !ctx.running.load(Ordering::Relaxed) {
                    warn!("active connection service closed");
                    break;
                }
                ctx.chain_db.await_background_jobs();
                if !ctx.running.load(Ordering::Relaxed) {
                    warn!("active connection service closed");
                    break;
                }
                info!("active connection to {}", peer_addr);
                let logger = slog_scope::logger().new(o!(
                    "peer_addr" => peer_addr.clone(),
                ));
                match timeout(Duration::from_secs(10), TcpStream::connect(&peer_addr)).await {
                    Err(_) => slog_warn!(logger, "connect timeout"),
                    Ok(Err(e)) => slog_warn!(logger, "connect failed: {}", e),
                    Ok(Ok(sock)) => {
                        ctx.num_active_connections.fetch_add(1, Ordering::SeqCst);
                        let ctx = ctx.clone();
                        tokio::spawn(async move {
                            let _ = handshake_handler(ctx.clone(), sock).with_logger(logger).await;
                            ctx.num_active_connections.fetch_sub(1, Ordering::SeqCst);
                        });
                    }
                }
            }
        })
    };
    active_service.await?;
    Ok(())
}

async fn handshake_handler(ctx: Arc<AppContext>, mut sock: TcpStream) -> Result<(), Box<dyn Error>> {
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

    let block_height = ctx.chain_db.get_block_height();
    let block_headers = ctx.chain_db.get_block_headers_by_number(block_height as u64);
    if block_headers.len() != 1 {
        panic!("TODO: should handle fork");
    }
    let head_block_id = Some(block_headers[0].block_id());

    info!("handshake with block id {}", head_block_id.as_ref().unwrap());

    let _solid_block_id = if block_height > 27 {
        ctx.chain_db
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
                slog_info!(slog_scope::logger(), "handshake request";
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

                // only syncing if remote >= local?
                let need_syncing =
                    peer_head_block_id.as_ref().unwrap().number >= head_block_id.as_ref().unwrap().number;

                info!("handshake finished, need sync = {}", need_syncing);
                let ret = sync_channel_handler(ctx, need_syncing, reader, writer).await;
                match ret {
                    Ok(_) => info!("channel finished"),
                    Err(e) => warn!("channel finished with error={:?}", e),
                }
                return Ok(());
            }
            Ok(ChannelMessage::HandshakeDisconnect(HandshakeDisconnect { reason })) => {
                warn!(
                    "disconnect in handshake, reason={}",
                    DisconnectReasonCode::from_i32(reason).unwrap_or(DisconnectReasonCode::Unknown)
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
    mut syncing: bool,
    mut reader: impl Stream<Item = Result<ChannelMessage, io::Error>> + Unpin,
    mut writer: impl Sink<ChannelMessage, Error = io::Error> + Unpin,
) -> Result<(), Box<dyn Error>> {
    let config = &ctx.config.protocol.channel;
    let batch_size = config.sync_batch_size;

    let highest_block = ctx
        .chain_db
        .get_block_by_number(ctx.chain_db.get_block_height() as u64)
        .ok();
    let highest_block_id = highest_block
        .as_ref()
        .map(|blk| blk.block_id())
        .unwrap_or(ctx.genesis_block_id.clone().unwrap());

    let mut last_block_number = highest_block_id.number;
    let mut last_block_number_in_this_batch = 0_i64;
    if syncing {
        info!("sync block from {}", highest_block_id);
        let inv = BlockInventory {
            ids: vec![highest_block_id],
            ..Default::default()
        };

        writer.send(ChannelMessage::SyncBlockchain(inv)).await?;
    }

    let mut syncing_block_ids: Vec<Vec<u8>> = vec![];
    let mut pinged = false;
    let (tx, mut rx) = mpsc::channel::<ChannelMessage>(1000);

    let mut done = ctx.termination_signal.subscribe();

    const READING_TIMEOUT: u64 = 18;
    loop {
        tokio::select! {
            sending_message = rx.recv().fuse() => {
                if let Some(msg) = sending_message {
                    writer.send(msg).await?;
                }
            }
            _ = done.recv() => {
                debug!("termination, close channel connection");
                return Ok(());
            }
            task = timeout(Duration::from_secs(READING_TIMEOUT), reader.next().fuse()) => {
                let payload = match task {
                    Err(_) if pinged => {
                        warn!("timeout");
                        return Ok(());
                    },
                    Err(_) => {
                        debug!("timeout, try pinging remote");
                        writer.send(ChannelMessage::Ping).await?;
                        pinged = true;
                        continue;
                    },
                    Ok(None) => {
                        warn!("connection closed");
                        return Ok(());
                    }
                    Ok(Some(payload)) => {
                        payload
                    }
                };

                debug!("receive message, payload={}", format!("{:?}", payload));
                match payload {
                    Err(e) => {
                        error!("error disconnect, {:?}", e);
                        return Err(e).map_err(From::from);
                    },
                    Ok(ChannelMessage::HandshakeDisconnect(HandshakeDisconnect { reason })) => {
                        warn!(
                            "disconnect, reason={}",
                            DisconnectReasonCode::from_i32(reason).unwrap_or(DisconnectReasonCode::Unknown)
                        );
                        return Ok(());
                    },
                    Ok(ChannelMessage::Ping) => {
                        info!("ping");
                        writer.send(ChannelMessage::Pong).await?;
                    },
                    Ok(ChannelMessage::Pong) => {
                        debug!("pong");
                    },
                    Ok(ChannelMessage::TransactionInventory(inv)) => {
                        let Inventory { ids, r#type: _ } = inv;
                        for id in &ids {
                            debug!("transaction inventory, txn_id={}", hex::encode(id));
                        }
                        /*
                        if ids.len() == 1 {
                            // ids[0][0] = b'A';
                            let fake_inv = Inventory { ids, r#type };
                            info!("request inv");
                            // writer.send(ChannelMessage::TransactionInventory(fake_inv)).await?;
                            writer.send(ChannelMessage::FetchBlockInventory(fake_inv)).await?;
                        }
                        */
                    }
                    Ok(ChannelMessage::FetchTransactionInventory(inv)) => {
                        info!("fetch transactions {:?}", inv);
                    }
                    Ok(ChannelMessage::Transactions(Transactions { transactions })) => {
                        for txn in &transactions {
                            info!("got txn {:?}", txn);
                        }
                    }
                    Ok(ChannelMessage::BlockInventory(inv)) => {
                        if syncing {
                            continue;
                        }
                        let Inventory { ids, r#type } = inv;
                        let ids: Vec<_> = ids
                            .into_iter()
                            .filter(|blk_id| {
                                if ctx.recent_block_ids.read().unwrap().contains(&H256::from_slice(blk_id)) {
                                    debug!("block inventory, number={}, skip for seen", block_hash_to_number(&blk_id));
                                    false
                                } else {
                                    debug!("block inventory, number={}, fetch", block_hash_to_number(&blk_id));
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
                        let start_block_num = chain_inv.ids[0].number;
                        syncing_block_ids = chain_inv.ids.iter().skip(1).map(|blk_id| blk_id.hash.clone()).collect();
                        let last_block_id = chain_inv.ids.pop().unwrap();

                        info!(
                            "👀chain inventory, {}..={}, remain={}",
                            start_block_num,
                            last_block_id.number,
                            chain_inv.remain_num);

                        last_block_number = last_block_id.number;

                        let tail = if syncing_block_ids.len() >= batch_size {
                            syncing_block_ids.split_off(batch_size)
                        } else {
                            vec![]
                        };
                        if syncing_block_ids.is_empty() {
                            info!("🎉syncing finished, entering gossip loop");
                            // remore: peer.setNeedSyncFromUs = false
                            syncing = false;
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
                        let block = IndexedBlock::from_raw(block).unwrap();
                        if !ctx.recent_block_ids.read().unwrap().contains(&block.header.hash) {
                            if syncing {
                                if block.number() % 100 == 0 {
                                    info!(
                                        "✨syncing progress: block number={} hash={} txns={}",
                                        block.number(),
                                        block.hash(),
                                        block.transactions.len(),
                                    );
                                }
                            } else {
                                info!(
                                    "📦receive block number={} hash={} txns={:<3} witness={}",
                                    block.number(),
                                    block.hash(),
                                    block.transactions.len(),
                                    b58encode_check(block.witness()),
                                );
                            }

                            ctx.recent_block_ids.write().unwrap().insert(block.header.hash);
                            if !ctx.chain_db.has_block(&block)  {
                                ctx.chain_db.insert_block(&block)?;
                                ctx.chain_db.update_block_height(block.number());
                            } else {
                                warn!("block exists in db");
                            }
                        }
                        if syncing {
                            if block.number() == last_block_number {
                                if block.number() % 2_000 == 0 {
                                    ctx.chain_db.report_status();
                                }
                                info!("👀sync next bulk of blocks from {}", block.number());
                                let inv = BlockInventory {
                                    ids: vec![block.block_id()],
                                    ..Default::default()
                                };
                                writer.send(ChannelMessage::SyncBlockchain(inv)).await?;
                            } else if block.number() == last_block_number_in_this_batch {
                                info!("👀sync next bulk of blocks from={} batch={}", block.number(), batch_size);
                                let tail = if syncing_block_ids.len() >= batch_size {
                                    syncing_block_ids.split_off(batch_size)
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
                    // handle remote sync
                    Ok(ChannelMessage::SyncBlockchain(blk_inv)) => {
                        const SYNC_FETCH_BATCH_NUM: i64 = 2000;
                        let BlockInventory { ids, .. } = blk_inv;
                        info!("sync request {:?}", ids.iter().map(|blk_id| blk_id.number).collect::<Vec<_>>());
                        let unfork_id = ids.iter()
                            .rev()
                            .find(|blk_id| ctx.chain_db.has_block_id(&H256::from_slice(&blk_id.hash)));

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
                                let block_height = ctx.chain_db.get_block_height();
                                let max_block_num = block_height.min(unfork_id.number + SYNC_FETCH_BATCH_NUM);
                                let reply_ids:Vec<BlockId> =
                                    ctx.chain_db.block_hashes_from(
                                        &unfork_id.hash, (max_block_num - unfork_id.number) as usize + 1)
                                    .into_iter()
                                    .map(|block_hash| BlockId::from(block_hash))
                                    .collect();
                                let remain_num = block_height - reply_ids.last().unwrap().number;
                                info!("reply with remain_num={} ids={}", remain_num, reply_ids.len());
                                let chain_inv = ChainInventory {
                                    ids: reply_ids,
                                    remain_num: remain_num,
                                };
                                writer.send(ChannelMessage::BlockchainInventory(chain_inv)).await?
                            }
                        }
                    }
                    Ok(ChannelMessage::FetchBlockInventory(Inventory { ids, .. })) => {
                        if ids.is_empty() {
                            info!("fetch block request with 0 ids??");
                        } else {
                            info!(
                                "fetch block request, start={}, end={}, len={}",
                                block_hash_to_number(ids.first().unwrap()),
                                block_hash_to_number(ids.last().unwrap()),
                                ids.len());
                        }
                        // NOTE: hard-coded 500, in javatron, this defaults to 100, maximum is 2000
                        if ids.len() > 500 {
                            warn!("reject malformed node");
                            writer.send(
                                ChannelMessage::disconnect_with_reason(DisconnectReasonCode::BadProtocol))
                            .await?;
                            return Ok(());
                        }
                        for id in ids.iter().map(|raw| H256::from_slice(&*raw)) {
                            let block = ctx.chain_db.get_block_by_id(&id)?;
                            tx.send(ChannelMessage::Block(block.into())).await?;
                        }
                        info!("sent {} blocks", ids.len());
                    }
                    Ok(msg) => {
                        error!("unhandled message {:?}", msg);
                        return Ok(());
                    },
                }
            }
        }
    }
}

#[inline]
pub fn block_hash_to_number(hash: &[u8]) -> i64 {
    BE::read_u64(&hash[..8]) as _
}
