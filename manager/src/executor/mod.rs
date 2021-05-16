//! Transaction executor.

use std::collections::HashMap;
use std::str;

use ::keys::{b58encode_check, Address};
use chain::{IndexedBlockHeader, IndexedTransaction};
use log::{debug, error, warn};
use primitive_types::H256;
use prost::Message;
use proto::chain::{transaction::result::ContractStatus, transaction::Result as TransactionResult, ContractType};
use proto::common::ResourceCode;
use proto::contract as contract_pb;
use proto::state::{ResourceReceipt, TransactionLog, TransactionReceipt};
use proto::ContractExt;
use state::keys;

use self::actuators::BuiltinContractExecutorExt;
use crate::resource::BandwidthProcessor;
use crate::Manager;

pub mod actuators;

pub struct TransactionContext<'a> {
    // Transaction static context.
    pub block_header: &'a IndexedBlockHeader,
    pub transaction_hash: H256,
    // Bandwidth, including account creation.
    pub bandwidth_usage: i64,
    pub bandwidth_fee: i64,
    // Handled by actuator.
    pub contract_fee: i64,
    pub multisig_fee: i64,
    // NOTE: Account creation fee will overwrite bandwidth fee.
    // pub account_creation_fee: i64,
    // Set by actuator.valide().
    pub new_account_created: bool,
    pub withdrawal_amount: i64,
    pub unfrozen_amount: i64,
    pub fee_limit: i64,
    pub energy: i64,
    pub energy_limit: i64,
    pub energy_usage: i64,
    pub origin_energy_usage: i64,
    pub energy_fee: i64,
    pub result: Vec<u8>,
    pub logs: Vec<TransactionLog>,
    pub contract_status: ContractStatus,
}

impl<'a> TransactionContext<'a> {
    pub fn new<'b>(
        block_header: &'b IndexedBlockHeader,
        transaction: &'b IndexedTransaction,
    ) -> TransactionContext<'b> {
        TransactionContext {
            block_header,
            transaction_hash: transaction.hash,
            bandwidth_usage: 0,
            bandwidth_fee: 0,
            contract_fee: 0,
            multisig_fee: 0,
            new_account_created: false,
            withdrawal_amount: 0,
            unfrozen_amount: 0,
            fee_limit: transaction.raw.raw_data.as_ref().unwrap().fee_limit,
            // will be filled while validating
            energy: 0,
            energy_limit: 0,
            energy_usage: 0,
            origin_energy_usage: 0,
            energy_fee: 0,
            result: vec![],
            logs: vec![],
            contract_status: ContractStatus::default(),
        }
    }

    pub fn dummy<'b>(block_header: &'b IndexedBlockHeader) -> TransactionContext<'b> {
        TransactionContext {
            block_header,
            transaction_hash: H256::zero(),
            bandwidth_usage: 0,
            bandwidth_fee: 0,
            contract_fee: 0,
            multisig_fee: 0,
            new_account_created: false,
            withdrawal_amount: 0,
            unfrozen_amount: 0,
            fee_limit: 1000_000_000,
            // will be filled while validating
            energy: 0,
            energy_limit: 0,
            energy_usage: 0,
            origin_energy_usage: 0,
            energy_fee: 0,
            result: vec![],
            logs: vec![],
            contract_status: ContractStatus::default(),
        }
    }
}

impl From<TransactionContext<'_>> for TransactionReceipt {
    fn from(ctx: TransactionContext) -> TransactionReceipt {
        let mut receipt = TransactionReceipt {
            success: true,

            hash: ctx.transaction_hash.as_ref().to_vec(),
            block_number: ctx.block_header.number(),
            block_timestamp: ctx.block_header.timestamp(),

            resource_receipt: Some(ResourceReceipt {
                bandwidth_usage: ctx.bandwidth_usage,
                bandwidth_fee: ctx.bandwidth_fee,
                contract_fee: ctx.contract_fee,
                ..Default::default()
            }),
            ..Default::default()
        };

        // TODO: distinguish by builtin contract type
        if ctx.energy_limit > 0 {
            receipt.resource_receipt.as_mut().map(|r| {
                r.energy = ctx.energy;
                r.energy_usage = ctx.energy_usage;
                r.energy_fee = ctx.energy_fee;
                r.origin_energy_usage = ctx.origin_energy_usage;
            });
            receipt.vm_result = ctx.result;
            receipt.vm_status = ctx.contract_status as i32;
            receipt.vm_logs = ctx.logs;
        }
        // misc
        receipt.withdrawal_amount = ctx.withdrawal_amount;
        receipt.unfrozen_amount = ctx.unfrozen_amount;

        receipt
    }
}

impl ::std::fmt::Debug for TransactionContext<'_> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let mut dbg = f.debug_struct("TransactionContext");
        dbg.field("block", &self.block_header.number())
            .field("bandwidth_usage", &self.bandwidth_usage)
            .field("bandwidth_fee", &self.bandwidth_fee)
            .field("contract_fee", &self.contract_fee)
            .field("multisig_fee", &self.multisig_fee)
            .field("withdrawal_amount", &self.withdrawal_amount)
            .field("unfrozen_amount", &self.unfrozen_amount)
            .field("new_account_created", &self.new_account_created);

        // smart contract
        if self.energy_limit > 0 {
            dbg.field("energy_limit", &self.energy_limit)
                .field("energy", &self.energy)
                .field("energy_usage", &self.energy_usage)
                .field("origin_energy_usage", &self.origin_energy_usage)
                .field("energy_fee", &self.energy_fee)
                .field("result", &hex::encode(&self.result))
                .field("|logs|", &self.logs.len());
        }
        dbg.finish()
    }
}

/// TransactionTrace + RuntimeImpl.
pub struct TransactionExecutor<'m> {
    manager: &'m mut Manager,
}

impl<'m> TransactionExecutor<'m> {
    pub fn new<'a>(manager: &'a mut Manager) -> TransactionExecutor<'a> {
        TransactionExecutor { manager }
    }

    pub fn execute_smart_contract(
        &mut self,
        trigger: &contract_pb::TriggerSmartContract,
        energy_limit: i64,
    ) -> Result<TransactionReceipt, String> {
        debug!(
            "=> Execute Smart Contract, owner={} contract={}",
            b58encode_check(&trigger.owner_address()),
            b58encode_check(&trigger.contract_address),
        );

        let next_block_number = self.manager.latest_block_number() + 1;

        let block_header = IndexedBlockHeader::dummy(
            next_block_number,
            self.manager.latest_block_timestamp() + constants::BLOCK_PRODUCING_INTERVAL,
        );

        let mut ctx = TransactionContext::dummy(&block_header);
        ctx.energy_limit = energy_limit;

        let exec_result = self::actuators::smart_contract::execute_smart_contract(self.manager, &trigger, &mut ctx)?;
        debug!("context => {:?}", ctx);
        debug!("result => {:?}", exec_result);
        Ok(ctx.into())
    }

    pub fn execute_and_verify_result(
        &mut self,
        txn: &IndexedTransaction,
        recover_addrs: Vec<Address>,
        block_header: &IndexedBlockHeader,
    ) -> Result<TransactionReceipt, String> {
        let maybe_result = txn.raw.result.get(0);

        let mut ctx = TransactionContext::new(&block_header, &txn);
        let mut exec_result = self.execute_inner(txn, recover_addrs, &mut ctx)?;

        if block_header.version() == 0 {
            /*
            let contract_status = maybe_result
                .and_then(|ret| ContractStatus::from_i32(ret.contract_status))
                .unwrap_or_default();
            */
            //ctx.contract_status = contract_status;
            exec_result.contract_status = ContractStatus::Default as i32;
        }

        // NOTE: vm must be strictly checked.
        if !check_transaction_result(&exec_result, &maybe_result) {
            debug!("result => {:?}", exec_result);
            return Err("result check not passed!".into());
        }
        Ok(ctx.into())
    }

    /// Verifies the transaction, do not run.
    /// Requires rollback.
    pub fn verify(
        &mut self,
        txn: &IndexedTransaction,
        recover_addrs: Vec<Address>,
        block_header: &IndexedBlockHeader,
    ) -> Result<(), String> {
        let cntr = txn
            .raw
            .raw_data
            .as_ref()
            .ok_or_else(|| "empty raw_data".to_owned())?
            .contract
            .as_ref()
            .ok_or_else(|| "empty inner contract".to_owned())?;
        let cntr_type = ContractType::from_i32(cntr.r#type)
            .ok_or_else(|| format!("unhandled system contract type code: {}", cntr.r#type))?;

        let permission_id = cntr.permission_id;

        let mut ctx = TransactionContext::new(&block_header, &txn);
        let ctx = &mut ctx;

        match cntr_type {
            ContractType::TransferContract => {
                let cntr = contract_pb::TransferContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::ProposalCreateContract => {
                let cntr = contract_pb::ProposalCreateContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::ProposalApproveContract => {
                let cntr = contract_pb::ProposalApproveContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::ProposalDeleteContract => {
                let cntr = contract_pb::ProposalDeleteContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::WitnessCreateContract => {
                let cntr = contract_pb::WitnessCreateContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::WitnessUpdateContract => {
                let cntr = contract_pb::WitnessUpdateContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::UpdateBrokerageContract => {
                let cntr = contract_pb::UpdateBrokerageContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::FreezeBalanceContract => {
                let cntr = contract_pb::FreezeBalanceContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::UnfreezeBalanceContract => {
                let cntr = contract_pb::UnfreezeBalanceContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::VoteWitnessContract => {
                let cntr = contract_pb::VoteWitnessContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::AssetIssueContract => {
                let cntr = contract_pb::AssetIssueContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::UpdateAssetContract => {
                let cntr = contract_pb::UpdateAssetContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::UnfreezeAssetContract => {
                let cntr = contract_pb::UnfreezeAssetContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::TransferAssetContract => {
                let cntr = contract_pb::TransferAssetContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::ParticipateAssetIssueContract => {
                let cntr = contract_pb::ParticipateAssetIssueContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::AccountUpdateContract => {
                let cntr = contract_pb::AccountUpdateContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::SetAccountIdContract => {
                let cntr = contract_pb::SetAccountIdContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::AccountCreateContract => {
                let cntr = contract_pb::AccountCreateContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::AccountPermissionUpdateContract => {
                let cntr = contract_pb::AccountPermissionUpdateContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::WithdrawBalanceContract => {
                let cntr = contract_pb::WithdrawBalanceContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::UpdateSettingContract => {
                let cntr = contract_pb::UpdateSettingContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::UpdateEnergyLimitContract => {
                let cntr = contract_pb::UpdateEnergyLimitContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::ClearAbiContract => {
                let cntr = contract_pb::ClearAbiContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            // TVM: Should handle BW first, then remaining can be used for E.
            ContractType::CreateSmartContract => {
                let raw_cntr = &cntr.parameter.as_ref().unwrap().value[..];
                let cntr = contract_pb::CreateSmartContract::decode(raw_cntr).unwrap();
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.validate(self.manager, ctx)?;
            }
            ContractType::TriggerSmartContract => {
                let cntr = contract_pb::TriggerSmartContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.validate(self.manager, ctx)?;
            }
            ContractType::ExchangeCreateContract => {
                let cntr = contract_pb::ExchangeCreateContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::ExchangeWithdrawContract => {
                let cntr = contract_pb::ExchangeWithdrawContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
            }
            ContractType::ExchangeInjectContract => {
                let cntr = contract_pb::ExchangeInjectContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.validate(self.manager, ctx)?;
            }
            ContractType::ExchangeTransactionContract => {
                let cntr = contract_pb::ExchangeTransactionContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.validate(self.manager, ctx)?;
            }
            #[cfg(feature = "nile")]
            ContractType::ShieldedTransferContract => {
                let cntr = contract_pb::ShieldedTransferContract::from_any(
                    cntr.parameter.as_ref().ok_or_else(|| "empty inner Any pb".to_owned())?,
                )
                .ok_or_else(|| "invalid inner Any pb".to_owned())?;

                log::warn!("=> Shielded Transaction, use dummy implementation");
                // NOTE: dummy implementation
                // NOTE: no need to verify signature
                // cntr.validate_signature(permission_id, recover_addrs, self.manager,ctx)?;
                cntr.validate(self.manager, ctx)?;
                // NOTE: Shielded transaction won't consume bandwidth.
            }
            ContractType::ObsoleteVoteAssetContract |
            ContractType::ObsoleteCustomContract |
            ContractType::ObsoleteGetContract => unreachable!("OBSOLETE: {:?}", cntr_type),
            #[allow(unreachable_patterns)]
            _ => unimplemented!("TODO: handle contract type {:?}", cntr_type),
        }
        Ok(())
    }

    pub fn execute(
        &mut self,
        txn: &IndexedTransaction,
        recover_addrs: Vec<Address>,
        block_header: &IndexedBlockHeader,
    ) -> Result<(TransactionResult, TransactionReceipt), String> {
        let mut ctx = TransactionContext::new(&block_header, &txn);
        let exec_result = self.execute_inner(txn, recover_addrs, &mut ctx)?;
        Ok((exec_result, ctx.into()))
    }

    // runtime.execute
    fn execute_inner(
        &mut self,
        txn: &IndexedTransaction,
        recover_addrs: Vec<Address>,
        ctx: &mut TransactionContext,
    ) -> Result<TransactionResult, String> {
        let cntr = txn.raw.raw_data.as_ref().unwrap().contract.as_ref().unwrap();
        let cntr_type = ContractType::from_i32(cntr.r#type).expect("unhandled system contract type");

        let permission_id = cntr.permission_id;

        // NOTE: Routine to handle transactions of builtin contracts:
        //
        // - decode google.Any
        // - multisig verifiy
        // - validate (except bandwidth)
        // - handle bandwidth
        // - handle mutisig fee
        // - execute logic
        //
        // Which is diffent from java-tron:
        //
        // - bandwidth
        // - multisig
        // - runtime.validate
        // - runtime.execute
        //
        // Bandwidth consumption must come before transaction execution,
        // since some type of transaction cause bandwidth usage changes(freeze/unfreeze).
        match cntr_type {
            ContractType::TransferContract => {
                let cntr = contract_pb::TransferContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> transfer from {} to {} with amount {}",
                    b58encode_check(&cntr.owner_address),
                    b58encode_check(&cntr.to_address),
                    cntr.amount
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::ProposalCreateContract => {
                let cntr = contract_pb::ProposalCreateContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> Proposal by {} {:?}",
                    b58encode_check(&cntr.owner_address),
                    cntr.parameters
                        .iter()
                        .map(|(&k, v)| (
                            keys::ChainParameter::from_i32(k as i32).expect(&format!("unknown proposal {}={}", k, v)),
                            v
                        ))
                        .collect::<std::collections::HashMap<_, _>>()
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::ProposalApproveContract => {
                let cntr = contract_pb::ProposalApproveContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> Approve Proposal #{} by {} {}",
                    cntr.proposal_id,
                    b58encode_check(cntr.owner_address()),
                    cntr.is_approval
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::ProposalDeleteContract => {
                let cntr = contract_pb::ProposalDeleteContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> Delete Proposal #{} by {}",
                    cntr.proposal_id,
                    b58encode_check(cntr.owner_address()),
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::WitnessCreateContract => {
                let cntr = contract_pb::WitnessCreateContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> New Witness {} url={:?}",
                    b58encode_check(cntr.owner_address()),
                    str::from_utf8(&cntr.url)
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::WitnessUpdateContract => {
                let cntr = contract_pb::WitnessUpdateContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> Witness Update {} new_url={:?}",
                    b58encode_check(cntr.owner_address()),
                    String::from_utf8(cntr.new_url.clone()),
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::UpdateBrokerageContract => {
                let cntr = contract_pb::UpdateBrokerageContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> Update Witness Brokerage {}: new_brokerage_rate={}",
                    b58encode_check(cntr.owner_address()),
                    cntr.brokerage,
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::FreezeBalanceContract => {
                let cntr = contract_pb::FreezeBalanceContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Freeze Resource {} amount={} resource={:?}",
                    b58encode_check(cntr.owner_address()),
                    cntr.frozen_balance,
                    ResourceCode::from_i32(cntr.resource).unwrap()
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::UnfreezeBalanceContract => {
                let cntr = contract_pb::UnfreezeBalanceContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                if cntr.receiver_address.is_empty() {
                    debug!(
                        "=> Unfreeze {:?} {}",
                        ResourceCode::from_i32(cntr.resource).unwrap(),
                        b58encode_check(cntr.owner_address()),
                    );
                } else {
                    debug!(
                        "=> Unfreeze {:?} {} receiver={}",
                        ResourceCode::from_i32(cntr.resource).unwrap(),
                        b58encode_check(cntr.owner_address()),
                        b58encode_check(&cntr.receiver_address)
                    );
                }

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::VoteWitnessContract => {
                let cntr = contract_pb::VoteWitnessContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Vote Witness by {} votes: {:?}",
                    b58encode_check(cntr.owner_address()),
                    cntr.votes
                        .iter()
                        .fold(HashMap::<String, i64>::new(), |mut votes, vote| {
                            *votes.entry(b58encode_check(&vote.vote_address)).or_default() += vote.vote_count;
                            votes
                        })
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::AssetIssueContract => {
                let cntr = contract_pb::AssetIssueContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Issue Asset by {}: {:?}",
                    b58encode_check(&cntr.owner_address()),
                    cntr.name
                );

                // TODO: save created asset id in Receipt
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                // ? order?
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::UpdateAssetContract => {
                let cntr = contract_pb::UpdateAssetContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!("=> Asset Update {}: {:?}", b58encode_check(&cntr.owner_address()), cntr);

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::UnfreezeAssetContract => {
                let cntr = contract_pb::UnfreezeAssetContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> Asset Unfreeze {}: {:?}",
                    b58encode_check(&cntr.owner_address()),
                    cntr
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::TransferAssetContract => {
                let cntr = contract_pb::TransferAssetContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> Transfer Asset from {} to {}: amount={} asset_name={:?}",
                    b58encode_check(&cntr.owner_address()),
                    b58encode_check(&cntr.to_address),
                    cntr.amount,
                    cntr.asset_name
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::ParticipateAssetIssueContract => {
                let cntr =
                    contract_pb::ParticipateAssetIssueContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> Participate Asset Issue {}, to {}: token_id={} amount={}",
                    b58encode_check(&cntr.owner_address()),
                    b58encode_check(&cntr.to_address),
                    cntr.asset_name,
                    cntr.amount
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::AccountUpdateContract => {
                let cntr = contract_pb::AccountUpdateContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Account Set Name {}: name={:?}",
                    b58encode_check(&cntr.owner_address()),
                    cntr.account_name
                );
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::SetAccountIdContract => {
                let cntr = contract_pb::SetAccountIdContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Account Set ID {}: name={:?}",
                    b58encode_check(&cntr.owner_address()),
                    cntr.account_id
                );
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::AccountCreateContract => {
                let cntr = contract_pb::AccountCreateContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Create Account By {}: {:?}, type={:?}",
                    b58encode_check(&cntr.owner_address()),
                    b58encode_check(&cntr.account_address),
                    cntr.r#type,
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::AccountPermissionUpdateContract => {
                let cntr =
                    contract_pb::AccountPermissionUpdateContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Account Permission Update {}",
                    b58encode_check(&cntr.owner_address()),
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::WithdrawBalanceContract => {
                let cntr = contract_pb::WithdrawBalanceContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!("=> Withdraw Reward {}", b58encode_check(&cntr.owner_address()),);

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::UpdateSettingContract => {
                let cntr = contract_pb::UpdateSettingContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Update Contract setting {}, contract={}",
                    b58encode_check(&cntr.owner_address()),
                    b58encode_check(&cntr.contract_address)
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::UpdateEnergyLimitContract => {
                let cntr = contract_pb::UpdateEnergyLimitContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Update Contract origin_energy_limit {}, contract={}",
                    b58encode_check(&cntr.owner_address()),
                    b58encode_check(&cntr.contract_address)
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::ClearAbiContract => {
                let cntr = contract_pb::ClearAbiContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Clear Contract ABI {}, contract={}",
                    b58encode_check(&cntr.owner_address()),
                    b58encode_check(&cntr.contract_address)
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            // TVM: Should handle BW first, then remaining can be used for E.
            ContractType::CreateSmartContract => {
                // See-also: https://github.com/opentron/opentron/issues/34
                // Sea-also: https://github.com/opentron/opentron/issues/38
                let raw_cntr = &cntr.parameter.as_ref().unwrap().value[..];
                let maybe_cntr = contract_pb::CreateSmartContract::decode(raw_cntr);

                let cntr = match maybe_cntr {
                    Ok(cntr) => cntr,
                    Err(e) => {
                        warn!("pb error: {:?}", e);
                        warn!("try fix protobuf bug at {:?}", txn.hash);
                        let mut raw = raw_cntr.to_vec();
                        match &*format!("{:?}", txn.hash) {
                            "0xd7506ce73f42c802fedb367cd803975d328ef331767711313a965d7cb935fc3e" |
                            "0xc8b66021c09ec0e18bea68750630fa7dd066cd1d5e3162074e96baa652c3b884" => {
                                // rm trailing `220123`
                                let _ = raw.split_off(raw.len() - 3);
                                contract_pb::CreateSmartContract::decode(&raw[..]).expect("pb decode error")
                            }
                            "0xa58995a7160be51ec2388f749c8abe1468c0cac795a8e879f912837882e0d490" |
                            "0x73d96abda1756f724871dfba418aa1e8c1c7526070e4d69fb247171f753d1158" |
                            "0x46ff9d24e110296dadb7ad70b8ab817050999fdad147170a2d360997051db9e6" |
                            "0x0c4d57f340a94593dce4a87aa4d1d277c19edb3869d3427aa51a688f756b9af6" |
                            "0xa6b98c471b496d9f00ea2b7b0fc0173e84be0b26dd9cd7dab4907f822fbcf57a" |
                            "0x31ae94f0d236c7bda7c1776296497f5c073d0845e7214b9c3c46a55c44f6775e" => {
                                // rm trailing `22022727`
                                let _ = raw.split_off(raw.len() - 4);
                                contract_pb::CreateSmartContract::decode(&raw[..]).expect("pb decode error")
                            }
                            _ => {
                                warn!("HEX: {}", hex::encode(raw));
                                return Err("cannot handle protobuf bug".into());
                            }
                        }
                    }
                };

                debug!(
                    "=> Create Smart Contract by {}: name={:?} code_size={}",
                    b58encode_check(&cntr.owner_address()),
                    cntr.new_contract.as_ref().unwrap().name,
                    cntr.new_contract.as_ref().unwrap().bytecode.len(),
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.validate(self.manager, ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::TriggerSmartContract => {
                let cntr = contract_pb::TriggerSmartContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> Calling Smart Contract by {}: contract={}",
                    b58encode_check(&cntr.owner_address()),
                    b58encode_check(&cntr.contract_address),
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.validate(self.manager, ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::ExchangeCreateContract => {
                let cntr = contract_pb::ExchangeCreateContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> ExchangeCreate by {}: {}:{} <=> {}:{}",
                    b58encode_check(&cntr.owner_address()),
                    cntr.first_token_id,
                    cntr.first_token_balance,
                    cntr.second_token_id,
                    cntr.second_token_balance
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::ExchangeWithdrawContract => {
                let cntr = contract_pb::ExchangeWithdrawContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> ExchangeWithdraw by {}: exchange#{} {}:{}",
                    b58encode_check(&cntr.owner_address()),
                    cntr.exchange_id,
                    cntr.token_id,
                    cntr.quant,
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                cntr.validate(self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::ExchangeInjectContract => {
                let cntr = contract_pb::ExchangeInjectContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> ExchangeInject by {}: exchange#{} {}:{}",
                    b58encode_check(&cntr.owner_address()),
                    cntr.exchange_id,
                    cntr.token_id,
                    cntr.quant,
                );

                // ???
                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.validate(self.manager, ctx)?;
                cntr.execute(self.manager, ctx)
            }
            ContractType::ExchangeTransactionContract => {
                let cntr =
                    contract_pb::ExchangeTransactionContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> ExchangeTransaction by {}: exchange#{} {}:{} expected={}",
                    b58encode_check(&cntr.owner_address()),
                    cntr.exchange_id,
                    cntr.token_id,
                    cntr.quant,
                    cntr.expected
                );

                cntr.validate_signature(permission_id, recover_addrs, self.manager, ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(ctx)?;
                cntr.validate(self.manager, ctx)?;
                cntr.execute(self.manager, ctx)
            }
            #[cfg(feature = "nile")]
            ContractType::ShieldedTransferContract => {
                let cntr = contract_pb::ShieldedTransferContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                log::warn!("=> Shielded Transaction, use dummy implementation");
                // NOTE: dummy implementation
                // NOTE: no need to verify signature
                // cntr.validate_signature(permission_id, recover_addrs, self.manager,ctx)?;
                cntr.validate(self.manager, ctx)?;
                // NOTE: Shielded transaction won't consume bandwidth.
                cntr.execute(self.manager, ctx)
            }
            ContractType::ObsoleteVoteAssetContract |
            ContractType::ObsoleteCustomContract |
            ContractType::ObsoleteGetContract => unreachable!("OBSOLETE: {:?}", cntr_type),
            #[allow(unreachable_patterns)]
            _ => unimplemented!("TODO: handle contract type {:?}", cntr_type),
        }
    }
}

#[inline]
fn check_transaction_result(exec_result: &TransactionResult, maybe_result: &Option<&TransactionResult>) -> bool {
    if let Some(result) = maybe_result {
        // NOTE: only the following 2 fields is actually used in TransactionResult
        if result.status != exec_result.status || result.contract_status != exec_result.contract_status {
            error!(
                "execution result mismatch, expected: \n{:?}\ngot: \n{:?}",
                result, exec_result
            );
            return false;
        }
    } else {
        debug!("no result field in chain pb");
    }
    return true;
}
