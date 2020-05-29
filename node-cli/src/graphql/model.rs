use chain::{IndexedBlock, IndexedTransaction};
use chrono::{DateTime, TimeZone, Utc};
use juniper::meta::MetaType;
use juniper::FieldResult;
use juniper::{GraphQLType, Registry, ScalarValue};
use keys::Address;
use primitives::H256;
use std::convert::TryFrom;
use std::str;
use std::sync::Arc;

use super::contract::Contract;
use crate::context::AppContext;

#[derive(juniper::GraphQLEnum, PartialEq, Eq)]
#[repr(i32)]
enum ContractReturn {
    Default = 0,
    Success = 1,
    Revert = 2,
    IllegalOperation = 8,
    OutOfEnergy = 10,
    OutOfTime = 11,
    TransferFailed = 14,
    BadJumpDestination = 3,
    OutOfMemory = 4,
    PrecompiledContract = 5,
    StackTooSmall = 6,
    StackTooLarge = 7,
    StackOverflow = 9,
    JvmStackOverFlow = 12,
    Unknown = 13,
}

impl ContractReturn {
    fn from_i32(val: i32) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}

#[derive(juniper::GraphQLObject)]
struct RawTransaction {
    contract: Contract,
    timestamp: Option<DateTime<Utc>>,
    expiration: DateTime<Utc>,
    ref_block_bytes: String,
    ref_block_hash: String,
    permission_id: i32,
    fee_limit: i32,
    memo: Option<String>,
}

#[derive(juniper::GraphQLObject)]
/// A transaction of blockchain.
pub struct Transaction {
    /// Transaction hash.
    id: String,
    /// Signature of the transaction,
    signatures: Vec<String>,
    /// Inner transaction.
    inner: RawTransaction,
    contract_return: Option<ContractReturn>,
}

impl From<IndexedTransaction> for Transaction {
    fn from(txn: IndexedTransaction) -> Self {
        let IndexedTransaction { hash, mut raw } = txn;
        let origin_contract = raw.raw_data.as_mut().unwrap().contract.take().unwrap();
        let raw_txn = raw.raw_data.as_ref().unwrap();
        let permission_id = origin_contract.permission_id;
        // NOTE: take first result
        let result = raw.result.into_iter().next().unwrap_or_default();

        let inner = RawTransaction {
            contract: origin_contract.into(),
            timestamp: if raw_txn.timestamp != 0 {
                Some(Utc.timestamp(raw_txn.timestamp / 1_000, raw_txn.expiration as u32 % 1_000 * 1_000000))
            } else {
                None
            },
            expiration: Utc.timestamp(raw_txn.expiration / 1_000, raw_txn.expiration as u32 % 1_000 * 1_000000),
            ref_block_bytes: hex::encode(&raw_txn.ref_block_bytes),
            ref_block_hash: hex::encode(&raw_txn.ref_block_hash),
            permission_id,
            fee_limit: raw_txn.fee_limit as _,
            memo: if !raw_txn.data.is_empty() {
                str::from_utf8(&raw_txn.data)
                    .map(|s| s.to_owned())
                    .ok()
                    .or_else(|| Some(hex::encode(&raw_txn.data)))
            } else {
                None
            },
        };
        Transaction {
            id: hex::encode(hash.as_bytes()),
            signatures: raw.signatures.iter().map(|sig| hex::encode(sig)).collect(),
            inner,
            contract_return: Some(ContractReturn::from_i32(result.contract_status)),
        }
    }
}

#[derive(juniper::GraphQLObject)]
/// A block, on the block chain.
pub struct Block {
    /// Block hash.
    id: String,
    /// Block number.
    number: i32,
    /// Block timestamp.
    timestamp: DateTime<Utc>,
    /// Parent hash.
    parent_hash: String,
    /// Hash of all the hashes of all the transactions.
    merkle_root_hash: String,
    /// Block version.
    version: i32,
    /// Witness address of the block.
    witness: String,
    /// Signature of the witness.
    witness_signature: String,
    /// The transactions in this block.
    transactions: Vec<Transaction>,
}

#[derive(juniper::GraphQLObject)]
/// Misc node info
pub struct NodeInfo {
    /// Running code version.
    code_version: String,
    /// Is node syncing.
    syncing: bool,
    /// Number of currently running compactions.
    num_running_compactions: i32,
    /// Number of currently running flushes.
    num_running_flushes: i32,
    /// Number of immutable memtables that have not yet been flushed.
    num_immutable_mem_table: i32,
    /// If write has been stopped.
    is_write_stopped: bool,
    /// Total size (bytes) of all SST files belong to the latest LSM tree.
    total_size: f64,
}

#[derive(Clone)]
pub(crate) struct Context {
    pub app: Arc<AppContext>,
}

impl<S> GraphQLType<S> for Context
where
    S: ScalarValue,
{
    type Context = Self;
    type TypeInfo = ();

    fn name(_: &()) -> Option<&str> {
        Some("_Context")
    }

    fn meta<'r>(_: &(), registry: &mut Registry<'r, S>) -> MetaType<'r, S>
    where
        S: 'r,
    {
        registry.build_object_type::<Self>(&(), &[]).into_meta()
    }
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

impl Context {
    pub fn get_node_info(&self) -> NodeInfo {
        let ref db = self.app.db;
        NodeInfo {
            code_version: "0.1.0".to_owned(),
            syncing: *self.app.syncing.read().unwrap(),
            num_running_compactions: db.get_db_property("rocksdb.num-running-compactions") as _,
            num_running_flushes: db.get_db_property("rocksdb.num-running-flushes") as _,
            num_immutable_mem_table: db.get_accumulated_db_property("rocksdb.num-immutable-mem-table") as _,
            is_write_stopped: db.get_accumulated_db_property("rocksdb.is-write-stopped") > 0,
            total_size: db.get_accumulated_db_property("rocksdb.live-sst-files-size") as _,
        }
    }

    pub fn get_block(&self, id: Option<String>, num: Option<i32>) -> FieldResult<Block> {
        let block = match (id, num) {
            (Some(_), Some(_)) => return Err("either query by id or block num".into()),
            (Some(id), _) => {
                let block_id = H256::from_slice(&hex::decode(&id)?);
                self.app.db.get_block_by_hash(&block_id)?
            }
            (_, Some(num)) => self.app.db.get_block_by_number(num as _)?,
            (None, None) => self.app.db.highest_block()?,
        };

        let IndexedBlock { header, transactions } = block;
        let raw_header = header.raw.raw_data.as_ref().unwrap();

        let transactions = transactions.into_iter().map(From::from).collect();

        Ok(Block {
            id: hex::encode(header.hash.as_bytes()),
            number: header.number() as _,
            timestamp: Utc.timestamp(raw_header.timestamp / 1_000, 0),
            witness: Address::try_from(&raw_header.witness_address)
                .map(|addr| addr.to_string())
                .unwrap_or_else(|_| String::from_utf8(raw_header.witness_address.clone()).unwrap()),
            parent_hash: hex::encode(&raw_header.parent_hash),
            merkle_root_hash: hex::encode(&raw_header.merkle_root_hash),
            version: raw_header.version,
            witness_signature: hex::encode(&header.raw.witness_signature),
            transactions: transactions,
        })
    }

    pub fn get_transaction(&self, id: String) -> FieldResult<Transaction> {
        let txn_id = H256::from_slice(&hex::decode(&id)?);
        let txn = self.app.db.get_transaction_by_id(&txn_id).map(From::from)?;
        Ok(txn)
    }
}
