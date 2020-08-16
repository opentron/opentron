//! Resource related, freeze, unfreeze.

use std::convert::TryFrom;

use ::keys::Address;
use proto2::chain::transaction::Result as TransactionResult;
use proto2::common::{AccountType, ResourceCode};
use proto2::contract as contract_pb;
use proto2::state::ResourceDelegation;
use state::keys;

use super::super::executor::TransactionContext;
use super::super::Manager;
use super::BuiltinContractExecutorExt;

impl BuiltinContractExecutorExt for contract_pb::FreezeBalanceContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        let owner_acct = state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "error while querying db")?;
        if owner_acct.is_none() {
            return Err("owner account is not on chain".into());
        }
        let owner_acct = owner_acct.unwrap();

        if self.frozen_balance < 1_000_000 {
            return Err("frozen balance must be greater than 1_TRX".into());
        }
        // TODO: check account frozen count
        if self.frozen_balance > owner_acct.balance {
            return Err("insufficient balance".into());
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
                .map_err(|_| "error while querying db")?;
            if recv_acct.is_none() {
                return Err("receiver account is not on chain".into());
            }
            let recv_acct = recv_acct.unwrap();

            if manager
                .state_db
                .must_get(&keys::ChainParameter::AllowTvmConstantinopleUpgrade) ==
                1 &&
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

        let maybe_recv_addr = Address::try_from(&self.receiver_address).ok();

        // NOTE: In OpenTron, delegate to others and freeze for oneself is handled in the same logic.
        if let Some(resource_type) = ResourceCode::from_i32(self.resource) {
            if let Some(recv_addr) = maybe_recv_addr {
                delegate_resource(
                    manager,
                    owner_addr,
                    recv_addr,
                    resource_type,
                    self.frozen_balance,
                    expire_time,
                )?;
            } else {
                freeze_resource(manager, owner_addr, resource_type, self.frozen_balance, expire_time)?;
            }
        } else {
            unreachable!("already verified");
        }

        Ok(TransactionResult::success())
    }
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
        to_address: to.as_bytes().to_vec(),
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
    let maybe_indexed_addrs = manager
        .state_db
        .get(&keys::ResourceDelegationIndex(to))
        .map_err(|_| "db query error")?;
    let mut indexed_addrs = maybe_indexed_addrs.unwrap_or_default();

    if !indexed_addrs.contains(&from) {
        indexed_addrs.push(from);
        manager
            .state_db
            .put_key(keys::ResourceDelegationIndex(to), indexed_addrs)
            .map_err(|_| "db insert error")?;
    }

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
    let maybe_indexed_addrs = manager
        .state_db
        .get(&keys::ResourceDelegationIndex(from))
        .map_err(|_| "db query error")?;
    let mut indexed_addrs = maybe_indexed_addrs.unwrap_or_default();

    if !indexed_addrs.contains(&from) {
        indexed_addrs.push(from);
        manager
            .state_db
            .put_key(keys::ResourceDelegationIndex(from), indexed_addrs)
            .map_err(|_| "db insert error")?;
    }

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
