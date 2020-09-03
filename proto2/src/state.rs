include!(concat!(env!("OUT_DIR"), "/proto.state.rs"));

pub use crate::common::SmartContract;
pub use crate::common::AccountType;

use self::proposal::State as ProposalState;

impl Account {
    pub fn new(block_timestamp: i64) -> Self {
        Account {
            creation_time: block_timestamp,
            resource: Some(Default::default()),
            ..Default::default()
        }
    }

    pub fn new_contract_account(block_timestamp: i64) -> Self {
        Account {
            creation_time: block_timestamp,
            r#type: AccountType::Contract as i32,
            ..Default::default()
        }
    }

    pub fn adjust_balance(&mut self, diff: i64) -> Result<(), ()> {
        if let Some(new_balance) = self.balance.checked_add(diff) {
            // When self.balance is negative, this is a blackhole.
            if self.balance < 0 || new_balance >= 0 {
                self.balance = new_balance;
                return Ok(());
            }
        }
        Err(())
    }

    pub fn adjust_allowance(&mut self, diff: i64) -> Result<(), ()> {
        if let Some(new_allowance) = self.allowance.checked_add(diff) {
            if new_allowance >= 0 {
                self.allowance = new_allowance;
                return Ok(());
            }
        }
        Err(())
    }

    pub fn adjust_token_balance(&mut self, token_id: i64, diff: i64) -> Result<(), ()> {
        if let Some(balance) = self.token_balance.get_mut(&token_id) {
            if let Some(new_balance) = balance.checked_add(diff) {
                if new_balance >= 0 {
                    *balance = new_balance;
                    return Ok(());
                }
            }
        } else if diff >= 0 {
            self.token_balance.insert(token_id, diff);
            return Ok(());
        }
        Err(())
    }

    pub fn tron_power(&self) -> i64 {
        (self.frozen_amount_for_bandwidth + self.frozen_amount_for_energy + self.delegated_out_amount) / 1_000_000
    }

    pub fn amount_for_bandwidth(&self) -> i64 {
        self.frozen_amount_for_bandwidth + self.delegated_frozen_amount_for_bandwidth
    }

    pub fn amount_for_energy(&self) -> i64 {
        self.frozen_amount_for_energy + self.delegated_frozen_amount_for_energy
    }

    pub fn resource(&self) -> &AccountResource {
        self.resource.as_ref().unwrap()
    }

    pub fn resource_mut(&mut self) -> &mut AccountResource {
        if self.resource.is_none() {
            self.resource = Some(Default::default());
        }
        self.resource.as_mut().unwrap()
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


impl SmartContract {
    pub fn new_inner() -> Self {
        SmartContract {
            name: "CreatedByContract".into(),
            consume_user_energy_percent: 100,
            origin_energy_limit: 0,
            ..Default::default()
        }
    }
}
