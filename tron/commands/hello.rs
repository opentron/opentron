use clap::ArgMatches;
use proto::core::Transaction_raw;
use protobuf::parse_from_bytes;

pub fn run(matches: &ArgMatches) -> Result<(), String> {
    let hash = matches.value_of("HASH").expect("HASH is required in cli.yml; qed");

    println!("HASH = {:?}", hash);

    let raw = hex::decode(hash).expect("hex decode ok");
    let tx = parse_from_bytes::<Transaction_raw>(&raw).expect("parse ok");

    println!("{:?}", tx);

    Ok(())
}
