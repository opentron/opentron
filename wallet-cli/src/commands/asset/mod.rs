//! TRC10 related commands

use chrono::{DateTime, Utc};
use clap::ArgMatches;
use keys::Address;
use proto::core::{AssetIssueContract, AssetIssueContract_FrozenSupply as FrozenSupply};

use crate::error::Error;
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

    let mut period = matches.values_of("issuing-period").expect("required in cli.yml; qed");
    let start_time = period
        .next()
        .expect("required in cli.yml; qed")
        .parse::<DateTime<Utc>>()
        .map_err(|_| Error::Runtime("illegal datetime format"))?;
    let end_time = period
        .next()
        .expect("required in cli.yml; qed")
        .parse::<DateTime<Utc>>()
        .map_err(|_| Error::Runtime("illegal datetime format"))?;

    println!("! issuing period {:?} -> {:?}", start_time, end_time);

    let rate = matches.value_of("exchange-rate").expect("has default in cli.yml; qed");
    let (trx_num, ico_num) = match &rate.split(":").collect::<Vec<_>>()[..] {
        [trc, ico] => (
            trx::parse_amount_without_surfix(trc)?,
            trx::parse_amount_without_surfix(ico)?,
        ),
        _ => return Err(Error::Runtime("illegal exchange rate format")),
    };

    let freeze = match matches.values_of("freeze") {
        Some(raw) => raw
            .map(|params| match &params.split('=').collect::<Vec<_>>()[..] {
                [amount, days] => Ok(FrozenSupply {
                    frozen_amount: trx::parse_amount_without_surfix(amount)?,
                    frozen_days: days.parse()?,
                    ..Default::default()
                }),
                _ => Err(Error::Runtime("illegal freeze format")),
            })
            .collect::<Result<Vec<_>, Error>>()?,
        _ => vec![],
    };

    let mut issue_contract = AssetIssueContract::new();
    issue_contract.set_owner_address(sender.as_ref().to_owned());

    issue_contract.set_name(name.as_bytes().to_owned());
    issue_contract.set_abbr(abbr.as_bytes().to_owned());
    issue_contract.set_total_supply(trx::parse_amount_without_surfix(total_supply)?);
    issue_contract.set_precision(precision.parse()?);
    issue_contract.set_frozen_supply(freeze.into());

    issue_contract.set_description(description.as_bytes().to_owned());
    issue_contract.set_url(url.as_bytes().to_owned());

    issue_contract.set_start_time(start_time.timestamp_millis());
    issue_contract.set_end_time(end_time.timestamp_millis());

    issue_contract.set_trx_num(trx_num as i32);
    issue_contract.set_num(ico_num as i32);

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
