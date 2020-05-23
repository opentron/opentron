use chrono::Utc;
use futures::future::FutureExt;
use futures::select;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use log::{debug, error, info, warn};
use proto2::common::Endpoint;
use proto2::discovery::{FindPeers, Peers, Ping, Pong};
use rand::Rng;
use std::collections::HashSet;
use std::error::Error;
use std::future::Future;
use std::sync::Arc;
use tokio::net;
use tokio::net::UdpSocket;

use super::protocol::{DiscoveryMessage, DiscoveryMessageTransport};
use crate::context::AppContext;
use crate::util::Peer;

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

pub async fn discovery_server<F>(ctx: Arc<AppContext>, signal: F) -> Result<(), Box<dyn Error>>
where
    F: Future<Output = ()> + Unpin,
{
    let config = &ctx.config.protocol.discovery;

    if !config.enable {
        warn!("discover service disabled");
        return Ok(());
    }

    // let channel_config = &ctx.config.protocol.channel;
    let my_ip = &ctx.outbound_ip;
    let p2p_version = ctx.config.chain.p2p_version;

    let endpoint = &config.endpoint;

    let socket = UdpSocket::bind(endpoint).await?;
    info!("udp bind to socket {}", socket.local_addr()?);

    let peers_data = std::fs::read_to_string("./peers.json").unwrap_or("[]".to_string());
    let mut peers_db: HashSet<Peer> = serde_json::from_str(&peers_data)?;

    let my_endpoint = Endpoint {
        address: my_ip.clone(),
        port: 18888,
        node_id: ctx.node_id.to_vec(),
    };

    let mut transport = DiscoveryMessageTransport::new(socket);

    for peer in &ctx.config.protocol.seed_nodes {
        if let Some(peer_addr) = net::lookup_host(peer).await.ok().and_then(|mut it| it.next()) {
            let ping = Ping {
                from: Some(my_endpoint.clone()),
                to: Some(Endpoint {
                    address: peer_addr.ip().to_string(),
                    port: peer_addr.port() as _,
                    node_id: vec![63u8; 64],
                }),
                version: p2p_version,
                timestamp: Utc::now().timestamp_millis(),
            };
            transport.send((ping.into(), peer_addr)).await?;
            info!("ping {}", peer_addr);
        } else {
            warn!("unable to resove address {:?}", peer);
        }
    }

    let mut signal = signal.fuse();
    loop {
        let mut payload_fut = transport.next().fuse();
        select! {
            _ = signal => {
                    warn!("discover service closed");
                    break;
            }
            payload = payload_fut => {
                if payload.is_none() {
                    warn!("udp discoery closed");
                    return Ok(());
                }
                let payload = payload.unwrap();
                match payload {
                    Ok((DiscoveryMessage::Ping(ping), peer_addr)) => {
                        if ping.version != p2p_version {
                            warn!("{} version mismatch: version = {}", peer_addr, ping.version);
                            continue;
                        }
                        let pong = Pong {
                            from: Some(my_endpoint.clone()),
                            timestamp: Utc::now().timestamp_millis(),
                            echo_version: p2p_version,
                        };
                        transport.send((pong.into(), peer_addr)).await?;
                        debug!("{} pong", peer_addr);
                        let mut rng = rand::thread_rng();
                        let mut random_id = vec![0u8; 32];
                        rng.fill(&mut random_id[..]);
                        debug!("find peers target={}", hex::encode(&random_id));
                        if ["127.0.0.1", my_ip, "192.168.1.1"].contains(&&*peer_addr.ip().to_string()) {
                            continue;
                        }
                        let find = FindPeers {
                            from: Some(my_endpoint.clone()),
                            timestamp: Utc::now().timestamp_millis(),
                            target_id: random_id,
                        };
                        transport.send((find.into(), peer_addr)).await?;
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
                            version: p2p_version,
                            timestamp: Utc::now().timestamp_millis(),
                        };
                        transport.send((ping.into(), peer_addr)).await?;
                    }
                    Ok((DiscoveryMessage::Peers(peers), _)) => {
                        for peer in &peers.peers {
                            if ["127.0.0.1", my_ip, "192.168.1.1"].contains(&&*peer.address) {
                                continue;
                            }
                            if let Ok(peer_addr) = format!("{}:{}", peer.address, peer.port).parse() {
                                info!("=> ping peer {}", peer_addr);
                                let ping = Ping {
                                    from: Some(my_endpoint.clone()),
                                    to: Some(Endpoint {
                                        address: peer.address.clone(),
                                        port: peer.port,
                                        node_id: vec![63u8; 64],
                                    }),
                                    version: p2p_version,
                                    timestamp: Utc::now().timestamp_millis(),
                                };
                                transport.send((ping.into(), peer_addr)).await?;
                            } else {
                                warn!("unable to parse peer address {}:{}", peer.address, peer.port);
                            }
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
                        error!("error: {:?}", e);
                        return Err(e).map_err(From::from);
                    }
                }
            }
        }
    }
    /*
    while let Some(payload) = transport.next().await {
        if let Ok((ref frame, ref peer_addr)) = payload {
            let addr = peer_addr.to_string(); // SocketAddr does not support width format
            debug!("<= {:^24} {:?}", addr, frame);
        }

        match payload {
            Ok((DiscoveryMessage::Ping(ping), peer_addr)) => {
                if ping.version != p2p_version {
                    warn!("{} version mismatch: version = {}", peer_addr, ping.version);
                    continue;
                }
                let pong = Pong {
                    from: Some(my_endpoint.clone()),
                    timestamp: Utc::now().timestamp_millis(),
                    echo_version: p2p_version,
                };
                transport.send((pong.into(), peer_addr)).await?;
                debug!("{} pong", peer_addr);

                let mut rng = rand::thread_rng();
                let mut random_id = vec![0u8; 32];
                rng.fill(&mut random_id[..]);

                debug!("find peers target={}", hex::encode(&random_id));

                if ["127.0.0.1", my_ip, "192.168.1.1"].contains(&&*peer_addr.ip().to_string()) {
                    continue;
                }
                let find = FindPeers {
                    from: Some(my_endpoint.clone()),
                    timestamp: Utc::now().timestamp_millis(),
                    target_id: random_id,
                };
                transport.send((find.into(), peer_addr)).await?;
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
                    version: p2p_version,
                    timestamp: Utc::now().timestamp_millis(),
                };
                transport.send((ping.into(), peer_addr)).await?;
            }
            Ok((DiscoveryMessage::Peers(peers), _)) => {
                for peer in &peers.peers {
                    if ["127.0.0.1", my_ip, "192.168.1.1"].contains(&&*peer.address) {
                        continue;
                    }
                    if let Ok(peer_addr) = format!("{}:{}", peer.address, peer.port).parse() {
                        info!("=> ping peer {}", peer_addr);
                        let ping = Ping {
                            from: Some(my_endpoint.clone()),
                            to: Some(Endpoint {
                                address: peer.address.clone(),
                                port: peer.port,
                                node_id: vec![63u8; 64],
                            }),
                            version: p2p_version,
                            timestamp: Utc::now().timestamp_millis(),
                        };
                        transport.send((ping.into(), peer_addr)).await?;
                    } else {
                        warn!("unable to parse peer address {}:{}", peer.address, peer.port);
                    }
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
                error!("error: {:?}", e);
                return Err(e).map_err(From::from);
            }
        }
    }
    */
    Ok(())
}
