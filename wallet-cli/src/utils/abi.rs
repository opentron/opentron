

use crate::utils::crypto;


#[inline]
pub fn fnhash(fname: &str) -> [u8; 4] {
    let mut hash_code = [0u8; 4];
    (&mut hash_code[..]).copy_from_slice(&crypto::keccak256(fname.as_bytes())[..4]);
    hash_code
}
