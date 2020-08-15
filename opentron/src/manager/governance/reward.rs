//! Handle block reward, voting reward.

use ::keys::Address;
use state::keys;

use super::super::Manager;

/// Controller to handle rewards. Renamed from DelegationService, which is ambiguous.
pub struct RewardController<'m> {
    manager: &'m Manager,
}


impl RewardController<'_> {
    pub fn new<'a>(manager: &'a mut Manager) -> RewardController<'a> {
        RewardController {
            manager
        }
    }

    pub fn pay_standby_witnesses(&mut self) -> Result<(), String> {
        unimplemented!()
    }

    pub fn pay_block_reward(&mut self) -> Result<(), String> {
        unimplemented!()
    }

    /// This method only updates an account's allowance and reset voting status.
    // Renamed: withdrawReward. Actually
    pub fn update_voting_reward(&mut self, address: Address) -> Result<(), String> {
        if self.manager.state_db.must_get(&keys::ChainParameter::AllowChangeDelegation) == 0 {
            return Ok(())
        }

        log::debug!("TODO: allowance update required");
        //unimplemented!()
        Ok(())
    }
}
