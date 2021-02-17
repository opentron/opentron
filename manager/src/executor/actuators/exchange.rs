//! Exchange, the DEX on chain.
//!
//! NOTE: These builtin contracts are seldom used now.

use std::convert::TryFrom;

use ::keys::Address;
use proto::chain::transaction::Result as TransactionResult;
use proto::contract as contract_pb;
use proto::state::Exchange;
use state::keys;

use super::super::TransactionContext;
use super::asset::find_asset_by_name;
use super::BuiltinContractExecutorExt;
use crate::Manager;

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

        if self.first_token_id == self.second_token_id {
            return Err("cannot exchange then same tokens".into());
        }

        if owner_acct.balance < fee {
            return Err("insufficient TRX balance".into());
        }

        let first_token_id = get_exchange_token_id(manager, &self.first_token_id)?;
        let second_token_id = get_exchange_token_id(manager, &self.second_token_id)?;

        if first_token_id == 0 {
            if owner_acct.balance < self.first_token_balance + fee {
                return Err("insufficient TRX balance".into());
            }
        } else {
            if owner_acct
                .token_balance
                .get(&first_token_id)
                .copied()
                .unwrap_or_default() <
                self.first_token_balance
            {
                return Err("insufficient token balance".into());
            }
        }

        if second_token_id == 0 {
            if owner_acct.balance < self.second_token_balance + fee {
                return Err("insufficient TRX balance".into());
            }
        } else {
            if owner_acct
                .token_balance
                .get(&second_token_id)
                .copied()
                .unwrap_or_default() <
                self.second_token_balance
            {
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

        let first_token_id = get_exchange_token_id(manager, &self.first_token_id).unwrap();
        let second_token_id = get_exchange_token_id(manager, &self.second_token_id).unwrap();

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

        let token_id = get_exchange_token_id(manager, &self.token_id)?;
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

        let token_id = get_exchange_token_id(manager, &self.token_id).unwrap();

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

        let token_id = get_exchange_token_id(manager, &self.token_id)?;

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

        log::debug!("inject other token: #{}:{}", other_token_id, other_token_amount);

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
        let token_id = get_exchange_token_id(manager, &self.token_id).unwrap();

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

impl BuiltinContractExecutorExt for contract_pb::ExchangeTransactionContract {
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

        if self.quant <= 0 {
            return Err("invalid exchange transaction quant".into());
        }
        if self.expected <= 0 {
            return Err("invalid exchange transaction expected token amount".into());
        }
        if exch.first_token_balance == 0 || exch.second_token_balance == 0 {
            return Err("insufficient token balance in exchange".into());
        }

        let token_id = get_exchange_token_id(manager, &self.token_id)?;
        let token_balance = if token_id == exch.first_token_id {
            exch.first_token_balance
        } else if token_id == exch.second_token_id {
            exch.second_token_balance
        } else {
            return Err("token is not in the exchange".into());
        };
        if token_balance + self.quant > EXCHANGE_BALANCE_LIMIT {
            return Err(format!("token balance in exchange exceeds {}", EXCHANGE_BALANCE_LIMIT));
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

        let supply = 1_000_000_000_000_000_000_i64;
        let buy_token_amount = if exch.first_token_id == token_id {
            exchange(supply, exch.first_token_balance, exch.second_token_balance, self.quant)
        } else {
            exchange(supply, exch.second_token_balance, exch.first_token_balance, self.quant)
        };
        if buy_token_amount < self.expected {
            return Err("buy token amount must be greater than expected".into());
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_addr = Address::try_from(&self.owner_address).unwrap();
        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_addr));

        let mut exch = manager.state_db.must_get(&keys::Exchange(self.exchange_id));
        let sell_token_id = get_exchange_token_id(manager, &self.token_id).unwrap();

        let supply = 1_000_000_000_000_000_000_i64;

        let buy_token_amount;
        let buy_token_id;
        if exch.first_token_id == sell_token_id {
            buy_token_id = exch.second_token_id;
            buy_token_amount = exchange(supply, exch.first_token_balance, exch.second_token_balance, self.quant);
            exch.first_token_balance += self.quant;
            exch.second_token_balance -= buy_token_amount;
        } else {
            buy_token_id = exch.first_token_id;
            buy_token_amount = exchange(supply, exch.second_token_balance, exch.first_token_balance, self.quant);
            exch.first_token_balance -= buy_token_amount;
            exch.second_token_balance += self.quant;
        }

        log::debug!(
            "exchange sell #{}:{}, buy #{}:{}",
            sell_token_id,
            self.quant,
            buy_token_id,
            buy_token_amount
        );

        if sell_token_id == 0 {
            owner_acct.adjust_balance(-self.quant).unwrap();
        } else {
            owner_acct.adjust_token_balance(sell_token_id, -self.quant).unwrap();
        }
        if buy_token_id == 0 {
            owner_acct.adjust_balance(buy_token_amount).unwrap();
        } else {
            owner_acct.adjust_token_balance(buy_token_id, buy_token_amount).unwrap();
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

// NOTE: Sell and buy are different tokens.
// exchange(long sellTokenBalance, long buyTokenBalance, long sellTokenQuant)
/// Returns: buy token amount(buyTokenQuant).
fn exchange(mut supply: i64, sell_balance: i64, buy_balance: i64, sell_amount: i64) -> i64 {
    // exchangeToSupply(sellTokenBalance, sellTokenQuant)
    let new_balance = sell_balance + sell_amount;

    let issued_supply = -supply as f64 * (1.0 - java8_math_pow(1.0 + sell_amount as f64 / new_balance as f64, 0.0005));

    let relay = issued_supply as i64;
    supply += relay;

    // exchangeFromSupply(buyTokenBalance, relay)
    supply -= relay;
    // NOTE: OK to use Rust `f64::powf`, for X > 1.
    let exchange_balance = buy_balance as f64 * ((1.0 + relay as f64 / supply as f64).powf(2000.0) - 1.0);

    exchange_balance as i64
}

/// Get token id used by exchange. Use 0 to denote TRX.
#[inline]
fn get_exchange_token_id(manager: &Manager, token_id: &str) -> Result<i64, String> {
    if token_id == "_" {
        Ok(0)
    } else if manager.state_db.must_get(&keys::ChainParameter::AllowSameTokenName) != 0 {
        token_id.parse().map_err(|_| "invalid token id".into())
    } else {
        find_asset_by_name(manager, &token_id)
            .map(|asset| asset.id)
            .ok_or_else(|| "invalid token name".into())
    }
}

/// Java8 version of `Math.pow()` under x86.
///
/// See-also: https://github.com/opentron/opentron/issues/36
///
/// NOTE: This is a partial implementation, only operates on -1<=X<=1.
#[inline]
fn java8_math_pow(x: f64, y: f64) -> f64 {
    let mut ret = 0_f64;
    unsafe {
        asm!(
            "fld qword ptr [{y}]",
            "fld qword ptr [{x}]",
            "fyl2x",
            "f2xm1",
            "fld1",
            "faddp st(1), st(0)",
            "fstp qword ptr [{v}]",
            x = in(reg) &x,
            y = in(reg) &y,
            v = in(reg) &mut ret,
            options(nostack)
        );
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java8_math_pow_of_block_4137160() {
        let val = java8_math_pow(1.0061363892207218_f64, 0.0005_f64);
        let expected = 1.0000030588238054;
        let errored = 1.0000030588238051;

        assert_eq!(val, expected);
        assert!(val != errored);
    }
}
