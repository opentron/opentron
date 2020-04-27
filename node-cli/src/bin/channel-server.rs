use chrono::Utc;
use futures::sink::{Sink, SinkExt};
use futures::stream::Stream;
// use futures::stream::StreamExt;
use node_cli::channel::{ChannelMessage, ChannelMessageCodec};
use node_cli::config::Config;
use node_cli::genesis::GenesisConfig;
use node_cli::util::get_my_ip;
use proto2::chain::Block;
use proto2::channel::{
    BlockInventory, ChainInventory, HandshakeDisconnect, HandshakeHello, Inventory, ReasonCode as DisconnectReasonCode,
    Transactions,
};
use proto2::common::{BlockId, Endpoint};
// use slog::{debug, error, info, o, warn, Drain};
use log::{debug, error, info, warn};
use slog::{o, slog_info, Drain};
use slog_scope_futures::FutureExt;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
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

pub struct PeerConnection {}

pub struct AppContext {
    inbound_ip: Option<SocketAddr>,
    genesis_block_id: BlockId,
    config: Config,
}

impl AppContext {
    pub fn from_config<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let config = Config::load_from_file(path)?;

        if config.protocol.channel.advertised_endpoint.is_empty() {

        }


        let genesis_config = GenesisConfig::load_from_file(&config.chain.genesis)?;
        let genesis_blk = genesis_config.to_indexed_block()?;

        let genesis_block_id = BlockId {
            number: 0,
            hash: genesis_blk.header.hash.as_ref().to_owned(),
        };

        info!("genesis block id => {:?}", hex::encode(&genesis_block_id.hash));
        Ok(AppContext {
            inbound_ip: None,
            genesis_block_id: genesis_block_id,
            config: config,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = slog::LevelFilter(drain, slog::Level::Info).fuse();

    let logger = slog::Logger::root(drain, o!());

    let _scope_guard = slog_scope::set_global_logger(logger);
    let _log_guard = slog_stdlog::init().unwrap();

    let ctx = AppContext::from_config("./conf.toml")?;
    let ctx = Arc::new(ctx);

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
                                let _ = incoming_handshake_handler(ctx, sock).with_logger(logger).await;
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
                    let _ = incoming_handshake_handler(ctx, sock).with_logger(logger).await;
                }
                Err(e) => {
                    warn!("connect {} failed: {}", peer_addr, e);
                }
            }
        });
    });

    server.await;
    Ok(())
}

async fn incoming_handshake_handler(ctx: Arc<AppContext>, mut sock: TcpStream) -> Result<(), Box<dyn Error>> {
    info!("connected");

    let (reader, writer) = sock.split();

    let mut reader = ChannelMessageCodec::new_read(reader);
    let mut writer = ChannelMessageCodec::new_write(writer);
    // let mut transport = ChannelMessageCodec::new_framed(sock); //.timeout(Duration::from_secs(10));

    let p2p_version = ctx.config.chain.p2p_version;
    //
    let hello = HandshakeHello {
        from: Some(Endpoint {
            address: MY_IP.into(),
            port: 18888,
            node_id: b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC".to_vec(),
        }),
        version: p2p_version,
        timestamp: Utc::now().timestamp_millis(),
        genesis_block_id: Some(ctx.genesis_block_id.clone()),
        head_block_id: Some(ctx.genesis_block_id.clone()),
        solid_block_id: Some(ctx.genesis_block_id.clone()),
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
                genesis_block_id,
                head_block_id,
                solid_block_id,
                ..
            })) => {
                slog_info!(slog_scope::logger(), "handshake";
                    "version" => version,
                    "genesis_block" => hex::encode(&genesis_block_id.as_ref().unwrap().hash),
                    "head_block" => head_block_id.as_ref().unwrap().number,
                );
                let _hello_reply = HandshakeHello {
                    from: Some(Endpoint {
                        address: MY_IP.into(),
                        port: 18888,
                        node_id: b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC".to_vec(),
                    }),
                    timestamp: Utc::now().timestamp_millis(),
                    version,
                    genesis_block_id: genesis_block_id.clone(),
                    solid_block_id: solid_block_id.clone(),
                    head_block_id: head_block_id.clone(),
                    ..Default::default()
                };
                // writer.send(ChannelMessage::HandshakeHello(hello_reply)).await?;
                info!("handshake finished");
                let logger = slog_scope::logger().new(o!(
                    "protocol" => "channel"
                ));
                let ret = channel_handler(reader, writer).with_logger(logger).await;
                info!("channel finished, return={}", format!("{:?}", ret));
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

async fn channel_handler(
    reader: impl Stream<Item = Result<ChannelMessage, io::Error>> + Unpin,
    mut writer: impl Sink<ChannelMessage, Error = io::Error> + Unpin,
) -> Result<(), Box<dyn Error>> {
    let mut reader = reader.timeout(Duration::from_secs(20));

    while let Some(payload) = reader.next().await {
        debug!("receive message, payload={}", format!("{:?}", payload));
        if payload.is_err() {
            error!("timeout");
            return Ok(());
        }
        match payload.unwrap() {
            Ok(ChannelMessage::Ping) => {
                info!("ping");
                writer.send(ChannelMessage::Pong).await?;
            }
            Ok(ChannelMessage::TransactionInventory(inv)) => {
                let Inventory { ids, .. } = inv;
                for id in &ids {
                    info!("transaction inventory, txn_id={}", hex::encode(&id));
                }
                // warn!(logger, "will disconnect");
                let _disconn = HandshakeDisconnect {
                    reason: DisconnectReasonCode::UserReason as _,
                };
                // writer.send(disconn.into()).await?;
            }
            Ok(ChannelMessage::BlockInventory(inv)) => {
                let Inventory { ref ids, .. } = inv;
                for id in ids {
                    info!("block inventory, blk_id={}", hex::encode(&id));
                }
                writer.send(ChannelMessage::FetchBlockInventory(inv)).await?;
            }

            Ok(ChannelMessage::Block(block)) => {
                info!("receive block, block={}", block.to_string());
            }
            Ok(ChannelMessage::HandshakeDisconnect(HandshakeDisconnect { reason })) => {
                warn!(
                    "disconnect, reason={}",
                    DisconnectReasonCode::from_i32(reason).unwrap().to_string()
                );
                return Ok(());
            }
            Err(e) => {
                error!("{:?}", e);
                return Err(e).map_err(From::from);
            }
            Ok(msg) => {
                error!("unhandled => {:?}", msg);
            }
        }
    }
    Ok(())
}
