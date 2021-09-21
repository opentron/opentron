use std::collections::{HashMap, HashSet, LinkedList};
use std::error::Error;
use std::fs::{create_dir_all, OpenOptions};
use std::io::{self, Write};
use std::iter::FromIterator;
use std::path::Path;

use byteorder::{ByteOrder, BE};
use bytes::BytesMut;
use log::{error, info, warn};
use prost::Message;
use rand::Rng;
use rocks::prelude::*;
use types::H256;

use chain::{BlockHeader, IndexedBlock, IndexedBlockHeader, IndexedTransaction, Transaction};
use proto::chain::ContractType;

pub type BoxError = Box<dyn Error>;

#[derive(Debug)]
pub enum CheckResult {
    Ok,
    ForkAt(u64),
    BreakAt(u64),
}

pub struct ChainDB {
    db: DB,
    default: ColumnFamily,
    block_header: ColumnFamily,
    transaction: ColumnFamily,
    transaction_block: ColumnFamily,
}

impl Drop for ChainDB {
    fn drop(&mut self) {
        info!("chain-db closed successfully");
    }
}

impl ChainDB {
    pub fn new<P: AsRef<Path>>(db_path: P) -> ChainDB {
        create_dir_all(&db_path).expect("create db directory");

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
            // [block_hash, transaction_index: u64, transaction_hash] => Transaction
            ColumnFamilyDescriptor::new(
                "transaction",
                ColumnFamilyOptions::default()
                    .prefix_extractor_fixed(32)
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

    pub fn reset_node_id(&self) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut node_id = vec![b'A'; 64];
        rng.fill(&mut node_id[32..]);
        self.default
            .put(WriteOptions::default_instance(), b"NODE_ID", &node_id)
            .unwrap();
        node_id
    }

    pub fn get_node_id(&self) -> Vec<u8> {
        if let Ok(node_id) = self.default.get(ReadOptions::default_instance(), b"NODE_ID") {
            node_id.to_vec()
        } else {
            self.reset_node_id()
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

    pub fn force_update_block_height(&self, height: i64) -> Result<(), BoxError> {
        let mut val = [0u8; 8];
        BE::write_u64(&mut val, height as u64);
        self.default
            .put(WriteOptions::default_instance(), b"BLOCK_HEIGHT", &val)
            .map_err(From::from)
    }

    /// Highest block id, counted from 0
    pub fn highest_block(&self) -> Result<IndexedBlock, BoxError> {
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
        let mut lower_bound = [0u8; 32];
        BE::write_u64(&mut lower_bound[..8], num);
        let mut upper_bound = [0xffu8; 32];
        BE::write_u64(&mut upper_bound[..8], num);

        let it = self.block_header.new_iterator(
            &ReadOptions::default()
                .iterate_lower_bound(&lower_bound[..])
                .iterate_upper_bound(&upper_bound[..]),
        );
        it.count() > 0
    }

    pub fn get_block_from_header(&self, header: IndexedBlockHeader) -> Result<IndexedBlock, BoxError> {
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
                let txn = Transaction::decode(val)?;
                Ok(IndexedTransaction::new(H256::from_slice(&key[32 + 8..]), txn))
            })
            .collect::<Result<Vec<_>, BoxError>>();

        transactions.map(|txns| IndexedBlock::new(header, txns))
    }

    pub fn get_block_transactions(&self, hash: &H256) -> Result<Vec<IndexedTransaction>, BoxError> {
        let mut upper_bound = hash.as_bytes().to_vec();
        upper_bound.push(0xFF); // [0xcafebabe00 .. 0xcafebabeff]

        let ropts = ReadOptions::default()
            .iterate_lower_bound(&hash.as_bytes())
            .iterate_upper_bound(&upper_bound);
        let txns = self
            .transaction
            .new_iterator(&ropts)
            .map(|(key, val)| {
                let txn = Transaction::decode(val)?;
                Ok(IndexedTransaction::new(H256::from_slice(&key[32 + 8..]), txn))
            })
            .collect::<Result<Vec<_>, BoxError>>();
        drop(ropts);
        txns
    }

    pub fn get_transaction_hashes_by_block_number(&self, num: i64) -> Result<Vec<H256>, BoxError> {
        let mut lower_bound = [0u8; 8];
        BE::write_u64(&mut lower_bound[..], num as u64);
        let mut upper_bound = [0u8; 8];
        BE::write_u64(&mut upper_bound[..], num as u64 + 1);

        let ropts = ReadOptions::default()
            .iterate_lower_bound(&lower_bound)
            .iterate_upper_bound(&upper_bound);
        let txn_hashes = self
            .transaction
            .new_iterator(&ropts)
            .keys()
            .map(|key| Ok(H256::from_slice(&key[32 + 8..])))
            .collect::<Result<Vec<_>, BoxError>>();
        drop(ropts);
        txn_hashes
    }

    pub fn get_transaction_hashes_by_block_hash(&self, hash: &H256) -> Result<Vec<H256>, BoxError> {
        let mut upper_bound = hash.as_bytes().to_vec();
        upper_bound.push(0xFF); // [0xcafebabe00 .. 0xcafebabeff]

        let ropts = ReadOptions::default()
            .iterate_lower_bound(&hash.as_bytes())
            .iterate_upper_bound(&upper_bound);
        let txn_hashes = self
            .transaction
            .new_iterator(&ropts)
            .keys()
            .map(|key| Ok(H256::from_slice(&key[32 + 8..])))
            .collect::<Result<Vec<_>, BoxError>>();
        drop(ropts);
        txn_hashes
    }

    pub fn get_block_header_by_number(&self, num: i64) -> Result<IndexedBlockHeader, BoxError> {
        let mut headers = self.get_block_headers_by_number(num as u64);
        if headers.is_empty() {
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, "not found")));
        }
        if headers.len() != 1 {
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, "chain fork")));
        }
        let header = headers.pop().unwrap();
        Ok(header)
    }

    pub fn get_block_header(&self, hash: &H256) -> Result<IndexedBlockHeader, BoxError> {
        self.block_header
            .get(ReadOptions::default_instance(), hash.as_bytes())
            .map_err(From::from)
            .and_then(|raw_header| BlockHeader::decode(&*raw_header).map_err(From::from))
            .map(|header| IndexedBlockHeader::new(hash.clone(), header))
    }

    /// handles fork
    pub fn get_block_headers_by_number(&self, num: u64) -> Vec<IndexedBlockHeader> {
        let mut lower_bound = [0u8; 32];
        BE::write_u64(&mut lower_bound[..8], num);
        let mut upper_bound = [0xff_u8; 32];
        BE::write_u64(&mut upper_bound[..8], num);
        let ropt = ReadOptions::default()
            .iterate_lower_bound(&lower_bound[..])
            .iterate_upper_bound(&upper_bound[..])
            .pin_data(true);

        self.block_header
            .new_iterator(&ropt)
            .map(|(key, val)| IndexedBlockHeader::new(H256::from_slice(key), BlockHeader::decode(val).unwrap()))
            .collect()
    }

    pub fn get_block_by_number(&self, num: u64) -> Result<IndexedBlock, BoxError> {
        let mut lower_bound = [0u8; 32];
        BE::write_u64(&mut lower_bound[..8], num);
        let mut upper_bound = [0xff_u8; 32];
        BE::write_u64(&mut upper_bound[..8], num);

        let ropts = ReadOptions::default()
            .iterate_lower_bound(&lower_bound[..])
            .iterate_upper_bound(&upper_bound[..])
            .pin_data(true);
        let it = self.block_header.new_iterator(&ropts);

        // FIXME: iterator key lifetime leaks, key might becomes same key
        // ref: https://github.com/bh1xuw/rust-rocks/issues/15
        let found = it
            // .take_while(|(key, _)| &key[..8] == lower_bound)
            .map(|(key, val)| (key.to_vec(), val.to_vec()))
            .collect::<Vec<_>>();
        drop(ropts); // holds lifetime of bound slice.

        if found.is_empty() {
            return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "block not found")));
        }
        if found.len() > 1 {
            eprintln!("multiple blocks found for same number: {}", num);
            for item in &found {
                eprintln!("  => {}", hex::encode(&item.0));
                eprintln!("  => {}", hex::encode(&item.1));
            }
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, "fork found")));
        }

        let header = IndexedBlockHeader::new(H256::from_slice(&found[0].0), BlockHeader::decode(&*found[0].1)?);
        self.get_block_from_header(header)
    }

    pub fn get_block_by_hash(&self, hash: &H256) -> Result<IndexedBlock, BoxError> {
        self.get_block_by_id(hash)
    }

    pub fn get_block_by_id(&self, id: &H256) -> Result<IndexedBlock, BoxError> {
        self.block_header
            .get(ReadOptions::default_instance(), id.as_bytes())
            .map_err(From::from)
            .and_then(|raw_header| BlockHeader::decode(&*raw_header).map_err(From::from))
            .map(|header| IndexedBlockHeader::new(id.clone(), header))
            .and_then(|header| self.get_block_from_header(header))
    }

    pub fn get_genesis_block(&self) -> Result<IndexedBlock, BoxError> {
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

    pub fn get_transaction_index(&self, id: &H256) -> Result<i32, BoxError> {
        let key = self
            .transaction_block
            .get(ReadOptions::default_instance(), id.as_bytes())?;
        Ok(BE::read_u64(&key[32..]) as i32)
    }

    pub fn get_transaction_block_hash(&self, id: &H256) -> Result<H256, BoxError> {
        let key = self
            .transaction_block
            .get(ReadOptions::default_instance(), id.as_bytes())?;
        Ok(H256::from_slice(&key[..32]))
    }

    pub fn get_block_header_by_transaction_hash(&self, txn_hash: &H256) -> Result<IndexedBlockHeader, BoxError> {
        let block_key = self
            .transaction_block
            .get(ReadOptions::default_instance(), txn_hash.as_bytes())?;
        self.block_header
            .get(ReadOptions::default_instance(), &block_key[..32])
            .map(|raw| BlockHeader::decode(&*raw).unwrap())
            .map(|header| IndexedBlockHeader::new(H256::from_slice(&block_key[..32]), header))
            .map_err(From::from)
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

    pub fn delete_block_by_number(&self, num: u64) -> Result<(), BoxError> {
        let mut lower_bound = [0u8; 8];
        BE::write_u64(&mut lower_bound[..], num);

        let mut wb = WriteBatch::with_reserved_bytes(1024);

        self.block_header
            .new_iterator(&ReadOptions::default().iterate_lower_bound(&lower_bound))
            .keys()
            .take_while(|key| &key[..8] == &lower_bound)
            .for_each(|key| {
                info!("delete block {}", hex::encode(key));
                wb.delete_cf(&self.block_header, key);
            });
        self.transaction
            .new_iterator(&ReadOptions::default().iterate_lower_bound(&lower_bound))
            .keys()
            .take_while(|key| &key[..8] == &lower_bound)
            .for_each(|key| {
                info!("delete transaction {}", hex::encode(&key[32 + 8..]));
                wb.delete_cf(&self.transaction, key);
                wb.delete_cf(&self.transaction_block, &key[32 + 8..]);
            });

        self.db.write(WriteOptions::default_instance(), &wb)?;

        Ok(())
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

    fn delete_block_without_reverse_index(&self, block: &IndexedBlock, wb: &mut WriteBatch) {
        wb.delete_cf(&self.block_header, block.hash().as_bytes());

        let header = &block.header;
        self.transaction
            .new_iterator(&ReadOptions::default().iterate_lower_bound(&header.hash.as_bytes()))
            .keys()
            .take_while(|key| &key[..32] == header.hash.as_bytes())
            .for_each(|key| {
                wb.delete_cf(&self.transaction, &key);
            });
    }

    fn relink_transactions_to_block(&self, block: &IndexedBlock, wb: &mut WriteBatch) {
        if !block.verify_merkle_root_hash() {
            eprintln!("error while checking block merkle root hash");
            return;
        }
        block.transactions.iter().enumerate().for_each(|(i, txn)| {
            let mut corrent_reverse_index = vec![0u8; 32 + 8];
            (&mut corrent_reverse_index[..32]).copy_from_slice(block.hash().as_bytes());
            BE::write_u64(&mut corrent_reverse_index[32..], i as u64);

            let reverse_index = self
                .transaction_block
                .get(ReadOptions::default_instance(), txn.hash.as_bytes())
                .unwrap();

            if corrent_reverse_index != &*reverse_index {
                println!(
                    "! wrong reverse index {:?}\n=> {}\n=> {}",
                    txn.hash,
                    hex::encode(&*reverse_index),
                    hex::encode(&corrent_reverse_index),
                );
                wb.put_cf(&self.transaction_block, txn.hash.as_ref(), &corrent_reverse_index);
            }
        });
    }

    pub fn block_hashes_from(&self, start_block_hash: &[u8], count: usize) -> Vec<Vec<u8>> {
        self.block_header
            .new_iterator(&ReadOptions::default().iterate_lower_bound(start_block_hash))
            .keys()
            .take(count)
            .map(|key| key.to_vec())
            .collect()
    }

    pub fn handle_chain_fork_at(&self, mut num: u64, dry_run: bool) -> Result<(), BoxError> {
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
            if headers.is_empty() {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "can not determine longest fork",
                )));
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
                // wb.delete_cf(&self.block_header, header.hash.as_bytes());
                let block = self.get_block_from_header(header.clone()).unwrap();
                self.delete_block_without_reverse_index(&block, &mut wb);
                println!("! delete block {:?}", header.hash);
                for txn in block.transactions {
                    if !txn_whitelist.contains(&txn) {
                        println!("! found orphan txn: {:?}", txn.hash);
                        orphan_txns.insert(txn);
                    }
                }
            }
        }
        if dry_run {
            return Ok(());
        }
        self.db.write(WriteOptions::default_instance(), &wb)?;

        if !orphan_txns.is_empty() {
            let mut f = OpenOptions::new()
                .read(true)
                .create(true)
                .append(true)
                .open("./orphan_txns.log")?;
            for txn in &orphan_txns {
                write!(f, "{:?}\n", txn.hash)?;
            }
        }

        Ok(())
    }

    pub fn visit(&self) -> Result<(), Box<dyn Error>> {
        let it = self.transaction.new_iterator(ReadOptions::default_instance());

        for (key, raw) in it {
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

    pub fn block_headers<'a>(&'a self) -> impl Iterator<Item = IndexedBlockHeader> + 'a {
        self.block_header
            .new_iterator(ReadOptions::default_instance())
            .map(|(blk_id, raw_header)| {
                IndexedBlockHeader::new(H256::from_slice(blk_id), BlockHeader::decode(raw_header).unwrap())
            })
    }

    pub fn blocks<'a>(&'a self) -> impl Iterator<Item = IndexedBlock> + 'a {
        self.block_header
            .new_iterator(ReadOptions::default_instance())
            .map(|(blk_id, raw_header)| {
                IndexedBlockHeader::new(H256::from_slice(blk_id), BlockHeader::decode(raw_header).unwrap())
            })
            .map(move |header| self.get_block_from_header(header).unwrap())
    }

    pub fn ref_block_hashes_of_block_num(&self, num: i64) -> Vec<H256> {
        if num < 65536 {
            self.block_headers()
                .take(num as usize + 1)
                .map(|head| head.hash)
                .collect()
        } else {
            let mut lower_bound = [0u8; 32];
            BE::write_u64(&mut lower_bound[..8], num as u64 - 65535);
            let mut upper_bound = [0xff_u8; 32];
            BE::write_u64(&mut upper_bound[..8], num as u64);

            let mut ref_hashes = self
                .block_header
                .new_iterator(
                    &ReadOptions::default()
                        .iterate_lower_bound(&lower_bound[..])
                        .iterate_upper_bound(&upper_bound[..]),
                )
                .keys()
                .map(|raw_hash| H256::from_slice(raw_hash))
                .collect::<Vec<_>>();

            let wrap_pos = 65536 - (num + 1) % 65536;
            let mut new_ref_hashes = ref_hashes.split_off(wrap_pos as usize);
            new_ref_hashes.extend(ref_hashes);
            new_ref_hashes
        }
    }

    pub fn get_parent_hash_verified_block_number(&self) -> u64 {
        self.default
            .get(ReadOptions::default_instance(), b"PARENT_HASH_VERIFIED")
            .map(|raw| BE::read_u64(&*raw))
            .unwrap_or(0)
    }

    pub fn update_parent_hash_verified_block_number(&self, num: u64) -> Result<(), BoxError> {
        let mut raw = [0u8; 8];
        BE::write_u64(&mut raw[..], num);
        self.default
            .put(WriteOptions::default_instance(), b"PARENT_HASH_VERIFIED", &raw)
            .map_err(From::from)
    }

    pub fn verify_parent_hashes(&self) -> Result<CheckResult, BoxError> {
        let start_block_num = self.get_parent_hash_verified_block_number();
        let start_block = self.get_block_by_number(start_block_num)?;

        let mut parent_hash = start_block.header.raw.raw_data.as_ref().unwrap().parent_hash.to_vec();

        info!(
            "start from block {}, parent_hash = {}",
            start_block_num,
            hex::encode(&parent_hash)
        );

        for header in self
            .block_header
            .new_iterator(&ReadOptions::default().iterate_lower_bound(start_block.hash().as_bytes()))
            .map(|(blk_id, raw_header)| {
                IndexedBlockHeader::new(H256::from_slice(blk_id), BlockHeader::decode(raw_header).unwrap())
            })
        {
            if header.raw.raw_data.as_ref().unwrap().parent_hash != parent_hash {
                let parent_block_number = BE::read_u64(&parent_hash[..8]);
                // block_number - 1 to handle multiple forks
                self.update_parent_hash_verified_block_number(parent_block_number - 1)?;

                error!("❌ parent_hash verification error");
                warn!(
                    "parent block {}, hash = {}",
                    parent_block_number,
                    hex::encode(parent_hash)
                );
                warn!(
                    "current block {}, parent_hash = {}",
                    header.number(),
                    hex::encode(&header.raw.raw_data.as_ref().unwrap().parent_hash)
                );
                if parent_block_number == header.number() as u64 {
                    return Ok(CheckResult::ForkAt(parent_block_number));
                } else {
                    return Ok(CheckResult::BreakAt(parent_block_number));
                }
            }
            if header.number() % 10000 == 0 {
                info!("block => {} parent_hash => {:?}", header.number(), header.hash);
            }
            parent_hash = header.hash.as_bytes().to_vec();
        }

        let block_number = BE::read_u64(&parent_hash[..8]);
        self.update_parent_hash_verified_block_number(block_number)?;

        info!("✅ verification all passed!");
        Ok(CheckResult::Ok)
    }

    pub fn get_merkle_tree_verified_block_number(&self) -> u64 {
        self.default
            .get(ReadOptions::default_instance(), b"MERKLE_TREE_VERIFIED")
            .map(|raw| BE::read_u64(&*raw))
            .unwrap_or(0)
    }

    pub fn update_merkle_tree_verified_block_number(&self, num: u64) -> Result<(), BoxError> {
        let mut raw = [0u8; 8];
        BE::write_u64(&mut raw[..], num);
        self.default
            .put(WriteOptions::default_instance(), b"MERKLE_TREE_VERIFIED", &raw)
            .map_err(From::from)
    }

    pub fn verify_merkle_tree(&self, patch: &HashMap<H256, H256>) -> Result<bool, Box<dyn Error>> {
        let start_block = self.get_block_by_number(self.get_merkle_tree_verified_block_number())?;
        let ropt = ReadOptions::default().iterate_lower_bound(start_block.hash().as_bytes());
        info!("verify merkle tree from {}", start_block.number());

        for (blk_id, raw_header) in self.block_header.new_iterator(&ropt) {
            let header = IndexedBlockHeader::new(H256::from_slice(blk_id), BlockHeader::decode(raw_header).unwrap());
            let block = self.get_block_from_header(header).unwrap();

            if !block.verify_merkle_root_hash() {
                if block.verify_merkle_root_hash_with_patch(patch) {
                    info!("verified block {} with patch", block.number());
                } else {
                    error!("verify block {} failed", block.number());
                    return Ok(false);
                }
            }
            if block.number() % 1000 == 0 {
                println!("block {} {:?}", block.number(), block.hash());
                self.update_merkle_tree_verified_block_number(block.number() as _)?;
            }
        }
        Ok(true)
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
        let n_compactions = self
            .db
            .get_int_property("rocksdb.num-running-compactions")
            .unwrap_or_default();
        let n_flushes = self
            .db
            .get_int_property("rocksdb.num-running-flushes")
            .unwrap_or_default();
        info!(
            "background db status: compactions={}, flushes={}",
            n_compactions, n_flushes
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
            if n_compactions + n_flushes <= 1 {
                break;
            }
            info!(
                "⏳awaiting background jobs, compactions={}, flushes={}",
                n_compactions, n_flushes
            );
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
        info!("cancal background work ...");
        self.db.cancel_background_work(/* wait: */ true);
        info!("syncing WAL ... {:?}", self.db.sync_wal());
        // eprintln!("Close DB ... {:?}", self.db.close());
    }
}
