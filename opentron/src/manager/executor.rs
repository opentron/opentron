//! Transaction executor.

use ::keys::b58encode_check;
use chain::{IndexedBlock, IndexedBlockHeader, IndexedTransaction};
use log::debug;
use primitive_types::H256;
use proto2::chain::{transaction::result::ContractStatus, ContractType};
use proto2::contract as contract_pb;
use proto2::state::{ResourceReceipt, TransactionReceipt};

use super::actuators::{BuiltinContractExecutorExt, BuiltinContractExt};
use super::processors::BandwidthProcessor;
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

        debug!("cntr type => {:?}", cntr_type);
        debug!("TODO: verify signagures and multisig, recover_addrs => {:?}", recover_addrs);

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

                debug!(
                    "=> transfer from {} to {} with amount {}",
                    b58encode_check(&cntr.owner_address),
                    b58encode_check(&cntr.to_address),
                    cntr.amount
                );

                let mut ctx = TransactionContext::new(&block.header, &txn.hash);
                cntr.validate(self.manager, &mut ctx)?;
                debug!("execute => {:?}", cntr.execute(self.manager, &mut ctx)?);

                {
                    let mut bw_proc = BandwidthProcessor::new(self.manager);
                    bw_proc.consume(txn, &cntr, &mut ctx)?;
                }
                debug!("context => {:?}", ctx);

                Ok(ctx.into())
            }
            ContractType::TriggerSmartContract | ContractType::CreateSmartContract => {
                // smart contract status
                let contract_status = txn
                    .raw
                    .result
                    .get(0)
                    .and_then(|ret| ContractStatus::from_i32(ret.contract_status))
                    .unwrap_or_default();
                debug!("contract_status => {:?}", contract_status);

                unimplemented!()
            }
            _ => unimplemented!(),
        }
    }
}
