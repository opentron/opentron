//! The state-db implementation.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::io;
use std::iter;
use std::path::Path;

use ::keys::Address;
use config::genesis::GenesisConfig;
use config::ChainConfig;
use log::info;
use proto2::common::AccountType;
use proto2::state as state_pb;
use rocks::prelude::*;

use super::keys;
use super::ChainParameter;
use super::DynamicProperty;
// use crate::constants;

pub type BoxError = Box<dyn ::std::error::Error>;

pub trait KeyValueDB {
    type Column;

    /// Get a value by key.
    fn get(&self, col: &Self::Column, key: &[u8]) -> io::Result<Option<Vec<u8>>>;

    /// Get the first value matching the given prefix.
    fn get_by_prefix(&self, col: &Self::Column, prefix: &[u8]) -> Option<Box<[u8]>>;

    /// Write a batch of changes to the backing store.
    fn write(&self, batch: &WriteBatch) -> io::Result<()>;

    /// Iterate over the data for a given column.
    fn iter<'a>(&'a self, col: &Self::Column) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a>;

    /// Iterate over the data for a given column, returning all key/value pairs
    /// where the key starts with the given prefix.
    fn iter_with_prefix<'a>(
        &'a self,
        col: &Self::Column,
        prefix: &'a [u8],
    ) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a>;
}

pub struct OverlayWriteBatch {
    wb: WriteBatch,
    // CF => (Key => Value)
    // TODO: replace with VecMap
    cache: HashMap<u32, BTreeMap<Vec<u8>, Option<Vec<u8>>>>,
}

impl OverlayWriteBatch {
    pub fn new() -> Self {
        OverlayWriteBatch {
            wb: WriteBatch::new(),
            cache: HashMap::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        OverlayWriteBatch {
            wb: WriteBatch::with_reserved_bytes(cap),
            cache: HashMap::new(),
        }
    }

    pub fn put(&mut self, col: &ColumnFamilyHandle, key: &[u8], value: &[u8]) {
        self.wb.put_cf(col, key, value);
        self.cache
            .entry(col.id())
            .or_default()
            .insert(key.to_owned(), Some(value.to_owned()));
    }

    pub fn delete(&mut self, col: &ColumnFamilyHandle, key: &[u8]) {
        self.wb.delete_cf(col, key);
        self.cache.entry(col.id()).or_default().insert(key.to_owned(), None);
    }

    // Ok(None) => deleted
    // Err(_)   => non-exist
    pub fn get(&self, col: &ColumnFamilyHandle, key: &[u8]) -> io::Result<Option<Vec<u8>>> {
        self.cache
            .get(&col.id())
            .and_then(|cf| cf.get(key).cloned())
            .ok_or(io::Error::new(io::ErrorKind::NotFound, ""))
    }

    // None => deleted or not-found
    pub fn get_by_prefix(&self, col: &ColumnFamilyHandle, prefix: &[u8]) -> Option<Box<[u8]>> {
        self.cache.get(&col.id()).and_then(|cf| {
            cf.iter()
                .filter(|(key, value)| key.starts_with(prefix) && value.is_some())
                .map(|(_, value)| value.clone().unwrap().into_boxed_slice())
                .next()
        })
    }

    pub fn iter<'a>(&'a self, col: &ColumnFamilyHandle) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a> {
        self.cache
            .get(&col.id())
            .map(|cf| {
                Box::new(cf.iter().filter(|(_, value)| value.is_some()).map(|(key, value)| {
                    (
                        key.to_vec().into_boxed_slice(),
                        value.clone().unwrap().into_boxed_slice(),
                    )
                })) as Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)>>
            })
            .unwrap_or_else(|| Box::new(iter::empty()) as Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)>>)
    }

    /// Iterate over the data for a given column, returning all key/value pairs
    /// where the key starts with the given prefix.
    pub fn iter_with_prefix<'a>(
        &'a self,
        col: &ColumnFamilyHandle,
        prefix: &'a [u8],
    ) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a> {
        self.cache
            .get(&col.id())
            .map(|cf| {
                Box::new(
                    cf.iter()
                        .filter(move |(key, value)| key.starts_with(prefix) && value.is_some())
                        .map(|(key, value)| {
                            (
                                key.to_vec().into_boxed_slice(),
                                value.clone().unwrap().into_boxed_slice(),
                            )
                        }),
                ) as Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)>>
            })
            .unwrap_or_else(|| Box::new(iter::empty()) as Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)>>)
    }
}

pub struct OverlayDB {
    inner: DB,
    // Use push_back to add to the queue, and pop_front to remove from the queue.
    // push_back to add a new layer, pop_front to sync a layer to db, clear to discard all layers.
    layers: VecDeque<OverlayWriteBatch>,
}

impl OverlayDB {
    pub fn new(inner: DB) -> Self {
        OverlayDB {
            inner,
            // ceiling(27 - 27 * 70%) = 9
            layers: VecDeque::with_capacity(9),
        }
    }

    /// Fake `write` an OverlayWriteBath.
    pub fn write(&mut self, wb: OverlayWriteBatch) -> io::Result<()> {
        self.layers.push_back(wb);
        Ok(())
    }

    pub fn push_layer(&mut self, wb: OverlayWriteBatch) {
        self.layers.push_back(wb);
    }

    pub fn solidify_layers(&mut self) -> Result<(), BoxError> {
        for layer in self.layers.drain(..) {
            self.inner.write(WriteOptions::default_instance(), &layer.wb)?;
        }
        Ok(())
    }

    pub fn discard_layers(&mut self) -> io::Result<()> {
        self.layers.clear();
        Ok(())
    }

    /// Get a value by key.
    pub fn get(&self, col: &ColumnFamilyHandle, key: &[u8]) -> io::Result<Option<Vec<u8>>> {
        for layer in self.layers.iter().rev() {
            if let Ok(val) = layer.get(col, key) {
                return Ok(val);
            }
        }
        match self.inner.get_cf(ReadOptions::default_instance(), col, key) {
            Ok(val) => Ok(Some(val.to_vec())),
            Err(e) if e.is_not_found() => Ok(None),
            // Err(e) if e.is_not_found() => Err(io::Error::new(io::ErrorKind::NotFound, "")),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
        }
    }

    /// Get the first value matching the given prefix.
    pub fn get_by_prefix(&self, col: &ColumnFamilyHandle, prefix: &[u8]) -> Option<Box<[u8]>> {
        let mut deleted = HashSet::<&[u8]>::new();

        for layer in self.layers.iter().rev() {
            if let Some(cache) = layer.cache.get(&col.id()) {
                for (key, value) in cache.iter().filter(|(key, _)| key.starts_with(prefix)) {
                    if deleted.contains(&**key) {
                        continue;
                    }
                    match value {
                        Some(val) => {
                            return Some(val.clone().into_boxed_slice());
                        }
                        None => {
                            deleted.insert(key);
                        }
                    }
                }
            }
        }

        for (key, value) in self
            .inner
            .new_iterator_cf(&ReadOptions::default().iterate_lower_bound(prefix), col)
        {
            if !key.starts_with(prefix) {
                return None;
            }
            if deleted.contains(key) {
                continue;
            }
            return Some(value.to_vec().into_boxed_slice());
        }
        None
    }

    /// Iterate over the data for a given column.
    pub fn iter<'a>(&'a self, _col: &ColumnFamilyHandle) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a> {
        // let mut deleted: HashSet<&[u8]> = HashSet::new();
        unimplemented!("might not used")
    }

    /// Iterate over the data for a given column, returning all key/value pairs
    /// where the key starts with the given prefix.
    pub fn iter_with_prefix<'a>(
        &'a self,
        _col: &ColumnFamilyHandle,
        _prefix: &'a [u8],
    ) -> Box<dyn Iterator<Item = (Box<[u8]>, Box<[u8]>)> + 'a> {
        unimplemented!()
    }
}

// * Column family indices.
pub const COL_DEFAULT: usize = 0;
/// Account, with account resource.
pub const COL_ACCOUNT: usize = 1;
pub const COL_RESOURCE_DELEGATION: usize = 2;
pub const COL_RESOURCE_DELEGATION_INDEX: usize = 3;
pub const COL_VOTES: usize = 4;
pub const COL_CONTRACT: usize = 5;
pub const COL_CONTRACT_CODE: usize = 6;
pub const COL_CONTRACT_STORAGE: usize = 7;
pub const COL_WITNESS: usize = 8;
pub const COL_PROPOSAL: usize = 9;
pub const COL_ASSET: usize = 10;
pub const COL_TRANSACTION_RECEIPT: usize = 11;
pub const COL_INTERNAL_TRANSACTION: usize = 12;
pub const COL_TRANSACTION_LOG: usize = 13;

/// The State DB derived from Chain DB.
pub struct StateDB {
    db: OverlayDB,
    cols: Vec<ColumnFamily>,
}

impl Drop for StateDB {
    fn drop(&mut self) {
        info!("state db closed successfully, all cached layers will be droped");
    }
}

impl StateDB {
    pub fn new<P: AsRef<Path>>(db_path: P) -> StateDB {
        let db_options = DBOptions::default()
            .create_if_missing(true)
            .create_missing_column_families(true)
            .increase_parallelism(num_cpus::get() as _)
            .allow_mmap_reads(true) // for Cuckoo table
            .max_open_files(1024);

        let column_families = vec![
            ColumnFamilyDescriptor::new(
                DEFAULT_COLUMN_FAMILY_NAME,
                ColumnFamilyOptions::default()
                    .optimize_for_small_db()
                    .optimize_for_point_lookup(32)
                    .num_levels(2)
                    .compression(CompressionType::NoCompression),
            ),
            // address => Account
            ColumnFamilyDescriptor::new("account", ColumnFamilyOptions::default().optimize_for_point_lookup(128)),
            // address => AccountResource
            /*ColumnFamilyDescriptor::new(
                "account-resource",
                ColumnFamilyOptions::default().optimize_for_point_lookup(128),
            ),*/
            // to_address => AccountResourceDelegation
            ColumnFamilyDescriptor::new(
                "resource-delegation",
                ColumnFamilyOptions::default().optimize_for_point_lookup(128),
            ),
            // from_address => [to_address]
            ColumnFamilyDescriptor::new(
                "resource-delegation-index",
                ColumnFamilyOptions::default().optimize_for_point_lookup(128),
            ),
            // address => Votes
            ColumnFamilyDescriptor::new("account-votes", ColumnFamilyOptions::default()),
            // address => Contract
            ColumnFamilyDescriptor::new("contract", ColumnFamilyOptions::default().optimize_for_point_lookup(32)),
            // address => Code
            ColumnFamilyDescriptor::new(
                "contract-code",
                ColumnFamilyOptions::default().optimize_for_point_lookup(128),
            ),
            // <<contract_address: Address, storage_key: H256>> => H256
            ColumnFamilyDescriptor::new(
                "contract-storage",
                ColumnFamilyOptions::default()
                    .optimize_for_point_lookup(32)
                    .prefix_extractor_fixed(32),
            ),
            // <<Address>> => Witness
            ColumnFamilyDescriptor::new(
                "witness",
                ColumnFamilyOptions::default()
                    .optimize_for_small_db()
                    .optimize_for_point_lookup(16)
                    .num_levels(2)
                    .compression(CompressionType::NoCompression),
            ),
            // <<id: u64>> => Proposal
            ColumnFamilyDescriptor::new(
                "proposal",
                ColumnFamilyOptions::default()
                    .optimize_for_small_db()
                    .optimize_for_point_lookup(16)
                    .num_levels(2)
                    .compression(CompressionType::NoCompression),
            ),
            // <<id: u64>> => Asset
            ColumnFamilyDescriptor::new(
                "asset",
                ColumnFamilyOptions::default()
                    .optimize_for_small_db()
                    .optimize_for_point_lookup(16),
            ),
            // <<txid: H256>> -> TransactionReceipt
            ColumnFamilyDescriptor::new(
                "transaction-receipt",
                ColumnFamilyOptions::default().optimize_for_point_lookup(16),
            ),
            // <<txid: H256>> -> InternalTransaction
            ColumnFamilyDescriptor::new(
                "internal-transaction",
                ColumnFamilyOptions::default().optimize_for_point_lookup(16),
            ),
            // <<Address, Topic: H256, [IndexedParam]>> => Transaction
            ColumnFamilyDescriptor::new(
                "transaction-log",
                ColumnFamilyOptions::default().prefix_extractor_fixed(32),
            ),
        ];

        let (db, cols) = DB::open_with_column_families(&db_options, db_path, column_families).unwrap();

        StateDB {
            db: OverlayDB::new(db),
            cols,
        }
    }
}

impl StateDB {
    pub fn new_layer(&mut self) -> &mut OverlayWriteBatch {
        self.db.push_layer(OverlayWriteBatch::with_capacity(4 * 1024));
        self.db.layers.back_mut().unwrap()
    }

    pub fn put_key<T, K: keys::Key<T>>(&mut self, key: K, value: T) -> Result<(), BoxError> {
        let wb = self.db.layers.back_mut().unwrap();
        wb.put(&self.cols[K::COL], key.key().as_ref(), &*K::value(&value));
        Ok(())
    }

    pub fn get<T, K: keys::Key<T>>(&self, key: &K) -> Result<Option<T>, BoxError> {
        self.db
            .get(&self.cols[K::COL], key.key().as_ref())
            .map(|maybe_raw| maybe_raw.map(|raw| K::parse_value(&raw)))
            .map_err(|e| e.into())
    }

    pub fn init_genesis(&mut self, genesis: &GenesisConfig, chain: &ChainConfig) -> Result<(), BoxError> {
        if let Some(ver) = self.get(&keys::DynamicProperty::DbVersion)? {
            info!("state-db in already inited ver: {}", ver);
            // TODO: check migration here
            let latest_block_hash = self.get(&keys::LatestBlockHash)?.expect("db inited");
            info!("latest block hash {:?}", latest_block_hash);

            return Ok(());
        }

        self.new_layer();

        for (k, v) in ChainParameter::default_parameters_from_config(&chain.parameter) {
            self.put_key(k, v)?;
        }
        for (k, v) in DynamicProperty::default_properties() {
            self.put_key(k, v)?;
        }

        self.apply_genesis_config(genesis)?;

        // WitnessSchedule is inited in first maintenance cycle.
        // self.put_key(&mut wb, keys::WitnessSchedule, value: T)

        self.db.solidify_layers()?;
        info!("inited state-db from genesis");
        Ok(())
    }

    fn apply_genesis_config(&mut self, genesis: &GenesisConfig) -> Result<(), BoxError> {
        for witness in &genesis.witnesses {
            let addr = witness.address.parse::<Address>()?;
            println!("{:?}", witness);
            let wit = state_pb::Witness {
                address: addr.as_bytes().to_vec(),
                url: witness.url.clone(),
                vote_count: witness.votes,
                // assume all witness in genesis are active witnesses.
                is_active: true,
                ..Default::default()
            };
            let key = keys::Witness(addr.clone());

            self.put_key(key, wit)?;

            let key = keys::Account(addr);
            let acct = state_pb::Account {
                creation_time: genesis.timestamp,
                r#type: AccountType::Normal as i32,
                ..Default::default()
            };
            self.put_key(key, acct)?;
        }

        for alloc in &genesis.allocs {
            let addr: Address = alloc.address.parse()?;
            println!("{:?}", alloc);
            let acct = state_pb::Account {
                name: alloc.name.clone(),
                balance: alloc.balance,
                creation_time: genesis.timestamp,
                r#type: AccountType::Normal as i32,
                ..Default::default()
            };

            let key = keys::Account(addr);
            self.put_key(key, acct)?;
        }

        let genesis_block = genesis.to_indexed_block()?;
        self.put_key(keys::LatestBlockHash, *genesis_block.hash())?;

        Ok(())
    }

    /*fn get_absolut_slot(timestamp: i64) -> i64 {
         (timestamp - 0) / constants::BLOCK_PRODUCING_INTERVAL
    }

    pub fn apply_block(&mut self, block: &IndexedBlock) -> Result<(), BoxError> {
        // DPoSService.java
        // statisticManager.applyBlock
        // maintenanceManager.applyBlock
        // updateSolidBlock

        // statisticManager
        let addr = Address::from_bytes(block.witness());
        println!("witness => {:?}", addr);

        if let Some(mut witness) = self.get(&keys::Witness(*addr))? {
            witness.total_produced += 1;
            witness.latest_block_num = block.number();
            witness.latest_slot_num = Self::get_absolut_slot(block.header.timestamp());
            println!("wit => {:?}", witness);
        }

        Ok(())
    }*/
}
