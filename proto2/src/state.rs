include!(concat!(env!("OUT_DIR"), "/proto.state.rs"));

pub use crate::common::SmartContract;

impl Account {
    pub fn new(block_timestamp: i64) -> Self {
        Account {
            creation_time: block_timestamp,
            ..Default::default()
        }
    }
}
