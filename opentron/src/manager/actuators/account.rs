//! Account related builtin contracts.

use std::convert::TryFrom;

use ::keys::Address;
use proto2::chain::transaction::Result as TransactionResult;
use proto2::contract as contract_pb;
use proto2::state::Account;
use state::keys;

use super::super::executor::TransactionContext;
use super::super::Manager;
use super::BuiltinContractExecutorExt;

impl BuiltinContractExecutorExt for contract_pb::AccountUpdateContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        // validAccountName
        if self.account_name.as_bytes().len() > 200 {
            return Err("invalid account name".into());
        }

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;
        let maybe_acct = manager
            .state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "db query error")?;
        if maybe_acct.is_none() {
            return Err("account not exists".into());
        }
        let acct = maybe_acct.unwrap();

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
        Ok(TransactionResult::success())
    }
}

// TODO: impl index
fn find_account_by_name(manager: &Manager, acct_name: &str) -> Option<Account> {
    let mut found: Option<Account> = None;
    {
        let found = &mut found;
        manager.state_db.for_each(move |_key: &keys::Account, value: &Account| {
            if value.name == acct_name {
                *found = Some(value.clone());
            }
        });
    }
    found
}
