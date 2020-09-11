//! Tron 4.0 update, shielded contract support.
// 0000000000000000000000000000000000000000000000000000000001000001 - verifyMintProof
// 0000000000000000000000000000000000000000000000000000000001000002 - verifyTransferProof
// 0000000000000000000000000000000000000000000000000000000001000003 - verifyBurnProof
// 0000000000000000000000000000000000000000000000000000000001000004 - pedersenHash

use bellman::groth16::{Parameters, PreparedVerifyingKey, Proof};
use ff::PrimeField;
use lazy_static::lazy_static;
use pairing::bls12_381::{Bls12, Fr, FrRepr};
use primitive_types::U256;
use zcash_primitives::jubjub::edwards;
use zcash_primitives::jubjub::Unknown;
use zcash_primitives::merkle_tree::Hashable;
use zcash_primitives::redjubjub::{PublicKey, Signature};
use zcash_primitives::sapling::{merkle_hash, Node};
use zcash_primitives::transaction::components::Amount;
use zcash_primitives::JUBJUB;
use zcash_proofs::load_parameters;
use zcash_proofs::sapling::SaplingVerificationContext;

use self::helper::AbiArgIterator;

pub mod helper;

#[allow(dead_code)]
struct SaplingParameters {
    spend_vk: PreparedVerifyingKey<Bls12>,
    output_vk: PreparedVerifyingKey<Bls12>,
    spend_params: Parameters<Bls12>,
    output_params: Parameters<Bls12>,
}

lazy_static! {
    static ref SAPLING_PARAMETERS: SaplingParameters = {
        use std::path::Path;

        eprintln!("loading sapling parameters ...");

        lazy_static::initialize(&JUBJUB);

        let spend_path = "./ztron-params/sapling-spend.params";
        let output_path = "./ztron-params/sapling-output.params";

        let (spend_params, spend_vk, output_params, output_vk, _) =
            load_parameters(Path::new(spend_path), Path::new(output_path), None);

        SaplingParameters {
            spend_vk,
            output_vk,
            spend_params,
            output_params,
        }
    };
}

/// Error while running a precompile.
#[derive(Debug)]
pub enum Error {
    /// ABI decode error, invalid input.
    AbiDecode,
    /// Invalid value for librustzcash.
    InvalidValue,
    /// Error while `check_spend`.
    CheckSpend,
    /// Error while `check_output`.
    ChecknOutput,
    /// Error while `final_check`.
    FinalCheck,
    /// Io error.
    Io(std::io::Error),
    /// Customized error message.
    Runtime(&'static str),
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for Error {
    fn from(inner: std::io::Error) -> Self {
        Error::Io(inner)
    }
}

/// Get frontier slot from leaf index, i.e. current leaf count, from 0.
fn get_frontier_slot(index: usize) -> usize {
    let mut slot = 0;
    if index % 2 != 0 {
        let mut exp1 = 1;
        let mut pow1 = 2;
        let mut pow2 = pow1 << 1;
        while slot == 0 {
            if (index + 1 - pow1) % pow2 == 0 {
                slot = exp1;
            } else {
                pow1 = pow2;
                pow2 <<= 1;
                exp1 += 1;
            }
        }
    }
    slot
}

#[inline]
fn le_bytes_to_fr_repr(raw: &[u8]) -> FrRepr {
    let mut f = FrRepr::default();
    f.as_mut().copy_from_slice(raw);
    f
}

#[inline]
fn le_bytes_to_fr(raw: &[u8]) -> Result<Fr, Error> {
    Fr::from_repr(le_bytes_to_fr_repr(raw)).ok_or(Error::InvalidValue)
}

fn insert_leaf_to_merkle_tree(mut frontier: [[u8; 32]; 33], leaf_index: usize, leafs: &[[u8; 32]]) -> Vec<u8> {
    const TREE_WIDTH: usize = 0x100000000;

    let slots: Vec<usize> = (0..leafs.len()).map(|i| get_frontier_slot(leaf_index + i)).collect();
    let mut result = {
        let mut result_len = 32;
        for &i in &slots {
            result_len += (i + 1) * 32;
        }
        vec![0u8; result_len]
    };

    let mut offset = 0;

    let mut node_value = [0u8; 32];
    let mut node_index = 0;

    for (i, (&slot, &leaf_value)) in slots.iter().zip(leafs.iter()).enumerate() {
        node_value = leaf_value;

        if slot != 0 {
            assert!(slot < 0xff);
            let slot_value = U256::from(slot);
            slot_value.to_big_endian(&mut result[offset..offset + 32]);
        }
        offset += 32;

        node_index = i + leaf_index + TREE_WIDTH - 1;
        if slot == 0 {
            frontier[0].copy_from_slice(&node_value);
            continue;
        }

        for level in 1..=slot {
            let (left, right) = if node_index % 2 == 0 {
                let left = le_bytes_to_fr_repr(&frontier[level - 1]);
                let right = le_bytes_to_fr_repr(&node_value);
                node_index = (node_index - 1) / 2;
                (left, right)
            } else {
                let left = le_bytes_to_fr_repr(&node_value);
                let right = Fr::from(Node::empty_root(level - 1)).to_repr();
                node_index = node_index / 2;
                (left, right)
            };

            let hash_value = merkle_hash(level - 1, &left, &right);

            node_value[..].copy_from_slice(hash_value.as_ref());
            result[offset..offset + 32].copy_from_slice(hash_value.as_ref());
            offset += 32;
        }

        frontier[slot].copy_from_slice(&node_value);
    }

    for level in *slots.last().unwrap() + 1..=32 {
        let (left, right) = if node_index % 2 == 0 {
            let left = le_bytes_to_fr_repr(&frontier[level - 1]);
            let right = le_bytes_to_fr_repr(&node_value);
            node_index = (node_index - 1) / 2;
            (left, right)
        } else {
            let left = le_bytes_to_fr_repr(&node_value);
            let right = Fr::from(Node::empty_root(level - 1)).to_repr();
            node_index = node_index / 2;
            (left, right)
        };
        let hash_value = merkle_hash(level - 1, &left, &right);
        node_value[..].copy_from_slice(hash_value.as_ref());
    }
    result[offset..offset + 32].copy_from_slice(&node_value);

    result
}

pub fn verify_mint_proof(data: &[u8]) -> Result<Vec<u8>, Error> {
    // (bytes32 cm, bytes32 cv, bytes32 epk, bytes32[6] proof,
    //  bytes32[2] binding_sig, uint256 value, bytes32 sighash,
    //  bytes32[33] frontier, uint256 leaf_count)
    let mut it = AbiArgIterator::new(data);

    let cm = it.next_byte32()?;
    let cv = it.next_byte32()?;
    let epk = it.next_byte32()?;
    let zkproof = it.next_fixed_words(6)?;

    let binding_sig = it.next_fixed_words(2)?;
    let value = it.next_u256()?;
    let sighash = it.next_byte32_as_array()?;

    let frontier = it.next_fixed_words(33)?;
    // current leaf count
    let leaf_count = it.next_u256()?;

    assert!(it.is_ended());

    // librustzcashSaplingCheckOutput
    let cm = le_bytes_to_fr(cm)?;
    let cv = edwards::Point::<Bls12, Unknown>::read(cv, &JUBJUB)?;
    let epk = edwards::Point::<Bls12, Unknown>::read(epk, &JUBJUB)?;
    let zkproof = Proof::<Bls12>::read(zkproof)?;

    let mut ctx = SaplingVerificationContext::new();

    if !ctx.check_output(cv, cm, epk, zkproof, &SAPLING_PARAMETERS.output_vk, &JUBJUB) {
        return Err(Error::ChecknOutput);
    }

    // librustzcashSaplingFinalCheck
    let binding_sig = Signature::read(binding_sig)?;
    let value_balance = Amount::from_i64(-(value.as_u64() as i64)).map_err(|_| Error::InvalidValue)?;

    if !ctx.final_check(value_balance, &sighash, binding_sig, &JUBJUB) {
        return Err(Error::FinalCheck);
    }

    // insertLeaves
    let frontier = {
        let mut ret = [[0u8; 32]; 33];
        for (buf, val) in ret.iter_mut().zip(frontier.chunks(32)) {
            buf.copy_from_slice(val);
        }
        ret
    };
    let mut leafs = [[0u8; 32]];
    leafs[0].copy_from_slice(cm.to_repr().as_ref());

    Ok(insert_leaf_to_merkle_tree(frontier, leaf_count.as_usize(), &leafs))
}

pub fn verify_transfer_proof(data: &[u8]) -> Result<Vec<u8>, Error> {
    // (bytes32[10][] input, bytes32[2][] spend_auth_sig, bytes32[9][] output,
    //  bytes32[2] binding_sig, bytes32 sighash, uint256 value,
    //  bytes32[33] frontier, uint256 leafCount)
    let mut it = AbiArgIterator::new(data);

    let inputs = it.next_array_of_fixed_words(10)?;
    let spend_auth_sigs = it.next_array_of_fixed_words(2)?;
    if inputs.len() != spend_auth_sigs.len() {
        return Err(Error::Runtime("input and spend_auth_sig are of different length"));
    }

    let outputs = it.next_array_of_fixed_words(9)?;
    let binding_sig = it.next_fixed_words(2)?;
    let sighash = it.next_byte32_as_array()?;
    let value = it.next_u256()?;

    let frontier = it.next_fixed_words(33)?;
    let leaf_count = it.next_u256()?;

    let mut ctx = SaplingVerificationContext::new();

    // check spend - librustzcashSaplingCheckSpendNew
    // input: nf, anchor, cv, rk, proof
    for (&input, &spend_auth_sig) in inputs.iter().zip(spend_auth_sigs.iter()) {
        let mut iit = AbiArgIterator::new(input);

        let nullifier = iit.next_byte32_as_array()?;
        let anchor = iit.next_byte32()?;
        let cv = iit.next_byte32()?;
        let rk = iit.next_byte32()?;
        let zkproof = iit.next_fixed_words(6)?;

        let cv = edwards::Point::<Bls12, Unknown>::read(cv, &JUBJUB)?;
        let anchor = le_bytes_to_fr(anchor)?;

        let rk = PublicKey::<Bls12>::read(rk, &JUBJUB)?;
        let spend_auth_sig = Signature::read(spend_auth_sig)?;

        let zkproof = Proof::<Bls12>::read(zkproof)?;

        if !ctx.check_spend(
            cv,
            anchor,
            &nullifier,
            rk,
            &sighash,
            spend_auth_sig,
            zkproof,
            &SAPLING_PARAMETERS.spend_vk,
            &JUBJUB,
        ) {
            return Err(Error::CheckSpend);
        }
    }

    // check output - librustzcashSaplingCheckOutputNew
    // output: cm, cv, epk, proof
    let mut leafs: Vec<[u8; 32]> = Vec::with_capacity(2);
    for output in outputs {
        let mut oit = AbiArgIterator::new(output);

        let cm = oit.next_byte32()?;
        let cv = oit.next_byte32()?;
        let epk = oit.next_byte32()?;
        let zkproof = oit.next_fixed_words(6)?;

        let cm = le_bytes_to_fr(cm)?;
        let cv = edwards::Point::<Bls12, Unknown>::read(cv, &JUBJUB)?;
        let epk = edwards::Point::<Bls12, Unknown>::read(epk, &JUBJUB)?;
        let zkproof = Proof::<Bls12>::read(zkproof)?;

        if !ctx.check_output(cv, cm, epk, zkproof, &SAPLING_PARAMETERS.output_vk, &JUBJUB) {
            return Err(Error::ChecknOutput);
        }

        leafs.push([0u8; 32]);
        leafs.last_mut().map(|leaf| leaf.copy_from_slice(cm.to_repr().as_ref()));
    }

    // check binding sig - librustzcashSaplingFinalCheckNew
    // normally 0
    let value_balance = Amount::from_i64(value.as_u64() as i64).map_err(|_| Error::InvalidValue)?;
    let binding_sig = Signature::read(binding_sig)?;

    if !ctx.final_check(value_balance, &sighash, binding_sig, &JUBJUB) {
        return Err(Error::FinalCheck);
    }

    // insertLeaves
    let frontier = {
        let mut ret = [[0u8; 32]; 33];
        for (buf, val) in ret.iter_mut().zip(frontier.chunks(32)) {
            buf.copy_from_slice(val);
        }
        ret
    };

    Ok(insert_leaf_to_merkle_tree(frontier, leaf_count.as_usize(), &leafs))
}

pub fn verify_burn_proof(data: &[u8]) -> Result<(), Error> {
    // (bytes32[10] input, bytes32[2] spendAuthoritySignature, uint256 value,
    // bytes32[2] bindingSignature, bytes32 signHash)
    // input: nf, anchor, cv, rk, proof

    let mut it = AbiArgIterator::new(data);

    let nullifier = it.next_byte32_as_array()?;
    let anchor = it.next_byte32()?;
    let cv = it.next_byte32()?;
    let rk = it.next_byte32()?;
    let zkproof = it.next_fixed_words(6)?;

    let spend_auth_sig = it.next_fixed_words(2)?;
    let value = it.next_u256()?;
    let binding_sig = it.next_fixed_words(2)?;
    let sighash = it.next_byte32_as_array()?;

    let cv = edwards::Point::<Bls12, Unknown>::read(cv, &JUBJUB)?;
    let anchor = le_bytes_to_fr(anchor)?;
    let rk = PublicKey::<Bls12>::read(rk, &JUBJUB)?;
    let spend_auth_sig = Signature::read(spend_auth_sig)?;
    let zkproof = Proof::<Bls12>::read(zkproof)?;

    let mut ctx = SaplingVerificationContext::new();

    // librustzcashSaplingCheckSpend
    if !ctx.check_spend(
        cv,
        anchor,
        &nullifier,
        rk,
        &sighash,
        spend_auth_sig,
        zkproof,
        &SAPLING_PARAMETERS.spend_vk,
        &JUBJUB,
    ) {
        return Err(Error::CheckSpend);
    }

    // librustzcashSaplingFinalCheck
    let value_balance = Amount::from_i64(value.as_u64() as i64).map_err(|_| Error::InvalidValue)?;
    let binding_sig = Signature::read(binding_sig)?;

    if !ctx.final_check(value_balance, &sighash, binding_sig, &JUBJUB) {
        return Err(Error::FinalCheck);
    }

    Ok(())
}

/// pedersenhash, merklehash
pub fn pedersen_hash(data: &[u8]) -> Result<Vec<u8>, Error> {
    use std::convert::TryInto;

    let mut it = AbiArgIterator::new(data);

    let level: usize = it.next_u256()?.try_into().map_err(|s| Error::Runtime(s))?;
    let left = le_bytes_to_fr_repr(it.next_byte32()?);
    let right = le_bytes_to_fr_repr(it.next_byte32()?);

    let result = merkle_hash(level, &left, &right);
    Ok(result.as_ref().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pedersen_hash() {
        let a = hex::decode("05655316a07e6ec8c9769af54ef98b30667bfb6302b32987d552227dae86a087").unwrap();
        let b = hex::decode("06041357de59ba64959d1b60f93de24dfe5ea1e26ed9e8a73d35b225a1845ba7").unwrap();

        let a_repr = {
            let mut f = FrRepr::default();
            f.as_mut().copy_from_slice(&a);
            f
        };
        let b_repr = {
            let mut f = FrRepr::default();
            f.as_mut().copy_from_slice(&b);
            f
        };
        let result = merkle_hash(25, &a_repr, &b_repr);
        assert_eq!(
            hex::encode(result.as_ref()),
            "61a50a5540b4944da27cbd9b3d6ec39234ba229d2c461f4d719bc136573bf45b"
        );
    }
}
