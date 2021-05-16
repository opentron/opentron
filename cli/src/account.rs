use clap::ArgMatches;
use keys::Address;
use proto::chain::transaction::Contract;
use proto::contract as contract_pb;

pub fn main(matches: &ArgMatches) -> Option<Contract> {
    match matches.subcommand() {
        ("create", Some(arg_matches)) => create(arg_matches),
        ("set_name", Some(arg_matches)) => set_name(arg_matches),
        ("set_id", Some(arg_matches)) => set_id(arg_matches),
        _ => unimplemented!(),
    }
}

fn create(matches: &ArgMatches) -> Option<Contract> {
    use proto::common::AccountType;

    let from: Address = matches.value_of("SENDER")?.parse().ok()?;
    let to: Address = matches.value_of("RECIPIENT")?.parse().ok()?;

    let account_type = match matches.value_of("type") {
        Some("Normal") => AccountType::Normal,
        Some("AssetIssue") => AccountType::AssetIssue,
        Some("Contract") => AccountType::Contract,
        _ => unreachable!("values restricted; qed"),
    };

    let inner = contract_pb::AccountCreateContract {
        owner_address: from.as_bytes().into(),
        account_address: to.as_bytes().into(),
        r#type: account_type as i32,
    };
    Some(inner.into())
}

fn set_name(matches: &ArgMatches) -> Option<Contract> {
    let from: Address = matches.value_of("SENDER")?.parse().ok()?;
    let name = matches.value_of("NAME").expect("required; qed");

    let inner = contract_pb::AccountUpdateContract {
        owner_address: from.as_bytes().into(),
        account_name: name.into(),
    };
    Some(inner.into())
}

fn set_id(matches: &ArgMatches) -> Option<Contract> {
    let from: Address = matches.value_of("SENDER")?.parse().ok()?;
    let id = matches.value_of("ID").expect("required; qed");

    let inner = contract_pb::SetAccountIdContract {
        owner_address: from.as_bytes().into(),
        account_id: id.into(),
    };
    Some(inner.into())
}
