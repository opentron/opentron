//! Transaction executor.

use std::str;

use ::keys::b58encode_check;
use chain::{IndexedBlock, IndexedBlockHeader, IndexedTransaction};
use log::{debug, error, warn};
use primitive_types::H256;
use proto2::chain::{transaction::result::ContractStatus, transaction::Result as TransactionResult, ContractType};
use proto2::common::ResourceCode;
use proto2::contract as contract_pb;
use proto2::state::{ResourceReceipt, TransactionLog, TransactionReceipt};
use state::keys;

use super::actuators::{BuiltinContractExecutorExt, BuiltinContractExt};
use super::resource::BandwidthProcessor;
use super::Manager;

pub struct TransactionContext<'a> {
    // Transaction static context.
    pub block_header: &'a IndexedBlockHeader,
    pub transaction_hash: &'a H256,
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
            transaction_hash: &transaction.hash,
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
            receipt.vm_logs = ctx.logs;
        }
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

    // runtime.execut
    pub fn execute(&mut self, txn: &IndexedTransaction, block: &IndexedBlock) -> Result<TransactionReceipt, String> {
        let cntr = txn.raw.raw_data.as_ref().unwrap().contract.as_ref().unwrap();
        let cntr_type = ContractType::from_i32(cntr.r#type).expect("unhandled system contract type");
        let recover_addrs = txn.recover_owner().expect("error while verifying signature");
        let maybe_result = txn.raw.result.get(0);

        let permission_id = cntr.permission_id;

        // NOTE: Routine to handle transactions of builtin contracts:
        //
        // - decode google.Any
        // - multisig verifiy
        // - validate (except bandwidth)
        // - handle bandwidth
        // - TODO: handle mutisig fee
        // - execute logic
        //
        // Which is diffent from java-tron:
        //
        // - bandwidth
        // - multisig
        // - runtime.validate
        // - runtime.execute
        //
        // Bandwidth consumption must come before transaction execution, since some type of transaction cause bandwidth usage changes(freeze/unfreeze).
        match cntr_type {
            ContractType::TransferContract => {
                let cntr = contract_pb::TransferContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> transfer from {} to {} with amount {}",
                    b58encode_check(&cntr.owner_address),
                    b58encode_check(&cntr.to_address),
                    cntr.amount
                );

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
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

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::ProposalApproveContract => {
                let cntr = contract_pb::ProposalApproveContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> Approve Proposal #{} by {} {}",
                    cntr.proposal_id,
                    b58encode_check(cntr.owner_address()),
                    cntr.is_approval
                );

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::ProposalDeleteContract => {
                let cntr = contract_pb::ProposalDeleteContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> Delete Proposal #{} by {}",
                    cntr.proposal_id,
                    b58encode_check(cntr.owner_address()),
                );

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);
                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::WitnessCreateContract => {
                let cntr = contract_pb::WitnessCreateContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> New Witness {} url={:?}",
                    b58encode_check(cntr.owner_address()),
                    str::from_utf8(&cntr.url)
                );

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::WitnessUpdateContract => {
                let cntr = contract_pb::WitnessUpdateContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> Witness Update {} new_url={:?}",
                    b58encode_check(cntr.owner_address()),
                    String::from_utf8(cntr.update_url.clone()),
                );

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::UpdateBrokerageContract => {
                let cntr = contract_pb::UpdateBrokerageContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> Update Witness Brokerage {}: new_brokerage_rate={}",
                    b58encode_check(cntr.owner_address()),
                    cntr.brokerage,
                );
                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                check_transaction_result(&cntr.execute(self.manager, &mut ctx)?, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::FreezeBalanceContract => {
                let cntr = contract_pb::FreezeBalanceContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Freeze Resource {} amount={} resource={:?}",
                    b58encode_check(cntr.owner_address()),
                    cntr.frozen_balance,
                    ResourceCode::from_i32(cntr.resource).unwrap()
                );

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
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

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::VoteWitnessContract => {
                let cntr = contract_pb::VoteWitnessContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Vote Witness by {} votes: {:?}",
                    b58encode_check(cntr.owner_address()),
                    cntr.votes
                        .iter()
                        .map(|vote| (b58encode_check(&vote.vote_address), vote.vote_count))
                        .collect::<std::collections::HashMap<_, _>>()
                );

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::AssetIssueContract => {
                let cntr = contract_pb::AssetIssueContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> Issue Asset by {}: {:?}",
                    b58encode_check(&cntr.owner_address()),
                    cntr.name
                );

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                // TODO: Fill TransactionReceipt with newly created asset token_id.
                Ok(ctx.into())
            }
            ContractType::UpdateAssetContract => {
                let cntr = contract_pb::UpdateAssetContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!("=> Asset Update {}: {:?}", b58encode_check(&cntr.owner_address()), cntr);

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::UnfreezeAssetContract => {
                let cntr = contract_pb::UnfreezeAssetContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                debug!(
                    "=> Asset Unfreeze {}: {:?}",
                    b58encode_check(&cntr.owner_address()),
                    cntr
                );

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
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

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
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

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::AccountUpdateContract => {
                let cntr = contract_pb::AccountUpdateContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Account Set Name {}: name={:?}",
                    b58encode_check(&cntr.owner_address()),
                    cntr.account_name
                );
                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::SetAccountIdContract => {
                let cntr = contract_pb::SetAccountIdContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Account Set ID {}: name={:?}",
                    b58encode_check(&cntr.owner_address()),
                    cntr.account_id
                );
                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::AccountCreateContract => {
                let cntr = contract_pb::AccountCreateContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Create Account By {}: {:?}, type={:?}",
                    b58encode_check(&cntr.owner_address()),
                    b58encode_check(&cntr.account_address),
                    cntr.r#type,
                );

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::AccountPermissionUpdateContract => {
                let cntr =
                    contract_pb::AccountPermissionUpdateContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Account Permission Update {}",
                    b58encode_check(&cntr.owner_address()),
                );

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                check_transaction_result(&cntr.execute(self.manager, &mut ctx)?, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::WithdrawBalanceContract => {
                let cntr = contract_pb::WithdrawBalanceContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!("=> Withdraw Reward {}", b58encode_check(&cntr.owner_address()),);
                let mut ctx = TransactionContext::new(&block.header, &txn);

                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                check_transaction_result(&cntr.execute(self.manager, &mut ctx)?, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::UpdateSettingContract => {
                let cntr = contract_pb::UpdateSettingContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Update Contract setting {}, contract={}",
                    b58encode_check(&cntr.owner_address()),
                    b58encode_check(&cntr.contract_address)
                );
                let mut ctx = TransactionContext::new(&block.header, &txn);

                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                check_transaction_result(&cntr.execute(self.manager, &mut ctx)?, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::UpdateEnergyLimitContract => {
                let cntr = contract_pb::UpdateEnergyLimitContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Update Contract origin_energy_limit {}, contract={}",
                    b58encode_check(&cntr.owner_address()),
                    b58encode_check(&cntr.contract_address)
                );
                let mut ctx = TransactionContext::new(&block.header, &txn);

                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                check_transaction_result(&cntr.execute(self.manager, &mut ctx)?, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::ClearAbiContract => {
                let cntr = contract_pb::ClearAbiContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                debug!(
                    "=> Clear Contract ABI {}, contract={}",
                    b58encode_check(&cntr.owner_address()),
                    b58encode_check(&cntr.contract_address)
                );
                let mut ctx = TransactionContext::new(&block.header, &txn);

                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                check_transaction_result(&cntr.execute(self.manager, &mut ctx)?, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            // TVM: Should handle BW first, then remaining can be used for E.
            ContractType::CreateSmartContract => {
                let cntr = contract_pb::CreateSmartContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                let contract_status = maybe_result
                    .and_then(|ret| ContractStatus::from_i32(ret.contract_status))
                    .unwrap_or_default();

                debug!(
                    "=> Create Smart Contract by {}: name={:?} code_size={}",
                    b58encode_check(&cntr.owner_address()),
                    cntr.new_contract.as_ref().unwrap().name,
                    cntr.new_contract.as_ref().unwrap().bytecode.len(),
                );

                let mut ctx = TransactionContext::new(&block.header, &txn);
                ctx.contract_status = contract_status;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                // NOTE: vm must be strictly checked.
                if !check_transaction_result(&exec_result, &maybe_result) {
                    debug!("result => {:?}", exec_result);
                    return Err("result check not passed!".into());
                }

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::TriggerSmartContract => {
                let cntr = contract_pb::TriggerSmartContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                // smart contract status
                let contract_status = maybe_result
                    .and_then(|ret| ContractStatus::from_i32(ret.contract_status))
                    .unwrap_or_default();
                debug!("contract_status => {:?}", contract_status);
                debug!(
                    "=> Calling Smart Contract by {}: contract={}",
                    b58encode_check(&cntr.owner_address()),
                    b58encode_check(&cntr.contract_address),
                );

                let mut ctx = TransactionContext::new(&block.header, &txn);
                ctx.contract_status = contract_status;
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                if !check_transaction_result(&exec_result, &maybe_result) {
                    debug!("result => {:?}", exec_result);
                    return Err("result check not passed!".into());
                }
                debug!("context => {:?}", ctx);
                Ok(ctx.into())
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

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);
                debug!("context => {:?}", ctx);

                Ok(ctx.into())
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

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);
                debug!("context => {:?}", ctx);

                Ok(ctx.into())
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

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);
                debug!("context => {:?}", ctx);

                Ok(ctx.into())
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

                let mut ctx = TransactionContext::new(&block.header, &txn);
                cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                BandwidthProcessor::new(self.manager, txn, &cntr)?.consume(&mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);
                debug!("context => {:?}", ctx);

                Ok(ctx.into())
            }
            #[cfg(feature = "nile")]
            ContractType::ShieldedTransferContract => {
                let cntr = contract_pb::ShieldedTransferContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();

                warn!("=> Shielded Transaction");
                // NOTE: dummy implementation

                let mut ctx = TransactionContext::new(&block.header, &txn);
                // cntr.validate_signature(permission_id, recover_addrs, self.manager, &mut ctx)?;
                cntr.validate(self.manager, &mut ctx)?;
                // NOTE: Shielded transaction won't consume bandwidth.
                check_transaction_result(&cntr.execute(self.manager, &mut ctx)?, &maybe_result);

                debug!("context => {:?}", ctx);
                Ok(ctx.into())
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
        if result != &exec_result {
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
