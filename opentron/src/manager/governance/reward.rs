//! Handle block reward, voting reward.

use ::keys::Address;
use log::warn;
use state::keys;

use super::super::Manager;

/// Controller to handle rewards. Renamed from DelegationService, which is ambiguous.
pub struct RewardController<'m> {
    manager: &'m mut Manager,
}

impl RewardController<'_> {
    pub fn new<'a>(manager: &'a mut Manager) -> RewardController<'a> {
        RewardController { manager }
    }

    pub fn pay_standby_witnesses(&mut self) -> Result<(), String> {
        unimplemented!()
    }

    pub fn pay_block_reward(&mut self) -> Result<(), String> {
        unimplemented!()
    }

    /// This method only updates an account's allowance and reset voting status.
    // Renamed: withdrawReward. Actually
    pub fn update_voting_reward(&mut self, _address: Address) -> Result<(), String> {
        if self
            .manager
            .state_db
            .must_get(&keys::ChainParameter::AllowChangeDelegation) ==
            0
        {
            return Ok(());
        }

        unimplemented!("TODO: allowance update required")
    }
}

pub struct RewardUtil<'m> {
    manager: &'m Manager,
}

impl RewardUtil<'_> {
    pub fn new<'a>(manager: &'a Manager) -> RewardUtil<'a> {
        RewardUtil { manager }
    }

    pub fn query_reward(&self, _addr: Address) -> i64 {
        let allow_change_delegation = self
            .manager
            .state_db
            .must_get(&keys::ChainParameter::AllowChangeDelegation) !=
            0;

        if !allow_change_delegation {
            return 0;
        }

        // unimplemented!()
        warn!("TODO: fake query_reward implementation");
        16_000_000
    }
}
