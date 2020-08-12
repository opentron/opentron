include!(concat!(env!("OUT_DIR"), "/proto.state.rs"));

pub use crate::common::SmartContract;

impl Account {
    pub fn new(block_timestamp: i64) -> Self {
        Account {
            creation_time: block_timestamp,
            resource: Some(Default::default()),
            ..Default::default()
        }
    }

    pub fn adjust_balance(&mut self, diff: i64) -> Result<(), ()> {
        if let Some(new_balance) = self.balance.checked_add(diff) {
            self.balance = new_balance;
            Ok(())
        } else {
            Err(())
        }
    }
}
