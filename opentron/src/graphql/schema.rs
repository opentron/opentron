use std::convert::TryFrom;
use std::sync::{Arc, RwLock};

use ::state::keys;
use async_graphql::{Context, Enum, FieldError, FieldResult, InputObject, Object, SimpleObject};
use byteorder::{ByteOrder, BE};
use chain::{IndexedBlockHeader, IndexedTransaction};
use primitive_types::H256;
use proto2::state;
use std::mem;

use super::model::NodeInfo;
use super::scalar::{Address, Bytes, Bytes32, Long};
use super::contract::Contract;
use crate::context::AppContext;

const CODE_VERSION: &'static str = "0.1.0";
const API_VERSION: &'static str = "0.1.0";
const MAX_NUMBER_OF_BATCH_ITEMS_PER_REQUEST: i64 = 1000;

/// Account is an Tron account.
pub struct Account {
    address: Address,
    inner: state::Account,
}

#[Object]
impl Account {
    /// Address is the address owning the account.
    async fn address(&self) -> Address {
        self.address
    }

    /// Balance is the balance of the account, in sun.
    async fn balance(&self) -> Long {
        Long(self.inner.balance)
    }

    /// Code contains the smart contract code for this account, if the account
    /// is a (non-self-destructed) contract.
    async fn code(&self, ctx: &Context<'_>) -> FieldResult<Bytes> {
        if self.inner.r#type != state::AccountType::Contract as i32 {
            return Ok(Bytes(vec![]));
        }
        let ref manager = ctx.data_unchecked::<Arc<AppContext>>().manager.read().unwrap();
        manager
            .state()
            .get(&keys::ContractCode(self.address.0))
            .map(|maybe_code| maybe_code.unwrap_or_default())
            .map(Bytes)
            .map_err(From::from)
    }

    /// Storage provides access to the storage of a contract account, indexed
    /// by its 32 byte slot identifier.
    async fn storage(&self, ctx: &Context<'_>, slot: Bytes32) -> FieldResult<Bytes32> {
        if self.inner.r#type != state::AccountType::Contract as i32 {
            return Ok(Bytes32::from(H256::zero()));
        }
        let ref manager = ctx.data_unchecked::<Arc<AppContext>>().manager.read().unwrap();
        let val = manager
            .state()
            .get(&keys::ContractStorage(self.address.0, slot.0))?
            .unwrap_or_default();
        Ok(Bytes32(val))
    }

    /// Token balance of token id, in minimum unit.
    async fn token_balance(&self, id: i64) -> Long {
        Long(self.inner.token_balance.get(&id).copied().unwrap_or_default())
    }

    async fn name(&self) -> &str {
        &self.inner.name
    }

    async fn r#type(&self) -> i32 {
        self.inner.r#type
    }
}

#[derive(Debug)]
enum BlockIdentifier {
    Number(Long),
    Hash(Bytes32),
}

/// Block is a Tron block.
pub struct Block {
    /// Either block number or block hash.
    identifier: BlockIdentifier,
    header: RwLock<Option<IndexedBlockHeader>>,
    transactions: RwLock<Option<Vec<IndexedTransaction>>>,
}

impl Block {
    fn from_number(num: Long) -> Block {
        Block {
            identifier: BlockIdentifier::Number(num),
            header: RwLock::default(),
            transactions: RwLock::default(),
        }
    }

    fn from_hash(hash: Bytes32) -> Block {
        Block {
            identifier: BlockIdentifier::Hash(hash),
            header: RwLock::default(),
            transactions: RwLock::default(),
        }
    }

    fn require_header(&self, ctx: &Context<'_>) -> FieldResult<()> {
        if self.header.read().unwrap().is_none() {
            let ref db = ctx.data_unchecked::<Arc<AppContext>>().chain_db;
            match self.identifier {
                BlockIdentifier::Hash(hash) => {
                    let header = db.get_block_header(&hash.0)?;
                    *self.header.write().unwrap() = Some(header);
                }
                BlockIdentifier::Number(num) => {
                    let header = db.get_block_header_by_number(num.0)?;
                    *self.header.write().unwrap() = Some(header);
                }
            }
        }
        Ok(())
    }

    fn require_txns(&self, ctx: &Context<'_>) -> FieldResult<()> {
        if self.transactions.read().unwrap().is_none() {
            self.require_header(ctx)?;
            if let Some(ref header) = *self.header.read().unwrap() {
                if H256::from_slice(header.merkle_root_hash()) == H256::zero() {
                    // empty transaction
                    *self.transactions.write().unwrap() = Some(vec![]);
                } else {
                    let ref block_hash = header.hash;
                    let ref db = ctx.data_unchecked::<Arc<AppContext>>().chain_db;
                    let txns = db.get_block_transactions(block_hash)?;
                    *self.transactions.write().unwrap() = Some(txns);
                }
            }
        }
        Ok(())
    }
}

#[Object]
impl Block {
    /// Number is the number of this block, starting at 0 for the genesis block.
    async fn number(&self) -> Long {
        match self.identifier {
            BlockIdentifier::Number(num) => num,
            BlockIdentifier::Hash(hash) => Long(BE::read_u64(&hash.0.as_bytes()[..8]) as i64),
        }
    }

    /// Hash is the block hash of this block.
    async fn hash(&self, ctx: &Context<'_>) -> FieldResult<Bytes32> {
        match self.identifier {
            BlockIdentifier::Hash(hash) => Ok(hash),
            _ => {
                self.require_header(ctx)?;
                self.header
                    .read()
                    .unwrap()
                    .as_ref()
                    .map(|header| header.hash.into())
                    .ok_or_else(|| unreachable!())
            }
        }
    }

    /// Parent is the parent block of this block.
    async fn parent(&self, ctx: &Context<'_>) -> FieldResult<Option<Block>> {
        if let Some(ref header) = *self.header.read().unwrap() {
            if header.number() == 0 {
                return Ok(None);
            } else {
                let block_hash = H256::from_slice(header.parent_hash());
                return Ok(Some(Block::from_hash(block_hash.into())));
            }
        }
        let num = self.number(ctx).await?;
        if num.0 == 0 {
            Ok(None)
        } else {
            Ok(Some(Block::from_number(Long(num.0 - 1))))
        }
    }

    /// TransactionsRoot is the hash of the root of the trie of transactions in this block.
    // "txTrieRoot" in TRON
    async fn transactions_root(&self, ctx: &Context<'_>) -> FieldResult<Bytes32> {
        self.require_header(ctx)?;
        if let Some(ref header) = *self.header.read().unwrap() {
            let txn_root = H256::from_slice(header.merkle_root_hash());
            Ok(txn_root.into())
        } else {
            unreachable!()
        }
    }

    /// TransactionCount is the number of transactions in this block. if
    /// transactions are not available for this block, this field will be null.
    async fn transaction_count(&self, ctx: &Context<'_>) -> FieldResult<Option<i32>> {
        self.require_txns(ctx)?;
        if let Some(ref txns) = *self.transactions.read().unwrap() {
            Ok(Some(txns.len() as _))
        } else {
            Ok(None) // unreachable ??
        }
    }

    /// Witness is the account that mined this block.
    async fn witness(&self, ctx: &Context<'_>) -> FieldResult<Account> {
        self.require_header(ctx)?;
        match *self.header.read().unwrap() {
            Some(ref header) => {
                let addr = ::keys::Address::try_from(header.witness())?;
                let ref manager = ctx.data_unchecked::<Arc<AppContext>>().manager.read().unwrap();

                let acct = manager
                    .state()
                    .get(&keys::Account(addr))?
                    .ok_or_else(|| "account not found")?;
                Ok(Account {
                    address: addr.into(),
                    inner: acct,
                })
            }
            _ => unreachable!(),
        }
    }

    /// Timestamp is the unix timestamp at which this block was mined.
    async fn timestamp(&self, ctx: &Context<'_>) -> FieldResult<i64> {
        self.require_header(ctx)?;
        match *self.header.read().unwrap() {
            Some(ref header) => Ok(header.timestamp()),
            _ => unreachable!(),
        }
    }

    /// Transactions is a list of transactions associated with this block. If
    /// transactions are unavailable for this block, this field will be null.
    async fn transactions(&self, ctx: &Context<'_>) -> FieldResult<Vec<Transaction>> {
        self.require_txns(ctx)?;
        match *self.transactions.read().unwrap() {
            Some(ref txns) => {
                // TODO: avoid clone
                Ok(txns.iter().cloned().map(|txn| Transaction { inner: txn }).collect())
            }
            _ => unreachable!(),
        }
    }

    /// TransactionAt returns the transaction at the specified index. If
    /// transactions are unavailable for this block, or if the index is out of
    /// bounds, this field will be null.
    async fn transaction_at(&self, ctx: &Context<'_>, index: i32) -> FieldResult<Option<Transaction>> {
        self.require_txns(ctx)?;
        match self.transactions.read().unwrap().as_ref().unwrap().get(index as usize) {
            // TODO: avoid clone
            Some(txn) => Ok(Some(Transaction { inner: txn.clone() })),
            _ => Ok(None),
        }
    }

    /// Logs returns a filtered set of logs from this block.
    async fn logs(&self, _ctx: &Context<'_>, _filter: BlockFilterCriteria) -> FieldResult<Vec<Log>> {
        unimplemented!()
    }

    // eip1767:
    //
    // nonce
    // stateRoot
    // receiptsRoot
    // miner: = witness
    // extraData
    // gasLimit
    // gasUsed
    // logsBloom
    // mixHash
    // difficulty
    // totalDifficulty
    // ommerCount
    // ommers
    // ommerAt
    // ommerHash
    // account, call, estimateGas: block state not supported
}

/// Rename from `ContractStatus`, or `contractResult`.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[repr(i32)]
enum VmStatus {
    Default = 0,
    Success = 1,
    Revert = 2,
    IllegalOperation = 8,
    OutOfTime = 11,
    OutOfEnergy = 10,
    TransferFailed = 14,
    Unknown = 13,
}

/// Transaction is a Tron transaction.
pub struct Transaction {
    inner: IndexedTransaction,
}

#[Object]
impl Transaction {
    /// Hash is the hash of this transaction.
    async fn hash(&self) -> Bytes32 {
        Bytes32(self.inner.hash)
    }

    /// Index is the index of this transaction in the parent block. This will
    /// be null if the transaction has not yet been mined.
    async fn index(&self, ctx: &Context<'_>) -> FieldResult<i32> {
        let ref db = ctx.data_unchecked::<Arc<AppContext>>().chain_db;
        Ok(db.get_transaction_index(&self.inner.hash)?)
    }

    /// Block is the block this transaction was mined in. This will be null if
    /// the transaction has not yet been mined.
    async fn block(&self, ctx: &Context<'_>) -> FieldResult<Block> {
        let ref db = ctx.data_unchecked::<Arc<AppContext>>().chain_db;
        let block_hash = db.get_transaction_block_hash(&self.inner.hash)?;
        Ok(Block::from_hash(Bytes32(block_hash)))
    }

    /// Return status of TVM. Only meaningful for VM involved transactions.
    async fn vm_status(&self) -> VmStatus {
        let maybe_result = self.inner.raw.result.get(0);
        let contract_status = maybe_result.map(|ret| ret.contract_status).unwrap_or_default();
        // FIXME: unsafe
        unsafe { mem::transmute(contract_status) }
    }

    /*
    /// Builtin contract type.
    async fn contract_type(&self) -> String {
        let cntr = self.inner.raw.raw_data.as_ref().unwrap().contract.as_ref().unwrap();
        format!("{:?}", ContractType::from_i32(cntr.r#type).unwrap())
    }
    */

    /// Permission ID of this transaction. 0 for owner permission, 2 and above for active permission.
    async fn permission_id(&self) -> i32 {
        let cntr = self.inner.raw.raw_data.as_ref().unwrap().contract.as_ref().unwrap();
        cntr.permission_id
    }

    /// Expiration timestamp of this transaction.
    async fn expiration(&self) -> i64 {
        self.inner.raw.raw_data.as_ref().unwrap().expiration
    }

    /// Memo data of this transaction.
    async fn memo(&self) -> Bytes {
        Bytes(self.inner.raw.raw_data.as_ref().unwrap().data.clone())
    }

    /// Inner system contract of this transaction. (builtin contract)
    async fn contract(&self) -> Contract {
        let cntr = self.inner.raw.raw_data.as_ref().unwrap().contract.as_ref().unwrap();
        Contract::from(cntr)
    }

    // eip1767:
    //
    // nonce
    // status
    // gasUsed
    // cumulativeGasUsed
}

/// # Log is a Tron event log.
pub struct Log {}

#[Object]
impl Log {}

#[derive(InputObject)]
/// FilterCriteria encapsulates log filter criteria for searching log entries.
struct FilterCriteria {
    /// FromBlock is the block at which to start searching, inclusive. Defaults
    /// to the latest block if not supplied.
    from_block: Option<Long>,
    /// ToBlock is the block at which to stop searching, inclusive. Defaults
    /// to the latest block if not supplied.
    to_block: Option<Long>,
    /// Addresses is a list of addresses that are of interest. If this list is
    /// empty, results will not be filtered by address.
    // [Address!]
    addresses: Option<Vec<Address>>,
    /// Topics list restricts matches to particular event topics. Each event has a list
    /// of topics. Topics matches a prefix of that list. An empty element array matches any
    /// topic. Non-empty elements represent an alternative that matches any of the
    /// contained topics.
    ///
    /// Examples:
    ///  - [] or nil          matches any topic list
    ///  - [[A]]              matches topic A in first position
    ///  - [[], [B]]          matches any topic in first position, B in second position
    ///  - [[A], [B]]         matches topic A in first position, B in second position
    ///  - [[A, B]], [C, D]]  matches topic (A OR B) in first position, (C OR D) in second position
    // [[Bytes32!]!]
    topics: Option<Vec<Vec<Bytes32>>>,
}

#[derive(InputObject)]
/// BlockFilterCriteria encapsulates log filter criteria for a filter applied
/// to a single block.
struct BlockFilterCriteria {
    /// Addresses is a list of addresses that are of interest. If this list is
    /// empty, results will not be filtered by address.
    // [Address!]
    addresses: Option<Vec<Address>>,
    /// Topics list restricts matches to particular event topics. Each event has a list
    /// of topics. Topics matches a prefix of that list. An empty element array matches any
    /// topic. Non-empty elements represent an alternative that matches any of the
    /// contained topics.
    ///
    /// Examples:
    ///  - [] or nil          matches any topic list
    ///  - [[A]]              matches topic A in first position
    ///  - [[], [B]]          matches any topic in first position, B in second position
    ///  - [[A], [B]]         matches topic A in first position, B in second position
    ///  - [[A, B]], [C, D]]  matches topic (A OR B) in first position, (C OR D) in second position
    // [[Bytes32!]!]
    topics: Option<Vec<Vec<Bytes32>>>,
}

/// CallData represents the data associated with a local contract call.
/// All fields are optional.
#[derive(InputObject)]
pub struct CallData {
    /// From is the address making the call.
    from: Option<Address>,
    /// To is the address the call is sent to. (contract address)
    to: Option<Address>,
    /// FeeLimit is the max amount of energy sent with the call.
    fee_limit: Option<Long>,
    /// Data is the data sent to the callee.
    data: Option<Bytes>,
    /// Value is the value, in sun, sent along with the call.
    value: Option<Long>,
    /// TokenId is the TRC10 token ID.
    token_id: Option<i64>,
    /// TokenValue is the TRC10 token value.
    token_value: Option<Long>,
}

/// CallResult is the result of a local call operation.
#[derive(SimpleObject)]
pub struct CallResult {
    /// FromData is the return data of the called contract.
    data: Bytes,
    /// EnergyUsed is the amount of gas used by the call, after any refunds.
    energy_used: Long,
    /// VmStatus is the result of the call.
    vm_status: VmStatus,
}

/// SyncState contains the current synchronisation state of the client.
#[derive(SimpleObject)]
pub struct SyncState {
    /// StartingBlock is the block number at which synchronisation started.
    starting_block: Long,
    /// CurrentBlock is the point at which synchronisation has presently reached.
    current_block: Long,
    /// HighestBlock is the latest known block number.
    highest_block: Long,
    /// PulledStates is the number of state entries fetched so far, or null
    /// if this is not known or not relevant.
    pulled_states: Option<Long>,
    /// KnownStates is the number of states the node knows of so far, or null
    /// if this is not known or not relevant.
    known_states: Option<Long>,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Current API version.
    async fn api_version(&self, _ctx: &Context<'_>) -> &'static str {
        API_VERSION
    }

    /// Current Node info.
    async fn node_info(&self, ctx: &Context<'_>) -> NodeInfo {
        let ref db = ctx.data_unchecked::<Arc<AppContext>>().chain_db;
        NodeInfo {
            code_version: CODE_VERSION.into(),
            syncing: *ctx.data_unchecked::<Arc<AppContext>>().syncing.read().unwrap(),
            num_running_compactions: db.get_db_property("rocksdb.num-running-compactions") as _,
            num_running_flushes: db.get_db_property("rocksdb.num-running-flushes") as _,
            num_immutable_mem_table: db.get_accumulated_db_property("rocksdb.num-immutable-mem-table") as _,
            is_write_stopped: db.get_accumulated_db_property("rocksdb.is-write-stopped") > 0,
            total_size: db.get_accumulated_db_property("rocksdb.live-sst-files-size") as _,
        }
    }

    /// Block fetches an Tron block by number or by hash. If neither is
    /// supplied, the most recent known block is returned.
    async fn block(&self, ctx: &Context<'_>, number: Option<Long>, hash: Option<Bytes32>) -> FieldResult<Block> {
        if let Some(num) = number {
            return Ok(Block::from_number(num));
        }
        if let Some(hash) = hash {
            return Ok(Block::from_hash(hash));
        }
        let ref chain_db = ctx.data_unchecked::<Arc<AppContext>>().chain_db;
        let header = chain_db.get_block_header_by_number(chain_db.get_block_height())?;
        Ok(Block {
            identifier: BlockIdentifier::Hash(header.hash.into()),
            header: RwLock::new(Some(header)),
            transactions: RwLock::default(),
        })
    }

    /// Blocks returns all the blocks between two numbers, inclusive. If
    /// to is not supplied, it defaults to the most recent known block.
    async fn blocks(&self, ctx: &Context<'_>, from: Long, to: Option<Long>) -> FieldResult<Vec<Block>> {
        let ref chain_db = ctx.data_unchecked::<Arc<AppContext>>().chain_db;
        let block_height = chain_db.get_block_height();

        if from.0 > block_height {
            return Err(FieldError::from("from is out of range"));
        }
        let to_num = match to {
            Some(to) => {
                if to.0 < from.0 {
                    return Err(FieldError::from("from <= to check failed"));
                }
                to.0
            }
            None => block_height,
        };
        if to_num - from.0 > MAX_NUMBER_OF_BATCH_ITEMS_PER_REQUEST {
            return Err(FieldError::from("exceeds the maximum number of items per request"));
        }
        Ok((from.0..=to_num).map(|num| Block::from_number(Long(num))).collect())
    }

    // # Pending returns the current pending state.
    // pending: Pending!

    /// Transaction returns a transaction specified by its hash.
    async fn transaction(&self, ctx: &Context<'_>, hash: Bytes32) -> FieldResult<Transaction> {
        let ref db = ctx.data_unchecked::<Arc<AppContext>>().chain_db;

        Ok(Transaction {
            inner: db.get_transaction_by_id(&hash.0)?,
        })
    }

    /// Logs returns log entries matching the provided filter.
    async fn logs(&self, _ctx: &Context<'_>, _filter: FilterCriteria) -> FieldResult<Vec<Log>> {
        unimplemented!()
    }

    /// Syncing returns information on the current synchronisation state.
    async fn syncing(&self, ctx: &Context<'_>) -> SyncState {
        let ref db = ctx.data_unchecked::<Arc<AppContext>>().chain_db;

        SyncState {
            starting_block: Long(db.get_block_height()),
            current_block: Long(db.get_block_height()),
            highest_block: Long(db.get_block_height()),
            pulled_states: None,
            known_states: None,
        }
    }

    // NOTE: Tron does not support block history, so the following query is moved from Block to Query.

    /// Account fetches an Tron account at the current block's state.
    async fn account(&self, ctx: &Context<'_>, address: Address) -> FieldResult<Account> {
        let ref manager = ctx.data_unchecked::<Arc<AppContext>>().manager.read().unwrap();

        let acct = manager
            .state()
            .get(&keys::Account(address.0))?
            .ok_or_else(|| "account not found")?;

        Ok(Account { address, inner: acct })
    }

    /// Call executes a local call operation at the current block's state.
    async fn call(&self, _ctx: &Context<'_>, _data: CallData) -> FieldResult<CallResult> {
        unimplemented!()
    }

    /// EstimateEnergy estimates the amount of energy that will be required for
    /// successful execution of a transaction at the current block's state.
    async fn estimate_energy(&self, _ctx: &Context<'_>, _data: CallData) -> FieldResult<Long> {
        unimplemented!()
    }
}
