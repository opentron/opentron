//! Creating smart contracts, calling smart contract methods.

use std::convert::TryFrom;
use std::i64;
use std::rc::Rc;

use ::keys::Address;
use constants::block_version::BlockVersion;
use log::{debug, warn};
use primitive_types::{H160, H256};
use proto::chain::transaction::{result::ContractStatus, Result as TransactionResult};
use proto::contract as contract_pb;
use proto::state::{Account, SmartContract};
use state::keys;
use tvm::{backend::ApplyBackend, ExitError, ExitFatal, ExitReason, TvmUpgrade};

use super::super::super::resource::{EnergyProcessor, EnergyUtil};
use super::super::super::version_fork::ForkController;
use super::super::super::vm::StateBackend;
use super::super::TransactionContext;
use super::BuiltinContractExecutorExt;
use crate::Manager;

const MAX_CONTRACT_NAME_LENGTH: usize = 32;
const MAX_FEE_LIMIT: i64 = 1_000_000_000;
const MIN_TOKEN_ID: i64 = 1_000_000;
const SAVE_CODE_ENERGY_PER_BYTE: usize = 200;

// Create a smart contract and deploy it on chain.
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
        if new_cntr.consume_user_energy_percent < 0 || new_cntr.consume_user_energy_percent > 100 {
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

        // NOTE: VMConfig.getEnergyLimitHardFork is a const false?
        let energy_limit = if ForkController::new(manager)
            .pass_version(BlockVersion::ENERGY_LIMIT())
            .unwrap()
        {
            // old style
            if call_value < 0 {
                return Err("invalid call_value".into());
            }
            if call_token_value < 0 {
                return Err("invalid call_token_value".into());
            }

            if new_cntr.origin_energy_limit < 0 {
                return Err("origin_energy_limit must be greater than 0".into());
            }

            get_account_energy_limit_with_fixed_ratio(manager, &acct, ctx.fee_limit, call_value)
        } else {
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
        let owner_address = Address::try_from(&self.owner_address).unwrap();
        let cntr_address = generate_created_contract_address(&ctx.transaction_hash, &owner_address);

        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_address));

        // Routine to handle smart contract creation:
        // . create contract account
        // . create smart contract
        // . save code if before AllowTvmConstantinopleUpgrade
        // . transfer TRX and TRC10
        // . execute opcode in VM (vm.play)
        // . save code if after AllowTvmConstantinopleUpgrade

        // If contract creation is failed, all creation will be discarded.
        manager.new_layer();

        let mut cntr_acct = Account::new_contract_account(manager.latest_block_timestamp());
        let mut cntr = self.new_contract.as_ref().unwrap().clone();
        cntr.contract_address = cntr_address.as_bytes().to_vec();

        let call_value = new_cntr.call_value;
        if call_value > 0 {
            if owner_acct.adjust_balance(-call_value).is_err() {
                return Err(format!(
                    "insufficient balance, balance={} required={}",
                    owner_acct.balance, call_value
                )); // validate error
            }
            cntr_acct.adjust_balance(call_value).unwrap();
        }
        if self.call_token_value > 0 {
            if owner_acct
                .adjust_token_balance(self.call_token_id, -self.call_token_value)
                .is_err()
            {
                return Err("insufficient token balance".into()); // validate error
            }
            cntr_acct
                .adjust_token_balance(self.call_token_id, self.call_token_value)
                .unwrap();
        }

        manager
            .state_db
            .put_key(keys::Account(cntr_address), cntr_acct)
            .unwrap();
        manager
            .state_db
            .put_key(keys::Account(owner_address), owner_acct)
            .unwrap();
        manager.state_db.put_key(keys::Contract(cntr_address), cntr).unwrap();

        let allow_tvm_constantinople = manager
            .state_db
            .must_get(&keys::ChainParameter::AllowTvmConstantinopleUpgrade) !=
            0;
        if !allow_tvm_constantinople {
            let code = legacy_get_runtime_code(&new_cntr.bytecode);
            log::debug!("save legacy code size => {}", code.len());
            manager
                .state_db
                .put_key(keys::ContractCode(cntr_address), code.to_vec())
                .unwrap();
        }

        // execution
        let energy_limit = ctx.energy_limit as usize;

        let upgrade = get_current_tvm_upgrade(manager);
        let precompile = upgrade.precompile();
        let config = upgrade.to_tvm_config();

        let mut backend = StateBackend::new(owner_address, manager, ctx);
        let mut executor = tvm::StackExecutor::new_with_precompile(&backend, energy_limit, &config, precompile);

        let vm_ctx = tvm::Context {
            // contract address
            address: H160::from_slice(cntr_address.as_tvm_bytes()),
            caller: H160::from_slice(owner_address.as_tvm_bytes()),
            call_value: new_cntr.call_value.into(),
            call_token_id: self.call_token_id.into(),
            call_token_value: self.call_token_value.into(),
        };

        let code = Rc::new(new_cntr.bytecode.clone());
        let data = Rc::default();

        let mut rt = tvm::Runtime::new(code, data, vm_ctx, &config);
        let mut exit_reason = executor.execute(&mut rt);
        let mut used_energy = executor.used_gas();
        let ret_val = rt.machine().return_value();

        let (applies, logs) = executor.deconstruct();

        let save_code_energy = ret_val.len() * SAVE_CODE_ENERGY_PER_BYTE;
        let remain_energy = energy_limit - used_energy;

        if save_code_energy > remain_energy {
            // use up
            used_energy = energy_limit;
            exit_reason = ExitReason::Error(ExitError::OutOfGas);
            log::warn!(
                "insufficient energy to save code! energy={} exit={:?}",
                used_energy,
                exit_reason
            );
        }

        if exit_reason.is_succeed() {
            backend.apply(applies, logs, false);
            if allow_tvm_constantinople {
                manager
                    .state_db
                    .put_key(keys::ContractCode(cntr_address), ret_val.clone())
                    .unwrap();
            }
        } else {
            drop(backend);
            drop(applies);
            drop(logs);
            manager.rollback_layers(1);
        }
        ctx.result = ret_val;

        let energy_usage = if exit_reason.is_succeed() {
            (used_energy + save_code_energy) as i64
        } else if exit_reason.is_fatal() {
            energy_limit as i64
        } else {
            used_energy as i64
        };
        ctx.energy = energy_usage;
        log::debug!(
            "energy usage: {}/{} vm_energy={}",
            energy_usage,
            energy_limit,
            used_energy,
        );
        // consume energy
        EnergyProcessor::new(manager).consume(
            owner_address,
            owner_address,
            energy_usage,
            0,
            new_cntr.origin_energy_limit,
            ctx,
        )?;
        let mut ret = TransactionResult::success();
        ret.contract_status = exit_reason.as_contrat_status() as i32;
        debug!(
            "deploy contract, vm_exit_reason={:?} contract_status={:?}",
            exit_reason,
            exit_reason.as_contrat_status()
        );
        Ok(ret)
    }
}

// Calling smart contract method.
impl BuiltinContractExecutorExt for contract_pb::TriggerSmartContract {
    fn validate(&self, manager: &Manager, ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        if state_db.must_get(&keys::ChainParameter::AllowTvm) == 0 {
            return Err("TVM is disabled".into());
        }

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;
        let cntr_address = Address::try_from(&self.contract_address).map_err(|_| "invalid contract address")?;

        let maybe_cntr = manager
            .state_db
            .get(&keys::Contract(cntr_address))
            .map_err(|_| "db query error")?;
        if maybe_cntr.is_none() {
            return Err("contract not found".into());
        }
        let cntr = maybe_cntr.unwrap();
        let origin_address = *Address::from_bytes(&cntr.origin_address);

        let call_value = self.call_value;
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

        if ForkController::new(manager).pass_version(BlockVersion::ENERGY_LIMIT())? {
            if call_value < 0 {
                return Err("invalid call_value".into());
            }
            if call_token_value < 0 {
                return Err("invalid call_token_value".into());
            }
        }

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

        let code = manager
            .state_db
            .get(&keys::ContractCode(cntr_address))
            .map_err(|_| "db query error")?;
        if code.is_some() && !code.as_ref().unwrap().is_empty() {
            if ctx.fee_limit < 0 || ctx.fee_limit > MAX_FEE_LIMIT {
                return Err("invalid fee_limit".into());
            }

            // TODO: check constant call
            let caller_acct = manager
                .state_db
                .get(&keys::Account(owner_address))
                .map_err(|_| "db query error")?
                .ok_or_else(|| "owner account is not on chain")?;
            let origin_acct = manager.state_db.must_get(&keys::Account(origin_address));

            let energy_limit = if owner_address == origin_address {
                get_account_energy_limit(manager, &caller_acct, ctx.fee_limit, call_value)
            } else {
                get_total_energy_limit(manager, &caller_acct, &origin_acct, &cntr, ctx.fee_limit, call_value)
            };
            ctx.energy_limit = energy_limit;
        } else {
            warn!("contract code is empty!");
        }
        Ok(())
    }

    fn execute(&self, manager: &mut Manager, ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_address = Address::try_from(&self.owner_address).unwrap();
        let cntr_address = Address::try_from(&self.contract_address).unwrap();

        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_address));
        let mut cntr_acct = manager.state_db.must_get(&keys::Account(cntr_address));

        let cntr = manager.state_db.must_get(&keys::Contract(cntr_address));
        let origin_address = Address::try_from(&cntr.origin_address).unwrap();

        let energy_limit = ctx.energy_limit as usize;

        // NOTE: OutOfTime is a design flaw, skip VM and accept the result.
        if ctx.contract_status == ContractStatus::OutOfTime {
            warn!("contract status is OutOfTime, skip!");
            let energy_usage = energy_limit as i64;
            ctx.energy = energy_usage;
            log::debug!("energy usage: {} (all)", energy_usage);
            EnergyProcessor::new(manager).consume(
                owner_address,
                origin_address,
                energy_usage,
                cntr.consume_user_energy_percent,
                cntr.origin_energy_limit,
                ctx,
            )?;

            let mut ret = TransactionResult::success();
            ret.contract_status = ContractStatus::OutOfTime as i32;
            debug!("execute contract failed, OutOfTime");
            return Ok(ret);
        }

        manager.new_layer();

        let mut has_transfer = false;
        // transfer TRX
        if self.call_value > 0 {
            has_transfer = true;
            if owner_acct.adjust_balance(-self.call_value).is_err() {
                return Err(format!(
                    "insufficient balance, balance={} required={}",
                    owner_acct.balance, self.call_value
                )); // validate error
            }
            cntr_acct.adjust_balance(self.call_value).unwrap();
        }
        // transfer TRC10
        let mut call_token_value = 0_i64;
        let mut call_token_id = 0_i64;
        // See-also: https://github.com/opentron/opentron/issues/41
        // NOTE: Ignores value if allow_trc10_transfer is off.
        if self.call_token_value > 0 &&
            manager
                .state_db
                .must_get(&keys::ChainParameter::AllowTvmTransferTrc10Upgrade) !=
                0
        {
            has_transfer = true;
            call_token_value = self.call_token_value;
            call_token_id = self.call_token_id;
            if owner_acct
                .adjust_token_balance(call_token_id, -call_token_value)
                .is_err()
            {
                return Err("insufficient token balance".into()); // validate error
            }
            cntr_acct.adjust_token_balance(call_token_id, call_token_value).unwrap();
        }

        if has_transfer {
            manager
                .state_db
                .put_key(keys::Account(cntr_address), cntr_acct)
                .unwrap();
            manager
                .state_db
                .put_key(keys::Account(owner_address), owner_acct)
                .unwrap();
        }

        // build execution context
        let code = manager
            .state_db
            .get(&keys::ContractCode(cntr_address))
            .map_err(|_| "db query error")?
            .unwrap_or_default();
        let code = Rc::new(code);
        let data = Rc::new(self.data.to_vec());
        debug!("calling data = {:?}", hex::encode(&self.data));

        let upgrade = get_current_tvm_upgrade(manager);
        let precompile = upgrade.precompile();
        let config = upgrade.to_tvm_config();

        let mut backend = StateBackend::new(owner_address, manager, ctx);
        let mut executor = tvm::StackExecutor::new_with_precompile(&backend, energy_limit, &config, precompile);

        let vm_ctx = tvm::Context {
            // contract address
            address: H160::from_slice(cntr_address.as_tvm_bytes()),
            caller: H160::from_slice(owner_address.as_tvm_bytes()),
            call_value: self.call_value.into(),
            call_token_id: call_token_id.into(),
            call_token_value: call_token_value.into(),
        };

        let mut rt = tvm::Runtime::new(code, data, vm_ctx, &config);
        let exit_reason = executor.execute(&mut rt);
        let used_energy = executor.used_gas();
        let ret_val = rt.machine().return_value();

        let (applies, logs) = executor.deconstruct();

        if exit_reason.is_succeed() {
            backend.apply(applies, logs, false);
        } else {
            drop(backend);
            drop(applies);
            drop(logs);
            manager.rollback_layers(1);
        }

        if !ret_val.is_empty() {
            debug!("return value: {:?}", hex::encode(&ret_val));
            ctx.result = ret_val;
        }

        let energy_usage = if exit_reason.is_fatal() {
            energy_limit as i64
        } else {
            used_energy as i64
        };
        ctx.energy = energy_usage;
        // consume energy
        EnergyProcessor::new(manager).consume(
            owner_address,
            origin_address,
            energy_usage,
            cntr.consume_user_energy_percent,
            cntr.origin_energy_limit,
            ctx,
        )?;
        let mut ret = TransactionResult::success();
        ret.contract_status = exit_reason.as_contrat_status() as i32;
        debug!(
            "trigger contract, vm_exit_reason={:?} contract_status={:?}",
            exit_reason,
            exit_reason.as_contrat_status()
        );
        Ok(ret)
    }
}

// Update a contract's `consume_user_energy_percent` setting.
impl BuiltinContractExecutorExt for contract_pb::UpdateSettingContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        if state_db.must_get(&keys::ChainParameter::AllowTvm) == 0 {
            return Err("TVM is disabled".into());
        }

        if self.consume_user_energy_percent < 0 || self.consume_user_energy_percent > 100 {
            return Err("percent must be in the range [0, 100]".into());
        }

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;
        let cntr_address = Address::try_from(&self.contract_address).map_err(|_| "invalid contract_address")?;

        let maybe_cntr = manager
            .state_db
            .get(&keys::Contract(cntr_address))
            .map_err(|_| "db query error")?;
        if maybe_cntr.is_none() {
            return Err("contract not found".into());
        }
        let cntr = maybe_cntr.unwrap();
        let origin_address = *Address::from_bytes(&cntr.origin_address);

        if origin_address != owner_address {
            return Err("owner address is not the origin creator of contract".into());
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let cntr_address = Address::try_from(&self.contract_address).unwrap();
        let mut cntr = manager.state_db.must_get(&keys::Contract(cntr_address));

        cntr.consume_user_energy_percent = self.consume_user_energy_percent;
        manager
            .state_db
            .put_key(keys::Contract(cntr_address), cntr)
            .map_err(|_| "db insert error")?;

        Ok(TransactionResult::success())
    }
}

// Update a contract's `origin_energy_limit` setting.
impl BuiltinContractExecutorExt for contract_pb::UpdateEnergyLimitContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        if state_db.must_get(&keys::ChainParameter::AllowTvm) == 0 {
            return Err("TVM is disabled".into());
        }

        if self.origin_energy_limit <= 0 {
            return Err("origin energy limit must be greater than 0".into());
        }

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;
        let cntr_address = Address::try_from(&self.contract_address).map_err(|_| "invalid contract_address")?;

        let cntr = manager
            .state_db
            .get(&keys::Contract(cntr_address))
            .map_err(|_| "db query error")?
            .ok_or_else(|| "contract not found on chain")?;

        let origin_address = *Address::from_bytes(&cntr.origin_address);

        if origin_address != owner_address {
            return Err("owner address is not the origin creator of contract".into());
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let cntr_address = Address::try_from(&self.contract_address).unwrap();
        let mut cntr = manager.state_db.must_get(&keys::Contract(cntr_address));

        cntr.origin_energy_limit = self.origin_energy_limit;
        manager
            .state_db
            .put_key(keys::Contract(cntr_address), cntr)
            .map_err(|_| "db insert error")?;

        Ok(TransactionResult::success())
    }
}

// Update a contract's ABI.
//
// NOTE: This is a design flaw, to deceive oneself.
impl BuiltinContractExecutorExt for contract_pb::ClearAbiContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        if state_db.must_get(&keys::ChainParameter::AllowTvm) == 0 {
            return Err("TVM is disabled".into());
        }

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;
        let cntr_address = Address::try_from(&self.contract_address).map_err(|_| "invalid contract_address")?;

        let maybe_cntr = manager
            .state_db
            .get(&keys::Contract(cntr_address))
            .map_err(|_| "db query error")?;
        if maybe_cntr.is_none() {
            return Err("contract not found".into());
        }
        let cntr = maybe_cntr.unwrap();
        let origin_address = *Address::from_bytes(&cntr.origin_address);

        if origin_address != owner_address {
            return Err("owner address is not the origin creator of contract".into());
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let cntr_address = Address::try_from(&self.contract_address).unwrap();
        let mut cntr = manager.state_db.must_get(&keys::Contract(cntr_address));

        cntr.abi.as_mut().map(|abi| abi.entries = vec![]);
        manager
            .state_db
            .put_key(keys::Contract(cntr_address), cntr)
            .map_err(|_| "db insert error")?;

        Ok(TransactionResult::success())
    }
}

// Dry run a TriggerSmartContract. Use 1 db layer and rollback.
pub fn execute_smart_contract(
    manager: &mut Manager,
    trigger: &contract_pb::TriggerSmartContract,
    ctx: &mut TransactionContext,
) -> Result<TransactionResult, String> {
    let owner_address = Address::try_from(&trigger.owner_address).map_err(|_| "invalid owner address")?;
    let cntr_address = Address::try_from(&trigger.contract_address).map_err(|_| "invalid contract address")?;

    let energy_limit = ctx.energy_limit as usize;

    manager.new_layer();

    // transfer
    if trigger.call_value > 0 || trigger.call_token_value > 0 {
        let mut owner_acct = manager
            .state_db
            .get(&keys::Account(owner_address))
            .unwrap()
            .ok_or_else(|| "owner account not found")?;
        let mut cntr_acct = manager
            .state_db
            .get(&keys::Account(cntr_address))
            .unwrap()
            .ok_or_else(|| "contract not found")?;
        if trigger.call_value > 0 {
            if owner_acct.adjust_balance(-trigger.call_value).is_err() {
                return Err(format!(
                    "insufficient balance, balance={} required={}",
                    owner_acct.balance, trigger.call_value
                ));
            }
            cntr_acct.adjust_balance(trigger.call_value).unwrap();
        }
        if trigger.call_token_value > 0 {
            if owner_acct
                .adjust_token_balance(trigger.call_token_id, -trigger.call_token_value)
                .is_err()
            {
                return Err("insufficient token balance".into());
            }
            cntr_acct
                .adjust_token_balance(trigger.call_token_id, trigger.call_token_value)
                .unwrap();
        }
        manager
            .state_db
            .put_key(keys::Account(cntr_address), cntr_acct)
            .unwrap();
        manager
            .state_db
            .put_key(keys::Account(owner_address), owner_acct)
            .unwrap();
    }

    // build execution context
    let code = manager
        .state()
        .get(&keys::ContractCode(cntr_address))
        .map_err(|_| "db query error")?
        .unwrap_or_default();
    let code = Rc::new(code);
    let data = Rc::new(trigger.data.to_vec());

    let upgrade = get_current_tvm_upgrade(manager);
    let precompile = upgrade.precompile();
    let config = upgrade.to_tvm_config();

    let mut backend = StateBackend::new(owner_address, manager, ctx);
    let mut executor = tvm::StackExecutor::new_with_precompile(&backend, energy_limit, &config, precompile);

    let vm_ctx = tvm::Context {
        // contract address
        address: H160::from_slice(cntr_address.as_tvm_bytes()),
        caller: H160::from_slice(owner_address.as_tvm_bytes()),
        call_value: trigger.call_value.into(),
        call_token_id: trigger.call_token_id.into(),
        call_token_value: trigger.call_token_value.into(),
    };

    let mut rt = tvm::Runtime::new(code, data, vm_ctx, &config);
    let exit_reason = executor.execute(&mut rt);
    let used_energy = executor.used_gas();
    let ret_val = rt.machine().return_value();

    let (applies, logs) = executor.deconstruct();
    backend.apply(applies, logs, false);
    drop(backend);

    manager.rollback_layers(1);

    if !ret_val.is_empty() {
        debug!("return value: {:?}", hex::encode(&ret_val));
        ctx.result = ret_val;
    }

    let energy_usage = if exit_reason.is_fatal() {
        energy_limit as i64
    } else {
        used_energy as i64
    };
    // NOTE: energy_usage, origin_usage is for actual consumption.
    // energy is for total energy used.
    ctx.energy = energy_usage;
    let mut ret = TransactionResult::success();
    ctx.contract_status = exit_reason.as_contrat_status();
    ret.contract_status = exit_reason.as_contrat_status() as i32;
    debug!(
        "trigger contract, vm_exit_reason={:?} contract_status={:?}",
        exit_reason,
        exit_reason.as_contrat_status()
    );
    Ok(ret)
}

// NOTE: This is a really bad implementation.
// It preserves constructor parameters and is inconsistent with save code energy.
// Anyway, we are not the inventors of bugs, instead, we are copiers.
fn legacy_get_runtime_code(deploy_code: &[u8]) -> &[u8] {
    const RETURN: u8 = 0xf3;
    const STOP: u8 = 0x00;
    const PUSH1: u8 = 0x60;
    const PUSH32: u8 = 0x7f;

    let mut pos = 0;
    while pos + 1 < deploy_code.len() {
        let op = deploy_code[pos];

        if op == RETURN && deploy_code[pos + 1] == STOP {
            return &deploy_code[pos + 2..];
        }
        if op >= PUSH1 && op <= PUSH32 {
            pos += (op - PUSH1) as usize + 1;
        }
        pos += 1;
    }
    return &[0u8; 32];
}

fn generate_created_contract_address(txn_hash: &H256, owner_address: &Address) -> Address {
    use sha3::Digest;

    let mut hasher = sha3::Keccak256::new();
    hasher.update(txn_hash.as_ref());
    hasher.update(owner_address.as_bytes());
    Address::from_tvm_bytes(&hasher.finalize()[12..])
}

#[inline]
fn get_account_energy_limit(manager: &Manager, acct: &Account, fee_limit: i64, call_value: i64) -> i64 {
    if ForkController::new(manager)
        .pass_version(BlockVersion::ENERGY_LIMIT())
        .unwrap()
    {
        get_account_energy_limit_with_fixed_ratio(manager, &acct, fee_limit, call_value)
    } else {
        get_account_energy_limit_with_float_ratio(manager, &acct, fee_limit, call_value)
    }
}

/// getAccountEnergyLimitWithFixRatio
fn get_account_energy_limit_with_fixed_ratio(
    manager: &Manager,
    acct: &Account,
    fee_limit: i64,
    call_value: i64,
) -> i64 {
    let energy_price = manager.state_db.must_get(&keys::ChainParameter::EnergyFee);

    let left_energy = EnergyUtil::new(manager).get_left_frozen_energy(acct);
    let energy_from_balance = (acct.balance - call_value).max(0) / energy_price;

    let available_energy = left_energy + energy_from_balance;

    let energy_from_fee_limit = fee_limit / energy_price;

    available_energy.min(energy_from_fee_limit)
}

// getEnergyFee(long callerEnergyUsage, long callerEnergyFrozen, long callerEnergyTotal)
#[inline]
fn legacy_get_energy_fee(energy_usage: i64, frozen_energy: i64, total_energy: i64) -> i64 {
    if total_energy <= 0 {
        0
    } else {
        ((frozen_energy as i128) * (energy_usage as i128) / (total_energy as i128)) as i64
    }
}

/// getAccountEnergyLimitWithFloatRatio, before ENERGY_LIMIT fork.
fn get_account_energy_limit_with_float_ratio(
    manager: &Manager,
    acct: &Account,
    fee_limit: i64,
    call_value: i64,
) -> i64 {
    let energy_price = manager.state_db.must_get(&keys::ChainParameter::EnergyFee);

    // getAccountLeftEnergyFromFreeze
    let left_energy_from_freeze = EnergyUtil::new(manager).get_left_frozen_energy(acct);
    let energy_from_left_balance = (acct.balance - call_value.max(0)).max(0) / energy_price;
    let amount_for_energy = acct.amount_for_energy();

    let energy_from_fee_limit = if amount_for_energy == 0 {
        fee_limit / energy_price
    } else {
        let energy_limit = EnergyUtil::new(manager).calculate_global_energy_limit(acct);
        // getEnergyFee(totalBalanceForEnergyFreeze, leftEnergyFromFreeze, totalEnergyFromFreeze)
        // getEnergyFee(callerEnergyUsage, callerEnergyFrozen, callerEnergyTotal) ->
        //   callerEnergyFrozen * callerEnergyUsage / callerEnergyTotal
        let left_balance_for_freeze = legacy_get_energy_fee(amount_for_energy, left_energy_from_freeze, energy_limit);

        if left_balance_for_freeze >= fee_limit {
            ((energy_limit as i128) * (fee_limit as i128) / (amount_for_energy as i128)) as i64
        } else {
            left_energy_from_freeze + (fee_limit - left_balance_for_freeze) / energy_price
        }
    };
    i64::min(
        left_energy_from_freeze + energy_from_left_balance,
        energy_from_fee_limit,
    )
}

// getTotalEnergyLimit
#[inline]
fn get_total_energy_limit(
    manager: &Manager,
    caller: &Account,
    origin: &Account,
    cntr: &SmartContract,
    fee_limit: i64,
    call_value: i64,
) -> i64 {
    // TODO: Can origin be null? (use getAccountEnergyLimitWithFixRatio)
    // if block.number > BlockNumForEneryLimit
    if ForkController::new(manager)
        .pass_version(BlockVersion::ENERGY_LIMIT())
        .unwrap()
    {
        get_total_energy_limit_with_fixed_ratio(manager, caller, origin, cntr, fee_limit, call_value)
    } else {
        get_total_energy_limit_with_float_ratio(manager, caller, origin, cntr, fee_limit, call_value)
    }
}

/// getTotalEnergyLimitWithFixRatio
fn get_total_energy_limit_with_fixed_ratio(
    manager: &Manager,
    caller: &Account,
    origin: &Account,
    cntr: &SmartContract,
    fee_limit: i64,
    call_value: i64,
) -> i64 {
    let caller_energy_limit = get_account_energy_limit_with_fixed_ratio(manager, caller, fee_limit, call_value);
    let consume_user_energy_percent = cntr.consume_user_energy_percent;
    assert!(cntr.origin_energy_limit >= 0);

    let origin_energy_left = EnergyUtil::new(manager).get_left_frozen_energy(origin);
    let origin_energy_limit = if consume_user_energy_percent > 0 {
        assert!(consume_user_energy_percent <= 100);
        i64::min(
            caller_energy_limit * (100 - consume_user_energy_percent) / consume_user_energy_percent,
            i64::min(origin_energy_left, cntr.origin_energy_limit),
        )
    } else {
        i64::min(origin_energy_left, cntr.origin_energy_limit)
    };

    caller_energy_limit + origin_energy_limit
}

/// getTotalEnergyLimitWithFloatRatio, before ENERGY_LIMIT fork.
fn get_total_energy_limit_with_float_ratio(
    manager: &Manager,
    caller: &Account,
    origin: &Account,
    cntr: &SmartContract,
    fee_limit: i64,
    call_value: i64,
) -> i64 {
    let caller_energy_limit = get_account_energy_limit_with_float_ratio(manager, caller, fee_limit, call_value);
    let user_energy_percent = cntr.consume_user_energy_percent;
    let origin_energy_percent = 100 - user_energy_percent;

    // creatorEnergyFromFreeze
    let origin_energy_limit = EnergyUtil::new(manager).get_left_frozen_energy(origin);

    // orgin/caller > origin_percent/user_percent
    if origin_energy_limit * user_energy_percent > caller_energy_limit * origin_energy_percent {
        caller_energy_limit * 100 / user_energy_percent
    } else {
        caller_energy_limit + origin_energy_limit
    }
}

// TODO: Optimize and cache values.
fn get_current_tvm_upgrade(manager: &Manager) -> TvmUpgrade {
    TvmUpgrade {
        asset_transfer: manager
            .state_db
            .must_get(&keys::ChainParameter::AllowTvmTransferTrc10Upgrade) !=
            0,
        constantinople: manager
            .state_db
            .must_get(&keys::ChainParameter::AllowTvmConstantinopleUpgrade) !=
            0,
        solidity059: manager
            .state_db
            .must_get(&keys::ChainParameter::AllowTvmSolidity059Upgrade) !=
            0,
        shielded: manager
            .state_db
            .must_get(&keys::ChainParameter::AllowTvmShieldedUpgrade) !=
            0,
        stake: false,
        istanbul: false,
        asset_issue: false,
        multisig: manager.state_db.must_get(&keys::ChainParameter::AllowMultisig) != 0,
    }
}

/// Helper for `tvm::ExitReason`.
trait ExitReasonExt {
    /// Convert VM exit reason to ContractStatus.
    fn as_contrat_status(&self) -> ContractStatus;
    /// Fatal spends all remain energy.
    fn is_fatal(&self) -> bool;
    /// Is exit reason `ExitReason::Succeed`.
    fn is_succeed(&self) -> bool;
}

impl ExitReasonExt for ExitReason {
    fn as_contrat_status(&self) -> ContractStatus {
        match *self {
            ExitReason::Succeed(_) => ContractStatus::Success,
            ExitReason::Error(ExitError::OutOfGas) => ContractStatus::OutOfEnergy,
            ExitReason::Error(ExitError::IllegalOperation) => ContractStatus::IllegalOperation,
            ExitReason::Fatal(ExitFatal::CallErrorAsFatal(ExitError::TransferException)) |
            ExitReason::Error(ExitError::TransferException) => ContractStatus::TransferFailed,
            ExitReason::Fatal(ExitFatal::CallErrorAsFatal(ExitError::Unknown)) |
            ExitReason::Error(ExitError::Unknown) => ContractStatus::Unknown,
            ExitReason::Revert(_) => ContractStatus::Revert,
            _ => unimplemented!("TODO: handle code {:?}", self),
        }
    }

    fn is_fatal(&self) -> bool {
        match *self {
            ExitReason::Fatal(ExitFatal::CallErrorAsFatal(ExitError::TransferException)) |
            ExitReason::Error(ExitError::TransferException) |
            ExitReason::Fatal(ExitFatal::CallErrorAsFatal(ExitError::Unknown)) |
            ExitReason::Error(ExitError::Unknown) => true,
            _ => false,
        }
    }

    fn is_succeed(&self) -> bool {
        match *self {
            ExitReason::Succeed(_) => true,
            _ => false,
        }
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
