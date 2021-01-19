use std::convert::TryFrom;
use std::mem;
use std::str;
use std::sync::{Arc, RwLock};

use async_graphql::{Context, Enum, Error, InputObject, Object, Result, SimpleObject};
use byteorder::{ByteOrder, BE};
use chrono::{DateTime, TimeZone, Utc};
use primitive_types::H256;

use ::state::keys;
use chain::{IndexedBlockHeader, IndexedTransaction};
use context::AppContext;
use proto::state;

use super::contract::{AccountType, Contract};
use super::model::NodeInfo;
use super::scalar::{Address, Bytes, Bytes32, Long};

const CODE_VERSION: &'static str = "0.1.0";
const API_VERSION: &'static str = "0.1.0";
const MAX_NUMBER_OF_BATCH_ITEMS_PER_REQUEST: i64 = 1000;

/// Account is an Tron account.
pub struct Account {
    address: Address,
    inner: RwLock<Option<state::Account>>,
}

impl Account {
    fn require_inner(&self, ctx: &Context<'_>) -> Result<()> {
        if self.inner.read().unwrap().is_none() {
            let ref manager = ctx.data_unchecked::<Arc<AppContext>>().manager.read().unwrap();
            let acct = manager
                .state()
                .get(&keys::Account(self.address.0))?
                .ok_or_else(|| "account not found")?;
            *self.inner.write().unwrap() = Some(acct);
        }
        Ok(())
    }
}

#[Object]
impl Account {
    /// Address is the address owning the account.
    async fn address(&self) -> Address {
        self.address
    }

    /// Balance is the balance of the account, in sun.
    async fn balance(&self, ctx: &Context<'_>) -> Result<Long> {
        self.require_inner(ctx)?;
        let inner = self.inner.read().unwrap();
        Ok(inner.as_ref().unwrap().balance.into())
    }

    /// Code contains the smart contract code for this account, if the account
    /// is a (non-self-destructed) contract.
    async fn code(&self, ctx: &Context<'_>) -> Result<Bytes> {
        self.require_inner(ctx)?;
        let inner = self.inner.read().unwrap();
        if inner.as_ref().unwrap().r#type != state::AccountType::Contract as i32 {
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
    async fn storage(&self, ctx: &Context<'_>, slot: Bytes32) -> Result<Bytes32> {
        self.require_inner(ctx)?;
        let inner = self.inner.read().unwrap();
        if inner.as_ref().unwrap().r#type != state::AccountType::Contract as i32 {
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
    async fn token_balance(&self, ctx: &Context<'_>, id: i64) -> Result<Long> {
        self.require_inner(ctx)?;
        let inner = self.inner.read().unwrap();
        Ok(inner
            .as_ref()
            .unwrap()
            .token_balance
            .get(&id)
            .copied()
            .unwrap_or(0)
            .into())
    }

    /// Allowance of the account.
    async fn allowance(&self, ctx: &Context<'_>) -> Result<Long> {
        self.require_inner(ctx)?;
        let inner = self.inner.read().unwrap();
        Ok(inner.as_ref().unwrap().allowance.into())
    }

    /// Name of this account.
    async fn name(&self, ctx: &Context<'_>) -> Result<String> {
        self.require_inner(ctx)?;
        let inner = self.inner.read().unwrap();
        Ok(inner.as_ref().unwrap().name.clone())
    }

    /// Type of this account.
    async fn r#type(&self, ctx: &Context<'_>) -> Result<AccountType> {
        self.require_inner(ctx)?;
        let inner = self.inner.read().unwrap();
        Ok(AccountType::from_i32(inner.as_ref().unwrap().r#type))
    }

    /// Tron Power of the account.
    async fn power(&self, ctx: &Context<'_>) -> Result<Long> {
        self.require_inner(ctx)?;
        let inner = self.inner.read().unwrap();
        Ok(inner.as_ref().unwrap().tron_power().into())
    }
}

/// Asset is a TRC10 token.
pub struct Asset(state::Asset);

#[Object]
impl Asset {
    /// Asset id, aka. token id.
    async fn id(&self) -> i64 {
        self.0.id
    }

    /// Asset name, used as identifier before AllowSameTokenName.
    async fn name(&self) -> &str {
        &self.0.name
    }

    /// Asset symbol, aka. abbr.
    async fn symbol(&self) -> &str {
        &self.0.abbr
    }

    /// Description of the asset.
    async fn description(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0.description) }
    }

    /// URL of the asset.
    async fn url(&self) -> &str {
        &self.0.url
    }

    /// Totol supply of the asset.
    async fn total_supply(&self) -> Long {
        self.0.total_supply.into()
    }

    /// Decimals of the asset, aka. precision.
    async fn decimals(&self) -> i32 {
        self.0.precision
    }

    /// Issuer of the asset.
    async fn owner(&self) -> Address {
        Address(TryFrom::try_from(&self.0.owner_address).unwrap())
    }

    /// Returns the amount of tokens owned by account.
    async fn balance_of(&self, ctx: &Context<'_>, account: Address) -> Result<Long> {
        let ref manager = ctx.data_unchecked::<Arc<AppContext>>().manager.read().unwrap();
        let acct = manager
            .state()
            .get(&keys::Account(account.0))?
            .ok_or_else(|| "account not found")?;
        Ok(acct.token_balance.get(&self.0.id).copied().unwrap_or(0).into())
    }
}

/// Rename from `ContractStatus`, or `contractResult`.
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
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

/// Log is a Tron event log.
pub struct Log {
    index: i32,
    inner: state::TransactionLog,
    txn_hash: H256,
}

#[Object]
impl Log {
    /// Index is the index of this log in the block.
    async fn index(&self) -> i32 {
        self.index
    }

    /// Account is the account which generated this log - this will always
    /// be a contract account.
    async fn account(&self) -> Account {
        let address = TryFrom::try_from(&self.inner.address).map(Address).unwrap();
        Account {
            address,
            inner: RwLock::default(),
        }
    }

    /// Topics is a list of 0-4 indexed topics for the log.
    async fn topics(&self) -> Vec<Bytes32> {
        self.inner
            .topics
            .iter()
            .map(|topic| H256::from_slice(topic).into())
            .collect()
    }

    /// Data is unindexed data for this log.
    async fn data(&self) -> Bytes {
        Bytes(self.inner.data.clone())
    }

    /// Transaction is the transaction that generated this log entry.
    async fn transaction(&self, ctx: &Context<'_>) -> Result<Transaction> {
        let ref db = ctx.data_unchecked::<Arc<AppContext>>().chain_db;
        Ok(Transaction {
            inner: db.get_transaction_by_id(&self.txn_hash)?,
        })
    }
}

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

impl FilterCriteria {
    fn matches(&self, log: &state::TransactionLog) -> bool {
        matches_addrs(&self.addresses, log) && matches_topics(&self.topics, &log.topics)
    }
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

impl BlockFilterCriteria {
    fn matches(&self, log: &state::TransactionLog) -> bool {
        matches_addrs(&self.addresses, log) && matches_topics(&self.topics, &log.topics)
    }
}

fn matches_addrs(addrs: &Option<Vec<Address>>, log: &state::TransactionLog) -> bool {
    if let Some(ref addrs) = *addrs {
        if !addrs.is_empty() {
            if addrs
                .iter()
                .find(|addr| addr.0.as_bytes() == &log.address[..])
                .is_none()
            {
                return false;
            }
        }
    }
    true
}

fn matches_topics(topics: &Option<Vec<Vec<Bytes32>>>, target: &[Vec<u8>]) -> bool {
    match *topics {
        None => true,
        Some(ref topics) if topics.is_empty() => true,
        Some(ref topics) if topics.len() > target.len() => false,
        Some(ref topics) => {
            for (lhs_topics, rhs) in topics.iter().zip(target.iter()) {
                if !lhs_topics.is_empty() {
                    if lhs_topics.iter().find(|topic| topic.0.as_bytes() == &rhs[..]).is_none() {
                        return false;
                    }
                }
            }
            true
        }
    }
}

/// CallData represents the data associated with a local contract call.
/// All fields are optional.
#[derive(InputObject)]
pub struct CallData {
    /// From is the address making the call.
    from: Option<Address>,
    /// To is the address the call is sent to. (contract address)
    to: Option<Address>,
    /// EnergyLimit is the max amount of energy sent with the call.
    energy_limit: Option<Long>,
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
pub struct CallResult {
    receipt: state::TransactionReceipt,
}

#[Object]
impl CallResult {
    /// FromData is the return data of the called contract.
    async fn data(&self) -> Bytes {
        Bytes(self.receipt.vm_result.clone())
    }
    /// EnergyUsed is the amount of energy used by the call, after any refunds.
    async fn energy_used(&self) -> Long {
        self.receipt
            .resource_receipt
            .as_ref()
            .map(|receipt| receipt.energy)
            .unwrap_or_default()
            .into()
    }
    /// BandwidthUsed is the amount of bandwidth used by the call.
    async fn bandwidth_used(&self) -> Long {
        self.receipt
            .resource_receipt
            .as_ref()
            .map(|receipt| receipt.bandwidth_usage)
            .unwrap_or_default()
            .into()
    }
    /// VmStatus is the result of the call.
    async fn vm_status(&self) -> VmStatus {
        unsafe { mem::transmute(self.receipt.vm_status) }
    }
    /// Result VM logs.
    async fn logs(&self) -> Vec<Log> {
        self.receipt
            .vm_logs
            .iter()
            .map(|log| Log {
                inner: log.clone(),
                index: 0,
                txn_hash: H256::default(),
            })
            .collect()
    }
}

/// SyncState contains the current synchronisation state of the client.
#[derive(SimpleObject)]
pub struct SyncState {
    /// CurrentBlock is the point at which synchronisation has presently reached.
    current_block: Long,
    /// HighestBlock is the latest known block number.
    highest_block: Long,
    /// SolidBlock is the safe block for comfirmations.
    solid_block: Long,
    /// StateBlock is the block number of StateDB.
    state_block: Long,
    /// PulledStates is the number of state entries fetched so far, or null
    /// if this is not known or not relevant.
    pulled_states: Option<Long>,
    /// KnownStates is the number of states the node knows of so far, or null
    /// if this is not known or not relevant.
    known_states: Option<Long>,
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
    async fn index(&self, ctx: &Context<'_>) -> Result<i32> {
        let ref db = ctx.data_unchecked::<Arc<AppContext>>().chain_db;
        Ok(db.get_transaction_index(&self.inner.hash)?)
    }

    /// Block is the block this transaction was mined in. This will be null if
    /// the transaction has not yet been mined.
    async fn block(&self, ctx: &Context<'_>) -> Result<Block> {
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

    /// From is the account that sent this transaction - this will always be
    /// an externally owned account.
    async fn from(&self) -> Account {
        let cntr = self.inner.raw.raw_data.as_ref().unwrap().contract.as_ref().unwrap();
        let address = Contract::from(cntr).owner_address();
        Account {
            address,
            inner: RwLock::default(),
        }
    }

    /// To is the account the transaction was sent to. This is null for
    /// contract-creating transactions.
    async fn to(&self) -> Option<Account> {
        let cntr = self.inner.raw.raw_data.as_ref().unwrap().contract.as_ref().unwrap();
        Contract::from(cntr).to_address().map(|address| Account {
            address,
            inner: RwLock::default(),
        })
    }

    // NOTE: for debug
    async fn receipt(&self, ctx: &Context<'_>) -> Result<String> {
        let ref manager = ctx.data_unchecked::<Arc<AppContext>>().manager.read().unwrap();
        if let Some(receipt) = manager.state().get(&keys::TransactionReceipt(self.inner.hash))? {
            Ok(format!(
                "resource_receipt={:?} vm_logs={}",
                receipt.resource_receipt,
                receipt.vm_logs.len()
            ))
        } else {
            Ok("not found".to_owned())
        }
    }
    // nonce
    // status
    // gasUsed
    // cumulativeGasUsed
    // value
    // gasPrice
    // gas
    // inputData
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

    fn require_header(&self, ctx: &Context<'_>) -> Result<()> {
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

    fn require_txns(&self, ctx: &Context<'_>) -> Result<()> {
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
    async fn hash(&self, ctx: &Context<'_>) -> Result<Bytes32> {
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
    async fn parent(&self, ctx: &Context<'_>) -> Result<Option<Block>> {
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
    async fn transactions_root(&self, ctx: &Context<'_>) -> Result<Bytes32> {
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
    async fn transaction_count(&self, ctx: &Context<'_>) -> Result<Option<i32>> {
        self.require_txns(ctx)?;
        if let Some(ref txns) = *self.transactions.read().unwrap() {
            Ok(Some(txns.len() as _))
        } else {
            Ok(None) // unreachable ??
        }
    }

    /// Witness is the account that mined this block.
    async fn witness(&self, ctx: &Context<'_>) -> Result<Account> {
        self.require_header(ctx)?;
        let header = self.header.read().unwrap();
        let address = ::keys::Address::try_from(header.as_ref().unwrap().witness())?;
        Ok(Account {
            address: address.into(),
            inner: RwLock::default(),
        })
    }

    /// Timestamp is the unix timestamp at which this block was mined.
    async fn timestamp(&self, ctx: &Context<'_>) -> Result<i64> {
        self.require_header(ctx)?;
        match *self.header.read().unwrap() {
            Some(ref header) => Ok(header.timestamp()),
            _ => unreachable!(),
        }
    }

    /// Transactions is a list of transactions associated with this block. If
    /// transactions are unavailable for this block, this field will be null.
    async fn transactions(&self, ctx: &Context<'_>) -> Result<Vec<Transaction>> {
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
    async fn transaction_at(&self, ctx: &Context<'_>, index: i32) -> Result<Option<Transaction>> {
        self.require_txns(ctx)?;
        match self.transactions.read().unwrap().as_ref().unwrap().get(index as usize) {
            // TODO: avoid clone
            Some(txn) => Ok(Some(Transaction { inner: txn.clone() })),
            _ => Ok(None),
        }
    }

    /// Logs returns a filtered set of logs from this block.
    async fn logs(&self, ctx: &Context<'_>, filter: BlockFilterCriteria) -> Result<Vec<Log>> {
        self.require_txns(ctx)?;
        let ref manager = ctx.data_unchecked::<Arc<AppContext>>().manager.read().unwrap();
        let mut logs = vec![];
        for (index, txn) in self.transactions.read().unwrap().as_ref().unwrap().iter().enumerate() {
            if let Some(receipt) = manager.state().get(&keys::TransactionReceipt(txn.hash))? {
                receipt
                    .vm_logs
                    .into_iter()
                    .filter(|log_entry| filter.matches(log_entry))
                    .for_each(|log_entry| {
                        logs.push(Log {
                            index: index as i32,
                            inner: log_entry,
                            txn_hash: txn.hash,
                        });
                    });
            }
        }
        Ok(logs)
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

#[derive(SimpleObject)]
pub struct ChainParameter {
    id: i32,
    key: String,
    value: i64,
}

pub struct Chain;

#[Object]
impl Chain {
    /// Chain parameters.
    async fn parameters(&self, ctx: &Context<'_>) -> Result<Vec<ChainParameter>> {
        let ref manager = ctx.data_unchecked::<Arc<AppContext>>().manager.read().unwrap();
        let mut params = Vec::with_capacity(50);
        {
            let params = &mut params;
            manager.state().for_each(move |key: &keys::ChainParameter, value| {
                params.push(ChainParameter {
                    id: *key as i32,
                    key: format!("{:?}", key),
                    value: *value,
                });
            });
        }
        Ok(params)
    }

    /// Get a chain parameter.
    async fn parameter(&self, ctx: &Context<'_>, id: i32) -> Result<ChainParameter> {
        let ref manager = ctx.data_unchecked::<Arc<AppContext>>().manager.read().unwrap();
        let param = keys::ChainParameter::from_i32(id).ok_or_else(|| "invalid parameter id")?;
        let value = manager.state().must_get(&param);
        Ok(ChainParameter {
            id: id,
            key: format!("{:?}", param),
            value,
        })
    }

    /// Next maintenance time.
    async fn next_maintenance_time(&self, ctx: &Context<'_>) -> DateTime<Utc> {
        let ref manager = ctx.data_unchecked::<Arc<AppContext>>().manager.read().unwrap();
        let ts = manager.state().must_get(&keys::DynamicProperty::NextMaintenanceTime);
        Utc.timestamp(ts / 1_000, ts as u32 % 1_000 * 1_000_000)
    }
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
        let app = ctx.data_unchecked::<Arc<AppContext>>();
        let ref db = app.chain_db;
        NodeInfo {
            code_version: CODE_VERSION,
            syncing: app.syncing.load(std::sync::atomic::Ordering::Relaxed),
            num_active_connections: app.num_active_connections.load(std::sync::atomic::Ordering::Relaxed),
            num_passive_connections: 0,
            num_running_compactions: db.get_db_property("rocksdb.num-running-compactions") as _,
            num_running_flushes: db.get_db_property("rocksdb.num-running-flushes") as _,
            num_immutable_mem_table: db.get_accumulated_db_property("rocksdb.num-immutable-mem-table") as _,
            is_write_stopped: db.get_accumulated_db_property("rocksdb.is-write-stopped") > 0,
            total_size: db.get_accumulated_db_property("rocksdb.live-sst-files-size") as _,
        }
    }

    /// Block fetches an Tron block by number or by hash. If neither is
    /// supplied, the most recent known block is returned.
    async fn block(&self, ctx: &Context<'_>, number: Option<Long>, hash: Option<Bytes32>) -> Result<Block> {
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
    async fn blocks(&self, ctx: &Context<'_>, from: Long, to: Option<Long>) -> Result<Vec<Block>> {
        let ref chain_db = ctx.data_unchecked::<Arc<AppContext>>().chain_db;
        let block_height = chain_db.get_block_height();

        if from.0 > block_height {
            return Err(Error::from("from is out of range"));
        }
        let to_num = match to {
            Some(to) => {
                if to.0 < from.0 {
                    return Err(Error::from("from <= to check failed"));
                }
                to.0
            }
            None => block_height,
        };
        if to_num - from.0 > MAX_NUMBER_OF_BATCH_ITEMS_PER_REQUEST {
            return Err(Error::from("exceeds the maximum number of blocks per request"));
        }
        Ok((from.0..=to_num).map(|num| Block::from_number(Long(num))).collect())
    }

    // # Pending returns the current pending state.
    // pending: Pending!

    /// Transaction returns a transaction specified by its hash.
    async fn transaction(&self, ctx: &Context<'_>, hash: Bytes32) -> Result<Transaction> {
        let ref db = ctx.data_unchecked::<Arc<AppContext>>().chain_db;
        Ok(Transaction {
            inner: db.get_transaction_by_id(&hash.0)?,
        })
    }

    /// Logs returns log entries matching the provided filter.
    async fn logs(&self, ctx: &Context<'_>, filter: FilterCriteria) -> Result<Vec<Log>> {
        let ref db = ctx.data_unchecked::<Arc<AppContext>>().chain_db;
        let ref manager = ctx.data_unchecked::<Arc<AppContext>>().manager.read().unwrap();
        let defaut_block = manager.latest_block_number();

        let from_block = filter.from_block.unwrap_or(defaut_block.into()).0;
        let to_block = filter.to_block.unwrap_or(defaut_block.into()).0;

        if from_block > to_block {
            return Err("fromBlock should be lower than toBlock".into());
        }

        if to_block - from_block > MAX_NUMBER_OF_BATCH_ITEMS_PER_REQUEST {
            return Err(Error::from("exceeds the maximum number of blocks per request"));
        }

        let mut logs = vec![];
        for block_num in from_block..=to_block {
            let txn_hashes = db.get_transaction_hashes_by_block_number(block_num)?;
            for (index, &txn_hash) in txn_hashes.iter().enumerate() {
                if let Some(receipt) = manager.state().get(&keys::TransactionReceipt(txn_hash))? {
                    receipt
                        .vm_logs
                        .into_iter()
                        .filter(|log_entry| filter.matches(log_entry))
                        .for_each(|log_entry| {
                            logs.push(Log {
                                index: index as i32,
                                inner: log_entry,
                                txn_hash: txn_hash,
                            });
                        });
                }
            }
        }
        Ok(logs)
    }

    /// Syncing returns information on the current synchronisation state.
    async fn syncing(&self, ctx: &Context<'_>) -> SyncState {
        let ref db = ctx.data_unchecked::<Arc<AppContext>>().chain_db;
        let ref manager = ctx.data_unchecked::<Arc<AppContext>>().manager.read().unwrap();

        SyncState {
            current_block: Long(db.get_block_height()),
            // FIXME: wrong impl
            highest_block: Long(db.get_block_height()),
            solid_block: Long(manager.solid_block_number()),
            state_block: Long(manager.latest_block_number()),
            pulled_states: None,
            known_states: None,
        }
    }

    // NOTE: Tron does not support block history, so the following query is moved from Block to Query.

    /// Account fetches an Tron account at the current block's state.
    async fn account(&self, ctx: &Context<'_>, address: Address) -> Result<Account> {
        let ref manager = ctx.data_unchecked::<Arc<AppContext>>().manager.read().unwrap();
        let acct = manager
            .state()
            .get(&keys::Account(address.0))?
            .ok_or_else(|| "account not found")?;

        Ok(Account {
            address,
            inner: RwLock::new(Some(acct)),
        })
    }

    /// Call executes a local call operation at the current block's state.
    async fn call(&self, ctx: &Context<'_>, data: CallData) -> Result<CallResult> {
        use manager::executor::TransactionExecutor;
        use proto::contract::TriggerSmartContract;

        let trigger = TriggerSmartContract {
            owner_address: data.from.unwrap_or_else(Default::default).0.as_bytes().to_vec(),
            contract_address: data.to.unwrap().0.as_bytes().to_vec(),
            data: data.data.unwrap().0,
            call_value: data.value.map(|val| val.0).unwrap_or_default(),
            call_token_id: data.token_id.unwrap_or_default(),
            call_token_value: data.token_value.map(|val| val.0).unwrap_or_default(),
        };

        let ref mut manager = ctx.data_unchecked::<Arc<AppContext>>().manager.write().unwrap();
        let energy_limit = data.energy_limit.map(|val| val.0).unwrap_or(100_000_000);

        let receipt = TransactionExecutor::new(manager).execute_smart_contract(&trigger, energy_limit)?;
        Ok(CallResult { receipt })
    }

    /// EstimateEnergy estimates the amount of energy that will be required for
    /// successful execution of a transaction at the current block's state.
    async fn estimate_energy(&self, ctx: &Context<'_>, data: CallData) -> Result<Long> {
        self.call(ctx, data).await.and_then(|result| {
            if result.receipt.vm_status == VmStatus::Default as i32 ||
                result.receipt.vm_status == VmStatus::Success as i32
            {
                Ok(result
                    .receipt
                    .resource_receipt
                    .map(|res| res.energy)
                    .unwrap_or(0)
                    .into())
            } else {
                Err(format!("vm execution result: {:?}", result.receipt.vm_status).into())
            }
        })
    }

    // Tron extensions.

    /// Asset fetches an Tron asset(TRC10 token).
    async fn asset(&self, ctx: &Context<'_>, issuer: Option<Address>, id: Option<i64>) -> Result<Asset> {
        let ref manager = ctx.data_unchecked::<Arc<AppContext>>().manager.read().unwrap();
        let token_id = match (issuer, id) {
            (None, Some(token_id)) => token_id,
            (Some(issuer_addr), None) => {
                let acct = manager
                    .state()
                    .get(&keys::Account(issuer_addr.0))?
                    .ok_or_else(|| "issuer not found")?;
                acct.issued_asset_id
            }
            _ => return Err("either issuer or asset id should be provided".into()),
        };
        let asset = manager
            .state()
            .get(&keys::Asset(token_id))?
            .ok_or_else(|| "asset not found")?;
        Ok(Asset(asset))
    }

    /// Chain query.
    async fn chain(&self) -> Chain {
        Chain
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// SendRawTransaction sends an protobuf-encoded transaction to the network.
    async fn send_raw_transaction(&self, _data: Bytes) -> Result<Bytes32> {
        unimplemented!()
    }

    /// DryRunRawTransaction runs an protobuf-encoded transaction and returns the receipt as json.
    async fn dry_run_raw_transaction(&self, ctx: &Context<'_>, data: Bytes) -> Result<CallResult> {
        use chain::IndexedTransaction;
        use prost::Message;
        use proto::chain::Transaction;

        let ref mut manager = ctx.data_unchecked::<Arc<AppContext>>().manager.write().unwrap();

        let txn = Transaction::decode(&*data.0)?;
        let indexed_txn = IndexedTransaction::from_raw(txn).ok_or("invalid transaction")?;

        let receipt = manager.dry_run_transaction(&indexed_txn)?;

        Ok(CallResult { receipt })
    }
}
