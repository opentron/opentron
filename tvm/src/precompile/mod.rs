//! The precompiles.

use crate::backend::Backend;
use crate::{ExitError, ExitSucceed};

use digest::Digest;
use num_bigint::BigUint;
use num_traits::Zero;
use primitive_types::{H160, H256, U256};
use sha2::Sha256;
use std::convert::TryFrom;
use ztron::precompiles::{pedersen_hash, verify_burn_proof, verify_mint_proof, verify_transfer_proof};

mod alt_bn128;
pub mod helper;
mod tron;

const WORD_SIZE: usize = 32;

// 0000000000000000000000000000000000000000000000000000000000000001
// ecrecover(bytes32 hash, uint8 v, bytes32 r, bytes32 s) returns (address)
// 0000000000000000000000000000000000000000000000000000000000000002
// sha256(...) returns (bytes32)
// 0000000000000000000000000000000000000000000000000000000000000003
// ripemd160(...) returns (bytes20)
// 0000000000000000000000000000000000000000000000000000000000000004
// identity(...) returns (...)
// The Identity function simply returns whatever its input is.
// 0000000000000000000000000000000000000000000000000000000000000005
// modexp: modular exponentiation on big numbers
// 0000000000000000000000000000000000000000000000000000000000000006
// altBN128Add: alt_bn128 Addition
// 0000000000000000000000000000000000000000000000000000000000000007
// altBN128Mul: alt_bn128 Scalar Multiplication
// 0000000000000000000000000000000000000000000000000000000000000008
// altBN128Pairing: pairing check
// TRON 3.6 update
// 0000000000000000000000000000000000000000000000000000000000000009
// batchvalidatesign(bytes32 hash, bytes[] signatures, address[] addresses) returns (bytes32)
// 000000000000000000000000000000000000000000000000000000000000000a
// validatemultisign(address addr, uint256 permissionId, bytes32 hash, bytes[] signatures) returns (bool)
// TRON 4.0 update: shielded contracts, implemented in ztron.
// 0000000000000000000000000000000000000000000000000000000001000001 - verifyMintProof
// 0000000000000000000000000000000000000000000000000000000001000002 - verifyTransferProof
// 0000000000000000000000000000000000000000000000000000000001000003 - verifyBurnProof
// 0000000000000000000000000000000000000000000000000000000001000004 - pedersenHash
pub fn tron_precompile(
    address: H160,
    input: &[u8],
    _target_gas: Option<usize>,
    backend: &dyn Backend,
) -> Option<Result<(ExitSucceed, Vec<u8>, usize), ExitError>> {
    if address > H160::from_low_u64_be(0xffffffff) {
        return None;
    }
    match address.to_low_u64_be() {
        0x1 => {
            const COST: usize = 3000;
            let ret = tron::ecrecover(input).unwrap_or_default();
            Some(Ok((ExitSucceed::Returned, ret.as_bytes().to_vec(), COST)))
        }
        0x2 => {
            const COST: usize = 60;
            let cost = COST + 12 * ((input.len() + 31) / 32);

            let mut hasher = Sha256::new();
            hasher.update(input);
            let ret = hasher.finalize().to_vec();

            Some(Ok((ExitSucceed::Returned, ret, cost)))
        }
        0x3 => {
            const COST: usize = 600;
            let cost = COST + 120 * ((input.len() + 31) / 32);

            let mut hasher = Sha256::new();
            hasher.update(input);
            let orig = hasher.finalize().to_vec();

            let mut hasher = Sha256::new();
            hasher.update(&orig[..20]);
            let ret = hasher.finalize().to_vec();

            Some(Ok((ExitSucceed::Returned, ret, cost)))
        }

        0x4 => {
            const COST: usize = 15;
            let cost = COST + 3 * ((input.len() + 31) / 32);
            Some(Ok((ExitSucceed::Returned, input.to_vec(), cost)))
        }
        0x5 => {
            let words: Vec<_> = input.chunks(32).take(3).collect();

            let base_len = i32::try_from(U256::from_big_endian(&words[0])).unwrap() as usize;
            let exp_len = i32::try_from(U256::from_big_endian(&words[1])).unwrap() as usize;
            let modulus_len = i32::try_from(U256::from_big_endian(&words[2])).unwrap() as usize;

            let mut offset = 32 * 3;
            let base = BigUint::from_bytes_be(&input[offset..offset + base_len]);
            offset += base_len;

            let exp = BigUint::from_bytes_be(&input[offset..offset + exp_len]);
            offset += exp_len;

            let modulus = BigUint::from_bytes_be(&input[offset..offset + modulus_len]);

            let max_len = base_len.max(modulus_len);
            let mul_complexity = if max_len <= 64 {
                max_len.pow(2)
            } else if max_len <= 1024 {
                max_len.pow(2) / 4 + 96 * max_len - 3072
            } else {
                max_len.pow(2) / 16 + 480 * max_len - 199680
            };
            let adj_exp_len = exp.bits() as usize;
            let cost = mul_complexity * adj_exp_len.max(1) / 20;

            if modulus == BigUint::zero() {
                return Some(Ok((ExitSucceed::Returned, vec![], cost)));
            }

            let ret = base.modpow(&exp, &modulus).to_bytes_be();
            let ret_with_leading_zeros = if ret.len() < modulus_len {
                let mut fixed = vec![0u8; modulus_len - ret.len()];
                fixed.extend_from_slice(&ret);
                fixed
            } else {
                ret
            };

            Some(Ok((ExitSucceed::Returned, ret_with_leading_zeros, cost)))
        }
        0x6 => {
            const COST: usize = 500;

            let ret = alt_bn128::ecadd(input).unwrap_or_default();
            Some(Ok((ExitSucceed::Returned, ret, COST)))
        }
        0x7 => {
            const COST: usize = 40000;

            let ret = alt_bn128::ecmul(input).unwrap_or_default();
            Some(Ok((ExitSucceed::Returned, ret, COST)))
        }
        0x8 => {
            const COST: usize = 100000;
            const PAIR_SIZE: usize = 192;

            let cost = COST + 80000 * (input.len() / PAIR_SIZE);
            let ret = alt_bn128::ecpairing(input).unwrap_or_default();

            Some(Ok((ExitSucceed::Returned, ret, cost)))
        }
        0x9 => {
            const COST_PER_SIGN: usize = 1500;
            let cost = COST_PER_SIGN * ((input.len() / WORD_SIZE - 5) / 6);

            let ret = tron::batchvalidatesign(input).unwrap_or_default();
            Some(Ok((ExitSucceed::Returned, ret, cost)))
        }
        0xa => {
            const COST_PER_SIGN: usize = 1500;
            let cost = COST_PER_SIGN * ((input.len() / WORD_SIZE - 5) / 6);

            let validated = tron::validatemultisign(input, backend).unwrap_or(false);
            let encoded = U256::from(validated as u8);

            let mut ret = vec![0u8; 32];
            encoded.to_big_endian(&mut ret[..]);

            Some(Ok((ExitSucceed::Returned, ret, cost)))
        }
        0x1000001 => {
            // verifymintproof, fixed size input.
            const COST: usize = 150000;
            const SIZE: usize = 1504;

            if input.len() != SIZE {
                Some(Ok((ExitSucceed::Returned, H256::zero().as_bytes().to_owned(), COST)))
            } else {
                let output = verify_mint_proof(input);
                match output {
                    Ok(raw) => {
                        let mut ret = Vec::with_capacity(raw.len() + 32);
                        ret.extend_from_slice(H256::from_low_u64_be(1).as_bytes());
                        ret.extend_from_slice(&raw);
                        Some(Ok((ExitSucceed::Returned, ret, COST)))
                    }
                    Err(e) => {
                        eprintln!("verifymintproof error: {:?}", e);
                        Some(Ok((ExitSucceed::Returned, H256::zero().as_bytes().to_owned(), COST)))
                    }
                }
            }
        }
        0x1000002 => {
            // verifyTransferProof
            const COST: usize = 200000;
            const SIZES: [usize; 4] = [2080, 2368, 2464, 2752];

            if !SIZES.contains(&input.len()) {
                eprintln!("verifytransferproof input size mismatch, len={}", input.len());
                Some(Ok((ExitSucceed::Returned, H256::zero().as_bytes().to_owned(), COST)))
            } else {
                let output = verify_transfer_proof(input);
                match output {
                    Ok(raw) => {
                        println!("output => {}", hex::encode(&raw));
                        let mut ret = Vec::with_capacity(raw.len() + 32);
                        ret.extend_from_slice(H256::from_low_u64_be(1).as_bytes());
                        ret.extend_from_slice(&raw);
                        Some(Ok((ExitSucceed::Returned, ret, COST)))
                    }
                    Err(e) => {
                        eprintln!("verifytransferproof error: {:?}", e);
                        Some(Ok((ExitSucceed::Returned, H256::zero().as_bytes().to_owned(), COST)))
                    }
                }
            }
        }
        0x1000003 => {
            // verifyBurnProof
            const COST: usize = 150000;
            const SIZE: usize = 512;

            if input.len() != SIZE {
                Some(Ok((ExitSucceed::Returned, H256::zero().as_bytes().to_owned(), COST)))
            } else {
                let output = verify_burn_proof(input);
                match output {
                    Ok(_) => Some(Ok((
                        ExitSucceed::Returned,
                        H256::from_low_u64_be(1).as_bytes().to_owned(),
                        COST,
                    ))),
                    Err(e) => {
                        eprintln!("verifyburnproof error: {:?}", e);
                        Some(Ok((ExitSucceed::Returned, H256::zero().as_bytes().to_owned(), COST)))
                    }
                }
            }
        }
        0x1000004 => {
            // pedersenHash, aka. merkleHash in java-tron
            const COST: usize = 500;
            let ret = pedersen_hash(input);
            // On error, return an empty array.
            Some(Ok((ExitSucceed::Returned, ret.unwrap_or_default(), COST)))
        }
        _ => None,
    }
}
