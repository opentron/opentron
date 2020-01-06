use base58::ToBase58;
use clap::ArgMatches;
use hex::FromHex;
use keys::KeyPair;
use proto::core::{
    Transaction_Contract_ContractType as ContractType, Transaction_raw as TransactionRaw, VoteWitnessContract,
};
use protobuf::parse_from_bytes;
use sha2::{Digest, Sha256};

fn base58_checked<T: AsRef<[u8]>>(raw: T) -> String {
    let mut hasher = Sha256::new();
    hasher.input(raw.as_ref());
    let digest1 = hasher.result();

    let mut hasher = Sha256::new();
    hasher.reset();
    hasher.input(&digest1);
    let digest = hasher.result();

    let mut raw = raw.as_ref().to_owned();
    raw.extend(&digest[..4]);
    raw.to_base58()
}

pub fn run(matches: &ArgMatches) -> Result<(), String> {
    let hash = matches.value_of("HASH").expect("HASH is required in cli.yml; qed");

    println!("HASH = {:?}", hash);

    let raw = Vec::from_hex(hash).expect("hex decode ok");
    let tx = parse_from_bytes::<TransactionRaw>(&raw).expect("parse ok");

    println!("json =>\n{:}", serde_json::to_string(&tx).expect("json serilize"));

    for contr in &tx.contract {
        println!("got {:?}", contr.field_type);
        if contr.field_type == ContractType::VoteWitnessContract {
            let param = contr.parameter.as_ref().map(|p| &p.value).expect("parameter body");
            let vote_witness = parse_from_bytes::<VoteWitnessContract>(param).expect("pb VoteWitnessContract error");
            println!("param = {:?}", vote_witness);

            let addr = vote_witness.owner_address;
            let base58_addr = base58_checked(addr);
            println!("addr => {:?}", base58_addr);
        }
    }

    Ok(())
}

pub fn run_vanity(matches: &ArgMatches) -> Result<(), String> {
    let start = matches.value_of("start").unwrap_or("");
    let end = matches.value_of("end").unwrap_or("");

    loop {
        let kp = KeyPair::generate();
        let addr = kp.address().to_string();
        if addr.starts_with(start) && addr.ends_with(end) {
            println!("Addr: {}\nPrivate: {}", kp.address(), kp.private());
            break;
        }
    }
    Ok(())
}
