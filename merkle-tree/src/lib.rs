mod merkle_tree;
mod tree;
use types::H256;

pub use crate::merkle_tree::MerkleTree;

/// A hashable type
pub trait MerkleHasher {
    type Input;
    // type Output;
    fn hash(input: &Self::Input) -> H256;

    fn hash_nodes(left: &H256, right: &H256) -> H256;

    fn hash_empty() -> H256 {
        H256::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::mem;

    pub struct BytesSha256Hasher;

    impl MerkleHasher for BytesSha256Hasher {
        type Input = Vec<u8>;

        fn hash(input: &Self::Input) -> H256 {
            let mut sha256 = Sha256::new();
            sha256.update(input);
            unsafe { mem::transmute(sha256.finalize()) }
        }

        fn hash_nodes(left: &H256, right: &H256) -> H256 {
            let result = Sha256::new().chain(left.as_bytes()).chain(right.as_bytes()).finalize();
            unsafe { mem::transmute(result) }
        }
    }

    #[test]
    fn empty_tree() {
        let list: Vec<Vec<u8>> = vec![];
        let tree: MerkleTree<BytesSha256Hasher> = MerkleTree::from_vec(list);
        assert_eq!(&H256::zero(), tree.root_hash());
    }

    #[test]
    fn tree_with_one_node() {
        let list: Vec<Vec<u8>> = vec![b"\x00\x00\x00\x00".to_vec()];
        let tree: MerkleTree<BytesSha256Hasher> = MerkleTree::from_vec(list);
        // sha256 of "0x00_00_00_00"
        assert_eq!(
            &"df3f619804a92fdb4057192dc43dd748ea778adc52bc498ce80524c014b81119"
                .parse::<H256>()
                .unwrap(),
            tree.root_hash()
        );
    }

    #[test]
    fn tree_with_two_nodes() {
        let list: Vec<Vec<u8>> = vec![b"\x00\x00\x00\x00".to_vec(), b"\x00\x00\x00\x01".to_vec()];
        let tree: MerkleTree<BytesSha256Hasher> = MerkleTree::from_vec(list);
        // hashlib.sha256(
        //   bytes.fromhex(
        //     'df3f619804a92fdb4057192dc43dd748ea778adc52bc498ce80524c014b81119' + # 00_00_00_00
        //     'b40711a88c7039756fb8a73827eabe2c0fe5a0346ca7e0a104adc0fc764f528d'   # 00_00_00_01
        // )).hexdigest()
        assert_eq!(
            &"430ebda8b2441cf6a796f7f2a9b3377ae2fc8b23fe022fc018bed864b0fa1815"
                .parse::<H256>()
                .unwrap(),
            tree.root_hash()
        );
    }

    #[test]
    fn tree_with_three_nodes() {
        let list: Vec<Vec<u8>> = vec![
            b"\x00\x00\x00\x00".to_vec(),
            b"\x00\x00\x00\x01".to_vec(),
            b"\x00\x00\x00\x02".to_vec(),
        ];
        let tree: MerkleTree<BytesSha256Hasher> = MerkleTree::from_vec(list);
        // hashlib.sha256(
        //   bytes.fromhex(
        //     '430ebda8b2441cf6a796f7f2a9b3377ae2fc8b23fe022fc018bed864b0fa1815' + # above
        //     '433ebf5bc03dffa38536673207a21281612cef5faa9bc7a4d5b9be2fdb12cf1a'   # 00_00_00_02
        // )).hexdigest()
        assert_eq!(
            &"baab99a32bb15f1d10b9dd6958f98a729e8d237207b7d8b9e7789e382834d1eb"
                .parse::<H256>()
                .unwrap(),
            tree.root_hash()
        );
    }
}
