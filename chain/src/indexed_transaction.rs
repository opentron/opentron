use crypto::sha256;
use primitives::H256;
use prost::Message;
use std::cmp;

use proto2::chain::Transaction;

#[derive(Default, Clone, Debug)]
pub struct IndexedTransaction {
    pub hash: H256,
    pub raw: Transaction,
}

impl IndexedTransaction {
    pub fn new(hash: H256, transaction: Transaction) -> Self {
        IndexedTransaction {
            hash: hash,
            raw: transaction,
        }
    }

    /// Explicit conversion of the raw Transaction into IndexedTransaction.
    ///
    /// Hashes transaction contents.
    pub fn from_raw<T>(transaction: T) -> Self
    where
        Transaction: From<T>,
    {
        let transaction = Transaction::from(transaction);
        Self::new(get_transaction_hash(&transaction), transaction)
    }
}

impl cmp::PartialEq for IndexedTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

fn get_transaction_hash(transaction: &Transaction) -> H256 {
    let mut buf = Vec::with_capacity(255);
    transaction.encode(&mut buf).unwrap(); // won't fail?
    sha256(&buf)
}
