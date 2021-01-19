//! Resource related, freeze, unfreeze.

use std::convert::TryFrom;

use ::keys::Address;
use proto2::chain::transaction::Result as TransactionResult;
use proto2::common::{AccountType, ResourceCode};
use proto2::contract as contract_pb;
use proto2::state::ResourceDelegation;
use state::keys;

use super::super::super::governance::reward::RewardController;
use crate::Manager;
use super::super::TransactionContext;
use super::BuiltinContractExecutorExt;

impl BuiltinContractExecutorExt for contract_pb::FreezeBalanceContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;
        let owner_acct = state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "error while querying db")?
            .ok_or_else(|| "owner account is not on chain")?;

        if self.frozen_balance < 1_000_000 {
            return Err("frozen balance must be greater than 1_TRX".into());
        }
        if self.frozen_balance > owner_acct.balance {
            return Err(format!(
                "insufficient frozen balance, balance={}, required={}",
                owner_acct.balance, self.frozen_balance
            ));
        }

        // TODO: handle block.checkFrozenTime config
        if self.frozen_duration < constants::MIN_NUM_OF_FROZEN_DAYS_FOR_RESOURCE ||
            self.frozen_duration > constants::MAX_NUM_OF_FROZEN_DAYS_FOR_RESOURCE
        {
            return Err(format!(
                "frozen duration must be in range [{}, {}]",
                constants::MIN_NUM_OF_FROZEN_DAYS_FOR_RESOURCE,
                constants::MAX_NUM_OF_FROZEN_DAYS_FOR_RESOURCE
            ));
        }

        if ResourceCode::from_i32(self.resource).is_none() {
            return Err("resource code is invalid, possible values: [BANDWIDTH, ENERGY]".into());
        }

        if !self.receiver_address.is_empty() &&
            manager.state_db.must_get(&keys::ChainParameter::AllowDelegateResource) == 1
        {
            if self.receiver_address == self.owner_address {
                return Err("the owner and receiver address cannot be the same".into());
            }

            let receiver_address = Address::try_from(&self.receiver_address).map_err(|_| "invalid receiver_address")?;
            let recv_acct = state_db
                .get(&keys::Account(receiver_address))
                .map_err(|_| "error while querying db")?
                .ok_or_else(|| "receiver account is not on chain")?;

            if manager
                .state_db
                .must_get(&keys::ChainParameter::AllowTvmConstantinopleUpgrade) !=
                0 &&
                recv_acct.r#type == AccountType::Contract as i32
            {
                return Err(
                    "delegate resource to contract address is disabled since the Constantinople upgrade".into(),
                );
            }
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        const DAY_IN_MS: i64 = 86_400_000;

        let owner_addr = Address::try_from(&self.owner_address).unwrap();

        let now = manager.latest_block_timestamp();
        let duration = self.frozen_duration * DAY_IN_MS;
        let expire_time = now + duration;

        // NOTE: before AllowDelegateResource, receiver_address is not checked in `validate`.
        // Avoid using receiver_address before AllowDelegateResource.
        //
        // See-also: https://github.com/opentron/opentron/issues/44
        let maybe_recv_addr = if manager.state_db.must_get(&keys::ChainParameter::AllowDelegateResource) != 0 {
            Address::try_from(&self.receiver_address).ok()
        } else {
            None
        };

        // NOTE: In OpenTron, delegate to others and freeze for oneself is handled ~~in almost the same logic~~.
        let res_type = ResourceCode::from_i32(self.resource).unwrap();
        if let Some(recv_addr) = maybe_recv_addr {
            delegate_resource(
                manager,
                owner_addr,
                recv_addr,
                res_type,
                self.frozen_balance,
                expire_time,
            )?;
        } else {
            freeze_resource(manager, owner_addr, res_type, self.frozen_balance, expire_time)?;
        }

        Ok(TransactionResult::success())
    }
}

// Unfreeze and get frozen amount back. Will also remove all votes.
impl BuiltinContractExecutorExt for contract_pb::UnfreezeBalanceContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let owner_addr = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;
        let owner_acct = state_db
            .get(&keys::Account(owner_addr))
            .map_err(|_| "error while querying db")?
            .ok_or_else(|| "owner account is not on chain")?;

        let res_type = ResourceCode::from_i32(self.resource).ok_or("invalid resource type")?;

        let now = manager.latest_block_timestamp();

        if !self.receiver_address.is_empty() &&
            manager.state_db.must_get(&keys::ChainParameter::AllowDelegateResource) != 0
        {
            if self.owner_address == self.receiver_address {
                return Err("the owner and receiver address cannot be the same".into());
            }
            let recv_addr = Address::try_from(&self.receiver_address).map_err(|_| "invalid receiver_address")?;
            let recv_acct = state_db
                .get(&keys::Account(recv_addr))
                .map_err(|_| "error while querying db")?
                .ok_or_else(|| "receiver account is not on chain")?;

            let del = manager
                .state_db
                .get(&keys::ResourceDelegation(owner_addr, recv_addr))
                .map_err(|_| "error while querying db")?
                .ok_or_else(|| "delegation does not exist")?;

            let allow_tvm_constantinople = manager
                .state_db
                .must_get(&keys::ChainParameter::AllowTvmConstantinopleUpgrade) !=
                0;
            let allow_tvm_solidity059 = manager
                .state_db
                .must_get(&keys::ChainParameter::AllowTvmSolidity059Upgrade) !=
                0;

            // TODO: refactor logic
            match res_type {
                ResourceCode::Bandwidth => {
                    if del.amount_for_bandwidth <= 0 {
                        return Err("delegated bandwidth does not exist".into());
                    }
                    if !allow_tvm_constantinople {
                        if recv_acct.delegated_frozen_amount_for_bandwidth < del.amount_for_bandwidth {
                            return Err("inconsistent state-db!!".into());
                        }
                    } else if !allow_tvm_solidity059 &&
                        recv_acct.r#type != AccountType::Contract as i32 &&
                        recv_acct.delegated_frozen_amount_for_bandwidth < del.amount_for_bandwidth
                    {
                        return Err("inconsistent state-db!!".into());
                    }
                    if del.expiration_timestamp_for_bandwidth > now {
                        return Err("delegation is not expired".into());
                    }
                }
                ResourceCode::Energy => {
                    if del.amount_for_energy <= 0 {
                        return Err("delegated energy does not exist".into());
                    }
                    if !allow_tvm_constantinople {
                        if recv_acct.delegated_frozen_amount_for_energy < del.amount_for_energy {
                            return Err("inconsistent state-db!!".into());
                        }
                    } else if !allow_tvm_solidity059 &&
                        recv_acct.r#type != AccountType::Contract as i32 &&
                        recv_acct.delegated_frozen_amount_for_energy < del.amount_for_energy
                    {
                        return Err("inconsistent state-db!!".into());
                    }
                    if del.expiration_timestamp_for_energy > now {
                        return Err("delegation is not expired".into());
                    }
                }
            }
        } else {
            // NOTE: there will be only 1 freeze!
            let del = state_db.must_get(&keys::ResourceDelegation(owner_addr, owner_addr));
            match res_type {
                ResourceCode::Bandwidth => {
                    // NOTE: FrozenCount is not checked
                    if owner_acct.frozen_amount_for_bandwidth > 0 {
                        // check delegated from onself
                        if del.expiration_timestamp_for_bandwidth > now {
                            return Err("freeze is not expired yet, cannot unfreeze".into());
                        }
                    }
                }
                ResourceCode::Energy => {
                    if owner_acct.frozen_amount_for_energy > 0 {
                        // check delegated from onself
                        if del.expiration_timestamp_for_energy > now {
                            return Err("freeze is not expired yet, cannot unfreeze".into());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_addr = Address::try_from(&self.owner_address).unwrap();

        // withdrawReward
        RewardController::new(manager).withdraw_reward(owner_addr)?;

        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_addr));
        let res_type = ResourceCode::from_i32(self.resource).unwrap();

        let mut unfrozen_amount = 0;
        if !self.receiver_address.is_empty() &&
            manager.state_db.must_get(&keys::ChainParameter::AllowDelegateResource) == 1
        {
            // handle delegated resource
            let recv_addr = Address::try_from(&self.receiver_address).unwrap();
            let mut del = manager
                .state_db
                .must_get(&keys::ResourceDelegation(owner_addr, recv_addr));

            unfrozen_amount = del.amount_for_resource(res_type);
            del.reset_resource(res_type);
            owner_acct.delegated_out_amount -= unfrozen_amount;

            let mut recv_acct = manager.state_db.must_get(&keys::Account(recv_addr));

            let allow_tvm_constantinople = manager
                .state_db
                .must_get(&keys::ChainParameter::AllowTvmConstantinopleUpgrade) !=
                0;
            let allow_tvm_solidity059 = manager
                .state_db
                .must_get(&keys::ChainParameter::AllowTvmSolidity059Upgrade) !=
                0;

            if !allow_tvm_constantinople || recv_acct.r#type != AccountType::Contract as i32 {
                // TODO: is `saturating_add` can be used here to optimize?
                match res_type {
                    ResourceCode::Bandwidth => {
                        if allow_tvm_solidity059 && recv_acct.delegated_frozen_amount_for_bandwidth < unfrozen_amount {
                            recv_acct.delegated_frozen_amount_for_bandwidth = 0;
                        } else {
                            recv_acct.delegated_frozen_amount_for_bandwidth -= unfrozen_amount;
                        }
                    }
                    ResourceCode::Energy => {
                        if allow_tvm_solidity059 && recv_acct.delegated_frozen_amount_for_energy < unfrozen_amount {
                            recv_acct.delegated_frozen_amount_for_energy = 0;
                        } else {
                            recv_acct.delegated_frozen_amount_for_energy -= unfrozen_amount;
                        }
                    }
                }
                manager
                    .state_db
                    .put_key(keys::Account(recv_addr), recv_acct)
                    .map_err(|_| "db insert error")?;
            }

            owner_acct.adjust_balance(unfrozen_amount).unwrap();

            if del.is_empty() {
                remove_from_delegation_index(manager, owner_addr, recv_addr)?;
                manager
                    .state_db
                    .delete_key(&keys::ResourceDelegation(owner_addr, recv_addr))
                    .map_err(|_| "db delete error")?;
            } else {
                manager
                    .state_db
                    .put_key(keys::ResourceDelegation(owner_addr, recv_addr), del)
                    .map_err(|_| "db insert error")?;
            }
        } else {
            // handle frozen resource of oneself
            let mut del = manager
                .state_db
                .must_get(&keys::ResourceDelegation(owner_addr, owner_addr));
            match res_type {
                ResourceCode::Bandwidth => {
                    // ctx.withdrawal_amount = del.amount_for_bandwidth;
                    unfrozen_amount += del.amount_for_bandwidth;

                    owner_acct.adjust_balance(del.amount_for_bandwidth).unwrap();
                    del.amount_for_bandwidth = 0;
                    del.expiration_timestamp_for_bandwidth = 0;
                    owner_acct.frozen_amount_for_bandwidth = 0;
                }
                ResourceCode::Energy => {
                    unfrozen_amount += del.amount_for_energy;

                    // ctx.withdrawal_amount = del.amount_for_energy;
                    owner_acct.adjust_balance(del.amount_for_energy).unwrap();
                    del.amount_for_energy = 0;
                    del.expiration_timestamp_for_energy = 0;
                    owner_acct.frozen_amount_for_energy = 0;
                }
            }
            ctx.unfrozen_amount = unfrozen_amount;

            if del.is_empty() {
                remove_from_delegation_index(manager, owner_addr, owner_addr)?;
            }
            manager
                .state_db
                .put_key(keys::ResourceDelegation(owner_addr, owner_addr), del)
                .map_err(|_| "db insert error")?;
        }

        // handle global weight
        let weight_key = match res_type {
            ResourceCode::Bandwidth => keys::DynamicProperty::TotalBandwidthWeight,
            ResourceCode::Energy => keys::DynamicProperty::TotalEnergyWeight,
        };
        let weight = manager.state_db.must_get(&weight_key);
        manager
            .state_db
            .put_key(weight_key, weight - unfrozen_amount / 1_000_000)
            .map_err(|_| "db insert error")?;

        // clear votes
        let maybe_votes = manager
            .state_db
            .get(&keys::Votes(owner_addr))
            .map_err(|_| "db query error")?;
        if let Some(votes) = maybe_votes {
            for vote in &votes.votes {
                let wit_addr = Address::try_from(&vote.vote_address).unwrap();
                let mut wit = manager.state_db.must_get(&keys::Witness(wit_addr));
                wit.vote_count -= vote.vote_count;
                manager
                    .state_db
                    .put_key(keys::Witness(wit_addr), wit)
                    .map_err(|_| "db insert error")?;
            }
            manager
                .state_db
                .delete_key(&keys::Votes(owner_addr))
                .map_err(|_| "db delete error")?;
        }

        // save owner_acct at last
        manager
            .state_db
            .put_key(keys::Account(owner_addr), owner_acct)
            .map_err(|_| "db insert error")?;

        ctx.unfrozen_amount = unfrozen_amount;
        Ok(TransactionResult::success())
    }
}

fn add_to_delegation_index(manager: &mut Manager, from: Address, to: Address) -> Result<(), String> {
    let maybe_indexed_addrs = manager
        .state_db
        .get(&keys::ResourceDelegationIndex(from))
        .map_err(|_| "db query error")?;
    let mut indexed_addrs = maybe_indexed_addrs.unwrap_or_default();
    if !indexed_addrs.contains(&to) {
        indexed_addrs.push(to);
        manager
            .state_db
            .put_key(keys::ResourceDelegationIndex(from), indexed_addrs)
            .map_err(|_| "db insert error")?;
    }
    Ok(())
}

fn remove_from_delegation_index(manager: &mut Manager, from: Address, to: Address) -> Result<(), String> {
    let maybe_indexed_addrs = manager
        .state_db
        .get(&keys::ResourceDelegationIndex(from))
        .map_err(|_| "db query error")?;
    let indexed_addrs = maybe_indexed_addrs.unwrap_or_default();
    let indexed_addrs: Vec<_> = indexed_addrs.into_iter().filter(|addr| addr != &to).collect();
    if !indexed_addrs.is_empty() {
        manager
            .state_db
            .put_key(keys::ResourceDelegationIndex(from), indexed_addrs)
            .map_err(|_| "db insert error")?;
    } else {
        manager
            .state_db
            .delete_key(&keys::ResourceDelegationIndex(from))
            .map_err(|_| "db delete eerror")?;
    }
    Ok(())
}

fn delegate_resource(
    manager: &mut Manager,
    from: Address,
    to: Address,
    resouce_code: ResourceCode,
    amount: i64,
    expired_time: i64,
) -> Result<(), String> {
    let key = keys::ResourceDelegation(from, to);

    let maybe_delegated = manager.state_db.get(&key).map_err(|_| "db query error")?;
    let mut delegated = maybe_delegated.unwrap_or_else(|| ResourceDelegation {
        from_address: from.as_bytes().to_vec(),
        to_address: to.as_bytes().to_vec(),
        ..Default::default()
    });

    let weight_key;

    match resouce_code {
        ResourceCode::Bandwidth => {
            delegated.amount_for_bandwidth += amount;
            delegated.expiration_timestamp_for_bandwidth = expired_time;

            weight_key = keys::DynamicProperty::TotalBandwidthWeight;
        }
        ResourceCode::Energy => {
            delegated.amount_for_energy += amount;
            delegated.expiration_timestamp_for_energy = expired_time;

            weight_key = keys::DynamicProperty::TotalEnergyWeight;
        }
    }

    manager
        .state_db
        .put_key(key, delegated)
        .map_err(|_| "db insert error")?;

    let old_total_weight = manager.state_db.must_get(&weight_key);
    manager
        .state_db
        .put_key(weight_key, old_total_weight + amount / 1_000_000)
        .map_err(|_| "db insert error")?;

    // handle delegated-resource-index
    add_to_delegation_index(manager, from, to)?;

    // handle to_account resource
    let mut to_acct = manager.state_db.must_get(&keys::Account(to));
    match resouce_code {
        ResourceCode::Bandwidth => {
            to_acct.delegated_frozen_amount_for_bandwidth += amount;
        }
        ResourceCode::Energy => {
            to_acct.delegated_frozen_amount_for_energy += amount;
        }
    }
    manager
        .state_db
        .put_key(keys::Account(to), to_acct)
        .map_err(|_| "db insert error")?;

    // handle from_account balance
    let mut from_acct = manager.state_db.must_get(&keys::Account(from));
    from_acct.delegated_out_amount += amount;
    from_acct.adjust_balance(-amount).unwrap();
    manager
        .state_db
        .put_key(keys::Account(from), from_acct)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn freeze_resource(
    manager: &mut Manager,
    from: Address,
    resouce_code: ResourceCode,
    amount: i64,
    expired_time: i64,
) -> Result<(), String> {
    let key = keys::ResourceDelegation(from, from);

    let maybe_delegated = manager.state_db.get(&key).map_err(|_| "db query error")?;
    let mut delegated = maybe_delegated.unwrap_or_else(|| ResourceDelegation {
        to_address: from.as_bytes().to_vec(),
        from_address: from.as_bytes().to_vec(),
        ..Default::default()
    });

    let weight_key;

    match resouce_code {
        ResourceCode::Bandwidth => {
            delegated.amount_for_bandwidth += amount;
            delegated.expiration_timestamp_for_bandwidth = expired_time;

            weight_key = keys::DynamicProperty::TotalBandwidthWeight;
        }
        ResourceCode::Energy => {
            delegated.amount_for_energy += amount;
            delegated.expiration_timestamp_for_energy = expired_time;

            weight_key = keys::DynamicProperty::TotalEnergyWeight;
        }
    }

    manager
        .state_db
        .put_key(key, delegated)
        .map_err(|_| "db insert error")?;

    let old_total_weight = manager.state_db.must_get(&weight_key);
    manager
        .state_db
        .put_key(weight_key, old_total_weight + amount / 1_000_000)
        .map_err(|_| "db insert error")?;

    // handle delegated-resource-index
    add_to_delegation_index(manager, from, from)?;

    // handle account resource
    let mut from_acct = manager.state_db.must_get(&keys::Account(from));

    match resouce_code {
        ResourceCode::Bandwidth => {
            from_acct.frozen_amount_for_bandwidth += amount;
        }
        ResourceCode::Energy => {
            from_acct.frozen_amount_for_energy += amount;
        }
    }

    // handle account balance
    from_acct.adjust_balance(-amount).unwrap();

    manager
        .state_db
        .put_key(keys::Account(from), from_acct)
        .map_err(|_| "db insert error")?;
    Ok(())
}
