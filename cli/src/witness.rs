use clap::ArgMatches;
use keys::Address;
use proto::chain::transaction::Contract;
use proto::contract as contract_pb;

pub fn main(matches: &ArgMatches) -> Option<Contract> {
    match matches.subcommand() {
        ("create", Some(arg_matches)) => create(arg_matches),
        _ => unimplemented!(),
    }
}

fn create(matches: &ArgMatches) -> Option<Contract> {
    let from: Address = matches.value_of("SENDER")?.parse().ok()?;
    let url = matches.value_of("URL").expect("required; qed");

    let inner = contract_pb::WitnessCreateContract {
        owner_address: from.as_bytes().into(),
        url: url.as_bytes().into(),
    };

    Some(inner.into())
}
