use std::cmp;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

use crypto::sha256;
use keys::{Address, Public, Signature};
use primitive_types::H256;
use prost::Message;
use proto::chain::Transaction;

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
    pub fn from_raw<T>(transaction: T) -> Option<Self>
    where
        Transaction: From<T>,
    {
        let transaction = Transaction::from(transaction);
        get_transaction_hash(&transaction).map(|hash| Self::new(hash, transaction))
    }

    /// Recover owner address.
    pub fn recover_owner(&self) -> Result<Vec<Address>, keys::Error> {
        let mut buf = Vec::with_capacity(255);
        self.raw.raw_data.as_ref().unwrap().encode(&mut buf).unwrap();

        self.raw
            .signatures
            .iter()
            .map(|raw_sig| {
                Signature::try_from(raw_sig)
                    .and_then(|sig| Public::recover(&buf, &sig))
                    .map(|pk| Address::from_public(&pk))
            })
            .collect()
    }

    pub fn expiration(&self) -> i64 {
        self.raw.raw_data.as_ref().unwrap().expiration
    }

    pub fn verify(&self) -> bool {
        get_transaction_hash(&self.raw)
            .map(|hash| hash == self.hash)
            .unwrap_or(false)
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

fn get_transaction_hash(transaction: &Transaction) -> Option<H256> {
    let mut buf = Vec::with_capacity(255);
    transaction.raw_data.as_ref()?.encode(&mut buf).ok()?; // won't fail?
    Some(sha256(&buf))
}
