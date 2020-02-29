use clap::ArgMatches;
use itertools::Itertools;
use keys::Address;
use proto::core::{
    UpdateBrokerageContract, VoteWitnessContract, VoteWitnessContract_Vote as Vote, WithdrawBalanceContract,
    WitnessCreateContract, WitnessUpdateContract,
};

use crate::error::Error;
use crate::utils::trx;

pub fn vote(matches: &ArgMatches) -> Result<(), Error> {
    let sender = matches
        .value_of("SENDER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong from address format"))?;

    let votes = match matches.values_of("VOTES") {
        Some(vote_args) => vote_args
            .chunks(2)
            .into_iter()
            .map(|chunk| {
                if let &[addr, count] = &chunk.collect::<Vec<_>>()[..] {
                    Ok(Vote {
                        vote_address: addr.parse::<Address>()?.as_bytes().to_owned(),
                        vote_count: count.parse()?,
                        ..Default::default()
                    })
                } else {
                    unreachable!("restricted by cli.yml; qed")
                }
            })
            .collect::<Result<Vec<_>, Error>>()?,
        _ => vec![],
    };

    let vote_contract = VoteWitnessContract {
        owner_address: sender.as_bytes().to_owned(),
        votes: votes.into(),
        ..Default::default()
    };

    trx::TransactionHandler::handle(vote_contract, matches).run()
}

pub fn create(matches: &ArgMatches) -> Result<(), Error> {
    let sender = matches
        .value_of("SENDER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong from address format"))?;
    let url = matches.value_of("URL").expect("required in cli.yml; qed");

    let create_contract = WitnessCreateContract {
        owner_address: sender.as_bytes().to_owned(),
        url: url.as_bytes().to_owned(),
        ..Default::default()
    };
    trx::TransactionHandler::handle(create_contract, matches).run()
}

pub fn update(matches: &ArgMatches) -> Result<(), Error> {
    let sender = matches
        .value_of("SENDER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong from address format"))?;
    let url = matches.value_of("URL").expect("required in cli.yml; qed");

    let update_contract = WitnessUpdateContract {
        owner_address: sender.as_bytes().to_owned(),
        update_url: url.as_bytes().to_owned(),
        ..Default::default()
    };
    trx::TransactionHandler::handle(update_contract, matches).run()
}

pub fn withdraw_reward(matches: &ArgMatches) -> Result<(), Error> {
    let sender = matches
        .value_of("SENDER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong from address format"))?;

    let withdraw_contract = WithdrawBalanceContract {
        owner_address: sender.as_bytes().to_owned(),
        ..Default::default()
    };
    trx::TransactionHandler::handle(withdraw_contract, matches).run()
}

pub fn update_brokerage(matches: &ArgMatches) -> Result<(), Error> {
    let sender = matches
        .value_of("SENDER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong from address format"))?;
    let brokerage = matches
        .value_of("BROKERAGE")
        .expect("required in cli.yml; qed")
        .parse()?;

    let withdraw_contract = UpdateBrokerageContract {
        owner_address: sender.as_bytes().to_owned(),
        brokerage: brokerage,
        ..Default::default()
    };
    trx::TransactionHandler::handle(withdraw_contract, matches).run()
}
