//! TRC10 related commands

use clap::ArgMatches;
use keys::Address;
use proto::core::AssetIssueContract;
use serde_json::json;
use std::io;
use std::io::Read;

use crate::error::Error;
use crate::utils::jsont;
use crate::utils::trx;

pub fn issue_asset(matches: &ArgMatches) -> Result<(), Error> {
    let sender = matches
        .value_of("SENDER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong from address format"))?;

    let name = matches.value_of("NAME").expect("required in cli.yml; qed");
    let abbr = matches.value_of("abbr").unwrap_or(name);
    let total_supply = matches.value_of("SUPPLY").expect("required in cli.yml; qed");
    let precision = matches.value_of("precision").unwrap_or("0");

    let description = matches.value_of("description").unwrap_or_default();
    let url = matches.value_of("url").expect("required in cli.yml; qed");


    let mut issue_contract = AssetIssueContract::new();
    issue_contract.set_owner_address(sender.as_ref().to_owned());

    issue_contract.set_name(name.as_bytes().to_owned());
    issue_contract.set_abbr(abbr.as_bytes().to_owned());
    issue_contract.set_total_supply(total_supply.parse()?);
    issue_contract.set_precision(precision.parse()?);

    issue_contract.set_description(description.as_bytes().to_owned());
    issue_contract.set_url(url.as_bytes().to_owned());


    trx::TransactionHandler::handle(issue_contract, matches).run()
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("issue", Some(arg_matches)) => issue_asset(arg_matches),
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
