//! Shielded transfer, only available in nile testnet.
//! This is a dummy implementation.

use std::convert::TryFrom;

use ::keys::Address;
use log::warn;
use proto::chain::transaction::Result as TransactionResult;
use proto::contract as contract_pb;
use proto::state::Account;
use state::keys;

use crate::Manager;
use super::super::TransactionContext;
use super::BuiltinContractExecutorExt;

/// The TRZ token id.
const SHIELDED_TOKEN_ID: i64 = 1000016;
// TODO: This following 2 should be chain parameters. Never mind, this is a dead feature.
const SHIELDED_TRANSACTION_FEE: i64 = 10_000_000;
const SHIELDED_TRANSACTION_CREATE_ACCOUNT_FEE: i64 = 10_000_000;

impl BuiltinContractExecutorExt for contract_pb::ShieldedTransferContract {
    fn validate(&self, manager: &Manager, ctx: &mut TransactionContext) -> Result<(), String> {
        if !self.transparent_from_address.is_empty() {
            let fee = self.fee(manager);
            ctx.contract_fee = fee;
        }
        Ok(())
    }

    fn execute(&self, manager: &mut Manager, ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        if !self.transparent_from_address.is_empty() {
            let from_addr = Address::try_from(&self.transparent_from_address).unwrap();
            let mut from_acct = manager.state_db.must_get(&keys::Account(from_addr));

            from_acct
                .adjust_token_balance(SHIELDED_TOKEN_ID, -self.from_amount)
                .unwrap();
            manager
                .add_token_to_blackhole(SHIELDED_TOKEN_ID, ctx.contract_fee)
                .unwrap();
            ctx.contract_fee = 0; // in TRZ, not TRX

            manager.state_db.put_key(keys::Account(from_addr), from_acct).unwrap();
        } else if !self.transparent_to_address.is_empty() {
            let to_addr = Address::try_from(&self.transparent_to_address).unwrap();
            let maybe_to_acct = manager.state_db.get(&keys::Account(to_addr)).unwrap();
            let mut to_acct = maybe_to_acct.unwrap_or_else(|| Account::new(manager.latest_block_timestamp()));
            to_acct.adjust_token_balance(SHIELDED_TOKEN_ID, self.to_amount).unwrap();

            manager.state_db.put_key(keys::Account(to_addr), to_acct).unwrap();
        } else {
            warn!("fake handling z to z shielded transfer");
        }
        Ok(TransactionResult::success())
    }

    fn fee(&self, manager: &Manager) -> i64 {
        if !self.transparent_to_address.is_empty() {
            let to_addr = Address::try_from(&self.transparent_to_address).unwrap();
            if manager.state_db.get(&keys::Account(to_addr)).unwrap().is_none() {
                return SHIELDED_TRANSACTION_CREATE_ACCOUNT_FEE;
            }
        }
        SHIELDED_TRANSACTION_FEE
    }
}
