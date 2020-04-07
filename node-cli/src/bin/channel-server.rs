use chrono::Utc;
use futures::sink::{Sink, SinkExt};
use futures::stream::Stream;
// use futures::stream::StreamExt;
use node_cli::channel::{ChannelMessage, ChannelMessageCodec};
use node_cli::config::Config;
use node_cli::util::get_my_ip;
use proto2::chain::Block;
use proto2::channel::{
    BlockInventory, ChainInventory, HandshakeDisconnect, HandshakeHello, Inventory, ReasonCode as DisconnectReasonCode,
    Transactions,
};
use proto2::common::{BlockId, Endpoint};
use slog::{debug, error, info, o, warn, Drain};
use std::error::Error;
use std::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio::time::timeout;
use tokio::time::Duration;
use std::net::SocketAddr;
// use tokio_util::codec::{Framed, FramedRead, FramedWrite};

const P2P_VERSION: i32 = 11111;

const MY_IP: &str = "0.0.0.0";

pub struct AppContext {
    inboud_ip: Option<SocketAddr>,
    genesis_block_id: BlockId,
    config: Config,
}

impl AppContext {

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = slog::LevelFilter(drain, slog::Level::Info).fuse();

    let logger = slog::Logger::root(drain, o!());

    let addr = "0.0.0.0:18888";
    let mut listener = TcpListener::bind(addr).await?;

    info!(logger, "server starts listening at {}", listener.local_addr().unwrap());

    let server = async move {
        let mut incoming = listener.incoming();
        while let Some(conn) = incoming.next().await {
            match conn {
                Err(e) => error!(logger, "accept failed = {:?}", e),
                Ok(sock) => match sock.peer_addr() {
                    Ok(peer_addr) => {
                        let logger = logger.new(o!("peer_addr" => peer_addr));
                        tokio::spawn(async move {
                            let _ = incoming_handshake_handler(logger, sock).await;
                        });
                    }
                    Err(e) => error!(logger, "accept failed = {:?}", e),
                },
            }
        }
    };
    server.await;
    Ok(())
}

async fn incoming_handshake_handler(logger: slog::Logger, mut sock: TcpStream) -> Result<(), Box<dyn Error>> {
    let peer_addr = sock.peer_addr()?;

    info!(logger, "accept connection");

    let (reader, writer) = sock.split();

    let mut reader = ChannelMessageCodec::new_read(reader);
    let mut writer = ChannelMessageCodec::new_write(writer);
    // let mut transport = ChannelMessageCodec::new_framed(sock); //.timeout(Duration::from_secs(10));

    //
    let hello = HandshakeHello {
        from: Some(Endpoint {
            address: MY_IP.into(),
            port: 18888,
            node_id: b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC".to_vec(),
        }),
        version: P2P_VERSION,
        timestamp: Utc::now().timestamp_millis(),
        ..Default::default()
    };

    while let Ok(payload) = timeout(Duration::from_secs(10), reader.next()).await {
        if payload.is_none() {
            warn!(logger, "empty payload");
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
                info!(logger, "handshake";
                    "version" => version,
                    "genesis_block" => hex::encode(&genesis_block_id.as_ref().unwrap().hash),
                    "head_block" => head_block_id.as_ref().unwrap().number,
                );
                let hello_reply = HandshakeHello {
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
                writer.send(ChannelMessage::HandshakeHello(hello_reply)).await?;
                info!(logger, "handshake finished");
                let ret = channel_handler(logger.clone(), peer_addr, reader, writer).await;
                info!(logger, "channel finished"; "return" => format!("{:?}", ret));
                return Ok(());
            }
            Err(e) => {
                error!(logger, "error: {:?}", e);
                return Ok(());
            }
            Ok(message) => {
                error!(logger, "unhandled message {:?}", &message);
                return Ok(());
            }
        }
    }

    warn!(logger, "disconnect");

    Ok(())
}

async fn channel_handler(
    logger: slog::Logger,
    peer_addr: ::std::net::SocketAddr,
    reader: impl Stream<Item = Result<ChannelMessage, io::Error>> + Unpin,
    mut writer: impl Sink<ChannelMessage, Error = io::Error> + Unpin,
) -> Result<(), Box<dyn Error>> {
    let mut reader = reader.timeout(Duration::from_secs(20));

    while let Some(payload) = reader.next().await {
        debug!(logger, "receive message"; "payload" => format!("{:?}", payload));
        if payload.is_err() {
            error!(logger, "timeout");
            return Ok(());
        }
        match payload.unwrap() {
            Ok(ChannelMessage::Ping) => {
                info!(logger, "ping");
                writer.send(ChannelMessage::Pong).await?;
            }
            Ok(ChannelMessage::TransactionInventory(inv)) => {
                // |ids| = 1, 2
                let Inventory { ids, .. } = inv;
                for id in &ids {
                    info!(logger, "transaction inventory"; "txn_id" => hex::encode(&id));
                }
                // warn!(logger, "will disconnect");
                let disconn = HandshakeDisconnect {
                    reason: DisconnectReasonCode::UserReason as _,
                };
                // writer.send(disconn.into()).await?;
            }
            Ok(ChannelMessage::BlockInventory(inv)) => {
                let Inventory { ref ids, .. } = inv;
                for id in ids {
                    info!(logger, "block inventory"; "blk_id" => hex::encode(&id));
                }
                //  println!("  ! fetch block inventory");
                writer.send(ChannelMessage::FetchBlockInventory(inv)).await?;
            }

            Ok(ChannelMessage::Block(block)) => {
                info!(logger, "receive block"; "block" => block.to_string());
                // println!("  ! block => {}", &block);
            }
            Ok(ChannelMessage::HandshakeDisconnect(HandshakeDisconnect { reason })) => {
                warn!(logger, "disconnect";
                    "reason" => DisconnectReasonCode::from_i32(reason).unwrap().to_string()
                );
                return Ok(());
            }
            Err(e) => {
                error!(logger, "{:?}", e);
                return Err(e).map_err(From::from);
            }
            Ok(msg) => {
                error!(logger, "unhandled => {:?}", msg);
            }
        }
    }
    Ok(())
}
