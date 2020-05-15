use chrono::Utc;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use node_cli::discovery::{DiscoveryMessage, DiscoveryMessageTransport};
use node_cli::util::{get_my_ip, Peer};
use proto2::common::Endpoint;
use proto2::discovery::{FindPeers, Peers, Ping, Pong};
use rand::Rng;
use std::collections::HashSet;
use std::error::Error;
use tokio::net::UdpSocket;

const P2P_VERSION: i32 = 11111;
// const P2P_VERSION: i32 = 1;

fn common_prefix_bits(a: &[u8], b: &[u8]) -> u32 {
    let mut acc = 0;
    for (&lhs, &rhs) in a.iter().zip(b.iter()) {
        if lhs != rhs {
            return acc + (lhs ^ rhs).leading_zeros();
        } else {
            acc += 8;
        }
    }
    acc
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind("0.0.0.0:18888").await?;
    println!("! udp bind to sock => {:?}", socket);

    let my_ip = get_my_ip().await?;
    println!("! detect my ip {}", my_ip);

    let mut peers_db: HashSet<Peer> = serde_json::from_str(&std::fs::read_to_string("./peers.json")?)?;

    let my_endpoint = Endpoint {
        address: my_ip.clone(),
        port: 18888,
        node_id: b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFE".to_vec(),
    };

    let mut transport = DiscoveryMessageTransport::new(socket);

    for peer in &peers_db {
        let to_ip = &peer.received_ip;
        let to_port = peer.received_port;

        let ping = Ping {
            from: Some(my_endpoint.clone()),
            to: Some(Endpoint {
                address: to_ip.clone(),
                port: to_port as _,
                node_id: vec![63u8; 64],
            }),
            version: P2P_VERSION,
            timestamp: Utc::now().timestamp_millis(),
        };
        transport
            .send((ping.into(), format!("{}:{}", to_ip, to_port).parse().unwrap()))
            .await?;
        println!("! pinging {}", to_ip);
    }

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

                let reply_ping = Ping {
                    from: Some(my_endpoint.clone()),
                    to: ping.from.clone(),
                    version: P2P_VERSION,
                    timestamp: Utc::now().timestamp_millis(),
                };
                transport.send((reply_ping.into(), peer_addr)).await?;
            }
            Ok((DiscoveryMessage::FindPeers(find), peer_addr)) => {
                let target = &find.target_id;
                let mut known_peers = peers_db.iter().collect::<Vec<_>>();
                known_peers.sort_by(|a, b| {
                    common_prefix_bits(&hex::decode(&b.id).unwrap(), target)
                        .cmp(&common_prefix_bits(&hex::decode(&a.id).unwrap(), target))
                });

                let nearby_peers = known_peers.into_iter().take(10).map(Endpoint::from).collect::<Vec<_>>();
                let peers = Peers {
                    from: Some(my_endpoint.clone()),
                    timestamp: Utc::now().timestamp_millis(),
                    peers: nearby_peers,
                };
                transport.send((peers.into(), peer_addr)).await?;

                let ping = Ping {
                    from: Some(my_endpoint.clone()),
                    to: find.from.clone(),
                    version: P2P_VERSION,
                    timestamp: Utc::now().timestamp_millis(),
                };
                transport.send((ping.into(), peer_addr)).await?;
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
            Ok((DiscoveryMessage::Pong(pong), peer_addr)) => {
                let ep = pong.from.as_ref().unwrap();
                let peer = Peer {
                    id: hex::encode(&ep.node_id),
                    version: pong.echo_version,
                    advertised_ip: ep.address.clone(),
                    advertised_port: ep.port as _,
                    received_ip: peer_addr.ip().to_string(),
                    received_port: peer_addr.port(),
                };

                if !peers_db.contains(&peer) {
                    peers_db.insert(peer);
                    std::fs::write("./peers.json", serde_json::to_string_pretty(&peers_db)?.as_bytes())?;
                }
            }
            Err(e) => {
                eprintln!("error: {:?}", e);
                return Err(e).map_err(From::from);
            }
        }
    }
    Ok(())
}
