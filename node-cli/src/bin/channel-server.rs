use chrono::Utc;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use node_cli::channel::{ChannelMessage, ChannelMessageCodec};
use proto2::chain::Block;
use proto2::channel::{
    BlockInventory, ChainInventory, HandshakeDisconnect, HandshakeHello, Inventory, ReasonCode as DisconnectReasonCode,
    Transactions,
};
use proto2::common::{BlockId, Endpoint};
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

const MY_IP: &str = "0.0.0.0";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "0.0.0.0:18888";
    let mut listener = TcpListener::bind(addr).await?;

    let server = async move {
        let mut incoming = listener.incoming();
        while let Some(conn) = incoming.next().await {
            match conn {
                Err(e) => eprintln!("accept failed = {:?}", e),
                Ok(sock) => {
                    tokio::spawn(async move {
                        if let Err(e) = incoming_handshake_handler(sock).await {
                            eprintln!("handler error: {:?}", e);
                        }
                    });
                }
            }
        }
    };
    println!("server running on {}", addr);
    server.await;

    Ok(())
}

async fn incoming_handshake_handler(sock: TcpStream) -> Result<(), Box<dyn Error>> {
    let peer_addr = sock.peer_addr()?;
    println!("! accepted connection from {}", peer_addr);

    // let (mut reader, mut writer) = sock.split();

    let mut transport = ChannelMessageCodec::new_framed(sock);

    while let Some(payload) = transport.next().await {
        println!("! <= {} {:?}", peer_addr, payload);
        match payload {
            Ok(ChannelMessage::HandshakeHello(HandshakeHello {
                version,
                genesis_block_id,
                solid_block_id,
                head_block_id,
                ..
            })) => {
                let hello_reply = HandshakeHello {
                    from: Some(Endpoint {
                        address: MY_IP.into(),
                        port: 18888,
                        node_id: b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC".to_vec(),
                    }),
                    timestamp: Utc::now().timestamp_millis(),
                    version,
                    genesis_block_id,
                    solid_block_id,
                    head_block_id,
                    ..Default::default()
                };
                transport.send(ChannelMessage::HandshakeHello(hello_reply)).await?;
                return channel_handler(peer_addr, transport).await;
            }
            Err(e) => {
                println!("! error {} {:?}", peer_addr, e);
                return Err(e).map_err(From::from);
            }
            _ => {
                println!("unhandled got => {:?}", payload);
            }
        }
    }

    Ok(())
}

async fn channel_handler(
    peer_addr: ::std::net::SocketAddr,
    mut transport: Framed<TcpStream, ChannelMessageCodec>,
) -> Result<(), Box<dyn Error>> {
    while let Some(payload) = transport.next().await {
        println!("! <= {} {:?}", peer_addr, payload);
        match payload {
            Ok(ChannelMessage::Ping) => {
                transport.send(ChannelMessage::Pong).await?;
            }
            Ok(ChannelMessage::TransactionInventory(inv)) => {
                // |ids| = 1, 2
                let Inventory { ids, .. } = inv;
                for id in &ids {
                    println!("  ! txn id = {:?}", hex::encode(&id));
                }
            }
            Ok(ChannelMessage::BlockInventory(inv)) => {
                let Inventory { ref ids, .. } = inv;
                for id in ids {
                    println!("  ! blk id = {:?}", hex::encode(&id));
                }
                println!("  ! fetch block inventory");
                transport.send(ChannelMessage::FetchBlockInventory(inv)).await?;
            }

            Ok(ChannelMessage::Block(block)) => {
                println!("  ! block => {}", &block);
            }
            Ok(ChannelMessage::HandshakeDisconnect(HandshakeDisconnect { reason })) => {
                println!("  ! disconnect = {:?}", DisconnectReasonCode::from_i32(reason));
                return Ok(());
            }
            Err(e) => {
                println!("! error {} {:?}", peer_addr, e);
                return Err(e).map_err(From::from);
            }
            _ => {
                println!("unhandled => {:?}", payload);
            }
        }
    }
    Ok(())
}
