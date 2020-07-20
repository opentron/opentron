//! Tron 4.0 update, shielded contract support.
// 0000000000000000000000000000000000000000000000000000000001000001 - verifyMintProof
// 0000000000000000000000000000000000000000000000000000000001000002 - verifyTransferProof
// 0000000000000000000000000000000000000000000000000000000001000003 - verifyBurnProof
// 0000000000000000000000000000000000000000000000000000000001000004 - pedersenHash

use bellman::groth16::{Parameters, PreparedVerifyingKey, Proof};
use ff::{Field, PrimeField};
use lazy_static::lazy_static;
use pairing::bls12_381::{Bls12, Fr, FrRepr};
use primitive_types::U256;
use zcash_primitives::jubjub::edwards;
use zcash_primitives::jubjub::fs::{Fs, FsRepr};
use zcash_primitives::jubjub::Unknown;
use zcash_primitives::merkle_tree::{CommitmentTree, Hashable, IncrementalWitness, MerklePath};
use zcash_primitives::redjubjub::Signature;
use zcash_primitives::sapling::{merkle_hash, Node};
use zcash_primitives::transaction::components::Amount;
use zcash_primitives::JUBJUB;
use zcash_proofs::load_parameters;
use zcash_proofs::sapling::SaplingVerificationContext;

use self::helper::AbiArgIterator;

mod helper;

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

        let spend_path = "../ztron-params/sapling-spend.params";
        let output_path = "../ztron-params/sapling-output.params";

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

const TREE_WIDTH: usize = 0x100000000;

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
fn bytes_to_fr_repr(raw: &[u8]) -> FrRepr {
    let mut f = FrRepr::default();
    f.as_mut().copy_from_slice(raw);
    f
}

fn insert_leaf_to_merkle_tree(mut frontier: [[u8; 32]; 33], leaf_index: usize, leafs: &[[u8; 32]]) -> Vec<u8> {
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
                let left = bytes_to_fr_repr(&frontier[level - 1]);
                let right = bytes_to_fr_repr(&node_value);
                node_index = (node_index - 1) / 2;
                (left, right)
            } else {
                let left = bytes_to_fr_repr(&node_value);
                let right = Fr::from(Node::empty_root(level - 1)).to_repr();
                node_index = node_index / 2;
                (left, right)
            };

            let hash_value = merkle_hash(level - 1, &left, &right);
            // println!("hash_value => {:?}", hash_value);

            node_value[..].copy_from_slice(hash_value.as_ref());
            result[offset..offset + 32].copy_from_slice(hash_value.as_ref());
            offset += 32;
        }

        frontier[slot].copy_from_slice(&node_value);
    }

    for level in *slots.last().unwrap() + 1..=32 {
        let (left, right) = if node_index % 2 == 0 {
            let left = bytes_to_fr_repr(&frontier[level - 1]);
            let right = bytes_to_fr_repr(&node_value);
            node_index = (node_index - 1) / 2;
            (left, right)
        } else {
            let left = bytes_to_fr_repr(&node_value);
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

pub fn verify_mint_proof(input: &[u8]) -> Option<Vec<u8>> {
    // (bytes32 cm, bytes32 cv, bytes32 epk, bytes32[6] proof,
    //  bytes32[2] binding_sig, uint256 value, bytes32 sighash,
    //  bytes32[33] frontier, uint256 leaf_count)
    let mut it = AbiArgIterator::new(input);

    let cm = it.next_byte32()?;
    let cv = it.next_byte32()?;
    let epk = it.next_byte32()?;
    let zkproof = it.next_words_as_bytes(6)?;

    let bingding_sig = it.next_words_as_bytes(2)?;
    let value = it.next_u256()?;
    let sighash = it.next_byte32()?;

    let frontier = it.next_words_as_bytes(33)?;
    // current leaf count
    let leaf_count = it.next_u256()?;

    assert!(it.is_ended());

    // librustzcashSaplingCheckOutput
    let cm = Fr::from_repr(bytes_to_fr_repr(cm))?;
    let cv = edwards::Point::<Bls12, Unknown>::read(cv, &JUBJUB).ok()?;
    let epk = edwards::Point::<Bls12, Unknown>::read(epk, &JUBJUB).ok()?;
    let zkproof = Proof::<Bls12>::read(zkproof).ok()?;

    let mut ctx = SaplingVerificationContext::new();

    if !ctx.check_output(cv, cm, epk, zkproof, &SAPLING_PARAMETERS.output_vk, &JUBJUB) {
        return None;
    }

    // librustzcashSaplingFinalCheck
    let bingding_sig = Signature::read(bingding_sig).ok()?;
    let value_balance = Amount::from_i64(-(value.as_u64() as i64)).ok()?;
    let sighash = {
        let mut raw = [0u8; 32];
        raw.copy_from_slice(sighash);
        raw
    };

    if !ctx.final_check(value_balance, &sighash, bingding_sig, &JUBJUB) {
        return None;
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

    return Some(insert_leaf_to_merkle_tree(frontier, leaf_count.as_usize(), &leafs));
}

pub fn verify_transfer_proof(input: &[u8]) -> Option<Vec<u8>> {
    unimplemented!()
}

pub fn verify_burn_proof(input: &[u8]) -> Option<Vec<u8>> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_mint_proof() {
        let raw = "a4a318b818080697617d9cc681624570570b1ff2d76d126ef11d59d5845d5f39297543de8c3536545890e96bc16ef5a76c7f85e7f9d626d8d31c8d05652ad0be5a1135a5e85bf8b89a482050b69899898314914a0d1556b5c30b50f499e5b3188b474e588ac50837de987e15b65cee71742fa83d78b526820b4d1858c8a76a514c8d6b9978f92c392e4bd892a77dec41a129f3e38f452e1cb68ad34ad4d685c68567ba5b5b516e30986ab23d33c80bd670c46f8fcef9dba9b3cbbf0d2568533210c2f81270762e71dbef95d0189d06ad0760cac8effe21732a8eb5c69a29070d9db9b85e311cb30dc24e2b23e20fffdc9017c6edac380c4092c1eea85856b55168944f2c6fba78f68dc3ea172a97369654995c090e8e1b2bdfaacff0bf1e47a233c9c087fe4428578fc65db6aeaad4d4ccba124d6f4a247ab08af5a999a994d24dd249b890259cd18542a17b4a773ec678e4efebfd138e25e16546c790f2c2040000000000000000000000000000000000000000000000000000000000000001354a47315acce4ed131b7884426e62371b9415a103b1ee2495e7f062d37188480000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

        let input = hex::decode(raw).unwrap();

        println!("len = {} {}", input.len(), input.len() as f64 / 32.0);

        let ret = verify_mint_proof(&input).unwrap();
        for word in ret.chunks(32) {
            println!("=> {}", hex::encode(word));
        }
    }
}
