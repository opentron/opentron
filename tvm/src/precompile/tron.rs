use types::{H160, H256, U256};
use secp256k1::{Message, RecoveryId, Signature};
use sha3::{Digest, Keccak256};
use std::convert::TryInto;

use super::helper::AbiArgIterator;
use crate::backend::Backend;

pub fn ecrecover(input: &[u8]) -> Option<H256> {
    let v: u8 = U256::from_big_endian(&input[32..64]).try_into().ok()?;

    let msg = Message::parse_slice(&input[0..32]).ok()?;
    let sig = Signature::parse_slice(&input[64..128]).ok()?;
    // TRON: rec_id fix is same as EVM
    let rec_id = RecoveryId::parse(v.wrapping_sub(27)).ok()?;

    let pub_key = secp256k1::recover(&msg, &sig, &rec_id).ok()?;
    let raw_pub_key = pub_key.serialize();

    let mut hasher = Keccak256::new();
    hasher.update(&raw_pub_key[1..]); // skip [0], type byte
    let digest = hasher.finalize();

    let mut ret = H256::zero();
    ret.as_bytes_mut()[12..32].copy_from_slice(&digest[digest.len() - 20..]);
    Some(ret)
}

// [u8; 32], [u8; 65] => [u8; 20]
fn recover_addr(message: &[u8], signature: &[u8]) -> Option<H160> {
    let msg = Message::parse_slice(message).ok()?;
    let sig = Signature::parse_slice(&signature[..64]).ok()?;
    // NOTE: no wrapping_sub
    let rec_id = RecoveryId::parse(signature[64]).ok()?;

    let pub_key = secp256k1::recover(&msg, &sig, &rec_id).ok()?;
    let raw_pub_key = pub_key.serialize();

    let mut hasher = Keccak256::new();
    hasher.update(&raw_pub_key[1..]); // skip [0], type byte
    let digest = hasher.finalize();

    let mut ret = H256::zero();
    ret.as_bytes_mut()[12..32].copy_from_slice(&digest[digest.len() - 20..]);
    Some(ret.into())
}

/// batchvalidatesign(bytes32 hash, bytes[] signatures, address[] addresses) returns (bytes32)
pub fn batchvalidatesign(input: &[u8]) -> Option<Vec<u8>> {
    const MAX_NUM_OF_SIGNATURES: usize = 16;
    let mut it = AbiArgIterator::new(input);

    let hash = it.next_byte32()?;
    let sigs = it.next_array_of_bytes()?;
    let addrs = it.next_array_of_byte32()?;

    if sigs.len() != addrs.len() || sigs.is_empty() || sigs.len() > MAX_NUM_OF_SIGNATURES {
        return None;
    }

    let mut ret = vec![0u8; 32];
    for i in 0..sigs.len() {
        if let Some(addr) = recover_addr(hash, sigs[i]) {
            if addr == H256::from_slice(addrs[i]).into() {
                ret[i] = 1;
            }
        }
    }

    Some(ret)
}

/// validatemultisign(address addr, uint256 permissionId, bytes32 hash, bytes[] signatures) returns (bool)
pub fn validatemultisign(input: &[u8], backend: &dyn Backend) -> Option<bool> {
    let mut it = AbiArgIterator::new(input);

    let addr = it.next_h160()?;
    let perm_id = it.next_u256()?;
    let hash = it.next_h256()?;
    let sigs = it.next_array_of_bytes()?;

    Some(backend.validate_multisig(addr, perm_id, hash, &sigs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batchvalidatesign() {
        let raw = hex::decode("a166ceae7066e25689f134a16f08d82911363e16d4911ca3a0c23159ff92aaf0000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001c00000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000413f0449db639f3993d075dca4b0c0adfcc214c4a55a268a3c4c0617e822ed38bb29ef0035547e28cee2c35bd79642cdbb66ecc5594e5089cd858f232a0f957663000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000413f0449db639f3993d075dca4b0c0adfcc214c4a55a268a3c4c0617e822ed38bb29ef0035547e28cee2c35bd79642cdbb66ecc5594e5089cd858f232a0f9576630000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000003d14645130f22f0b3b03f6966fdc3c7e3322f070000000000000000000000415cbdd86a2fa8dc4bddd8a8f69dba48572eec07fb").unwrap();
        let ret = batchvalidatesign(&raw).unwrap();
        assert_eq!(ret[0], 0);
        assert_eq!(ret[1], 1);
    }
}
