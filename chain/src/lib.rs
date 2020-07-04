pub use primitive_types::H256;
pub use proto2::chain::{Block, BlockHeader, Transaction};

pub use indexed_block::IndexedBlock;
pub use indexed_header::IndexedBlockHeader;
pub use indexed_transaction::IndexedTransaction;

mod indexed_block;
mod indexed_header;
mod indexed_transaction;
mod merkle_root;
