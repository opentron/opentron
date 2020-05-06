use byteorder::{ByteOrder, BE};
use crypto::sha256;
use primitives::H256;
use prost::Message;
use proto2::chain::{Block, BlockHeader, Transaction};
use std::cmp;

use crate::merkle_root::MerkleTree;
use crate::{IndexedBlockHeader, IndexedTransaction};

#[derive(Debug, Clone)]
pub struct IndexedBlock {
    pub header: IndexedBlockHeader,
    pub transactions: Vec<IndexedTransaction>,
}

impl cmp::PartialEq for IndexedBlock {
    fn eq(&self, other: &Self) -> bool {
        self.header.hash == other.header.hash
    }
}

impl IndexedBlock {
    pub fn new(header: IndexedBlockHeader, transactions: Vec<IndexedTransaction>) -> Self {
        IndexedBlock {
            header: header,
            transactions: transactions,
        }
    }

    pub fn from_header_and_txns(header: BlockHeader, txns: Vec<Transaction>) -> Self {
        Self::from_raw(Block {
            block_header: Some(header),
            transactions: txns,
        })
    }

    /// Explicit conversion of the raw Block into IndexedBlock.
    ///
    /// Hashes block header + transactions.
    pub fn from_raw(block: Block) -> Self {
        let Block {
            block_header,
            transactions,
        } = block;
        let transactions = transactions
            .into_iter()
            .map(IndexedTransaction::from_raw)
            .collect::<Vec<_>>();
        let mut block_header = block_header.unwrap();
        if block_header.raw_data.as_ref().unwrap().merkle_root_hash.is_empty() {
            block_header
                .raw_data
                .as_mut()
                .map(|raw| raw.merkle_root_hash = merkle_root(&transactions).as_bytes().to_owned());
        }
        Self::new(IndexedBlockHeader::from_raw(block_header), transactions)
    }

    pub fn hash(&self) -> &H256 {
        &self.header.hash
    }

    pub fn number(&self) -> u64 {
        BE::read_u64(&self.header.hash.as_bytes()[..8])
    }

    pub fn into_raw_block(self) -> Block {
        Block {
            block_header: Some(self.header.raw),
            transactions: self.transactions.into_iter().map(|tx| tx.raw).collect(),
        }
    }

    pub fn size(&self) -> usize {
        self.clone().into_raw_block().encoded_len()
    }

    pub fn merkle_root_hash(&self) -> &[u8] {
        &self.header.raw.raw_data.as_ref().unwrap().merkle_root_hash
    }
}

fn merkle_root(transactions: &[IndexedTransaction]) -> H256 {
    let hashes = transactions
        .iter()
        .map(|txn| get_transaction_hash_for_merkle_root(&txn.raw))
        .collect::<Vec<_>>();
    let tree = MerkleTree::from_vec(hashes);
    *tree.root_hash()
}

fn get_transaction_hash_for_merkle_root(transaction: &Transaction) -> H256 {
    let mut buf = Vec::with_capacity(255);
    transaction.encode(&mut buf).unwrap(); // won't fail?
    sha256(&buf)
}
