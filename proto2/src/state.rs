include!(concat!(env!("OUT_DIR"), "/proto.state.rs"));

impl Account {
    pub fn new(block_timestamp: i64) -> Self {
        Account {
            creation_time: block_timestamp,
            ..Default::default()
        }
    }
}
