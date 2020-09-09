use std::convert::TryFrom;

use ::keys::Address;
use proto2::chain::transaction::Result as TransactionResult;
use proto2::contract as contract_pb;
use proto2::state::Exchange;
use state::keys;

use super::super::executor::TransactionContext;
use super::super::Manager;
use super::BuiltinContractExecutorExt;

const EXCHANGE_BALANCE_LIMIT: i64 = 1_000_000_000_000_000;

// Create an exchange pair.
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

// Withdraw exchange balance by owner.
impl BuiltinContractExecutorExt for contract_pb::ExchangeWithdrawContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let _ = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;
        let exch = state_db
            .get(&keys::Exchange(self.exchange_id))
            .map_err(|_| "db query error")?
            .ok_or_else(|| "exchange not found on chain")?;

        // NOTE: Exchange owner implies account existence.
        if exch.owner_address != self.owner_address {
            return Err("exchange is not created by owner address".into());
        }

        if self.quant <= 0 {
            return Err("invalid exchange withdraw quant".into());
        }
        if exch.first_token_balance == 0 || exch.second_token_balance == 0 {
            return Err("insufficient token balance in exchange".into());
        }

        let token_id = if self.token_id == "_" {
            0
        } else {
            self.token_id.parse().map_err(|_| "invalid token id")?
        };

        if token_id == exch.first_token_id {
            let other_token_amount = ((exch.second_token_balance as i128) * (self.quant as i128) /
                (exch.first_token_balance as i128)) as i64;
            if exch.first_token_balance < self.quant || exch.second_token_balance < other_token_amount {
                return Err("insufficient token balance in exchange".into());
            }
            if other_token_amount <= 0 {
                return Err("withdrawal token amount must be greater than 0".into());
            }

            // NOTE: The following logic is refactored from decimal arithmetic.
            let remainder = (exch.second_token_balance as i128) * (self.quant as i128) * 100000_i128 /
                (exch.first_token_balance as i128) -
                (other_token_amount as i128) * 100000_i128;
            if remainder / (other_token_amount as i128) > 10 {
                return Err("insufficient precision".into());
            }
        } else if token_id == exch.second_token_id {
            let other_token_amount = ((exch.first_token_balance as i128) * (self.quant as i128) /
                (exch.second_token_balance as i128)) as i64;
            if exch.second_token_balance < self.quant || exch.first_token_balance < other_token_amount {
                return Err("insufficient token balance in exchange".into());
            }
            if other_token_amount <= 0 {
                return Err("withdrawal token amount must be greater than 0".into());
            }

            let remainder = (exch.first_token_balance as i128) * (self.quant as i128) * 100000_i128 /
                (exch.second_token_balance as i128) -
                (other_token_amount as i128) * 100000_i128;
            if remainder / (other_token_amount as i128) > 10 {
                return Err("insufficient precision".into());
            }
        } else {
            return Err("token is not in the exchange".into());
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_addr = Address::try_from(&self.owner_address).unwrap();
        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_addr));

        let mut exch = manager.state_db.must_get(&keys::Exchange(self.exchange_id));

        let token_id = if self.token_id == "_" {
            0
        } else {
            self.token_id.parse().unwrap()
        };

        let (other_token_id, other_token_amount) = if token_id == exch.first_token_id {
            let other_token_amount = ((exch.second_token_balance as i128) * (self.quant as i128) /
                (exch.first_token_balance as i128)) as i64;
            exch.first_token_balance -= self.quant;
            exch.second_token_balance -= other_token_amount;
            (exch.second_token_id, other_token_amount)
        } else {
            let other_token_amount = ((exch.first_token_balance as i128) * (self.quant as i128) /
                (exch.second_token_balance as i128)) as i64;
            exch.second_token_balance -= self.quant;
            exch.first_token_balance -= other_token_amount;
            (exch.first_token_id, other_token_amount)
        };

        if token_id == 0 {
            owner_acct.adjust_balance(self.quant).unwrap();
        } else {
            owner_acct.adjust_token_balance(token_id, self.quant).unwrap();
        }

        if other_token_id == 0 {
            owner_acct.adjust_balance(other_token_amount).unwrap();
        } else {
            owner_acct
                .adjust_token_balance(other_token_id, other_token_amount)
                .unwrap();
        }

        manager
            .state_db
            .put_key(keys::Exchange(exch.id), exch)
            .map_err(|_| "db insert error")?;
        manager
            .state_db
            .put_key(keys::Account(owner_addr), owner_acct)
            .map_err(|_| "db insert error")?;

        Ok(TransactionResult::success())
    }
}

// Inject more tokens to exchange balance by owner.
//
// NOTE: This builtin contract has similar logic as ExchangeWithdrawContract.
impl BuiltinContractExecutorExt for contract_pb::ExchangeInjectContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let owner_addr = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;
        let owner_acct = state_db
            .get(&keys::Account(owner_addr))
            .map_err(|_| "db query error")?
            .ok_or_else(|| "owner account not found on chain")?;
        let exch = state_db
            .get(&keys::Exchange(self.exchange_id))
            .map_err(|_| "db query error")?
            .ok_or_else(|| "exchange not found on chain")?;

        if exch.owner_address != self.owner_address {
            return Err("exchange is not created by owner address".into());
        }

        if self.quant <= 0 {
            return Err("invalid exchange inject quant".into());
        }
        if exch.first_token_balance == 0 || exch.second_token_balance == 0 {
            return Err("insufficient token balance in exchange".into());
        }

        let token_id = if self.token_id == "_" {
            0
        } else {
            self.token_id.parse().map_err(|_| "invalid token id")?
        };

        let (other_token_id, other_token_amount) = if token_id == exch.first_token_id {
            let other_token_amount = ((exch.second_token_balance as i128) * (self.quant as i128) /
                (exch.first_token_balance as i128)) as i64;
            if exch.first_token_balance + self.quant > EXCHANGE_BALANCE_LIMIT ||
                exch.second_token_balance + other_token_amount > EXCHANGE_BALANCE_LIMIT
            {
                return Err(format!("token balance in exchange exceeds {}", EXCHANGE_BALANCE_LIMIT));
            }
            (exch.second_token_id, other_token_amount)
        } else if token_id == exch.second_token_id {
            let other_token_amount = ((exch.first_token_balance as i128) * (self.quant as i128) /
                (exch.second_token_balance as i128)) as i64;
            if exch.second_token_balance + self.quant > EXCHANGE_BALANCE_LIMIT ||
                exch.first_token_balance + other_token_amount > EXCHANGE_BALANCE_LIMIT
            {
                return Err(format!("token balance in exchange exceeds {}", EXCHANGE_BALANCE_LIMIT));
            }
            (exch.first_token_id, other_token_amount)
        } else {
            return Err("token is not in the exchange".into());
        };

        if other_token_amount <= 0 {
            return Err("inject token amount must be greater than 0".into());
        }

        if token_id == 0 {
            if owner_acct.balance < self.quant {
                return Err("insufficient balance".into());
            }
        } else {
            if owner_acct.token_balance.get(&token_id).copied().unwrap_or_default() < self.quant {
                return Err("insufficient token balance".into());
            }
        }

        if other_token_id == 0 {
            if owner_acct.balance < other_token_amount {
                return Err("insufficient balance".into());
            }
        } else {
            if owner_acct
                .token_balance
                .get(&other_token_id)
                .copied()
                .unwrap_or_default() <
                other_token_amount
            {
                return Err("insufficient token balance".into());
            }
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_addr = Address::try_from(&self.owner_address).unwrap();
        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_addr));

        let mut exch = manager.state_db.must_get(&keys::Exchange(self.exchange_id));
        let token_id = self.token_id.parse().unwrap_or(0);

        let (other_token_id, other_token_amount) = if token_id == exch.first_token_id {
            let other_token_amount = ((exch.second_token_balance as i128) * (self.quant as i128) /
                (exch.first_token_balance as i128)) as i64;
            exch.first_token_balance += self.quant;
            exch.second_token_balance += other_token_amount;
            (exch.second_token_id, other_token_amount)
        } else {
            let other_token_amount = ((exch.first_token_balance as i128) * (self.quant as i128) /
                (exch.second_token_balance as i128)) as i64;
            exch.second_token_balance += self.quant;
            exch.first_token_balance += other_token_amount;
            (exch.first_token_id, other_token_amount)
        };
        log::debug!("inject amount => {}", other_token_amount);

        if token_id == 0 {
            owner_acct.adjust_balance(-self.quant).unwrap();
        } else {
            owner_acct.adjust_token_balance(token_id, -self.quant).unwrap();
        }

        if other_token_id == 0 {
            owner_acct.adjust_balance(-other_token_amount).unwrap();
        } else {
            owner_acct
                .adjust_token_balance(other_token_id, -other_token_amount)
                .unwrap();
        }

        manager
            .state_db
            .put_key(keys::Exchange(exch.id), exch)
            .map_err(|_| "db insert error")?;
        manager
            .state_db
            .put_key(keys::Account(owner_addr), owner_acct)
            .map_err(|_| "db insert error")?;

        Ok(TransactionResult::success())
    }
}
