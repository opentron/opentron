use sha2::{Digest, Sha256};
use std::mem;
use types::H256;

pub enum HashedSha256Hasher {}

impl ::merkle_tree::MerkleHasher for HashedSha256Hasher {
    // FIXME: don't know how to impl for &Transaction
    type Input = H256;

    fn hash(input: &H256) -> H256 {
        /*
        let mut sha256 = Sha256::new();
        sha256.input(&input.write_to_bytes().unwrap());
        unsafe { mem::transmute(sha256.result()) }
        */
        *input
    }

    fn hash_nodes(left: &H256, right: &H256) -> H256 {
        let result = Sha256::new().chain(left.as_bytes()).chain(right.as_bytes()).finalize();
        unsafe { mem::transmute(result) }
    }
}

pub type MerkleTree = ::merkle_tree::MerkleTree<HashedSha256Hasher>;
