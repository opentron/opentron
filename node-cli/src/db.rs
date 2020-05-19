extern crate byteorder;

use byteorder::{ByteOrder, BE};
use bytes::BytesMut;
use chain::{BlockHeader, IndexedBlock, IndexedBlockHeader, IndexedTransaction, Transaction};
use log::{debug, info};
use primitives::H256;
use prost::Message;
use proto2::chain::ContractType;
use rocks::prelude::*;
use std::collections::{HashSet, LinkedList};
use std::error::Error;
use std::io;
use std::iter::FromIterator;
use std::path::Path;

pub type BoxError = Box<dyn Error>;

pub struct ChainDB {
    db: DB,
    default: ColumnFamily,
    block_header: ColumnFamily,
    transaction: ColumnFamily,
    transaction_block: ColumnFamily,
}

impl Drop for ChainDB {
    fn drop(&mut self) {
        info!("db closed successfully");
    }
}

impl ChainDB {
    pub fn new<P: AsRef<Path>>(db_path: P) -> ChainDB {
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
            // block_hash => BlockHeader
            ColumnFamilyDescriptor::new(
                "block-header",
                ColumnFamilyOptions::default().max_write_buffer_number(6),
            ),
            /*
            ColumnFamilyDescriptor::new(
                "block-transactions",
                ColumnFamilyOptions::default()
                    .optimize_for_point_lookup(128)
                    .max_write_buffer_number(6),
            ),*/
            // [block_hash, transaction_index: u64, transaction_hash] => Transaction
            ColumnFamilyDescriptor::new(
                "transaction",
                ColumnFamilyOptions::default()
                    .optimize_level_style_compaction(512 * 1024 * 1024)
                    .max_write_buffer_number(6),
            ),
            // transaction_hash => [block_hash, transaction_index: u64]
            // Key and value lengths are fixed
            ColumnFamilyDescriptor::new(
                "transaction-block",
                ColumnFamilyOptions::default()
                    .table_factory_cuckoo(CuckooTableOptions::default())
                    // .optimize_level_style_compaction(512 * 1024 * 1024)
                    // .optimize_for_point_lookup(32)
                    .max_write_buffer_number(6),
            ),
        ];

        let (db, mut handles) = DB::open_with_column_families(&db_options, db_path, column_families).unwrap();
        let txn_blk = handles.pop().unwrap();
        let txn = handles.pop().unwrap();
        let blk = handles.pop().unwrap();
        let default = handles.pop().unwrap();

        assert!(handles.is_empty());

        ChainDB {
            db: db,
            default: default,
            block_header: blk,
            transaction: txn,
            transaction_block: txn_blk,
        }
    }

    pub fn get_block_height(&self) -> i64 {
        self.default
            .get(ReadOptions::default_instance(), b"BLOCK_HEIGHT")
            .map(|val| BE::read_u64(&*val) as i64)
            .unwrap_or(0)
    }

    pub fn update_block_height(&self, height: i64) {
        assert!(height >= 0);
        if height > self.get_block_height() {
            let mut val = [0u8; 8];
            BE::write_u64(&mut val, height as u64);
            self.default
                .put(WriteOptions::default_instance(), b"BLOCK_HEIGHT", &val)
                .unwrap();
        }
    }

    pub fn force_update_block_height(&self, height: i64) {
        let mut val = [0u8; 8];
        BE::write_u64(&mut val, height as u64);
        self.default
            .put(WriteOptions::default_instance(), b"BLOCK_HEIGHT", &val)
            .unwrap();
    }

    /// Highest block id, counted from 0
    pub fn highest_block(&self) -> Option<IndexedBlock> {
        self.get_block_by_number(self.get_block_height() as u64)
    }

    pub fn insert_block(&self, block: &IndexedBlock) -> Result<(), Box<dyn Error>> {
        let mut batch = WriteBatch::with_reserved_bytes(1024);

        let mut buf = BytesMut::with_capacity(block.header.raw.encoded_len());
        block.header.raw.encode(&mut buf)?;
        batch.put_cf(&self.block_header, block.header.hash.as_bytes(), &buf);

        for (index, txn) in block.transactions.iter().enumerate() {
            buf.clear();
            txn.raw.encode(&mut buf)?;

            // [block_hash, transaction_index: u64, transaction_hash] => Transaction
            let mut idx_key = [0u8; 8];
            BE::write_u64(&mut idx_key[..], index as u64);

            batch.putv_cf(
                &self.transaction,
                &[block.hash().as_bytes(), &idx_key, txn.hash.as_bytes()],
                &[&buf],
            );
            // reverse index
            // transaction_hash => [block_hash, transaction_index: u64]
            batch.putv_cf(
                &self.transaction_block,
                &[txn.hash.as_bytes()],
                &[block.hash().as_bytes(), &idx_key],
            );
        }

        self.db.write(WriteOptions::default_instance(), &batch)?;
        Ok(())
    }

    pub fn has_block_id(&self, id: &H256) -> bool {
        self.block_header
            .get(ReadOptions::default_instance(), id.as_bytes())
            .is_ok()
    }

    pub fn has_block(&self, block: &IndexedBlock) -> bool {
        self.has_block_id(&block.header.hash)
    }

    pub fn has_block_number(&self, num: u64) -> bool {
        let mut lower_bound = [0u8; 8];
        BE::write_u64(&mut lower_bound, num);
        let mut upper_bound = [0u8; 8];
        BE::write_u64(&mut upper_bound, num + 1);

        let it = self.block_header.new_iterator(
            &ReadOptions::default()
                .iterate_lower_bound(&lower_bound[..])
                .iterate_upper_bound(&upper_bound[..]),
        );
        it.count() == 1
    }

    pub fn get_block_from_header(&self, header: IndexedBlockHeader) -> Option<IndexedBlock> {
        let mut upper_bound = header.hash.as_bytes().to_vec();
        upper_bound.push(0xFF); // [0xcafebabe00 .. 0xcafebabeff]

        let transactions = self
            .transaction
            .new_iterator(
                &ReadOptions::default()
                    .iterate_lower_bound(&header.hash.as_bytes())
                    .iterate_upper_bound(&upper_bound),
            )
            .map(|(key, val)| {
                let txn = Transaction::decode(val).unwrap();
                IndexedTransaction::new(H256::from_slice(&key[32 + 8..]), txn)
            })
            .collect();

        Some(IndexedBlock::new(header, transactions))
    }

    /// handles fork
    pub fn get_block_headers_by_number(&self, num: u64) -> Vec<IndexedBlockHeader> {
        let mut lower_bound = [0u8; 8];
        BE::write_u64(&mut lower_bound, num);
        let mut upper_bound = [0u8; 8];
        BE::write_u64(&mut upper_bound, num + 1);
        let ropt = ReadOptions::default()
            .iterate_lower_bound(&lower_bound[..])
            .iterate_upper_bound(&upper_bound[..])
            .pin_data(true);

        self.block_header
            .new_iterator(&ropt)
            .map(|(key, val)| IndexedBlockHeader::new(H256::from_slice(key), BlockHeader::decode(val).unwrap()))
            .collect()
    }

    pub fn get_block_by_number(&self, num: u64) -> Option<IndexedBlock> {
        let mut lower_bound = [0u8; 8];
        BE::write_u64(&mut lower_bound, num);
        let mut upper_bound = [0u8; 8];
        BE::write_u64(&mut upper_bound, num + 1);

        let it = self.block_header.new_iterator(
            &ReadOptions::default()
                .iterate_lower_bound(&lower_bound[..])
                .iterate_upper_bound(&upper_bound[..])
                .pin_data(true),
        );

        // FIXME: iterator key lifetime leaks, key might becomes same key
        // ref: https://github.com/bh1xuw/rust-rocks/issues/15
        let found = it.map(|(key, val)| (key.to_vec(), val.to_vec())).collect::<Vec<_>>();
        if found.is_empty() {
            return None;
        }
        if found.len() > 1 {
            eprintln!("multiple blocks found for same number: {}", num);
            for item in &found {
                eprintln!("  => {}", hex::encode(&item.0));
                eprintln!("  => {}", hex::encode(&item.1));
            }
            return None;
        }

        let header = IndexedBlockHeader::new(
            H256::from_slice(&found[0].0),
            BlockHeader::decode(&*found[0].1).unwrap(),
        );
        self.get_block_from_header(header)
    }

    pub fn get_block_by_hash(&self, hash: &H256) -> Option<IndexedBlock> {
        self.get_block_by_id(hash)
    }

    pub fn get_block_by_id(&self, id: &H256) -> Option<IndexedBlock> {
        self.block_header
            .get(ReadOptions::default_instance(), id.as_bytes())
            .map(|raw_header| BlockHeader::decode(&*raw_header).unwrap())
            .map(|header| IndexedBlockHeader::new(id.clone(), header))
            .ok()
            .and_then(|header| self.get_block_from_header(header))
    }

    pub fn get_genesis_block(&self) -> Option<IndexedBlock> {
        self.get_block_by_number(0)
    }

    pub fn get_transaction_by_id(&self, id: &H256) -> Result<IndexedTransaction, BoxError> {
        let mut key = self
            .transaction_block
            .get(ReadOptions::default_instance(), id.as_bytes())?
            .to_vec();
        key.extend_from_slice(id.as_bytes());
        let txn = self
            .transaction
            .get(ReadOptions::default_instance(), &key)
            .map(|raw| Transaction::decode(&*raw).unwrap())
            .map(|txn| IndexedTransaction::new(id.clone(), txn))?;
        Ok(txn)
    }

    pub fn get_block_header_by_transaction(&self, txn: &IndexedTransaction) -> Result<IndexedBlockHeader, BoxError> {
        let block_key = self
            .transaction_block
            .get(ReadOptions::default_instance(), txn.hash.as_bytes())?;
        let header = self
            .block_header
            .get(ReadOptions::default_instance(), &block_key[..32])
            .map(|raw| BlockHeader::decode(&*raw).unwrap())
            .map(|header| IndexedBlockHeader::new(H256::from_slice(&block_key[..32]), header))?;
        Ok(header)
    }

    pub fn delete_transaction(&self, txn: &IndexedTransaction, wb: &mut WriteBatch) -> Result<(), BoxError> {
        let block_key = self
            .transaction_block
            .get(ReadOptions::default_instance(), txn.hash.as_bytes())?;

        if let Err(e) = self.block_header.get(ReadOptions::default_instance(), &block_key[..32]) {
            if e.is_not_found() {
                wb.deletev_cf(&self.transaction, &[&*block_key, txn.hash.as_bytes()]);
                wb.delete_cf(&self.transaction_block, txn.hash.as_bytes());
                return Ok(());
            }
        }

        Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            "transaction is linked to a block, please delete the block first",
        )))
    }

    pub fn delete_block(&self, block: &IndexedBlock) -> bool {
        let mut wb = WriteBatch::with_reserved_bytes(1024);

        wb.delete_cf(&self.block_header, block.hash().as_bytes());

        let header = &block.header;
        self.transaction
            .new_iterator(&ReadOptions::default().iterate_lower_bound(&header.hash.as_bytes()))
            .keys()
            .take_while(|key| &key[..32] == header.hash.as_bytes())
            .for_each(|key| {
                wb.delete_cf(&self.transaction, &key);
                wb.delete_cf(&self.transaction_block, &key[32 + 8..]);
            });

        self.db.write(WriteOptions::default_instance(), &wb).is_ok()
    }

    // leaves dangling transactions
    fn delete_block_header(&self, header: &IndexedBlockHeader, wb: &mut WriteBatch) {
        wb.delete_cf(&self.block_header, header.hash.as_bytes());
    }

    fn delete_orphan_transaction(&self, txn: &IndexedTransaction, wb: &mut WriteBatch) -> bool {
        if let Ok(block_hash) = self
            .transaction_block
            .get(ReadOptions::default_instance(), txn.hash.as_bytes())
        {
            if self
                .block_header
                .get(ReadOptions::default_instance(), &*block_hash)
                .is_ok()
            {
                eprintln!(
                    "! txn not orphan {:?} => block {}",
                    txn.hash,
                    BE::read_u64(&block_hash[..8])
                );
                false
            } else {
                wb.delete_cf(&self.transaction, txn.hash.as_bytes());
                wb.delete_cf(&self.transaction_block, txn.hash.as_bytes());
                true
            }
        } else {
            eprintln!("Not Found");
            false
        }
    }

    fn relink_transactions_to_block(&self, block: &IndexedBlock, wb: &mut WriteBatch) {
        if !block.verify_merkle_root_hash() {
            eprintln!("error while checking block merkle root hash");
            return;
        }
        let correct_block_hash = block.hash().as_bytes();
        let txn_hashes: Vec<_> = block.transactions.iter().map(|txn| txn.hash.as_bytes()).collect();
        let block_hashes = self
            .transaction_block
            .multi_get(ReadOptions::default_instance(), &txn_hashes);

        for (txn_hash, block_hash) in txn_hashes.iter().zip(block_hashes.into_iter()) {
            let block_hash = block_hash.unwrap();

            if block_hash != correct_block_hash {
                println!(
                    "! wrong block hash {:?} => {:?}",
                    hex::encode(txn_hash),
                    hex::encode(&block_hash)
                );
                wb.put_cf(&self.transaction_block, txn_hash, correct_block_hash);
            }
        }
    }

    pub fn block_hashes_from(&self, start_block_hash: &[u8], count: usize) -> Vec<Vec<u8>> {
        self.block_header
            .new_iterator(&ReadOptions::default().iterate_lower_bound(start_block_hash))
            .keys()
            .take(count)
            .map(|key| key.to_vec())
            .collect()
    }

    pub fn handle_chain_fork_at(&self, mut num: u64) -> Result<(), BoxError> {
        // check
        assert!(num > 0, "cannot fork from genesis block");
        assert!(self.get_block_headers_by_number(num - 1).len() == 1);
        assert!(self.get_block_headers_by_number(num).len() > 1);

        let mut forks: Vec<LinkedList<IndexedBlockHeader>> = Vec::new();
        loop {
            let headers = self.get_block_headers_by_number(num);
            for header in &headers {
                // find by parent_hash
                if let Some(found) = forks.iter_mut().find(|h| {
                    h.front().unwrap().hash == H256::from_slice(&header.raw.raw_data.as_ref().unwrap().parent_hash)
                }) {
                    found.push_front(header.clone());
                } else {
                    forks.push(LinkedList::from_iter(vec![header.clone()]));
                }
            }
            if headers.len() == 1 {
                break;
            }
            num += 1;
        }
        let mut tobe_purged_forks = vec![];
        let longest_fork = forks.iter().max_by_key(|fork| fork.len()).unwrap();
        for fork in &forks {
            print!("fork => {}", fork.len());
            if fork == longest_fork {
                println!(" (longest)");
            } else {
                println!(" (will purge)");
                tobe_purged_forks.push(fork);
            }
            for head in fork.iter().rev() {
                println!("  |- {:?} {}", head.hash, head.number());
            }
        }

        let mut wb = WriteBatch::with_reserved_bytes(1024);

        let mut txn_whitelist = HashSet::new();
        let mut orphan_txns = HashSet::new();
        for header in longest_fork.iter() {
            let block = self.get_block_from_header(header.clone()).unwrap();
            // link to the right block
            self.relink_transactions_to_block(&block, &mut wb);
            for txn in block.transactions {
                txn_whitelist.insert(txn);
            }
        }

        for fork in tobe_purged_forks {
            for header in fork.iter() {
                self.delete_block_header(header, &mut wb);
                let block = self.get_block_from_header(header.clone()).unwrap();
                for txn in block.transactions {
                    if !txn_whitelist.contains(&txn) {
                        orphan_txns.insert(txn);
                    }
                }
            }
        }
        self.db.write(WriteOptions::default_instance(), &wb)?;

        if !orphan_txns.is_empty() {
            wb.clear();
            for txn in &orphan_txns {
                if self.delete_orphan_transaction(&txn, &mut wb) {
                    println!("! delete orphan txn: {:?}", txn.hash);
                }
            }
            self.db.write(WriteOptions::default_instance(), &wb)?;
        }

        Ok(())
    }

    pub fn visit(&self) -> Result<(), Box<dyn Error>> {
        let it = self.transaction.new_iterator(ReadOptions::default_instance());

        for (key, raw) in it
        // .take(2000)
        {
            let txn = Transaction::decode(raw)?;
            match ContractType::from_i32(txn.raw_data.as_ref().unwrap().contract.as_ref().unwrap().r#type) {
                Some(ContractType::TransferContract) => {
                    println!("txn id: {} => {:?}", hex::encode(key), txn.result);
                }
                Some(typ) => {
                    println!("txn: {:?}", typ);
                }
                None => unreachable!(),
            }
        }
        Ok(())
    }

    pub fn verify_parent_hashes_from(&self, num: u64) -> Result<bool, BoxError> {
        let start_block = self.get_block_by_number(num).unwrap();
        let mut parent_hash = start_block.header.raw.raw_data.as_ref().unwrap().parent_hash.to_vec();

        println!("start from parent hash = {}", hex::encode(&parent_hash));

        let mut lower_bound = [0u8; 8];
        BE::write_u64(&mut lower_bound, num);

        for (blk_id, raw_header) in self
            .block_header
            .new_iterator(&ReadOptions::default().iterate_lower_bound(&lower_bound))
        {
            let header = IndexedBlockHeader::new(H256::from_slice(blk_id), BlockHeader::decode(raw_header).unwrap());
            if header.raw.raw_data.as_ref().unwrap().parent_hash != parent_hash {
                eprintln!("❌ parent_hash verification error");
                eprintln!(
                    "parent block {} => {}",
                    BE::read_u64(&parent_hash[..8]),
                    hex::encode(parent_hash)
                );
                eprintln!("current block {} => {:?}", header.number(), header);
                return Ok(false);
            }
            if header.number() % 10000 == 0 {
                println!("block => {} parent_hash => {}", header.number(), hex::encode(blk_id));
            }
            parent_hash = blk_id.to_vec();
        }

        println!("✅ verification all passed!");
        Ok(true)
    }

    pub fn verify_parent_hashes(&self) -> Result<bool, Box<dyn Error>> {
        // genesis parent_hash: e58f33f9baf9305dc6f82b9f1934ea8f0ade2defb951258d50167028c780351f
        self.verify_parent_hashes_from(0)
    }

    /*
    pub fn blocks(&self) -> impl Iterator<Item = IndexedBlock> {
        self.block_header
            .new_iterator(ReadOptions::default_instance())
            .map(|(blk_id, raw_header)| {
                IndexedBlockHeader::new(H256::from_slice(blk_id), BlockHeader::decode(raw_header).unwrap())
            })
            .map(|header| self.get_block_from_header(header).unwrap())
    }
    */

    /**
     * From block 1102553, 1103364, 1103650, 1135972
     */
    pub fn verify_merkle_tree(&self) -> Result<(), Box<dyn Error>> {
        let ropt = ReadOptions::default();

        for (blk_id, raw_header) in self.block_header.new_iterator(&ropt) {
            let header = IndexedBlockHeader::new(H256::from_slice(blk_id), BlockHeader::decode(raw_header).unwrap());
            let block = self.get_block_from_header(header).unwrap();

            if block.number() % 1000 == 0 {
                println!("block {} {:?}", block.number(), block.hash());
            }
            if !block.verify_merkle_root_hash() {
                println!(
                    "! mismatch block => {} {:?}\n  merkle tree={}",
                    block.number(),
                    block.hash(),
                    hex::encode(block.merkle_root_hash())
                );
                for (i, txn) in block.transactions.iter().enumerate() {
                    println!("  txn {} {:?} verify={}", i, txn.hash, txn.verify());
                }
            }
        }
        Ok(())
    }

    pub fn get_db_property(&self, key: &str) -> u64 {
        self.db.get_int_property(key).unwrap_or_default()
    }

    pub fn get_accumulated_db_property(&self, key: &str) -> u64 {
        [
            &self.default,
            &self.block_header,
            &self.transaction,
            &self.transaction_block,
        ]
        .iter()
        .map(|cf| cf.get_int_property(key).unwrap_or_default())
        .sum()
    }

    pub fn report_status(&self) {
        info!(
            "rocksdb.num-running-compactions = {:?}",
            self.db
                .get_int_property("rocksdb.num-running-compactions")
                .unwrap_or_default()
        );
        info!(
            "rocksdb.num-running-flushes = {:?}",
            self.db
                .get_int_property("rocksdb.num-running-flushes")
                .unwrap_or_default()
        );
        debug!(
            "rocksdb.compaction-pending = {:?}",
            self.db
                .get_int_property("rocksdb.compaction-pending")
                .unwrap_or_default()
        );
        debug!(
            "rocksdb.mem-table-flush-pending = {:?}",
            self.db
                .get_int_property("rocksdb.mem-table-flush-pending")
                .unwrap_or_default()
        );
    }

    pub fn await_background_jobs(&self) {
        loop {
            let n_compactions = self
                .db
                .get_int_property("rocksdb.num-running-compactions")
                .unwrap_or_default();
            let n_flushes = self
                .db
                .get_int_property("rocksdb.num-running-flushes")
                .unwrap_or_default();
            info!(
                "awaiting background jobs, compactions={}, flushes={}",
                n_compactions, n_flushes
            );
            if n_compactions + n_flushes <= 1 {
                return;
            }
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
    }

    pub fn compact_db(&self) -> Result<(), BoxError> {
        self.default.compact_range(&Default::default(), ..)?;
        self.block_header.compact_range(&Default::default(), ..)?;
        self.transaction.compact_range(&Default::default(), ..)?;
        self.transaction_block.compact_range(&Default::default(), ..)?;
        Ok(())
    }

    pub unsafe fn prepare_close(&self) {
        info!("flush db ... {:?}", self.db.flush(&FlushOptions::default()));
        self.db.cancel_background_work(/* wait: */ true);
        info!("cancel background work");
        info!("syncing WAL ... {:?}", self.db.sync_wal());
        // eprintln!("Close DB ... {:?}", self.db.close());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn check_parent_hash() {
        println!("opening db ...");
        let db = ChainDB::new("./data");
        println!("db opened!");
        db.report_status();
        //assert!(db.verify_parent_hashes().unwrap());

        assert!(db.verify_parent_hashes_from(19750000).unwrap());
    }

    #[test]
    #[ignore]
    fn fix_forked_chain() {
        println!("opening db ...");
        let db = ChainDB::new("./data");
        println!("db opened!");
        db.report_status();

        let num = 19752249;

        if let Some(block) = db.get_block_by_number(num) {
            println!("block found and is unique: {:?}", block.hash());
        } else {
            db.handle_chain_fork_at(num).unwrap();
        }
    }

    #[test]
    #[ignore]
    fn create_db() {
        let db = ChainDB::new("./data");

        println!("ok");

        db.report_status();

        assert!(db.highest_block().is_some());

        assert!(db.has_block_id(&H256::from_slice(
            b"\x00\x00\x00\x00\x00\x00\x00\x00\x1e\xbf\x88P\x8a\x03\x86\\q\xd4R\xe2_MQ\x19A\x96\xa1\xd2+fS\xdc"
        )));

        // let blk = db.get_block_by_number(0).unwrap();
        let blk = db.get_genesis_block().unwrap();
        // println!("blk => {:?}", blk.header.hash);

        for txn in blk.transactions {
            println!("=> {:?}", txn.hash);
            println!("{:?}", txn.raw.raw_data.unwrap().contract.unwrap().parameter);
        }

        // db.visit();
        println!("==================================");
        db.verify_parent_hashes().unwrap();
        // db.verify_merkle_tree();
    }
}
