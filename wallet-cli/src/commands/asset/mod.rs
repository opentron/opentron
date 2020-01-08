//! TRC10 related commands

use chrono::{DateTime, Utc};
use clap::ArgMatches;
use keys::Address;
use proto::core::{
    AssetIssueContract, AssetIssueContract_FrozenSupply as FrozenSupply, TransferAssetContract, UpdateAssetContract,
};

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

    if let Some(limit) = matches.value_of("bandwidth-limit-for-issuer") {
        issue_contract.set_free_asset_net_limit(limit.parse()?);
    }
    if let Some(public_limit) = matches.value_of("bandwidth-limit-per-account") {
        issue_contract.set_public_free_asset_net_limit(public_limit.parse()?);
    }

    trx::TransactionHandler::handle(issue_contract, matches).run()
}

pub fn transfer_asset(matches: &ArgMatches) -> Result<(), Error> {
    let sender = matches
        .value_of("SENDER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong sender address format"))?;
    let recipient = matches
        .value_of("RECIPIENT")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong recipient address format"))?;
    let amount = matches.value_of("AMOUNT").expect("required in cli.yml; qed");
    let memo = matches.value_of("MEMO").unwrap_or("").as_bytes().to_owned();
    let assert_id = matches.value_of("token-id").expect("required in cli.yml; qed");

    let transfer_contract = TransferAssetContract {
        owner_address: sender.to_bytes().to_owned(),
        to_address: recipient.to_bytes().to_owned(),
        amount: trx::parse_amount_without_surfix(amount)?,
        asset_name: assert_id.as_bytes().to_owned(),
        ..Default::default()
    };

    eprintln!("sender:    {:}", sender);
    eprintln!("recipient: {:}", recipient);

    trx::TransactionHandler::handle(transfer_contract, matches)
        .map_raw_transaction(move |raw| raw.set_data(memo.clone()))
        .run()
}

pub fn update_asset_settings(matches: &ArgMatches) -> Result<(), Error> {
    let sender = matches
        .value_of("SENDER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong from address format"))?;

    let description = matches.value_of("description").expect("required in cli.yml; qed");
    let url = matches.value_of("url").expect("required in cli.yml; qed");

    let mut update_contract = UpdateAssetContract::new();
    update_contract.set_owner_address(sender.as_ref().to_owned());

    update_contract.set_description(description.as_bytes().to_owned());
    update_contract.set_url(url.as_bytes().to_owned());

    if let Some(limit) = matches.value_of("bandwidth-limit-for-issuer") {
        update_contract.set_new_limit(limit.parse()?);
    }
    if let Some(public_limit) = matches.value_of("bandwidth-limit-per-account") {
        update_contract.set_new_public_limit(public_limit.parse()?);
    }

    trx::TransactionHandler::handle(update_contract, matches).run()
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("issue", Some(arg_matches)) => issue_asset(arg_matches),
        ("transfer", Some(arg_matches)) => transfer_asset(arg_matches),
        ("update", Some(arg_matches)) => update_asset_settings(arg_matches),
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
