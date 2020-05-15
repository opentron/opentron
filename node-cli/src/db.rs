extern crate byteorder;

use byteorder::{ByteOrder, BE};
use bytes::BytesMut;
use chain::{BlockHeader, IndexedBlock, IndexedBlockHeader, IndexedTransaction, Transaction};
use log::{debug, error, info, warn};
use primitives::H256;
use prost::Message;
use proto2::chain::ContractType;
use rocks::prelude::*;
use std::collections::{HashMap, HashSet, LinkedList};
use std::error::Error;
use std::iter::FromIterator;
use std::path::Path;

pub type BoxError = Box<dyn Error>;

pub struct ChainDB {
    db: DB,
    default: ColumnFamily,
    block_header: ColumnFamily,
    block_transactions: ColumnFamily,
    transaction: ColumnFamily,
    transaction_block: ColumnFamily,
}

impl Drop for ChainDB {
    fn drop(&mut self) {
        println!("db closed");
    }
}

impl ChainDB {
    pub fn new<P: AsRef<Path>>(db_path: P) -> ChainDB {
        let db_options = DBOptions::default()
            .create_if_missing(true)
            .create_missing_column_families(true)
            .increase_parallelism(6)
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
            ColumnFamilyDescriptor::new(
                "block-header",
                ColumnFamilyOptions::default().max_write_buffer_number(6),
            ),
            ColumnFamilyDescriptor::new(
                "block-transactions",
                ColumnFamilyOptions::default()
                    .optimize_for_point_lookup(128)
                    .max_write_buffer_number(6),
            ),
            ColumnFamilyDescriptor::new(
                "transaction",
                ColumnFamilyOptions::default()
                    .optimize_level_style_compaction(512 * 1024 * 1024)
                    .optimize_for_point_lookup(256)
                    .max_write_buffer_number(6),
            ),
            ColumnFamilyDescriptor::new(
                "transaction-block",
                ColumnFamilyOptions::default()
                    .optimize_level_style_compaction(512 * 1024 * 1024)
                    .optimize_for_point_lookup(32)
                    .max_write_buffer_number(6),
            ),
        ];

        let (db, mut handles) = DB::open_with_column_families(&db_options, db_path, column_families).unwrap();
        let txn_blk = handles.pop().unwrap();
        let txn = handles.pop().unwrap();
        let blk_txns = handles.pop().unwrap();
        let blk = handles.pop().unwrap();
        let default = handles.pop().unwrap();

        assert!(handles.is_empty());

        ChainDB {
            db: db,
            default: default,
            block_header: blk,
            block_transactions: blk_txns,
            transaction: txn,
            transaction_block: txn_blk,
        }
    }

    pub fn new_for_sync<P: AsRef<Path>>(db_path: P) -> ChainDB {
        let options = Options::default().prepare_for_bulk_load();

        let db_options = DBOptions::default()
            .create_if_missing(true)
            .create_missing_column_families(true)
            .increase_parallelism(6)
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
            ColumnFamilyDescriptor::new("block-header", ColumnFamilyOptions::from_options(&options)),
            ColumnFamilyDescriptor::new(
                "block-transactions",
                ColumnFamilyOptions::from_options(&options).optimize_for_point_lookup(128),
            ),
            ColumnFamilyDescriptor::new(
                "transaction",
                ColumnFamilyOptions::from_options(&options).optimize_for_point_lookup(256),
            ),
            ColumnFamilyDescriptor::new(
                "transaction-block",
                ColumnFamilyOptions::from_options(&options).optimize_for_point_lookup(32),
            ),
        ];

        let (db, mut handles) = DB::open_with_column_families(&db_options, db_path, column_families).unwrap();
        let txn_blk = handles.pop().unwrap();
        let txn = handles.pop().unwrap();
        let blk_txns = handles.pop().unwrap();
        let blk = handles.pop().unwrap();
        let default = handles.pop().unwrap();

        assert!(handles.is_empty());

        ChainDB {
            db: db,
            default: default,
            block_header: blk,
            block_transactions: blk_txns,
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

        if block.transactions.is_empty() {
            batch.put_cf(&self.block_transactions, block.header.hash.as_bytes(), b"");
        } else {
            for txn in &block.transactions {
                buf.clear();
                txn.raw.encode(&mut buf)?;
                batch.put_cf(&self.transaction, txn.hash.as_bytes(), &buf);
                batch.put_cf(
                    &self.transaction_block,
                    txn.hash.as_bytes(),
                    block.header.hash.as_bytes(),
                );
            }
            let txn_ids: Vec<_> = block
                .transactions
                .iter()
                .map(|txn| txn.hash.as_bytes())
                .collect::<Vec<_>>();
            batch.putv_cf(&self.block_transactions, &[block.header.hash.as_bytes()], &txn_ids);
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
        let raw_txn_ids = self
            .block_transactions
            .get(ReadOptions::default_instance(), header.hash.as_bytes())
            .unwrap_or_default();
        let txn_ids: Vec<&[u8]> = raw_txn_ids.chunks_exact(32).collect();

        let transactions = self
            .transaction
            .multi_get(ReadOptions::default_instance(), &txn_ids)
            .into_iter()
            .zip(txn_ids.iter())
            .map(|(maybe_raw, txn_id)| {
                maybe_raw
                    .ok()
                    .and_then(|raw| Transaction::decode(&*raw).ok())
                    .map(|txn| IndexedTransaction::new(H256::from_slice(txn_id), txn))
            })
            .collect::<Option<Vec<_>>>();

        Some(IndexedBlock::new(header, transactions.unwrap_or_default()))
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
        if found.len() != 1 {
            eprintln!("multiple blocks found for same number: {}", num);
            for item in &found {
                println!("  => {}", hex::encode(&item.0));
                println!("  => {}", hex::encode(&item.1));
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

    pub fn delete_block(&self, block: &IndexedBlock) -> bool {
        let mut wb = WriteBatch::with_reserved_bytes(1024);

        wb.delete_cf(&self.block_header, block.hash().as_bytes());
        wb.delete_cf(&self.block_transactions, block.hash().as_bytes());
        for txn in &block.transactions {
            wb.delete_cf(&self.transaction, txn.hash.as_bytes());
            wb.delete_cf(&self.transaction_block, txn.hash.as_bytes());
        }

        self.db.write(WriteOptions::default_instance(), &wb).is_ok()
    }

    // leaves dangling transactions
    fn delete_block_header(&self, header: &IndexedBlockHeader, wb: &mut WriteBatch) {
        wb.delete_cf(&self.block_header, header.hash.as_bytes());
        wb.delete_cf(&self.block_transactions, header.hash.as_bytes());
    }

    fn delete_transaction(&self, txn: &IndexedTransaction, wb: &mut WriteBatch) {
        wb.delete_cf(&self.transaction, txn.hash.as_bytes());
        wb.delete_cf(&self.transaction_block, txn.hash.as_bytes());
    }

    fn relink_transactions_to_block(&self, block: &IndexedBlock, wb: &mut WriteBatch) {
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
                        println!("delete orphan txn: {:?}", txn.hash);
                        self.delete_transaction(&txn, &mut wb);
                    }
                }
            }
        }

        /*
        let mut handler = rocks::write_batch::WriteBatchIteratorHandler::default();
        wb.iterate(&mut handler).unwrap();
        for entry in handler.entries {
            println!("{:?}", entry);
        }
        */
        self.db.write(WriteOptions::default_instance(), &wb)?;
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

    pub fn verify_parent_hashes(&self) -> Result<bool, Box<dyn Error>> {
        let mut parent_hash = hex::decode("e58f33f9baf9305dc6f82b9f1934ea8f0ade2defb951258d50167028c780351f").unwrap();

        for (blk_id, raw_header) in self.block_header.new_iterator(ReadOptions::default_instance()) {
            let header = IndexedBlockHeader::new(H256::from_slice(blk_id), BlockHeader::decode(raw_header).unwrap());
            if header.raw.raw_data.as_ref().unwrap().parent_hash != parent_hash {
                eprintln!("❌ parent_hash verification error");
                eprintln!("parent => {}", hex::encode(parent_hash));
                eprintln!("current block {} => {:?}", header.number(), header);
                return Ok(false);
            }
            if header.number() % 10000 == 0 {
                println!("block => {} parent_hash => {}", header.number(), hex::encode(blk_id));
            }
            parent_hash = blk_id.to_vec();
        }
        /*

        let last_block_number = BE::read_u64(&parent_hash[..8]);
        println!(
            "last block => {} hash => {}",
            last_block_number,
            hex::encode(parent_hash)
        );
        if last_block_number as i64 != self.get_block_height() {
            println!("❌ block height in db is {}", self.get_block_height());
            println!("There is missing blocks!");
        } else {
            */
        println!("✅ verification all passed!");

        Ok(true)
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

    pub fn verify_merkle_tree(&self) -> Result<(), Box<dyn Error>> {
        for (blk_id, raw_header) in self.block_header.new_iterator(ReadOptions::default_instance()) {
            let header = IndexedBlockHeader::new(H256::from_slice(blk_id), BlockHeader::decode(raw_header).unwrap());
            let block = self.get_block_from_header(header);
        }
        Ok(())
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
        // info!("threads = {:?}", Env::default_instance().get_thread_list());
        /*
        println!(
            "[block-header] rocksdb.estimate-num-keys = {:?}",
            self.block_header
                .get_int_property("rocksdb.estimate-num-keys")
                .unwrap_or_default()
        );
        println!(
            "[transactions] rocksdb.estimate-num-keys = {:?}",
            self.transaction
                .get_int_property("rocksdb.estimate-num-keys")
                .unwrap_or_default()
        );
        */
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

    pub unsafe fn close(&self) {
        eprintln!("Flush ... {:?}", self.db.flush(&FlushOptions::default()));
        self.db.cancel_background_work(/* wait: */ true);
        eprintln!("Syncing WAL ... {:?}", self.db.sync_wal());
        eprintln!("Close DB ... {:?}", self.db.close());
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
        assert!(db.verify_parent_hashes().unwrap());
    }

    #[test]
    #[ignore]
    fn fix_forked_chain() {
        println!("opening db ...");
        let db = ChainDB::new("./data");
        println!("db opened!");
        db.report_status();

        let num = 19720484;

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
        db.verify_parent_hashes();
        // db.verify_merkle_tree();
    }
}
