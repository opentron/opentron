use byteorder::{ByteOrder, BE};
use proto2::common::Endpoint;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct Peer {
    pub id: String,
    pub version: i32,
    pub advertised_ip: String,
    pub advertised_port: u16,
    pub received_ip: String,
    pub received_port: u16,
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

pub fn get_my_ip() -> Result<String, Box<dyn Error>> {
    let ip = reqwest::blocking::get("http://httpbin.org/ip")?.json::<Ip>()?;
    Ok(ip.origin)
}

pub fn block_hash_to_number(hash: &[u8]) -> i64 {
    BE::read_u64(&hash[..8]) as _
}
