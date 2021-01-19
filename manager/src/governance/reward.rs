//! Handle block reward, standby reward, and voting reward.

use ::keys::Address;
use chain::IndexedBlock;
use log::debug;
use proto2::state::Votes;
use state::keys;

use crate::Manager;

/// Controller to handle rewards. Renamed from DelegationService, which is ambiguous.
///
/// In OpenTron, the reward paying logic is refactored, simpler than the original one.
pub struct RewardController<'m> {
    manager: &'m mut Manager,
}

impl RewardController<'_> {
    pub fn new<'a>(manager: &'a mut Manager) -> RewardController<'a> {
        RewardController { manager }
    }

    // NOTE: this is a merged logic of `payBlockReward` + `payStandbyWitness`.
    pub fn pay_reward(&mut self, block: &IndexedBlock) -> Result<(), String> {
        let curr_wit_addr = *Address::from_bytes(block.witness());

        let wit_sched = self.manager.state_db.must_get(&keys::WitnessSchedule);

        let mut wit_accts = Vec::with_capacity(constants::MAX_NUM_OF_ACTIVE_WITNESSES);
        let mut total_votes = 0_i64;
        for &(wit_addr, vote_count, _) in &wit_sched {
            wit_accts.push(self.manager.state_db.must_get(&keys::Account(wit_addr)));
            total_votes += vote_count;
        }

        let block_reward = self
            .manager
            .state_db
            .must_get(&keys::ChainParameter::WitnessPayPerBlock);
        let standby_reward = self
            .manager
            .state_db
            .must_get(&keys::ChainParameter::StandbyWitnessPayPerBlock);

        // NOTE: When there're no votes at all, none will be paid to standby witnesses.
        let standby_pay_per_vote = if total_votes > 0 {
            standby_reward as f64 / total_votes as f64
        } else {
            0.0
        };

        let epoch = self.manager.state_db.must_get(&keys::DynamicProperty::CurrentEpoch);
        // payReward
        for ((wit_addr, vote_count, brokerage), mut wit_acct) in wit_sched.into_iter().zip(wit_accts.into_iter()) {
            let mut voters_reward = 0;
            // payStandbyWitness
            let brokerage_rate = brokerage as f64 / 100.0;
            let pay = (standby_pay_per_vote * vote_count as f64) as i64;

            if pay > 0 {
                let brokerage_amount = (pay as f64 * brokerage_rate) as i64;
                voters_reward += pay - brokerage_amount;
                if wit_acct.adjust_allowance(brokerage_amount).is_err() {
                    return Err("math overflow while adding brokerage amount".into());
                }
            }

            // payBlockReward
            if wit_addr == curr_wit_addr {
                let brokerage_amount = (block_reward as f64 * brokerage_rate) as i64;
                voters_reward += block_reward - brokerage_amount;
                if wit_acct.adjust_allowance(brokerage_amount).is_err() {
                    return Err("math overflow while adding brokerage amount".into());
                }
            }
            // save reward of voters
            // delegationStore.addReward(cycle, witnessAddress, value);
            self.add_voter_reward(epoch, wit_addr, voters_reward);
            self.manager
                .state_db
                .put_key(keys::Account(wit_addr), wit_acct)
                .map_err(|_| "db insert error")?;
        }

        Ok(())
    }

    fn add_voter_reward(&mut self, epoch: i64, wit_addr: Address, amount: i64) {
        assert!(amount >= 0, "voter reward must be greater than or equal to 0");
        let key = keys::VoterReward(epoch, wit_addr);
        let mut reward = self.manager.state_db.must_get(&key);
        reward.reward_amount += amount;
        self.manager.state_db.put_key(key, reward).unwrap();
    }

    // withdrawReward
    /// Update an account's allowance and reset voting epoch status.
    pub fn withdraw_reward(&mut self, addr: Address) -> Result<(), String> {
        if self
            .manager
            .state_db
            .must_get(&keys::ChainParameter::AllowChangeDelegation) ==
            0
        {
            return Ok(());
        }

        if let Some(mut votes) = self
            .manager
            .state_db
            .get(&keys::Votes(addr))
            .map_err(|_| "db query error")?
        {
            let curr_epoch = self.manager.state_db.must_get(&keys::DynamicProperty::CurrentEpoch);
            if votes.epoch == curr_epoch {
                return Ok(());
            }

            let mut acct = self.manager.state_db.must_get(&keys::Account(addr));

            let begin_epoch = votes.epoch;
            let mut reward_amount = 0;
            for epoch in begin_epoch..curr_epoch {
                reward_amount += RewardUtil::new(self.manager).compute_reward(epoch, &votes)?;
            }
            debug!("withdraw reward={} epochs={}", reward_amount, curr_epoch - begin_epoch);

            if reward_amount != 0 {
                acct.adjust_allowance(reward_amount).unwrap();
                self.manager.state_db.put_key(keys::Account(addr), acct).unwrap();
            }

            votes.epoch = curr_epoch;
            self.manager.state_db.put_key(keys::Votes(addr), votes).unwrap();
        }

        Ok(())
    }
}

pub struct RewardUtil<'m> {
    manager: &'m Manager,
}

impl RewardUtil<'_> {
    pub fn new<'a>(manager: &'a Manager) -> RewardUtil<'a> {
        RewardUtil { manager }
    }

    // DelegationService.queryReward.
    pub fn query_reward(&self, addr: Address) -> Result<i64, String> {
        let allow_change_delegation = self
            .manager
            .state_db
            .must_get(&keys::ChainParameter::AllowChangeDelegation) !=
            0;
        if !allow_change_delegation {
            return Ok(0);
        }

        if let Some(votes) = self
            .manager
            .state_db
            .get(&keys::Votes(addr))
            .map_err(|_| "db query error")?
        {
            let curr_epoch = self.manager.state_db.must_get(&keys::DynamicProperty::CurrentEpoch);
            if votes.epoch == curr_epoch {
                return Ok(0);
            }

            let begin_epoch = votes.epoch;
            let mut reward_amount = 0;
            for epoch in begin_epoch..curr_epoch {
                reward_amount += self.compute_reward(epoch, &votes)?;
            }
            Ok(reward_amount)
        } else {
            Ok(0)
        }
    }

    fn compute_reward(&self, epoch: i64, votes: &Votes) -> Result<i64, String> {
        let mut reward_amount = 0_i64;
        for vote in &votes.votes {
            let wit_addr = *Address::from_bytes(&vote.vote_address);
            if let Some(total_reward) = self
                .manager
                .state_db
                .get(&keys::VoterReward(epoch, wit_addr))
                .map_err(|_| "db query error")?
            {
                if total_reward.vote_count == 0 {
                    continue;
                }
                let vote_rate = vote.vote_count as f64 / total_reward.vote_count as f64;
                reward_amount += (vote_rate * total_reward.reward_amount as f64) as i64;
            }
        }
        Ok(reward_amount)
    }
}
