use std::convert::TryFrom;

use ::keys::Address;
use proto2::chain::transaction::Result as TransactionResult;
use proto2::contract as contract_pb;
use proto2::state::Witness;
use state::keys;

use super::super::executor::TransactionContext;
use super::super::Manager;
use super::BuiltinContractExecutorExt;

impl BuiltinContractExecutorExt for contract_pb::WitnessCreateContract {
    fn validate(&self, manager: &Manager, ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        let fee = self.fee(manager);

        // validUrl
        if self.url.is_empty() || self.url.len() > 256 {
            return Err("invalid url".into());
        }

        let owner_acct = state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "error while querying db")?;
        if owner_acct.is_none() {
            return Err("owner account is not on chain".into());
        }
        let owner_acct = owner_acct.unwrap();

        let maybe_witness = state_db
            .get(&keys::Witness(owner_address))
            .map_err(|_| "error while querying db")?;
        if maybe_witness.is_some() {
            return Err(format!("witness {} already exists", owner_address));
        }

        if owner_acct.balance < fee {
            return Err("insufficient balance to create witness".into());
        }

        ctx.contract_fee = fee;

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_address = Address::try_from(&self.owner_address).unwrap();

        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_address));
        // createWitness

        let witness = Witness {
            address: owner_address.as_bytes().to_vec(),
            url: unsafe { String::from_utf8_unchecked(self.url.clone()) },
            vote_count: 0,
            // FIXME: is_active should be updated in vote counting
            is_active: false,
            ..Default::default()
        };

        manager
            .state_db
            .put_key(keys::Witness(owner_address), witness)
            .map_err(|_| "db insert error")?;

        // TODO: setIsWitness for account,  getAllowMultiSign for witness permission

        owner_acct.adjust_balance(-ctx.contract_fee).unwrap();
        manager
            .state_db
            .put_key(keys::Account(owner_address), owner_acct)
            .map_err(|e| e.to_string())?;

        Ok(TransactionResult::default())
    }

    fn fee(&self, manager: &Manager) -> i64 {
        manager.state_db.must_get(&keys::ChainParameter::WitnessCreateFee)
    }
}
