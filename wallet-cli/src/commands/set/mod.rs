use clap::ArgMatches;
use keys::Address;
use proto::core::{AccountPermissionUpdateContract, AccountUpdateContract};
use serde_json::json;
use std::io;
use std::io::Read;

use crate::error::Error;
use crate::utils::jsont;
use crate::utils::trx;

mod contract;

/// Set account permission info.
fn set_account_permission(matches: &ArgMatches) -> Result<(), Error> {
    let addr = matches
        .value_of("ADDR")
        .expect("account address is required is cli.yml; qed")
        .parse::<Address>()?;
    let permission = matches.value_of("PERMISSION").expect("required in cli.yml; qed");

    let mut permission_info: serde_json::Value = if permission == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        serde_json::from_str(&buffer)?
    } else {
        serde_json::from_str(permission)?
    };

    // convert from hex byte repr to Vec<u8>
    jsont::revert_permission_info(&mut permission_info);
    permission_info["owner_address"] = json!(addr.as_ref().to_owned());

    let perm_contract: AccountPermissionUpdateContract = serde_json::from_value(permission_info)?;

    trx::TransactionHandler::handle(perm_contract, matches).run()
}

/// Set account name.
fn set_account_name(matches: &ArgMatches) -> Result<(), Error> {
    let addr = matches
        .value_of("ADDR")
        .expect("account name is required is cli.yml; qed")
        .parse::<Address>()?;
    let name = matches.value_of("NAME").expect("required in cli.yml; qed");

    let update_contract: AccountUpdateContract = AccountUpdateContract {
        owner_address: addr.as_ref().to_owned(),
        account_name: name.as_bytes().to_owned(),
        ..Default::default()
    };

    trx::TransactionHandler::handle(update_contract, matches).run()
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("account_permission", Some(arg_matches)) => set_account_permission(arg_matches),
        ("account_name", Some(arg_matches)) => set_account_name(arg_matches),
        ("contract", Some(arg_matches)) => contract::run(arg_matches),
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
