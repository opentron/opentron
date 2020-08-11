use std::convert::TryFrom;

use ::keys::Address;
use proto2::chain::transaction::Result as TransactionResult;
use proto2::common::AccountType;
use proto2::contract as contract_pb;
use proto2::state::Account;
use state::keys;

use super::super::executor::TransactionContext;
use super::super::Manager;
use super::BuiltinContractExecutorExt;

impl BuiltinContractExecutorExt for contract_pb::ProposalCreateContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        let maybe_acct = manager
            .state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "db query error")?;
        if maybe_acct.is_none() {
            return Err("account not exists".into());
        }

        let maybe_wit = manager
            .state_db
            .get(&keys::Witness(owner_address))
            .map_err(|_| "db query error")?;
        if maybe_wit.is_none() {
            return Err("account is not a witness".into());
        }

        if self.parameters.is_empty() {
            return Err("empty parameter".into());
        }

        // TODO: validate parameter entry

        Ok(())
    }

    // TODO: for now, use String as Error type
    fn execute(&self, _manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        unimplemented!()
    }
}
