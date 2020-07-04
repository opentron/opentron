use digest::Digest;
use primitive_types::H256;
use sha2::Sha256;
use sha3::Keccak256;

#[inline]
pub fn sha256(input: &[u8]) -> H256 {
    let mut hasher = Sha256::new();
    hasher.input(input);
    let inner: [u8; 32] = hasher.result().into();
    H256::from(inner)
}

#[inline]
pub fn keccak256(input: &[u8]) -> H256 {
    let mut hasher = Keccak256::new();
    hasher.input(input);
    let inner: [u8; 32] = hasher.result().into();
    H256::from(inner)
}
