use byteorder::{ByteOrder, BE};
use crypto::sha256;
use prost::Message;
use proto::chain::BlockHeader;
use proto::common::BlockId;
use std::cmp;
use types::H256;

#[derive(Clone, Debug)]
pub struct IndexedBlockHeader {
    pub hash: H256,
    pub raw: BlockHeader,
}

impl IndexedBlockHeader {
    pub fn new(hash: H256, header: BlockHeader) -> Self {
        IndexedBlockHeader {
            hash: hash,
            raw: header,
        }
    }

    /// Explicit conversion of the raw BlockHeader into IndexedBlockHeader.
    ///
    /// Hashes the contents of block header.
    pub fn from_raw(header: BlockHeader) -> Option<Self> {
        get_block_header_hash(&header).map(|hash| IndexedBlockHeader::new(hash, header))
    }

    /// Create a dummy block header.
    pub fn dummy(number: i64, timestamp: i64) -> Self {
        let mut hash = H256::zero();
        BE::write_u64(&mut hash.as_bytes_mut()[..8], number as u64);
        let mut inner = BlockHeader {
            raw_data: Some(Default::default()),
            ..Default::default()
        };
        inner.raw_data.as_mut().unwrap().timestamp = timestamp;
        IndexedBlockHeader { hash, raw: inner }
    }

    pub fn number(&self) -> i64 {
        BE::read_u64(&self.hash.as_bytes()[..8]) as i64
    }

    pub fn timestamp(&self) -> i64 {
        self.raw.raw_data.as_ref().unwrap().timestamp
    }

    pub fn version(&self) -> i32 {
        self.raw.raw_data.as_ref().unwrap().version
    }

    pub fn parent_hash(&self) -> &[u8] {
        &self.raw.raw_data.as_ref().unwrap().parent_hash
    }

    pub fn merkle_root_hash(&self) -> &[u8] {
        &self.raw.raw_data.as_ref().unwrap().merkle_root_hash
    }

    pub fn witness(&self) -> &[u8] {
        &self.raw.raw_data.as_ref().unwrap().witness_address
    }

    pub fn block_id(&self) -> BlockId {
        BlockId {
            number: self.number(),
            hash: self.hash.as_bytes().to_vec(),
        }
    }

    pub fn verify(&self) -> bool {
        get_block_header_hash(&self.raw)
            .map(|hash| hash == self.hash)
            .unwrap_or(false)
    }
}

impl cmp::PartialEq for IndexedBlockHeader {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

fn get_block_header_hash(header: &BlockHeader) -> Option<H256> {
    let raw_header = header.raw_data.as_ref()?;
    let block_numer = raw_header.number;

    let mut buf: Vec<u8> = Vec::with_capacity(255);
    raw_header.encode(&mut buf).ok()?; // encode failure?

    let mut block_hash = sha256(&buf);
    BE::write_i64(&mut block_hash[..8], block_numer);
    Some(block_hash)
}
