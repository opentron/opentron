use clap::ArgMatches;
use keys::Address;
use proto::core::{FreezeBalanceContract, ResourceCode, UnfreezeBalanceContract};

use crate::error::Error;
use crate::utils::trx;

pub fn freeze(matches: &ArgMatches) -> Result<(), Error> {
    let from = matches
        .value_of("FROM")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong from address format"))?;
    let receiver = matches
        .value_of("RECEIVER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong receiver address format"))?;
    let amount = matches.value_of("AMOUNT").expect("required in cli.yml; qed");
    let duration = matches.value_of("duration").expect("has default in cli.yml; qed");

    // if receiver is self, receiver_address must be empty
    let freeze_contract = FreezeBalanceContract {
        owner_address: from.as_bytes().to_owned(),
        receiver_address: if receiver == from {
            vec![]
        } else {
            receiver.as_bytes().to_owned()
        },
        frozen_balance: trx::parse_amount_with_surfix(amount, "TRX", 6)?,
        frozen_duration: duration.parse()?,
        resource: if matches.is_present("energy") {
            ResourceCode::ENERGY
        } else {
            ResourceCode::BANDWIDTH
        },
        ..Default::default()
    };

    trx::TransactionHandler::handle(freeze_contract, matches).run()
}

pub fn unfreeze(matches: &ArgMatches) -> Result<(), Error> {
    let from = matches
        .value_of("FROM")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong from address format"))?;
    let receiver = matches
        .value_of("RECEIVER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong receiver address format"))?;

    // if receiver is self, receiver_address must be empty
    let unfreeze_contract = UnfreezeBalanceContract {
        owner_address: from.as_bytes().to_owned(),
        receiver_address: if receiver == from {
            vec![]
        } else {
            receiver.as_bytes().to_owned()
        },
        resource: if matches.is_present("energy") {
            ResourceCode::ENERGY
        } else {
            ResourceCode::BANDWIDTH
        },
        ..Default::default()
    };

    trx::TransactionHandler::handle(unfreeze_contract, matches).run()
}
