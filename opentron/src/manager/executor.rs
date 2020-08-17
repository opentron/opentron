//! Transaction executor.

use std::str;

use ::keys::b58encode_check;
use chain::{IndexedBlock, IndexedBlockHeader, IndexedTransaction};
use log::{debug, error, warn};
use primitive_types::H256;
use proto2::chain::{transaction::result::ContractStatus, transaction::Result as TransactionResult, ContractType};
use proto2::common::ResourceCode;
use proto2::contract as contract_pb;
use proto2::state::{ResourceReceipt, TransactionReceipt};
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
    // NOTE: Account creation fee will overwrite bandwidth fee.
    // pub account_creation_fee: i64,
    // Set by actuator.valide().
    pub new_account_created: bool,
}

impl<'a> TransactionContext<'a> {
    pub fn new<'b>(block_header: &'b IndexedBlockHeader, transaction_hash: &'b H256) -> TransactionContext<'b> {
        TransactionContext {
            block_header,
            transaction_hash,
            bandwidth_usage: 0,
            bandwidth_fee: 0,
            contract_fee: 0,
            new_account_created: false,
        }
    }
}

impl From<TransactionContext<'_>> for TransactionReceipt {
    fn from(ctx: TransactionContext) -> TransactionReceipt {
        TransactionReceipt {
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
        }
    }
}

impl ::std::fmt::Debug for TransactionContext<'_> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("TransactionContext<'_>")
            .field("block", &self.block_header.hash)
            .field("bandwidth_usage", &self.bandwidth_usage)
            .field("bandwidth_fee", &self.bandwidth_fee)
            .field("contract_fee", &self.contract_fee)
            .field("new_account_created", &self.new_account_created)
            .finish()
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

        debug!("cntr type => {:?}", cntr_type);
        // debug!(
        //    "TODO: verify signagures and multisig, recover_addrs => {:?}",
        //    recover_addrs
        // );

        // NOTE: Routine to handle transactions of builtin contracts:
        //
        // - decode google.Any
        // - TODO: multisig
        // - validate (except bandwidth)
        // - execute logic
        // - handle bandwidth
        //
        // Which is diffent from java-tron:
        //
        // - bandwidth
        // - multisig
        // - runtime.validate
        // - runtime.execute
        match cntr_type {
            ContractType::TransferContract => {
                let cntr = contract_pb::TransferContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                // TOOD: handle multisig, for now, use simple method
                if cntr.owner_address() != recover_addrs[0].as_bytes() {
                    return Err("invalid signature".into());
                }

                debug!(
                    "=> transfer from {} to {} with amount {}",
                    b58encode_check(&cntr.owner_address),
                    b58encode_check(&cntr.to_address),
                    cntr.amount
                );

                let mut ctx = TransactionContext::new(&block.header, &txn.hash);
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                BandwidthProcessor::new(self.manager).consume(txn, &cntr, &mut ctx)?;
                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::ProposalCreateContract => {
                let cntr = contract_pb::ProposalCreateContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                // TOOD: handle multisig, for now, use simple method
                if cntr.owner_address() != recover_addrs[0].as_bytes() {
                    return Err("invalid signature".into());
                }

                debug!(
                    "=> Proposal by {} {:?}",
                    b58encode_check(&cntr.owner_address),
                    cntr.parameters
                        .iter()
                        .map(|(&k, v)| (keys::ChainParameter::from_i32(k as i32).unwrap(), v))
                        .collect::<std::collections::HashMap<_, _>>()
                );

                let mut ctx = TransactionContext::new(&block.header, &txn.hash);
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                BandwidthProcessor::new(self.manager).consume(txn, &cntr, &mut ctx)?;
                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::ProposalApproveContract => {
                let cntr = contract_pb::ProposalApproveContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                // TOOD: handle multisig, for now, use simple method
                if cntr.owner_address() != recover_addrs[0].as_bytes() {
                    return Err("invalid signature".into());
                }

                debug!(
                    "=> Approve Proposal #{} by {} {}",
                    cntr.proposal_id,
                    b58encode_check(cntr.owner_address()),
                    cntr.is_approval
                );

                let mut ctx = TransactionContext::new(&block.header, &txn.hash);
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                BandwidthProcessor::new(self.manager).consume(txn, &cntr, &mut ctx)?;
                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::WitnessCreateContract => {
                let cntr = contract_pb::WitnessCreateContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                if cntr.owner_address() != recover_addrs[0].as_bytes() {
                    return Err("invalid signature".into());
                }
                debug!(
                    "=> New Witness {} url={:?}",
                    b58encode_check(cntr.owner_address()),
                    str::from_utf8(&cntr.url)
                );
                let mut ctx = TransactionContext::new(&block.header, &txn.hash);
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                BandwidthProcessor::new(self.manager).consume(txn, &cntr, &mut ctx)?;
                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::FreezeBalanceContract => {
                let cntr = contract_pb::FreezeBalanceContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                if cntr.owner_address() != recover_addrs[0].as_bytes() {
                    return Err("invalid signature".into());
                }

                debug!(
                    "=> Freeze Resource {} amount={} resource={:?}",
                    b58encode_check(cntr.owner_address()),
                    cntr.frozen_balance,
                    ResourceCode::from_i32(cntr.resource).unwrap()
                );

                let mut ctx = TransactionContext::new(&block.header, &txn.hash);
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                BandwidthProcessor::new(self.manager).consume(txn, &cntr, &mut ctx)?;
                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::VoteWitnessContract => {
                let cntr = contract_pb::VoteWitnessContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                if cntr.owner_address() != recover_addrs[0].as_bytes() {
                    return Err("invalid signature".into());
                }

                debug!(
                    "=> Vote Witness by {} votes: {:?}",
                    b58encode_check(cntr.owner_address()),
                    cntr.votes
                        .iter()
                        .map(|vote| (b58encode_check(&vote.vote_address), vote.vote_count))
                        .collect::<std::collections::HashMap<_, _>>()
                );

                let mut ctx = TransactionContext::new(&block.header, &txn.hash);
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                BandwidthProcessor::new(self.manager).consume(txn, &cntr, &mut ctx)?;
                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::AssetIssueContract => {
                let cntr = contract_pb::AssetIssueContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                if cntr.owner_address() != recover_addrs[0].as_bytes() {
                    return Err("invalid signature".into());
                }

                debug!(
                    "=> Issue Asset by {}: {:?}",
                    b58encode_check(&cntr.owner_address()),
                    cntr.name
                );

                let mut ctx = TransactionContext::new(&block.header, &txn.hash);
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                BandwidthProcessor::new(self.manager).consume(txn, &cntr, &mut ctx)?;
                debug!("context => {:?}", ctx);
                // TODO: Fill TransactionReceipt with newly created asset token_id.
                Ok(ctx.into())
            }
            ContractType::TransferAssetContract => {
                let cntr = contract_pb::TransferAssetContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                if cntr.owner_address() != recover_addrs[0].as_bytes() {
                    return Err("invalid signature".into());
                }

                debug!(
                    "=> Transfer Asset from {} to {}: amount={} asset_name={:?}",
                    b58encode_check(&cntr.owner_address()),
                    b58encode_check(&cntr.to_address),
                    cntr.amount,
                    cntr.asset_name
                );

                let mut ctx = TransactionContext::new(&block.header, &txn.hash);
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                BandwidthProcessor::new(self.manager).consume(txn, &cntr, &mut ctx)?;
                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::AccountUpdateContract => {
                let cntr = contract_pb::AccountUpdateContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                if cntr.owner_address() != recover_addrs[0].as_bytes() {
                    return Err("invalid signature".into());
                }

                debug!(
                    "=> Account Set Name {}: name={:?}",
                    b58encode_check(&cntr.owner_address()),
                    cntr.account_name
                );
                let mut ctx = TransactionContext::new(&block.header, &txn.hash);
                cntr.validate(self.manager, &mut ctx)?;
                let exec_result = cntr.execute(self.manager, &mut ctx)?;
                check_transaction_result(&exec_result, &maybe_result);

                BandwidthProcessor::new(self.manager).consume(txn, &cntr, &mut ctx)?;
                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            // TVM
            ContractType::CreateSmartContract => {
                let cntr = contract_pb::CreateSmartContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                if cntr.owner_address() != recover_addrs[0].as_bytes() {
                    return Err("invalid signature".into());
                }

                debug!(
                    "=> Create Smart Contract by {}: name={:?} code_size={}",
                    b58encode_check(&cntr.owner_address()),
                    cntr.new_contract.as_ref().unwrap().name,
                    cntr.new_contract.as_ref().unwrap().bytecode.len(),
                );
                warn!("TODO: TVM & energy");

                let mut ctx = TransactionContext::new(&block.header, &txn.hash);
                BandwidthProcessor::new(self.manager).consume(txn, &cntr, &mut ctx)?;
                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            ContractType::TriggerSmartContract => {
                let cntr = contract_pb::TriggerSmartContract::from_any(cntr.parameter.as_ref().unwrap()).unwrap();
                if cntr.owner_address() != recover_addrs[0].as_bytes() {
                    return Err("invalid signature".into());
                }
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
                warn!("TODO: TVM & energy");

                let mut ctx = TransactionContext::new(&block.header, &txn.hash);
                BandwidthProcessor::new(self.manager).consume(txn, &cntr, &mut ctx)?;
                debug!("context => {:?}", ctx);
                Ok(ctx.into())
            }
            _ => unimplemented!(),
        }
    }
}

#[inline]
fn check_transaction_result(exec_result: &TransactionResult, maybe_result: &Option<&TransactionResult>) -> bool {
    if let Some(result) = maybe_result {
        if result != &exec_result {
            error!(
                "execution result mismatch, expected: {:?}, got: {:?}",
                result, exec_result
            );
            return false;
        }
    } else {
        warn!("no result field in chain pb");
    }
    return true;
}
