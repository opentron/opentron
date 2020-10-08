use std::cmp;
use std::collections::HashMap;
use std::convert::TryFrom;

use byteorder::{ByteOrder, BE};
use crypto::sha256;
use keys::{Address, Public, Signature};
use primitive_types::H256;
use prost::Message;
use proto2::chain::{Block, BlockHeader, Transaction};
use proto2::common::BlockId;
use rayon::prelude::*;

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

impl std::convert::From<IndexedBlock> for Block {
    fn from(block: IndexedBlock) -> Block {
        block.into_raw_block()
    }
}

impl IndexedBlock {
    pub fn new(header: IndexedBlockHeader, transactions: Vec<IndexedTransaction>) -> Self {
        IndexedBlock {
            header: header,
            transactions: transactions,
        }
    }

    pub fn from_raw_header_and_txns(header: BlockHeader, txns: Vec<Transaction>) -> Option<Self> {
        Self::from_raw(Block {
            block_header: Some(header),
            transactions: txns,
        })
    }

    /// Explicit conversion of the raw Block into IndexedBlock.
    ///
    /// Hashes block header + transactions.
    pub fn from_raw(block: Block) -> Option<Self> {
        let Block {
            block_header,
            transactions,
        } = block;
        // Only compute in parallel if there is enough work to benefit it
        let transactions = if transactions.len() > 200 {
            transactions
                .into_par_iter()
                .map(IndexedTransaction::from_raw)
                .collect::<Option<Vec<_>>>()?
        } else {
            transactions
                .into_iter()
                .map(IndexedTransaction::from_raw)
                .collect::<Option<Vec<_>>>()?
        };
        let mut block_header = block_header?;
        if block_header.raw_data.as_ref()?.merkle_root_hash.is_empty() {
            block_header
                .raw_data
                .as_mut()
                .map(|raw| raw.merkle_root_hash = merkle_root(&transactions).as_bytes().to_owned());
        }
        IndexedBlockHeader::from_raw(block_header).map(|header| Self::new(header, transactions))
    }

    pub fn hash(&self) -> &H256 {
        &self.header.hash
    }

    pub fn number(&self) -> i64 {
        BE::read_u64(&self.header.hash.as_bytes()[..8]) as i64
    }

    pub fn block_id(&self) -> BlockId {
        BlockId {
            number: self.number(),
            hash: self.hash().as_bytes().to_vec(),
        }
    }

    pub fn witness(&self) -> &[u8] {
        &self.header.raw.raw_data.as_ref().unwrap().witness_address
    }

    /// Recover witness from block signature.
    pub fn recover_witness(&self) -> Result<Address, keys::Error> {
        let mut buf = Vec::with_capacity(255);
        self.header.raw.raw_data.as_ref().unwrap().encode(&mut buf).unwrap();
        let sig = Signature::try_from(&self.header.raw.witness_signature)?;

        Ok(Address::from_public(&Public::recover(&buf, &sig)?))
    }

    pub fn timestamp(&self) -> i64 {
        self.header.raw.raw_data.as_ref().unwrap().timestamp
    }

    pub fn parent_hash(&self) -> &[u8] {
        &self.header.raw.raw_data.as_ref().unwrap().parent_hash
    }

    pub fn version(&self) -> i32 {
        self.header.raw.raw_data.as_ref().unwrap().version
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

    pub fn verify_merkle_root_hash(&self) -> bool {
        self.merkle_root_hash() == merkle_root(&self.transactions).as_bytes()
    }

    pub fn verify_merkle_root_hash_with_patch(&self, patch: &HashMap<H256, H256>) -> bool {
        let node_hashes = self
            .transactions
            .iter()
            .map(|txn| {
                patch
                    .get(&txn.hash)
                    .cloned()
                    .or_else(|| Some(get_transaction_hash_for_merkle_tree(&txn.raw)))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let tree = MerkleTree::from_vec(node_hashes);
        self.merkle_root_hash() == tree.root_hash().as_bytes()
    }
}

fn merkle_root(transactions: &[IndexedTransaction]) -> H256 {
    let hashes = transactions
        .iter()
        .map(|txn| get_transaction_hash_for_merkle_tree(&txn.raw))
        .collect::<Vec<_>>();
    // println!("hashes => {:#?}", hashes);
    let tree = MerkleTree::from_vec(hashes);
    *tree.root_hash()
}

fn get_transaction_hash_for_merkle_tree(transaction: &Transaction) -> H256 {
    let mut buf = Vec::with_capacity(255);
    // won't fail?
    transaction.encode(&mut buf).unwrap();
    // println!("raw => {:?}", buf);
    sha256(&buf)
}
