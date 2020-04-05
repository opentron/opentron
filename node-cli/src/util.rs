use proto2::common::Endpoint;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct Peer {
    id: String,
    version: i32,
    advertised_ip: String,
    advertised_port: u16,
    received_ip: String,
    received_port: u16,
}

impl From<&Peer> for Endpoint {
    fn from(peer: &Peer) -> Endpoint {
        Endpoint {
            address: peer.advertised_ip.clone(),
            port: peer.advertised_port as _,
            node_id: hex::decode(&peer.id).unwrap(),
        }
    }
}

#[derive(Deserialize)]
struct Ip {
    origin: String,
}

pub async fn get_my_ip() -> Result<String, Box<dyn Error>> {
    let ip = reqwest::get("http://httpbin.org/ip").await?.json::<Ip>().await?;
    Ok(ip.origin)
}
