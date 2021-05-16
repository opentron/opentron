use clap::ArgMatches;
use keys::Address;
use proto::chain::transaction::Contract;
use proto::contract as contract_pb;

pub fn main(matches: &ArgMatches) -> Option<Contract> {
    match matches.subcommand() {
        ("create", Some(arg_matches)) => create(arg_matches),
        ("update_brokerage", Some(arg_matches)) => update_brokerage(arg_matches),
        ("update_url", Some(arg_matches)) => update_url(arg_matches),
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

fn update_brokerage(matches: &ArgMatches) -> Option<Contract> {
    let from: Address = matches.value_of("SENDER")?.parse().ok()?;
    let percent = matches.value_of("PERCENT").expect("required; qed");

    let inner = contract_pb::UpdateBrokerageContract {
        owner_address: from.as_bytes().into(),
        brokerage: percent.parse().ok()?,
    };

    Some(inner.into())
}

fn update_url(matches: &ArgMatches) -> Option<Contract> {
    let from: Address = matches.value_of("SENDER")?.parse().ok()?;
    let url = matches.value_of("URL").expect("required; qed");

    let inner = contract_pb::WitnessUpdateContract {
        owner_address: from.as_bytes().into(),
        new_url: url.as_bytes().into(),
    };

    Some(inner.into())
}
