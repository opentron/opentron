//! The TVM backend.

use ::keys::Address;
use primitive_types::{H160, H256, U256};
use proto2::state::{Account, AccountType, TransactionLog};
use state::db::StateDB;
use state::keys;
use tvm::backend::{Apply, ApplyBackend, Backend, Basic, Log};

use super::executor::TransactionContext;
use super::Manager;

/// StateDB backend, storing all state values in a RocksDB instance.
pub struct StateBackend<'m, 'c, 'ctx> {
    manager: &'m mut Manager,
    ctx: &'c mut TransactionContext<'ctx>,
    sender: Address,
}

impl<'m, 'c, 'ctx> StateBackend<'m, 'c, 'ctx> {
    /// Create a new StateDB backend.
    pub fn new(sender: Address, manager: &'m mut Manager, ctx: &'c mut TransactionContext<'ctx>) -> Self {
        Self { manager, ctx, sender }
    }

    /// Get the underlying `StateDB` storing the state.
    fn state(&self) -> &StateDB {
        &self.manager.state_db
    }

    /// Get the underlying `StateDB` storing the state, mutable.
    fn state_mut(&mut self) -> &mut StateDB {
        &mut self.manager.state_db
    }
}

#[allow(unused_variables)]
impl Backend for StateBackend<'_, '_, '_> {
    fn gas_price(&self) -> U256 {
        U256::zero()
    }

    // ORIGIN opcode, the transaction sender.
    fn origin(&self) -> H160 {
        H160::from_slice(self.sender.as_tvm_bytes())
    }

    fn block_hash(&self, number: U256) -> H256 {
        unimplemented!()
    }

    fn block_number(&self) -> U256 {
        self.ctx.block_header.number().into()
    }
    fn block_coinbase(&self) -> H160 {
        H160::from_slice(&self.ctx.block_header.witness()[1..])
    }
    fn block_timestamp(&self) -> U256 {
        self.ctx.block_header.timestamp().into()
    }
    fn block_difficulty(&self) -> U256 {
        0.into()
    }
    fn block_gas_limit(&self) -> U256 {
        0.into()
    }

    fn chain_id(&self) -> U256 {
        unimplemented!("hash of block #0")
    }

    fn exists(&self, address: H160) -> bool {
        let addr = Address::from_tvm_bytes(address.as_bytes());
        println!("!!! ex address {:?} {}", address, addr);
        self.state().get(&keys::Account(addr)).unwrap().is_some()
    }

    fn basic(&self, address: H160) -> Basic {
        let addr = Address::from_tvm_bytes(address.as_bytes());
        self.state()
            .get(&keys::Account(addr))
            .unwrap()
            .map(|a| Basic {
                balance: a.balance.into(),
                nonce: 0.into(),
                token_balance: a.token_balance.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
            })
            .unwrap_or_default()
    }

    fn code_hash(&self, address: H160) -> H256 {
        unimplemented!()
    }

    fn code_size(&self, address: H160) -> usize {
        unimplemented!()
    }

    fn code(&self, address: H160) -> Vec<u8> {
        unimplemented!()
    }

    fn storage(&self, address: H160, index: H256) -> Option<H256> {
        // debug!("!storage {:?} {:?}", address, index);
        // NOTE: ContractStorage must not save value of 0.
        self.state()
            .get(&keys::ContractStorage(
                Address::from_tvm_bytes(address.as_bytes()),
                index,
            ))
            .expect("db query")
    }
}

impl ApplyBackend for StateBackend<'_, '_, '_> {
    fn apply<A, I, L>(&mut self, values: A, logs: L, delete_empty: bool)
    where
        A: IntoIterator<Item = Apply<I>>,
        I: IntoIterator<Item = (H256, H256)>,
        L: IntoIterator<Item = Log>,
    {
        for apply in values {
            match apply {
                Apply::Modify {
                    address,
                    basic,
                    code,
                    storage,
                    reset_storage,
                } => {
                    let addr = Address::from_tvm_bytes(address.as_bytes());
                    if delete_empty &&
                        basic.balance == U256::zero() &&
                        basic.nonce == U256::zero() &&
                        code.is_none() &&
                        basic.token_balance.is_empty()
                    {
                        unimplemented!("TODO: delete empty");
                    }

                    let mut account = self
                        .state()
                        .get(&keys::Account(addr))
                        .expect("db query")
                        .unwrap_or_else(|| Account::new(self.manager.latest_block_timestamp()));

                    account.balance = basic.balance.as_u64() as i64;
                    for (token_id, token_value) in basic.token_balance {
                        account
                            .adjust_token_balance(token_id.as_u64() as i64, token_value.as_u64() as i64)
                            .unwrap();
                    }
                    // account.nonce = basic.nonce;
                    if let Some(code) = code {
                        self.state_mut().put_key(keys::ContractCode(addr), code).unwrap();
                        account.r#type = AccountType::Contract as i32;
                    }

                    if reset_storage {
                        unimplemented!("TODO: reset_storage")
                    }

                    for (index, value) in storage {
                        log::debug!("set storage: ({}, {}) => {}", addr, index, value);
                        if value == H256::default() {
                            self.state_mut()
                                .delete_key(&keys::ContractStorage(addr, index))
                                .unwrap();
                        } else {
                            self.state_mut()
                                .put_key(keys::ContractStorage(addr, index), value)
                                .unwrap();
                        }
                    }
                }
                Apply::Delete { address } => {
                    let addr = Address::from_tvm_bytes(address.as_bytes());
                    self.state_mut().delete_key(&keys::Account(addr)).unwrap();
                    unimplemented!("TODO: delete account")
                }
            }
        }

        for Log { address, topics, data } in logs {
            // let addr = Address::from_tvm_bytes(address.as_bytes());
            self.ctx.logs.push(TransactionLog {
                address: Address::from_tvm_bytes(address.as_bytes()).as_bytes().to_vec(),
                topics: topics.iter().map(|t| t.as_bytes().to_vec()).collect(),
                data: data,
            });
        }
    }
}
