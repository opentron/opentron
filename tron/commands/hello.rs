use clap::ArgMatches;
use proto::core::{
    Transaction_Contract_ContractType as ContractType, Transaction_raw as TransactionRaw, VoteWitnessContract,
};
use protobuf::parse_from_bytes;

pub fn run(matches: &ArgMatches) -> Result<(), String> {
    let hash = matches.value_of("HASH").expect("HASH is required in cli.yml; qed");

    println!("HASH = {:?}", hash);

    let raw = hex::decode(hash).expect("hex decode ok");
    let tx = parse_from_bytes::<TransactionRaw>(&raw).expect("parse ok");

    for contr in &tx.contract {
        println!("got {:?}", contr.field_type);
        match contr.field_type {
            ContractType::VoteWitnessContract => {
                let param = contr.parameter.as_ref().map(|p| &p.value).expect("parameter body");
                let vote_witness =
                    parse_from_bytes::<VoteWitnessContract>(param).expect("pb VoteWitnessContract error");
                println!("param = {:?}", vote_witness);
            }
            _ => (),
        }
    }

    Ok(())
}
