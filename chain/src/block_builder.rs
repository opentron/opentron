use crate::{BlockHeader, IndexedBlock, Transaction};
use keys::{Address, KeyPair};
use primitive_types::H256;
use prost::Message;

pub struct BlockBuilder {
    header: BlockHeader,
    txns: Vec<Transaction>,
}

impl BlockBuilder {
    pub fn new(number: i64) -> Self {
        let mut header = BlockHeader::default();
        header.raw_data = Some(Default::default());
        header.raw_data.as_mut().map(|h| {
            h.number = number;
        });
        BlockBuilder { header, txns: vec![] }
    }

    pub fn timestamp(mut self, timestamp: i64) -> Self {
        self.header.raw_data.as_mut().map(|h| h.timestamp = timestamp);
        self
    }

    pub fn version(mut self, version: i32) -> Self {
        self.header.raw_data.as_mut().map(|h| h.version = version);
        self
    }

    pub fn parent_hash(mut self, hash: &H256) -> Self {
        self.header
            .raw_data
            .as_mut()
            .map(|h| h.parent_hash = hash.as_bytes().into());
        self
    }

    pub fn push_transaction(mut self, txn: Transaction) -> Self {
        self.txns.push(txn);
        self
    }

    pub fn build(mut self, witness: &Address, keypair: &KeyPair) -> Option<IndexedBlock> {
        self.header
            .raw_data
            .as_mut()
            .map(|h| h.witness_address = witness.as_bytes().into());

        let block = IndexedBlock::from_raw_header_and_txns(self.header, self.txns);

        block.and_then(|mut b| {
            let mut buf = Vec::with_capacity(255);
            b.header.raw.raw_data.as_ref().unwrap().encode(&mut buf).unwrap();
            let signature = keypair.private().sign(&buf).ok()?;
            b.header.raw.witness_signature = signature.as_bytes().into();
            Some(b)
        })
    }
}
