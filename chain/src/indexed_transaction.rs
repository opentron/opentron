use crypto::sha256;
use primitives::H256;
use prost::Message;
use std::cmp;
use std::hash::{Hash, Hasher};

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

impl cmp::Eq for IndexedTransaction {}

impl Hash for IndexedTransaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

fn get_transaction_hash(transaction: &Transaction) -> H256 {
    let mut buf = Vec::with_capacity(255);
    transaction.raw_data.as_ref().unwrap().encode(&mut buf).unwrap(); // won't fail?
    sha256(&buf)
}
