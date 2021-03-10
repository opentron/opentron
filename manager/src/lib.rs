#![feature(asm)]

use ::keys::{b58encode_check, Address, KeyPair};
use chain::BlockBuilder;
use chain::{IndexedBlock, IndexedBlockHeader, IndexedTransaction};
use chrono::Utc;
use config::{Config, GenesisConfig};
use log::{debug, info, trace, warn};
use primitive_types::H256;
use prost::Message;
use proto::chain::transaction::Result as TransactionResult;
use proto::state::TransactionReceipt;
use state::db::StateDB;
use state::keys;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use self::executor::TransactionExecutor;
use self::governance::maintenance::MaintenanceManager;
use self::governance::proposal::ProposalController;
use self::governance::reward::RewardController;
use self::resource::EnergyProcessor;

pub mod executor;
pub mod governance;
pub mod resource;
pub mod version_fork;
pub mod vm;

type Error = Box<dyn ::std::error::Error>;
type Result<T, E = Error> = ::std::result::Result<T, E>;

#[inline]
fn new_error(msg: &str) -> Error {
    use std::io;

    Box::new(io::Error::new(io::ErrorKind::Other, msg))
}

/// State DB Manager.
pub struct Manager {
    state_db: StateDB,
    genesis_block_timestamp: i64,
    blackhole: Address,

    block_energy_usage: i64,
    // TaPoS check, size = 65536, 2MB.
    ref_block_hashes: Vec<H256>,
    config: Config,
    genesis_config: GenesisConfig,
    maintenance_started_at: i64,

    layers: usize,
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
            block_energy_usage: 0,
            ref_block_hashes: Vec::with_capacity(65536),
            config: config.clone(),
            genesis_config: genesis_config.clone(),
            maintenance_started_at: 0,
            layers: 0,
        }
    }

    pub fn state(&self) -> &StateDB {
        &self.state_db
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

    pub fn add_token_to_blackhole(&mut self, token_id: i64, fee: i64) -> Result<()> {
        let key = keys::Account(self.blackhole);
        let mut blackhole_acct = self.state_db.must_get(&key);
        blackhole_acct.adjust_token_balance(token_id, fee).unwrap();
        self.state_db.put_key(key, blackhole_acct).unwrap();
        Ok(())
    }

    pub fn add_to_blackhole(&mut self, fee: i64) -> Result<()> {
        let key = keys::Account(self.blackhole);
        let mut blackhole_acct = self.state_db.must_get(&key);
        blackhole_acct.balance += fee;
        self.state_db.put_key(key, blackhole_acct).unwrap();
        Ok(())
    }

    fn new_layer(&mut self) {
        self.layers += 1;
        self.state_db.new_layer();
    }

    fn commit_current_layers(&mut self) {
        for _ in 0..self.layers {
            self.state_db.solidify_layer();
        }
        self.layers = 0;
    }

    fn rollback_layers(&mut self, n: usize) {
        for _ in 0..n {
            self.state_db.discard_last_layer().unwrap();
        }
        self.layers -= n;
    }

    /// Receive incoming block, and verify witness block signature.
    pub fn push_incoming_block(&mut self, block: &IndexedBlock) -> Result<bool> {
        // . verify witness signature
        let recovered = block.recover_witness()?;
        if self.state_db.must_get(&keys::ChainParameter::AllowMultisig) == 1 {
            // warn!("TODO: handle multisig witness");
        }
        if recovered.as_bytes() != block.witness() {
            return Err(new_error("verifying block witness signature failed"));
        }
        self.push_block(block)
    }

    pub fn push_generated_block(&mut self, block: &IndexedBlock) -> Result<bool> {
        self.push_block(block)
    }

    fn push_block(&mut self, block: &IndexedBlock) -> Result<bool> {
        if block.number() <= 0 {
            panic!("only accepts block number > 1");
        }

        // . verify merkle root hash of transaction
        if !block.verify_merkle_root_hash() {
            return Err(new_error(&format!(
                "verify block merkle root hash failed, block={}",
                block.number(),
            )));
        }

        // TODO: check dup block? (StateManager.receiveBlock)

        // NOTE: mainnet does not support shielded TRC10 transaction.
        // So there's no need to check shielded transaction count.

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
            warn!("TODO: handle chain fork!");
            return Err(new_error("chain fork!"));
        }

        // . block version check
        if block.version() > constants::CURRENT_BLOCK_VERSION as i32 {
            warn!(
                "encounter newer block version, YOU MUST UPGRADE OpenTron. block_version={}",
                block.version()
            );
        }

        // basic check finished, begin process block
        let started_at = Utc::now().timestamp_nanos();
        self.new_layer();

        // . applyBlock = processBlock + updateFork
        self.process_block(block)?;

        // NOTE: OpenTron use different logic to handle verson fork. So `updateFork` is removed.
        // And no need to updateFork.
        self.commit_current_layers();

        let elapsed = (Utc::now().timestamp_nanos() - started_at) as f64 / 1_000_000.0;
        if !block.transactions.is_empty() {
            info!(
                "block #{} v{} txns={:<3} total_time={:.3}ms",
                block.number(),
                block.version(),
                block.transactions.len(),
                elapsed
            );
        } else {
            trace!(
                "block #{} v{} empty total_time={:.3}ms",
                block.number(),
                block.version(),
                elapsed
            );
        }

        Ok(true)
    }

    fn process_block(&mut self, block: &IndexedBlock) -> Result<()> {
        // 1. checkWitness - check block producing schedule
        // Block producer is strictly scheduled except block #1(where needSyncCheck=false).
        if !self.validate_block_schedule(block)? {
            return Err(new_error("validate witness schedule error"));
        }

        // 2. reset block energy statistics, used in adaptive energy
        self.block_energy_usage = 0;

        // 3. Pre-check transaction signature in parallel.
        let recovered_owners = match block.recover_transaction_owners() {
            Ok(recovered) => recovered,
            Err(e) => {
                for txn in &block.transactions {
                    if txn.recover_owner().is_err() {
                        warn!("cannot recover owner address: {:?}", txn.hash);
                    }
                }
                return Err(e.into());
            }
        };

        // 3. Execute Transaction, TransactionRet / TransactionReceipt
        // TODO: handle accountState - AccountStateCallBack
        for (txn, recovered_addrs) in block.transactions.iter().zip(recovered_owners.into_iter()) {
            debug!(
                "transaction => {:?} at block #{} v{}",
                txn.hash,
                block.number(),
                block.version()
            );
            self.process_transaction(&txn, recovered_addrs, block)?;
        }

        // 4. Adaptive energy processor:
        if self.block_energy_usage > 0 {
            if self.state_db.must_get(&keys::ChainParameter::AllowAdaptiveEnergy) != 0 {
                debug!("block energy = {}", self.block_energy_usage);
                // updateTotalEnergyAverageUsage + updateAdaptiveTotalEnergyLimit
                EnergyProcessor::new(self).update_adaptive_energy().unwrap();
            }
        }

        // 5. Block reward
        self.pay_reward(block);

        // 6. Handle proposal if maintenance
        if self.state_db.must_get(&keys::DynamicProperty::NextMaintenanceTime) <= block.timestamp() {
            self.maintenance_started_at = Utc::now().timestamp_nanos();
            info!("beigin maintenance at block #{}", block.number());
            ProposalController::new(self).process_proposals()?;
        }

        // 7. consensus.applyBlock (DposService.applyBlock)
        // - statisticManager.applyBlock
        // - maintenanceManager.applyBlock
        // - updateSolidBlock
        WitnessStatisticManager::new(self).apply_block(block)?;
        MaintenanceManager::new(self).apply_block(block)?;
        self.update_solid_block(block)?;

        self.update_ref_blocks(*block.hash());

        // 8. update latest block - updateDynamicProperties
        self.state_db
            .put_key(keys::DynamicProperty::LatestBlockNumber, block.number())?;
        self.state_db
            .put_key(keys::DynamicProperty::LatestBlockTimestamp, block.timestamp())?;
        self.state_db.put_key(keys::LatestBlockHash, *block.hash())?;

        Ok(())
    }

    // NOTE: TransactionInfo is renamed to TransactionReceipt
    /// Process the transaction and saves result to current StateDB layer.
    fn process_transaction(
        &mut self,
        txn: &IndexedTransaction,
        recovered_addrs: Vec<Address>,
        block: &IndexedBlock,
    ) -> Result<()> {
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

        // 4.validateSignature (NOTE: move partial logic to upstream executor)
        // 5.cusumeBandwidth (NOTE: move to executor)
        // 6.cusumeMultiSigFee (NOTE: move to BandwidthProcessor)

        // 7. transaction is executed by TransactionTrace.
        let txn_receipt =
            TransactionExecutor::new(self).execute_and_verify_result(txn, recovered_addrs, &block.header)?;
        self.state_db.put_key(keys::TransactionReceipt(txn.hash), txn_receipt)?;
        Ok(())
    }

    /// Validate transacton before push to mempool.
    ///
    /// Use almost the same logic as `process_transaction`.
    pub fn pre_push_transaction(&mut self, txn: &IndexedTransaction) -> Result<TransactionResult> {
        if !self.validate_transaction_tapos(txn) {
            return Err(new_error("tapos validation failed: invalid ref_block"));
        }
        if !self.valide_transaction_common(txn) {
            return Err(new_error("message size or expiration validation failed"));
        }
        if !self.validate_duplicated_transaction(txn) {
            return Err(new_error("duplicated transaction"));
        }
        let fake_block_number = self.latest_block_number() + 1;
        let block_header = IndexedBlockHeader::dummy(
            fake_block_number,
            self.latest_block_timestamp() + constants::BLOCK_PRODUCING_INTERVAL,
        );

        let owner_addrs = txn.recover_owner()?;

        let old_layers = self.layers;
        self.new_layer();

        let ret = TransactionExecutor::new(self).execute(txn, owner_addrs, &block_header);

        let added_layers = self.layers - old_layers;
        self.rollback_layers(added_layers);

        Ok(ret?.0)
    }

    /// Dry run the transaction, return Receipt.
    ///
    /// This does not validate some props.
    pub fn dry_run_transaction(&mut self, txn: &IndexedTransaction) -> Result<(TransactionResult, TransactionReceipt)> {
        /*if !self.validate_transaction_tapos(txn) {
            return Err(new_error("tapos validation failed"));
        }
        if !self.valide_transaction_common(txn) {
            return Err(new_error("message size or expiration validation failed"));
        }*/
        let fake_block_number = self.latest_block_number() + 1;
        let block_header = IndexedBlockHeader::dummy(
            fake_block_number,
            self.latest_block_timestamp() + constants::BLOCK_PRODUCING_INTERVAL,
        );

        let old_layers = self.layers;
        self.new_layer();

        let ret = TransactionExecutor::new(self).execute(txn, txn.recover_owner()?, &block_header);

        let added_layers = self.layers - old_layers;
        debug!("dry run, rollback layers={}", added_layers);
        self.rollback_layers(added_layers);
        Ok(ret?)
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
        // TODO: not used in sync. used in block producing
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
        let mut wit_addrs = self.state_db.must_get(&keys::WitnessSchedule);
        if wit_addrs.is_empty() {
            panic!("no witness found");
        }
        if wit_addrs.len() > constants::MAX_NUM_OF_ACTIVE_WITNESSES {
            let _ = wit_addrs.split_off(constants::MAX_NUM_OF_ACTIVE_WITNESSES);
        }
        let mut block_nums: Vec<_> = wit_addrs
            .into_iter()
            .map(|(addr, _, _)| self.state_db.must_get(&keys::Witness(addr)).latest_block_number)
            .collect();
        block_nums.sort();

        // NOTE: When there are 27 active witnesses, pos will be 8, that's 19 SR confirmations.
        let pos = (block_nums.len() as f64 * (1.0 - constants::SOLID_THRESHOLD_PERCENT as f64 / 100.0)) as usize;
        let new_solid_block_num = block_nums[pos];

        let old_solid_block_num = self.state_db.must_get(&keys::DynamicProperty::LatestSolidBlockNumber);
        if new_solid_block_num < old_solid_block_num {
            // NOTE: This warning must be ignored. When new active witness is ranked after maintenance,
            // new solid block number might become 0.
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

    /// Pay block producing reward.
    fn pay_reward(&mut self, block: &IndexedBlock) {
        let allow_change_delegation = self.state_db.must_get(&keys::ChainParameter::AllowChangeDelegation) != 0;
        if allow_change_delegation {
            // So-called new-style reward scheme.
            // 1. delegationService.payBlockReward
            // 2. delegationService.payStandbyWitness
            RewardController::new(self).pay_reward(block).unwrap();
        } else {
            // NOTE: In this legacy reward scheme, standby witnesses will be paid during maintenance cycle.
            let wit_key = keys::Account(block.witness().try_into().unwrap());
            let mut wit_acct = self.state_db.must_get(&wit_key);
            let reward_per_block = self.state_db.must_get(&keys::ChainParameter::WitnessPayPerBlock);
            wit_acct.allowance += reward_per_block;
            self.state_db.put_key(wit_key, wit_acct).unwrap();
        }
    }

    // * block producers

    /// Generate an empty block. This normally happens when producing the first block (block#1)
    /// after genesis block, which requires sync-check to be turned off.
    pub fn generate_empty_block(
        &mut self,
        timestamp: i64,
        witness: &Address,
        keypair: &KeyPair,
    ) -> Result<IndexedBlock> {
        let block_number = self.latest_block_number() + 1;
        BlockBuilder::new(block_number)
            .version(17) // GreatVoyage4_0_1
            .timestamp(timestamp)
            .parent_hash(&self.latest_block_hash())
            .build(witness, keypair)
            .ok_or(new_error("cannot produce block"))
    }

    // * DposSlot
    fn get_absolute_slot(&self, timestamp: i64) -> i64 {
        (timestamp - self.genesis_block_timestamp) / constants::BLOCK_PRODUCING_INTERVAL
    }

    pub fn get_slot(&self, timestamp: i64) -> i64 {
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

    // public long getTime(long slot)
    pub fn get_slot_timestamp(&self, mut slot: i64) -> i64 {
        assert!(slot >= 0, "unreachable");

        if slot == 0 {
            return Utc::now().timestamp_millis();
        }

        if self.state_db.must_get(&keys::DynamicProperty::LatestBlockNumber) == 0 {
            return self.genesis_block_timestamp + slot * constants::BLOCK_PRODUCING_INTERVAL;
        }

        if self.is_latest_block_maintenance() {
            slot += constants::NUM_OF_SKIPPED_SLOTS_IN_MAINTENANCE as i64;
        }

        let mut ts = self.latest_block_timestamp();
        ts -= (ts - self.genesis_block_timestamp) % constants::BLOCK_PRODUCING_INTERVAL;
        ts + constants::BLOCK_PRODUCING_INTERVAL * slot
    }

    fn get_active_witnesses(&self) -> Vec<Address> {
        let mut witnesses = self.state_db.get(&keys::WitnessSchedule).unwrap().unwrap();
        if witnesses.is_empty() {
            panic!("no witness found");
        }
        if witnesses.len() > constants::MAX_NUM_OF_ACTIVE_WITNESSES {
            let _ = witnesses.split_off(constants::MAX_NUM_OF_ACTIVE_WITNESSES);
        }
        witnesses.into_iter().map(|wit| wit.0).collect()
    }

    fn get_standby_witnesses(&self) -> Vec<Address> {
        let mut witnesses = self.state_db.get(&keys::WitnessSchedule).unwrap().unwrap();
        if witnesses.is_empty() {
            panic!("no witness found");
        }
        if witnesses.len() > constants::MAX_NUM_OF_STANDBY_WITNESSES {
            let _ = witnesses.split_off(constants::MAX_NUM_OF_STANDBY_WITNESSES);
        }
        witnesses.into_iter().map(|wit| wit.0).collect()
    }

    pub fn get_scheduled_witness(&self, slot: i64) -> Address {
        let mut witnesses = self.state_db.get(&keys::WitnessSchedule).unwrap().unwrap_or_default();
        if witnesses.is_empty() {
            panic!("no witness schedule found");
        }
        if witnesses.len() > constants::MAX_NUM_OF_ACTIVE_WITNESSES {
            let _ = witnesses.split_off(constants::MAX_NUM_OF_ACTIVE_WITNESSES);
        }
        let curr_slot = self.get_absolute_slot(self.latest_block_timestamp()) + slot;
        assert!(curr_slot >= 0, "slot must be positive");

        let mut idx = (curr_slot as usize) % (witnesses.len() * constants::NUM_OF_CONSECUTIVE_BLOCKS_PER_ROUND);
        idx /= constants::NUM_OF_CONSECUTIVE_BLOCKS_PER_ROUND;
        witnesses[idx].0
    }

    // consensus
    #[inline]
    fn is_latest_block_maintenance(&self) -> bool {
        self.state_db.must_get(&keys::DynamicProperty::IsMaintenance) != 0
    }

    #[inline]
    fn latest_block_timestamp(&self) -> i64 {
        self.state_db.must_get(&keys::DynamicProperty::LatestBlockTimestamp)
    }

    #[inline]
    pub fn solid_block_number(&self) -> i64 {
        self.state_db.must_get(&keys::DynamicProperty::LatestSolidBlockNumber)
    }

    #[inline]
    pub fn latest_block_number(&self) -> i64 {
        self.state_db.must_get(&keys::DynamicProperty::LatestBlockNumber)
    }

    #[inline]
    fn latest_block_hash(&self) -> H256 {
        self.state_db.must_get(&keys::LatestBlockHash)
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
        let mut missed_witnesses = HashMap::new();
        let mut missed_count = HashMap::<Address, i64>::new();
        for i in 1..slot {
            let wit_addr = self.manager.get_scheduled_witness(i);
            let mut wit = missed_witnesses
                .entry(wit_addr)
                .or_insert_with(|| self.manager.state_db.must_get(&keys::Witness(wit_addr)));
            *missed_count.entry(wit_addr).or_default() += 1;

            wit.total_missed += 1;

            self.filled_slots[self.filled_slots_index as usize] = 0;
            self.filled_slots_index = (self.filled_slots_index + 1) % constants::NUM_OF_BLOCK_FILLED_SLOTS as i64;
        }
        for (wit_addr, wit) in missed_witnesses.into_iter() {
            warn!(
                "block miss #{}, witness={}, total_missed={}, missed+={}",
                block.number(),
                wit_addr,
                wit.total_missed,
                missed_count[&wit_addr],
            );
            self.manager.state_db.put_key(keys::Witness(wit_addr), wit).unwrap();
        }

        // current block is filled
        self.filled_slots[self.filled_slots_index as usize] = 1;
        self.filled_slots_index = (self.filled_slots_index + 1) % constants::NUM_OF_BLOCK_FILLED_SLOTS as i64;

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
