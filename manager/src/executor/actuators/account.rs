//! Account related builtin contracts.

use std::convert::TryFrom;

use ::keys::Address;
use log::{debug, warn};
use proto::chain::transaction::Result as TransactionResult;
use proto::chain::ContractType;
use proto::common::{permission::PermissionType, AccountType, Permission};
use proto::contract as contract_pb;
use proto::state::{Account, ActivePermission, OwnerPermission, PermissionKey};
use state::keys;

use crate::Manager;
use super::super::TransactionContext;
use super::BuiltinContractExecutorExt;

// Set account's name.
impl BuiltinContractExecutorExt for contract_pb::AccountUpdateContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        // validAccountName
        if self.account_name.as_bytes().len() > 200 {
            return Err("invalid account name".into());
        }

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;
        let acct = state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "db query error")?
            .ok_or_else(|| "account not exists")?;

        let allow_update_account_name = state_db.must_get(&keys::ChainParameter::AllowUpdateAccountName) != 0;
        if !acct.name.is_empty() && !allow_update_account_name {
            return Err("account name already exists".into());
        }

        if !allow_update_account_name && find_account_by_name(manager, &self.account_name).is_some() {
            return Err("the same account name already exists".into());
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_address = Address::try_from(&self.owner_address).unwrap();
        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_address));

        owner_acct.name = self.account_name.clone();

        manager
            .state_db
            .put_key(keys::Account(owner_address), owner_acct)
            .map_err(|e| e.to_string())?;
        manager
            .state_db
            .put_key(keys::AccountIndex(self.account_name.clone()), owner_address)
            .map_err(|e| e.to_string())?;

        Ok(TransactionResult::success())
    }
}

// Update account's permission for multisig or transfering ownership.
impl BuiltinContractExecutorExt for contract_pb::AccountPermissionUpdateContract {
    fn validate(&self, manager: &Manager, ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        if state_db.must_get(&keys::ChainParameter::AllowMultisig) == 0 {
            return Err("multisig is disabled on chain".into());
        }

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;
        let acct = state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "db query error")?
            .ok_or_else(|| "account not exists")?;

        if self.owner.is_none() {
            return Err("missing owner permission".into());
        }

        let is_witness = state_db
            .get(&keys::Witness(owner_address))
            .map_err(|_| "error while querying db")?
            .is_some();
        if is_witness {
            if let Some(wit_perm) = self.witness.as_ref() {
                check_permission(wit_perm, PermissionType::Witness)?;
            } else {
                return Err("missing witness permission".into());
            }
        } else if self.witness.is_some() {
            return Err("account is not a witness".into());
        }

        if self.actives.is_empty() {
            return Err("missing active permissions".into());
        }
        if self.actives.len() > constants::MAX_NUM_OF_ACTIVE_PERMISSIONS {
            return Err("too many active permissions".into());
        }

        check_permission(self.owner.as_ref().unwrap(), PermissionType::Owner)?;

        for active in &self.actives {
            check_permission(active, PermissionType::Active)?;
        }

        let fee = self.fee(manager);
        if acct.balance < fee {
            return Err("insufficient balance to set account permission".into());
        }
        ctx.contract_fee = fee;

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_address = Address::try_from(&self.owner_address).unwrap();
        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_address));

        // updatePermissions
        if let Some(owner_perm) = self.owner.as_ref() {
            owner_acct.owner_permission = Some(OwnerPermission {
                threshold: owner_perm.threshold,
                keys: owner_perm
                    .keys
                    .iter()
                    .map(|key| PermissionKey {
                        address: key.address.clone(),
                        weight: key.weight,
                    })
                    .collect(),
            });
        }

        owner_acct.active_permissions = self
            .actives
            .iter()
            .map(|perm| ActivePermission {
                threshold: perm.threshold,
                keys: perm
                    .keys
                    .iter()
                    .map(|key| PermissionKey {
                        address: key.address.clone(),
                        weight: key.weight,
                    })
                    .collect(),
                operations: perm.operations.clone(),
                permission_name: perm.name.clone(),
            })
            .collect();

        if let Some(wit_perm) = self.witness.as_ref() {
            let mut wit = manager.state_db.must_get(&keys::Witness(owner_address));
            wit.signature_key = wit_perm.keys[0].address.clone();

            manager
                .state_db
                .put_key(keys::Witness(owner_address), wit)
                .map_err(|e| e.to_string())?;
        }

        if ctx.contract_fee != 0 {
            owner_acct.adjust_balance(-ctx.contract_fee).unwrap();
            manager.add_to_blackhole(ctx.contract_fee).unwrap();
        }
        manager
            .state_db
            .put_key(keys::Account(owner_address), owner_acct)
            .map_err(|e| e.to_string())?;

        Ok(TransactionResult::success())
    }

    fn fee(&self, manager: &Manager) -> i64 {
        manager
            .state_db
            .must_get(&keys::ChainParameter::AccountPermissionUpdateFee)
    }
}

// Create an account on chain.
//
// NOTE: This is a bad desgin, and is still vulnerable. One can create a contract of any type, which is meanningless.
impl BuiltinContractExecutorExt for contract_pb::AccountCreateContract {
    fn validate(&self, manager: &Manager, ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let fee = self.fee(manager);

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;
        let new_address = Address::try_from(&self.account_address).map_err(|_| "invalid account_address")?;

        let owner_acct = state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "db query error")?
            .ok_or_else(|| "account not exists")?;

        let maybe_new_acct = state_db
            .get(&keys::Account(new_address))
            .map_err(|_| "db query error")?;
        if maybe_new_acct.is_some() {
            return Err("account already exists".into());
        }

        if owner_acct.balance < fee {
            return Err("insufficient balance to create an account".into());
        }

        // NOTE: type is not checked here!

        ctx.new_account_created = true;
        Ok(())
    }

    fn execute(&self, manager: &mut Manager, ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_address = Address::try_from(&self.owner_address).unwrap();
        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_address));

        let new_address = Address::try_from(&self.account_address).unwrap();

        let fee = ctx.contract_fee;

        // NOTE: Account's creation_time is not current block timestamp, it's previous.
        let mut new_acct = Account::new(manager.latest_block_timestamp());
        if let Some(acct_type) = AccountType::from_i32(self.r#type as i32) {
            if acct_type != AccountType::Normal {
                warn!("create account with type={:?}", acct_type);
            }
            // NOTE: One can create account of any type
            new_acct.r#type = self.r#type;
        } else {
            panic!("invalid account type code: {}", self.r#type);
        }

        if fee != 0 {
            owner_acct.adjust_balance(-fee).unwrap();
            manager.add_to_blackhole(fee).unwrap();
            manager
                .state_db
                .put_key(keys::Account(owner_address), owner_acct)
                .map_err(|e| e.to_string())?;
        }

        manager
            .state_db
            .put_key(keys::Account(new_address), new_acct)
            .map_err(|e| e.to_string())?;

        Ok(TransactionResult::success())
    }

    fn fee(&self, manager: &Manager) -> i64 {
        // NOTE: CreateNewAccountFeeInSystemContract is 0.
        // Account creation fee(bandwidth) is handled by BandwidthProcessor.
        manager
            .state_db
            .must_get(&keys::ChainParameter::CreateNewAccountFeeInSystemContract)
    }
}

// Deprecated but not removed.
impl BuiltinContractExecutorExt for contract_pb::SetAccountIdContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        // validAccountId
        // Length: 8~32
        // Charset: '!' to '~'
        if self.account_id.is_empty() ||
            self.account_id.len() < 8 ||
            self.account_id.len() > 32 ||
            self.account_id.iter().any(|&c| c < 0x21 || c > 0x7e)
        {
            return Err("invalid account id".into());
        }

        let acct = state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "db query error")?
            .ok_or("account not exists")?;

        if !acct.account_id.is_empty() {
            return Err("account id is already set".into());
        }

        debug!("TODO: check account-id reverse index");

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_address = Address::try_from(&self.owner_address).unwrap();
        let mut acct = manager.state_db.must_get(&keys::Account(owner_address));

        acct.account_id = self.account_id.clone();

        manager
            .state_db
            .put_key(keys::Account(owner_address), acct)
            .map_err(|_| "db insert error")?;

        Ok(TransactionResult::success())
    }
}

/// Find an account in state-db by its name.
fn find_account_by_name(manager: &Manager, acct_name: &str) -> Option<Account> {
    let maybe_addr = manager
        .state_db
        .get(&keys::AccountIndex(acct_name.to_owned()))
        .ok()
        .flatten();
    maybe_addr.map(|addr| manager.state_db.must_get(&keys::Account(addr)))
}

/// Check permission pb definition.
fn check_permission(perm: &Permission, perm_type: PermissionType) -> Result<(), String> {
    if perm.keys.len() > constants::MAX_NUM_OF_KEYS_IN_PERMISSION {
        return Err(format!(
            "number of keys in permission should not be greater than {}",
            constants::MAX_NUM_OF_KEYS_IN_PERMISSION
        ));
    }
    if perm.keys.is_empty() {
        return Err("no permission key provided".into());
    }

    if perm.threshold <= 0 {
        return Err("permission threshold should be greater than 0".into());
    }
    if perm.name.len() > 32 {
        return Err("permission name is too long".into());
    }
    if perm.parent_id != 0 {
        return Err("parent_id must be 0(owner)".into());
    }

    let mut weight_sum = 0_i64;
    let mut addrs: Vec<&[u8]> = Vec::with_capacity(perm.keys.len());
    for key in &perm.keys {
        if Address::try_from(&key.address).is_err() {
            return Err("invalid key address".into());
        }
        if key.weight <= 0 {
            return Err("weight of key should be greater than 0".into());
        }
        weight_sum = weight_sum.checked_add(key.weight).ok_or("math overflow")?;

        if addrs.contains(&&*key.address) {
            return Err("duplicated address in keys".into());
        } else {
            addrs.push(&*key.address);
        }
    }

    if weight_sum < perm.threshold {
        return Err("sum of all weights should be greater than threshold".into());
    }

    match perm_type {
        PermissionType::Owner | PermissionType::Witness => {
            if !perm.operations.is_empty() {
                return Err("no operations vec needed".into());
            }
        }
        PermissionType::Active => {
            if perm.operations.is_empty() || perm.operations.len() != 32 {
                return Err("operations vec length must be 32".into());
            }
            // NOTE: The check logic is buggy in java-tron.
            for type_code in 0..256 {
                let mask = (perm.operations[type_code / 8] >> (type_code % 8)) & 1;
                if mask != 0 && ContractType::from_i32(type_code as i32).is_none() {
                    return Err(format!("operation of {} is undefined", type_code));
                }
            }
        }
    }

    Ok(())
}
