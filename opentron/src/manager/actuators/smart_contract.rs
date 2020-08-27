//! Creating smart contracts, calling smart contract methods.

use std::convert::TryFrom;
use std::rc::Rc;

use ::keys::Address;
use constants::block_version::BlockVersion;
use log::warn;
use primitive_types::{H160, H256};
use proto2::chain::transaction::Result as TransactionResult;
use proto2::contract as contract_pb;
use proto2::state::Account;
use state::keys;

use super::super::controllers::ForkController;
use super::super::executor::TransactionContext;
use super::super::resource::EnergyUtil;
use super::super::vm::StateBackend;
use super::super::Manager;
use super::BuiltinContractExecutorExt;

const MAX_CONTRACT_NAME_LENGTH: usize = 32;
const MAX_FEE_LIMIT: i64 = 1_000_000_000;
const MIN_TOKEN_ID: i64 = 1_000_000;

impl BuiltinContractExecutorExt for contract_pb::CreateSmartContract {
    fn validate(&self, manager: &Manager, ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        if state_db.must_get(&keys::ChainParameter::AllowTvm) == 0 {
            return Err("TVM is disabled".into());
        }

        let new_cntr = self
            .new_contract
            .as_ref()
            .ok_or_else(|| "invalid CreateSmartContract")?;
        if &self.owner_address != &new_cntr.origin_address {
            return Err("owner address and origin address must be the same".into());
        }
        if new_cntr.name.as_bytes().len() > MAX_CONTRACT_NAME_LENGTH {
            return Err("smart contract's name must not be greater than 32".into());
        }
        if new_cntr.consume_user_resource_percent < 0 || new_cntr.consume_user_resource_percent > 100 {
            return Err("user energy consume percent must be in [0, 100]".into());
        }

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        let cntr_address = generate_created_contract_address(&ctx.transaction_hash, &owner_address);
        if manager
            .state_db
            .get(&keys::Account(cntr_address))
            .map_err(|_| "db query error")?
            .is_some()
        {
            return Err("contract address already exists".into());
        }

        let call_value = new_cntr.call_value;
        let mut call_token_value = 0_i64;
        let mut call_token_id = 0_i64;

        let allow_trc10_transfer = manager
            .state_db
            .must_get(&keys::ChainParameter::AllowTvmTransferTrc10Upgrade) !=
            0;

        if allow_trc10_transfer {
            call_token_value = self.call_token_value;
            call_token_id = self.call_token_id;
        }

        log::debug!("fee_limit => {}", ctx.fee_limit);
        if ctx.fee_limit < 0 || ctx.fee_limit > MAX_FEE_LIMIT {
            return Err("invalid fee_limit".into());
        }

        let maybe_owner_acct = manager
            .state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "db query error")?;
        if maybe_owner_acct.is_none() {
            return Err("owner_account not found".into());
        }
        let acct = maybe_owner_acct.unwrap();

        // i.e. the ENERGY_LIMIT fork
        let energy_limit = if ForkController::new(manager).pass_version(BlockVersion::Odyssey3_2_2)? {
            if call_value < 0 {
                return Err("invalid call_value".into());
            }
            if call_token_value < 0 {
                return Err("invalid call_token_value".into());
            }

            // NOTE: This is a strange check, one can set it to 1 to bypass.
            if new_cntr.origin_energy_limit <= 0 {
                return Err("origin_energy_limit must be greater than 0".into());
            }

            get_account_energy_limit_with_fixed_ratio(manager, &acct, ctx.fee_limit, call_value)
        } else {
            warn!("use legacy energy limit calculation");
            get_account_energy_limit_with_float_ratio(manager, &acct, ctx.fee_limit, call_value)
        };

        log::debug!("energy_limit => {}", energy_limit);
        ctx.energy_limit = energy_limit;

        // checkTokenValueAndId
        if allow_trc10_transfer {
            // NOTE: also checks allowMultiSig
            if manager.state_db.must_get(&keys::ChainParameter::AllowMultisig) != 0 {
                if call_token_id != 0 && call_token_id <= MIN_TOKEN_ID {
                    return Err("invalid token id range".into());
                }
                if call_token_value > 0 && call_token_id == 0 {
                    return Err("invalid token id & token value".into());
                }
            }
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let new_cntr = self.new_contract.as_ref().unwrap();

        let backend = StateBackend::new(manager, ctx);
        let config = tvm::Config::odyssey_3_7();

        let energy_limit = ctx.energy_limit as usize;

        // new_with_precompile
        let mut executor = tvm::StackExecutor::new(&backend, energy_limit, &config);

        let vm_ctx = tvm::Context {
            // contract address
            address: H160::random(),
            caller: H160::from_slice(&self.owner_address[1..]),
            call_value: new_cntr.call_value.into(),
            call_token_id: self.call_token_id.into(),
            call_token_value: self.call_token_value.into(),
        };

        let code = Rc::new(new_cntr.bytecode.clone());
        let data = Rc::default();

        let mut rt = tvm::Runtime::new(code, data, vm_ctx, &config);
        let exit_reason = executor.execute(&mut rt);

        log::debug!("exit => {:?}", exit_reason);

        let ret_val = rt.machine().return_value();
        log::debug!("return => {:?}", hex::encode(&ret_val));
        let save_code_energy = ret_val.len() * 200;

        log::debug!("consumed gas/energy => {}", energy_limit - executor.gas());
        log::debug!("save code energy => {}", save_code_energy);

        let energy_usage = energy_limit - executor.gas() + save_code_energy;
        log::debug!("energy usage = {}", energy_usage);

        Ok(TransactionResult::success())
    }
}

fn generate_created_contract_address(txn_hash: &H256, owner_address: &Address) -> Address {
    use sha3::Digest;

    let mut hasher = sha3::Keccak256::new();
    hasher.update(txn_hash.as_ref());
    hasher.update(owner_address.as_bytes());
    Address::from_tvm_bytes(&hasher.finalize()[12..])
}

// getAccountEnergyLimitWithFixRatio
fn get_account_energy_limit_with_fixed_ratio(
    manager: &Manager,
    acct: &Account,
    fee_limit: i64,
    call_value: i64,
) -> i64 {
    let energy_price = manager.state_db.must_get(&keys::ChainParameter::EnergyFee);

    let left_energy = EnergyUtil::new(manager).get_left_energy(acct);
    let energy_from_balance = (acct.balance - call_value).max(0) / energy_price;

    let available_energy = left_energy + energy_from_balance;

    let energy_from_fee_limit = fee_limit / energy_price;

    available_energy.min(energy_from_fee_limit)
}

fn get_account_energy_limit_with_float_ratio(
    manager: &Manager,
    acct: &Account,
    fee_limit: i64,
    call_value: i64,
) -> i64 {
    let energy_price = manager.state_db.must_get(&keys::ChainParameter::EnergyFee);

    let left_energy = EnergyUtil::new(manager).get_left_energy(acct);
    let call_value = call_value.max(0);
    let energy_from_balance = (acct.balance - call_value).max(0) / energy_price;

    let energy_from_fee_limit = if acct.amount_for_energy() == 0 {
        fee_limit / energy_price
    } else {
        let energy_limit = EnergyUtil::new(manager).calculate_global_energy_limit(acct);
        // getEnergyFee(totalBalanceForEnergyFreeze, leftEnergyFromFreeze, totalEnergyFromFreeze)
        let left_balance = legacy_get_energy_fee(acct.amount_for_energy(), left_energy, energy_limit);

        if left_balance > fee_limit {
            energy_limit * fee_limit / acct.amount_for_energy()
        } else {
            left_energy + (fee_limit - left_balance) / energy_price
        }
    };
    (left_energy + energy_from_balance).min(energy_from_fee_limit)
}

// getEnergyFee(long callerEnergyUsage, long callerEnergyFrozen, long callerEnergyTotal)
#[inline]
fn legacy_get_energy_fee(energy_usage: i64, frozen_energy: i64, total_energy: i64) -> i64 {
    if total_energy <= 0 {
        0
    } else {
        // TODO: big integer?
        frozen_energy * energy_usage / total_energy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_address_of_create_smart_contract() {
        // This is a transaction from Nile testnet.
        let txn_hash: H256 = "b8e13dee62f8945b0c09790c5842b1c5414cf5853736db9ee2da72ec2388dd53"
            .parse()
            .unwrap();
        let owner_address: Address = "TN21Wx2yoNYiZ7znuQonmZMJnH5Vdfxu78".parse().unwrap();

        let new_contract_address = generate_created_contract_address(&txn_hash, &owner_address);
        assert_eq!(new_contract_address.to_string(), "TCCcBZEdTHmS1NfFtCYfwpjBKeTv515n71");
    }
}
