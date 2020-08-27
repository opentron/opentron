//! The TVM backend.

use ::keys::Address;
use log::debug;
use primitive_types::{H160, H256, U256};
use state::db::StateDB;
use state::keys;
use tvm::backend::{Apply, ApplyBackend, Backend, Basic, Log};

use super::executor::TransactionContext;
use super::Manager;

/// StateDB backend, storing all state values in a RocksDB instance.
pub struct StateBackend<'m, 'c> {
    manager: &'m Manager,
    ctx: &'c TransactionContext<'c>,
}

impl<'m, 'c> StateBackend<'m, 'c> {
    /// Create a new StateDB backend.
    pub fn new(manager: &'m Manager, ctx: &'c TransactionContext<'c>) -> Self {
        Self { manager, ctx }
    }

    /// Get the underlying `StateDB` storing the state.
    pub fn state(&self) -> &StateDB {
        &self.manager.state_db
    }
}

impl<'m, 'c> Backend for StateBackend<'m, 'c> {
    fn gas_price(&self) -> U256 {
        U256::zero()
    }
    fn origin(&self) -> H160 {
        H160::from_slice(Address::default().as_tvm_bytes())
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

    fn storage(&self, address: H160, index: H256) -> H256 {
        debug!("!storage {:?} {:?}", address, index);
        self.state()
            .get(&keys::ContractStorage(
                Address::from_tvm_bytes(address.as_bytes()),
                index,
            ))
            .unwrap()
            .unwrap_or_default()
    }
}
