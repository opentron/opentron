use clap::ArgMatches;
use keys::Address;
use proto::core::{ClearABIContract, UpdateEnergyLimitContract, UpdateSettingContract};

use crate::error::Error;
use crate::utils::trx;

mod call;
mod create;

pub fn update_contract_settings(matches: &ArgMatches) -> Result<(), Error> {
    let owner_address: Address = matches.value_of("OWNER").expect("required in cli.yml; qed").parse()?;
    let contract = matches
        .value_of("CONTRACT")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong contract address format"))?;

    match (
        matches.value_of("user-resource-percent"),
        matches.value_of("energy-limit"),
    ) {
        (Some(res_percent), None) => {
            let update_contract = UpdateSettingContract {
                owner_address: owner_address.as_bytes().to_owned(),
                contract_address: contract.as_bytes().to_owned(),
                consume_user_resource_percent: res_percent.parse()?,
                ..Default::default()
            };
            trx::TransactionHandler::handle(update_contract, matches).run()
        }
        (None, Some(limit)) => {
            let update_contract = UpdateEnergyLimitContract {
                owner_address: owner_address.as_bytes().to_owned(),
                contract_address: contract.as_bytes().to_owned(),
                origin_energy_limit: limit.parse()?,
                ..Default::default()
            };
            trx::TransactionHandler::handle(update_contract, matches).run()
        }
        _ => Err(Error::Runtime(
            "one of --user-resource-percent or --energy-limit required",
        )),
    }
}

pub fn clear_contract_abi(matches: &ArgMatches) -> Result<(), Error> {
    let owner_address: Address = matches.value_of("OWNER").expect("required in cli.yml; qed").parse()?;
    let contract = matches
        .value_of("CONTRACT")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong contract address format"))?;

    let clear_contract = ClearABIContract {
        owner_address: owner_address.as_bytes().to_owned(),
        contract_address: contract.as_bytes().to_owned(),
        ..Default::default()
    };
    trx::TransactionHandler::handle(clear_contract, matches).run()
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("create", Some(arg_matches)) => create::main(arg_matches),
        ("call", Some(arg_matches)) => call::main(arg_matches),
        ("update", Some(arg_matches)) => update_contract_settings(arg_matches),
        ("clear_abi", Some(arg_matches)) => clear_contract_abi(arg_matches),
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
