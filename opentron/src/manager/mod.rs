use ::keys::{b58encode_check, Address};
use chain::{IndexedBlock, IndexedTransaction};
use chrono::Utc;
use config::{Config, GenesisConfig};
use log::{debug, info, warn};
use primitive_types::H256;
use prost::Message;
use state::db::StateDB;
use state::keys;
use std::convert::TryFrom;

use self::executor::TransactionExecutor;
use self::maintenance::MaintenanceManager;

pub mod actuators;
pub mod executor;
pub mod maintenance;
pub mod processors;

type Error = Box<dyn ::std::error::Error>;
type Result<T, E = Error> = ::std::result::Result<T, E>;
// use crate::context::AppContext;

#[inline]
fn new_error(msg: &str) -> Error {
    use std::io;

    Box::new(io::Error::new(io::ErrorKind::Other, msg))
}

// DB Manager.
pub struct Manager {
    state_db: StateDB,
    genesis_block_timestamp: i64,
    blackhole: Address,
    my_witness: Vec<u8>,

    block_energy_usage: i64,
    // TaPoS check, size = 65536, 2MB.
    ref_block_hashes: Vec<H256>,
}

impl Manager {
    pub fn new(config: &Config, genesis_config: &GenesisConfig) -> Self {
        let mut state_db = StateDB::new(&config.storage.state_data_dir);

        state_db.init_genesis(&genesis_config, &config.chain).unwrap();
        let genesis_block_timestamp = genesis_config.timestamp;

        let blackhole = genesis_config
            .allocs
            .iter()
            .find(|n| n.name == "Blackhole")
            .and_then(|alloc| alloc.address.parse().ok())
            .expect("blackhole account not found");

        debug!("loaded the Blackhole address {}", blackhole);

        Manager {
            state_db,
            genesis_block_timestamp,
            blackhole,
            my_witness: vec![],
            block_energy_usage: 0,
            ref_block_hashes: Vec::with_capacity(65536),
        }
    }

    pub fn init_ref_blocks(&mut self, hashes: Vec<H256>) {
        debug!("update num of ref_hashes => {:?}", hashes.len());
        self.ref_block_hashes = hashes;
    }

    fn update_ref_blocks(&mut self, new_hash: H256) {
        if self.ref_block_hashes.len() < 65536 {
            self.ref_block_hashes.push(new_hash);
        } else {
            let ref_block_bytes = {
                let mut raw = [0u8; 2];
                raw.copy_from_slice(&new_hash.as_bytes()[6..8]);
                raw
            };
            let ref_slot_index = u16::from_be_bytes(ref_block_bytes) as usize;
            self.ref_block_hashes[ref_slot_index] = new_hash;
        }
    }

    pub fn add_to_blackhole(&mut self, fee: i64) -> Result<()> {
        let key = keys::Account(self.blackhole);
        let mut blackhole_acct = self.state_db.must_get(&key);
        blackhole_acct.balance += fee;
        self.state_db.put_key(key, blackhole_acct).unwrap();
        Ok(())
    }

    // Entry of db manager.
    pub fn push_block(&mut self, block: &IndexedBlock) -> Result<bool> {
        if block.number() <= 0 {
            panic!("only accepts block number > 1");
        }
        // 1. verify witness signature
        if self.my_witness.is_empty() || block.witness() != &*self.my_witness {
            let recovered = block.recover_witness()?;
            if self.state_db.get(&keys::ChainParameter::AllowMultisig)?.unwrap() == 1 {
                panic!("TODO: handle multisig witness");
            }
            if recovered.as_bytes() != block.witness() {
                return Err(new_error("verifying block witness signature failed"));
            }
        }
        // 2. verify merkle tree
        if !block.verify_merkle_root_hash() {
            return Err(new_error("verify block merkle root hash failed"));
        }

        // consensusInterface.receiveBlock = DposService.receiverBlock
        // StateManager.receiverBlock
        // TODO: check dup block?

        // NOTE: mainnet does not support shielded transaction.

        // . reject smaller block number
        if block.number() <= self.latest_block_number() {
            warn!(
                "reject smaller block number latest={}, got={}",
                self.latest_block_number(),
                block.number()
            );
            return Ok(false);
        }

        if block.parent_hash() != self.latest_block_hash().as_bytes() {
            // println!("TODO: handle fork");
        }

        if block.version() > constants::CURRENT_BLOCK_VERSION as i32 {
            warn!(
                "encounter newer block version, YOU MUST UPGRADE OpenTron. block_version={}",
                block.version()
            );
        }

        // basic check finished, begin process block
        self.state_db.new_layer();

        // . applyBlock
        self.process_block(block)?;

        self.state_db.solidify_layer();

        Ok(true)
    }

    fn process_block(&mut self, block: &IndexedBlock) -> Result<()> {
        // 1. checkWitness
        if !self.validate_block_schedule(block)? {
            // warn!("TODO: dev, ignore  validate_block_schedule error");
            return Err(new_error("validate witness schedule error"));
        }

        // 2. reset block energy statistics, used in adaptive energy
        self.block_energy_usage = 0;

        // NOTE: won't pre-check transaction signature. useless.

        // 3. Execute Transaction, TransactionRet / TransactionReceipt
        // TODO: handle accountState - AccountStateCallBack.java
        for txn in &block.transactions {
            info!("transaction => {:?}", txn.hash);
            self.process_transaction(&txn, block)?;
        }

        // 4. Adaptive energy processor: TODO

        // 5. Block reward - payReward(block): TODO

        // 6. Handle proposal if maintenance
        if self.state_db.must_get(&keys::DynamicProperty::NextMaintenanceTime) <= block.timestamp() {
            // TODO
        }

        // 7. consensus.applyBlock (DposService.applyBlock)
        // - statisticManager.applyBlock
        // - maintenanceManager.applyBlock
        // - updateSolidBlock
        WitnessStatisticManager::new(self).apply_block(block)?;
        MaintenanceManager::new(self).apply_block(block)?;
        self.update_solid_block(block)?;

        self.update_ref_blocks(*block.hash());

        // update latest block
        self.state_db
            .put_key(keys::DynamicProperty::LatestBlockNumber, block.number())?;
        self.state_db
            .put_key(keys::DynamicProperty::LatestBlockTimestamp, block.timestamp())?;
        self.state_db.put_key(keys::LatestBlockHash, *block.hash())?;

        Ok(())
    }

    // NOTE: rename TransactionInfo to TransactionReceipt
    fn process_transaction(&mut self, txn: &IndexedTransaction, block: &IndexedBlock) -> Result<()> {
        // 1.validateTapos
        if !self.validate_transaction_tapos(txn) {
            return Err(new_error("tapos validation failed"));
        }
        // 2.validateCommon
        if !self.valide_transaction_common(txn) {
            return Err(new_error("message size or expiration validation failed"));
        }
        // 3.validateDup
        if !self.validate_duplicated_transaction(txn) {
            return Err(new_error("duplicated transaction"));
        }

        // 4.validateSignature (NOTE: move to executor)
        // 5.cusumeBandwidth (NOTE: move to executor)
        // 6.cusumeMultiSigFee (NOTE: move to executor)

        // 7. transaction is executed by TransactionTrace.
        let txn_receipt = TransactionExecutor::new(self).execute(txn, block)?;
        self.state_db.put_key(keys::TransactionReceipt(txn.hash), txn_receipt)?;
        Ok(())
    }

    fn validate_transaction_tapos(&self, txn: &IndexedTransaction) -> bool {
        let ref_block_hash = &txn.raw.raw_data.as_ref().unwrap().ref_block_hash;
        let ref_block_bytes = {
            let mut raw = [0u8; 2];
            raw.copy_from_slice(&txn.raw.raw_data.as_ref().unwrap().ref_block_bytes);
            raw
        };
        // debug!("ref block bytes=> {:?}", hex::encode(&ref_block_bytes[..]));
        // debug!("ref block hash=> {:?}", hex::encode(&ref_block_hash[..]));
        let ref_slot_index = u16::from_be_bytes(ref_block_bytes) as usize;

        self.ref_block_hashes
            .get(ref_slot_index)
            .map(|block_hash| &block_hash.as_ref()[8..16] == &ref_block_hash[..])
            .unwrap_or(false)
    }

    fn valide_transaction_common(&self, txn: &IndexedTransaction) -> bool {
        if txn.raw.encoded_len() > constants::MAX_TRANSACTION_SIZE {
            warn!("transaction is too big");
            return false;
        }
        let latest_block_ts = self.latest_block_timestamp();
        if txn.expiration() <= latest_block_ts ||
            txn.expiration() > latest_block_ts + constants::MAX_TRANSACTION_EXPIRATION
        {
            warn!("transaction expired");
            return false;
        }
        true
    }

    fn validate_duplicated_transaction(&self, _txn: &IndexedTransaction) -> bool {
        // TODO
        true
    }

    // consensus.validBlock
    fn validate_block_schedule(&self, block: &IndexedBlock) -> Result<bool> {
        if self.state_db.get(&keys::DynamicProperty::LatestBlockNumber).unwrap() == Some(0) {
            return Ok(true);
        }

        let timestamp = block.timestamp();
        let block_slot = self.get_absolute_slot(timestamp);
        let head_slot = self.get_absolute_slot(
            self.state_db
                .get(&keys::DynamicProperty::LatestBlockTimestamp)?
                .unwrap_or(0),
        );
        if block_slot <= head_slot {
            warn!(
                "error while validate block: bSlot:{} <= hSlot:{}",
                block_slot, head_slot
            );
            return Ok(false);
        }

        let slot = self.get_slot(timestamp);
        let scheduled = self.get_scheduled_witness(slot);

        if block.witness() != scheduled.as_bytes() {
            warn!(
                "scheduled witness mismatch: scheduled={}, block.witness={}, #{}, ts={}",
                b58encode_check(scheduled),
                b58encode_check(block.witness()),
                block.number(),
                block.timestamp(),
            );
            return Ok(false);
        }
        Ok(true)
    }

    fn update_solid_block(&mut self, block: &IndexedBlock) -> Result<()> {
        let mut wit_addrs = self.state_db.get(&keys::WitnessSchedule).unwrap().unwrap();
        if wit_addrs.is_empty() {
            panic!("no witness found");
        }
        if wit_addrs.len() > constants::MAX_NUM_OF_ACTIVE_WITNESSES {
            let _ = wit_addrs.split_off(constants::MAX_NUM_OF_ACTIVE_WITNESSES);
        }
        let mut block_nums: Vec<_> = wit_addrs
            .into_iter()
            .map(|(addr, _)| self.state_db.must_get(&keys::Witness(addr)).latest_block_number)
            .collect();

        block_nums.sort();

        // NOTE: When there are 27 active witnesses, pos will be 8, that's 19 SR confirmations.
        let pos = (block_nums.len() as f64 * (1.0 - constants::SOLID_THRESHOLD_PERCENT as f64 / 100.0)) as usize;

        let new_solid_block_num = block_nums[pos];
        let old_solid_block_num = self.state_db.must_get(&keys::DynamicProperty::LatestSolidBlockNumber);
        if new_solid_block_num < old_solid_block_num {
            // NOTE: This warning is ignored.
            warn!(
                "cannot update solid block number backwards, current={}, update={}",
                old_solid_block_num, new_solid_block_num
            );
        } else {
            if block.number() % 1000 == 0 {
                info!("updated solid block number = {}", new_solid_block_num);
            }
            self.state_db
                .put_key(keys::DynamicProperty::LatestSolidBlockNumber, new_solid_block_num)
                .unwrap();
        }

        Ok(())
    }

    // * DposSlot
    fn get_absolute_slot(&self, timestamp: i64) -> i64 {
        (timestamp - self.genesis_block_timestamp) / constants::BLOCK_PRODUCING_INTERVAL
    }

    fn get_slot(&self, timestamp: i64) -> i64 {
        let first_slot_ts = self.get_slot_timestamp(1);
        if timestamp < first_slot_ts {
            0
        } else {
            (timestamp - first_slot_ts) / constants::BLOCK_PRODUCING_INTERVAL + 1
        }
    }

    fn get_head_slot(&self) -> i64 {
        self.get_absolute_slot(self.state_db.must_get(&keys::DynamicProperty::LatestBlockTimestamp))
    }

    fn get_slot_timestamp(&self, mut slot: i64) -> i64 {
        assert!(slot >= 0, "unreachable");

        if slot == 0 {
            return Utc::now().timestamp_millis();
        }

        if self.state_db.get(&keys::DynamicProperty::LatestBlockNumber).unwrap() == Some(0) {
            return self.genesis_block_timestamp + slot * constants::BLOCK_PRODUCING_INTERVAL;
        }

        if self.is_latest_block_maintenance() {
            slot += constants::NUM_OF_SKIPPED_SLOTS_IN_MAINTENANCE;
        }

        let mut ts = self.latest_block_timestamp();
        ts -= (ts - self.genesis_block_timestamp) % constants::BLOCK_PRODUCING_INTERVAL;
        ts + constants::BLOCK_PRODUCING_INTERVAL * slot
    }

    fn get_scheduled_witness(&self, slot: i64) -> Address {
        const SINGLE_REPEAT: usize = 1;

        let mut witnesses = self.state_db.get(&keys::WitnessSchedule).unwrap().unwrap();
        if witnesses.is_empty() {
            panic!("no witness found");
        }
        if witnesses.len() > constants::MAX_NUM_OF_ACTIVE_WITNESSES {
            let _ = witnesses.split_off(constants::MAX_NUM_OF_ACTIVE_WITNESSES);
        }
        let curr_slot = self.get_absolute_slot(self.latest_block_timestamp()) + slot;
        assert!(curr_slot >= 0, "slot must be positive");

        let mut idx = (curr_slot as usize) % (witnesses.len() * SINGLE_REPEAT);
        idx /= SINGLE_REPEAT;
        witnesses[idx].0
    }

    // consensus
    #[inline]
    fn is_latest_block_maintenance(&self) -> bool {
        self.state_db
            .get(&keys::DynamicProperty::IsMaintenance)
            .unwrap()
            .unwrap_or(0) ==
            1
    }

    #[inline]
    fn latest_block_timestamp(&self) -> i64 {
        self.state_db
            .get(&keys::DynamicProperty::LatestBlockTimestamp)
            .unwrap()
            .unwrap()
    }

    #[inline]
    pub fn latest_block_number(&self) -> i64 {
        self.state_db
            .get(&keys::DynamicProperty::LatestBlockNumber)
            .unwrap()
            .unwrap()
    }

    #[inline]
    fn latest_block_hash(&self) -> H256 {
        self.state_db.get(&keys::LatestBlockHash).unwrap().unwrap()
    }
}

/// Update witnesses' statistics, and BlockFilledSlots.
pub struct WitnessStatisticManager<'m> {
    manager: &'m mut Manager,
    filled_slots: Vec<u8>,
    filled_slots_index: i64,
}

impl WitnessStatisticManager<'_> {
    pub fn new<'a>(manager: &'a mut Manager) -> WitnessStatisticManager<'a> {
        let filled_slots = manager.state_db.must_get(&keys::BlockFilledSlots);
        let filled_slots_index = manager.state_db.must_get(&keys::DynamicProperty::BlockFilledSlotsIndex);

        WitnessStatisticManager {
            manager,
            filled_slots,
            filled_slots_index,
        }
    }

    pub fn apply_block(mut self, block: &IndexedBlock) -> Result<()> {
        let wit_addr = Address::try_from(block.witness()).unwrap();

        let mut wit = self.manager.state_db.must_get(&keys::Witness(wit_addr));

        wit.total_produced += 1;
        wit.latest_block_number = block.number();
        wit.latest_slot_number = self.manager.get_absolute_slot(block.timestamp());
        // NOTE: This is used for fork controller.
        wit.latest_block_version = block.version();

        self.manager.state_db.put_key(keys::Witness(wit_addr), wit).unwrap();

        let slot = if block.number() != 1 {
            self.manager.get_slot(block.timestamp())
        } else {
            1
        };

        // record missed blocks
        // TODO: reduce `put_key` operations.
        for i in 1..slot {
            let wit_addr = self.manager.get_scheduled_witness(i);
            let mut wit = self.manager.state_db.must_get(&keys::Witness(wit_addr));
            wit.total_missed += 1;
            warn!(
                "block #{}, witness={}, total_missed={}",
                block.number(),
                wit_addr,
                wit.total_missed
            );
            self.manager.state_db.put_key(keys::Witness(wit_addr), wit).unwrap();

            self.filled_slots[self.filled_slots_index as usize] = 0;
            self.filled_slots_index = (self.filled_slots_index + 1) % constants::NUM_OF_BLOCK_FILLED_SLOTS;
        }
        // current block is filled
        self.filled_slots[self.filled_slots_index as usize] = 1;
        self.filled_slots_index = (self.filled_slots_index + 1) % constants::NUM_OF_BLOCK_FILLED_SLOTS;

        self.manager
            .state_db
            .put_key(keys::DynamicProperty::BlockFilledSlotsIndex, self.filled_slots_index)
            .unwrap();
        self.manager
            .state_db
            .put_key(keys::BlockFilledSlots, self.filled_slots)
            .unwrap();

        Ok(())
    }
}
