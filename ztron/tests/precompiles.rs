extern crate ztron;

use ztron::precompiles::*;

#[test]
fn test_verify_mint_proof() {
    let raw = include_str!("./mint.hex").trim();
    let input = hex::decode(raw).unwrap();

    println!("len={} words={}", input.len(), input.len() as f64 / 32.0);

    let ret = verify_mint_proof(&input).unwrap();
    for word in ret.chunks(32) {
        println!("=> {}", hex::encode(word));
    }
}

#[test]
fn test_verify_transfer_proof() {
    let raw = include_str!("./transfer.hex").trim();
    let input = hex::decode(raw).unwrap();

    println!("len={} words={}", input.len(), input.len() as f64 / 32.0);

    let ret = verify_transfer_proof(&input).unwrap();
    for word in ret.chunks(32) {
        println!("=> {}", hex::encode(word));
    }
}

#[test]
fn test_verify_transfer_proof_1_to_2() {
    let raw = include_str!("./transfer.to2.hex").trim();
    let input = hex::decode(raw).unwrap();

    println!("len={} words={}", input.len(), input.len() as f64 / 32.0);

    let ret = verify_transfer_proof(&input).unwrap();
    for word in ret.chunks(32) {
        println!("=> {}", hex::encode(word));
    }
}

#[test]
fn test_verify_burn_proof() {
    let raw = include_str!("./burn.hex").trim();
    let input = hex::decode(raw).unwrap();

    println!("len={} words={}", input.len(), input.len() as f64 / 32.0);

    let verified = verify_burn_proof(&input).unwrap();
    println!("ret => {:?}", verified);
    assert!(verified);
}
