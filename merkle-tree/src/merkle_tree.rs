use crate::tree::{LeavesIntoIterator, LeavesIterator, Tree};
use primitive_types::H256;
use crate::MerkleHasher;

/// A Merkle tree is a binary tree, with values of type `T` at the leafs,
/// and where every internal node holds the hash of the concatenation of the hashes of its children nodes.
#[derive(Clone, Debug)]
pub struct MerkleTree<H: MerkleHasher> {
    /// The root of the inner binary tree
    root: Tree<H::Input>,

    /// The height of the tree
    height: usize,

    /// The number of leaf nodes in the tree
    count: usize,
}

impl<H: MerkleHasher> MerkleTree<H> {
    /// Constructs a Merkle Tree from a vector of data blocks.
    /// Returns `None` if `values` is empty.
    pub fn from_vec(values: Vec<H::Input>) -> Self {
        if values.is_empty() {
            return MerkleTree {
                root: Tree::empty(H::hash_empty()),
                height: 0,
                count: 0,
            };
        }

        let count = values.len();
        let mut height = 0;
        let mut cur = Vec::with_capacity(count);

        for v in values {
            let hash = H::hash(&v);
            let leaf = Tree::new(hash, v);
            cur.push(leaf);
        }

        while cur.len() > 1 {
            let mut next = Vec::new();
            while !cur.is_empty() {
                if cur.len() == 1 {
                    next.push(cur.remove(0));
                } else {
                    let left = cur.remove(0);
                    let right = cur.remove(0);

                    let combined_hash = H::hash_nodes(left.hash(), right.hash());

                    let node = Tree::Node {
                        hash: combined_hash,
                        left: Box::new(left),
                        right: Box::new(right),
                    };

                    next.push(node);
                }
            }

            height += 1;

            cur = next;
        }

        debug_assert!(cur.len() == 1);

        let root = cur.remove(0);

        MerkleTree {
            root: root,
            height: height,
            count: count,
        }
    }

    /// Returns the root hash of Merkle tree
    pub fn root_hash(&self) -> &H256 {
        self.root.hash()
    }

    /// Returns the height of Merkle tree
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns the number of leaves in the Merkle tree
    pub fn count(&self) -> usize {
        self.count
    }

    /// Returns whether the Merkle tree is empty or not
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Creates an `Iterator` over the values contained in this Merkle tree.
    pub fn iter(&self) -> LeavesIterator<H::Input> {
        self.root.iter()
    }
}

impl<H: MerkleHasher> IntoIterator for MerkleTree<H> {
    type Item = H::Input;
    type IntoIter = LeavesIntoIterator<H::Input>;

    /// Creates a consuming iterator, that is, one that moves each value out of the Merkle tree.
    /// The tree cannot be used after calling this.
    fn into_iter(self) -> Self::IntoIter {
        self.root.into_iter()
    }
}

impl<'a, H: MerkleHasher> IntoIterator for &'a MerkleTree<H> {
    type Item = &'a H::Input;
    type IntoIter = LeavesIterator<'a, H::Input>;

    /// Creates a borrowing `Iterator` over the values contained in this Merkle tree.
    fn into_iter(self) -> Self::IntoIter {
        self.root.iter()
    }
}
