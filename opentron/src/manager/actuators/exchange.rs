use std::convert::TryFrom;

use ::keys::Address;
use proto2::chain::transaction::Result as TransactionResult;
use proto2::contract as contract_pb;
use proto2::state::Exchange;
use state::keys;

use super::super::executor::TransactionContext;
use super::super::Manager;
use super::BuiltinContractExecutorExt;

impl BuiltinContractExecutorExt for contract_pb::ExchangeCreateContract {
    fn validate(&self, manager: &Manager, ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let fee = self.fee(manager);

        let owner_addr = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;
        let owner_acct = state_db
            .get(&keys::Account(owner_addr))
            .map_err(|_| "db query error")?
            .ok_or_else(|| "owner account is not on chain")?;

        if owner_acct.balance < fee {
            return Err("insufficient balance".into());
        }

        // NOTE: java-tron has no asset-v1 support here. So, AllowSameTokenName check will cause an error.
        assert!(
            state_db.must_get(&keys::ChainParameter::AllowSameTokenName) == 1,
            "are you joking"
        );

        // TODO: check (1000000, LatestTokenId]
        // let latest_token_id = state_db.must_get(&keys::DynamicProperty::LatestTokenId);

        if self.first_token_id == self.second_token_id {
            return Err("cannot exchange then same tokens".into());
        }

        if self.first_token_id == "_" {
            if owner_acct.balance < self.first_token_balance + fee {
                return Err("insufficient TRX balance".into());
            }
        } else {
            let token_id = self.first_token_id.parse().map_err(|_| "invalid token id format")?;
            if owner_acct.token_balance.get(&token_id).copied().unwrap_or_default() < self.first_token_balance {
                return Err("insufficient token balance".into());
            }
        }

        if self.second_token_id == "_" {
            if owner_acct.balance < self.second_token_balance + fee {
                return Err("insufficient TRX balance".into());
            }
        } else {
            let token_id = self.second_token_id.parse().map_err(|_| "invalid token id format")?;
            if owner_acct.token_balance.get(&token_id).copied().unwrap_or_default() < self.second_token_balance {
                return Err("insufficient token balance".into());
            }
        }

        ctx.contract_fee = fee;
        Ok(())
    }

    fn execute(&self, manager: &mut Manager, ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_addr = Address::try_from(&self.owner_address).unwrap();
        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_addr));

        let exchange_id = manager
            .state_db
            .get(&keys::DynamicProperty::LatestExchangeId)
            .unwrap()
            .unwrap_or(0) +
            1;

        // Use 0 to denote TRX.
        let first_token_id = self.first_token_id.parse().unwrap_or_default();
        let second_token_id = self.second_token_id.parse().unwrap_or_default();

        if first_token_id == 0 {
            owner_acct.adjust_balance(-self.first_token_balance).unwrap();
        } else {
            owner_acct
                .adjust_token_balance(first_token_id, -self.first_token_balance)
                .unwrap();
        }
        if second_token_id == 0 {
            owner_acct.adjust_balance(-self.second_token_balance).unwrap();
        } else {
            owner_acct
                .adjust_token_balance(second_token_id, -self.second_token_balance)
                .unwrap();
        }
        owner_acct.adjust_balance(-ctx.contract_fee).unwrap();

        let now = manager.latest_block_timestamp();
        let exch = Exchange {
            id: exchange_id,
            owner_address: self.owner_address.to_vec(),
            creation_time: now,
            first_token_id,
            first_token_balance: self.first_token_balance,
            second_token_id,
            second_token_balance: self.second_token_balance,
        };

        manager
            .state_db
            .put_key(keys::Exchange(exchange_id), exch)
            .map_err(|_| "db insert error")?;
        manager
            .state_db
            .put_key(keys::DynamicProperty::LatestExchangeId, exchange_id)
            .map_err(|_| "db insert error")?;
        manager
            .state_db
            .put_key(keys::Account(owner_addr), owner_acct)
            .map_err(|_| "db insert error")?;
        manager.add_to_blackhole(ctx.contract_fee).unwrap();

        Ok(TransactionResult::success())
    }

    fn fee(&self, manager: &Manager) -> i64 {
        manager.state_db.must_get(&keys::ChainParameter::ExchangeCreateFee)
    }
}
