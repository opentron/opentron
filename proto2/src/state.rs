include!(concat!(env!("OUT_DIR"), "/proto.state.rs"));

pub use crate::common::SmartContract;

use self::proposal::State as ProposalState;

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

    pub fn tron_power(&self) -> i64 {
        (self.frozen_amount_for_bandwidth + self.frozen_amount_for_energy + self.delegated_out_amount) / 1_000_000
    }
}

impl Proposal {
    pub fn is_processed(&self) -> bool {
        if self.state == ProposalState::Disapproved as i32 || self.state == ProposalState::Approved as i32 {
            true
        } else {
            false
        }
    }

    pub fn is_cancelled(&self) -> bool {
        if self.state == ProposalState::Cancelled as i32 {
            true
        } else {
            false
        }
    }
}
