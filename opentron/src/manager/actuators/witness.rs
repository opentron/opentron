use std::convert::TryFrom;

use ::keys::Address;
use proto2::chain::transaction::Result as TransactionResult;
use proto2::contract as contract_pb;
use proto2::state::{Votes, Witness};
use state::keys;

use super::super::executor::TransactionContext;
use super::super::governance::reward::{RewardController, RewardUtil};
use super::super::Manager;
use super::BuiltinContractExecutorExt;

impl BuiltinContractExecutorExt for contract_pb::WitnessCreateContract {
    fn validate(&self, manager: &Manager, ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        let fee = self.fee(manager);

        // validUrl
        if self.url.is_empty() || self.url.len() > 256 {
            return Err("invalid url".into());
        }

        let owner_acct = state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "error while querying db")?;
        if owner_acct.is_none() {
            return Err("owner account is not on chain".into());
        }
        let owner_acct = owner_acct.unwrap();

        let maybe_witness = state_db
            .get(&keys::Witness(owner_address))
            .map_err(|_| "error while querying db")?;
        if maybe_witness.is_some() {
            return Err(format!("witness {} already exists", owner_address));
        }

        if owner_acct.balance < fee {
            return Err("insufficient balance to create witness".into());
        }

        ctx.contract_fee = fee;

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_address = Address::try_from(&self.owner_address).unwrap();

        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_address));
        // createWitness

        let witness = Witness {
            address: owner_address.as_bytes().to_vec(),
            url: unsafe { String::from_utf8_unchecked(self.url.clone()) },
            vote_count: 0,
            // FIXME: is_active should be updated in vote counting
            is_active: false,
            ..Default::default()
        };

        manager
            .state_db
            .put_key(keys::Witness(owner_address), witness)
            .map_err(|_| "db insert error")?;

        // TODO: setIsWitness for account,  getAllowMultiSign for witness permission

        owner_acct.adjust_balance(-ctx.contract_fee).unwrap();
        manager
            .state_db
            .put_key(keys::Account(owner_address), owner_acct)
            .map_err(|e| e.to_string())?;

        Ok(TransactionResult::success())
    }

    fn fee(&self, manager: &Manager) -> i64 {
        manager.state_db.must_get(&keys::ChainParameter::WitnessCreateFee)
    }
}

impl BuiltinContractExecutorExt for contract_pb::VoteWitnessContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        if self.votes.is_empty() {
            return Err("no votes".into());
        }
        if self.votes.len() > constants::MAX_NUM_OF_VOTES {
            return Err("exceeds maximum number of votes".into());
        }

        let mut total_vote_count = 0_i64;
        for vote in &self.votes {
            let candidate_addr = Address::try_from(&vote.vote_address).map_err(|_| "invalid vote_address")?;
            if vote.vote_count <= 0 {
                return Err("vote count must be greater than 0".into());
            }
            // witness implies account
            let maybe_witness = state_db
                .get(&keys::Witness(candidate_addr))
                .map_err(|_| "db query error")?;
            if maybe_witness.is_none() {
                return Err("witness not found".into());
            }
            total_vote_count = total_vote_count
                .checked_add(vote.vote_count)
                .ok_or("mathematical overflow")?;
        }

        let maybe_owner_acct = state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "error while querying db")?;
        if maybe_owner_acct.is_none() {
            return Err("owner account is not on chain".into());
        }
        let owner_acct = maybe_owner_acct.unwrap();

        // 1_TRX for 1_TP
        let tp = owner_acct.tron_power();
        if total_vote_count > tp {
            return Err(format!(
                "total number of votes is greater than account's tron power, {} > {}",
                total_vote_count, tp
            ));
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        // countVoteAccount
        let owner_addr = Address::try_from(&self.owner_address).unwrap();

        // delegationService.withdrawReward(ownerAddress);
        RewardController::new(manager).update_voting_reward(owner_addr)?;

        manager
            .state_db
            .put_key(
                keys::Votes(owner_addr),
                Votes {
                    votes: self.votes.clone(),
                },
            )
            .map_err(|_| "db insert error")?;

        Ok(TransactionResult::success())
    }
}

// Withdraw block producing reward, standby witness reward, and voting reward.
impl BuiltinContractExecutorExt for contract_pb::WithdrawBalanceContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        const DAY_IN_MS: i64 = 86_400_000;

        let state_db = &manager.state_db;

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        let maybe_acct = state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "db query error")?;
        if maybe_acct.is_none() {
            return Err("account not exists".into());
        }
        let acct = maybe_acct.unwrap();

        let is_gr = manager
            .genesis_config
            .witnesses
            .iter()
            .find(|gr| gr.address.parse::<Address>().expect("address format error") == owner_address)
            .is_some();
        if is_gr {
            return Err("account is is a guard representative and is not allowed to withdraw balance".into());
        }

        let latest_withdraw_ts = acct.latest_withdraw_timestamp;
        let now = manager.latest_block_timestamp();
        let witness_allowance_frozen_time = constants::NUM_OF_FROZEN_DAYS_FOR_WITNESS_ALLOWANCE * DAY_IN_MS;

        if now - latest_withdraw_ts < witness_allowance_frozen_time {
            return Err("latest withdrawal is less than 24 hours ago".into());
        }

        if acct.allowance <= 0 && RewardUtil::new(manager).query_reward(owner_address) <= 0 {
            return Err("account does not have any reward".into());
        }

        if acct.balance.checked_add(acct.allowance).is_none() {
            return Err("math overflow".into());
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_addr = Address::try_from(&self.owner_address).unwrap();
        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_addr));

        // delegationService.withdrawReward(ownerAddress);
        RewardController::new(manager).update_voting_reward(owner_addr)?;

        let now = manager.latest_block_timestamp();
        owner_acct.adjust_balance(owner_acct.allowance).unwrap();
        owner_acct.allowance = 0;
        owner_acct.latest_withdraw_timestamp = now;

        manager
            .state_db
            .put_key(keys::Account(owner_addr), owner_acct)
            .map_err(|e| e.to_string())?;
        Ok(TransactionResult::success())
    }
}
