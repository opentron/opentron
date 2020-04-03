use chrono::Utc;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use node_cli::discovery::{DiscoveryMessage, DiscoveryMessageTransport};
use proto2::common::Endpoint;
use proto2::discovery::{FindPeers, Peers, Ping, Pong};
use rand::Rng;
use serde::Deserialize;
use std::error::Error;
use tokio::net::UdpSocket;

const P2P_VERSION: i32 = 11111;
// const P2P_VERSION: i32 = 1;

// seed list
const TO_IP: &str = "18.196.99.16";

#[derive(Deserialize)]
struct Ip {
    origin: String,
}

async fn get_my_ip() -> Result<String, Box<dyn Error>> {
    let ip = reqwest::get("http://httpbin.org/ip").await?.json::<Ip>().await?;
    Ok(ip.origin)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind("0.0.0.0:18888").await?;
    println!("! udp bind to sock => {:?}", socket);

    let my_ip = get_my_ip().await?;
    println!("! detect my ip {}", my_ip);

    let my_endpoint = Endpoint {
        address: my_ip.clone(),
        port: 18888,
        node_id: b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD".to_vec(),
    };

    let ping = Ping {
        from: Some(my_endpoint.clone()),
        to: Some(Endpoint {
            address: TO_IP.into(),
            port: 18888,
            node_id: vec![63u8; 64],
        }),
        version: P2P_VERSION,
        timestamp: Utc::now().timestamp_millis(),
    };

    let mut transport = DiscoveryMessageTransport::new(socket);

    transport
        .send((ping.into(), format!("{}:18888", TO_IP).parse().unwrap()))
        .await?;

    while let Some(payload) = transport.next().await {
        if let Ok((ref frame, ref peer_addr)) = payload {
            let addr = peer_addr.to_string(); // SocketAddr does not support width format
            println!("! <= {:^24} {:?}", addr, frame);
        }

        match payload {
            Ok((DiscoveryMessage::Ping(ping), peer_addr)) => {
                if ping.version != P2P_VERSION {
                    eprintln!("  ! <= {} version mismatch: version = {}", peer_addr, ping.version);
                    continue;
                }
                let pong = Pong {
                    from: Some(my_endpoint.clone()),
                    timestamp: Utc::now().timestamp_millis(),
                    echo_version: P2P_VERSION,
                };
                transport.send((pong.into(), peer_addr)).await?;
                println!("  => Pong");

                let mut rng = rand::thread_rng();
                let mut random_id = vec![0u8; 32];
                rng.fill(&mut random_id[..]);

                println!("  => FindPeers target={}", hex::encode(&random_id));

                let find = FindPeers {
                    from: Some(my_endpoint.clone()),
                    timestamp: Utc::now().timestamp_millis(),
                    target_id: random_id,
                };
                transport.send((find.into(), peer_addr)).await?;
            }
            Ok((DiscoveryMessage::FindPeers(_find), peer_addr)) => {
                let peers = Peers {
                    from: Some(my_endpoint.clone()),
                    timestamp: Utc::now().timestamp_millis(),
                    peers: vec![my_endpoint.clone()],
                };
                transport.send((peers.into(), peer_addr)).await?;
                // println!("  => Peers");
            }
            Ok((DiscoveryMessage::Peers(peers), _)) => {
                for peer in &peers.peers {
                    if peer.address == my_ip {
                        println!("    my ip, ignore");
                        continue;
                    }
                    let peer_addr = format!("{}:{}", peer.address, peer.port).parse().unwrap();
                    println!("  => ping peer {}", peer_addr);
                    let ping = Ping {
                        from: Some(my_endpoint.clone()),
                        to: Some(Endpoint {
                            address: peer.address.clone(),
                            port: peer.port,
                            node_id: vec![63u8; 64],
                        }),
                        version: P2P_VERSION,
                        timestamp: Utc::now().timestamp_millis(),
                    };
                    transport.send((ping.into(), peer_addr)).await?;
                }
            }
            Ok((DiscoveryMessage::Pong(_pong), _peer_addr)) => {}
            Err(e) => {
                eprintln!("error: {:?}", e);
                return Err(e).map_err(From::from);
            }
        }
    }
    Ok(())
}
