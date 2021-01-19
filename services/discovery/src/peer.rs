use proto2::common::Endpoint;
use serde::{Deserialize, Serialize};

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
