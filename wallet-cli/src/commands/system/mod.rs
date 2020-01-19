//! Commands for system contracts.

use clap::ArgMatches;
use itertools::Itertools;
use keys::Address;
use proto::core::{
    ProposalApproveContract, ProposalCreateContract, ProposalDeleteContract, VoteWitnessContract,
    VoteWitnessContract_Vote as Vote, WithdrawBalanceContract, WitnessCreateContract, WitnessUpdateContract,
};
use std::collections::HashMap;

use crate::error::Error;
use crate::utils::trx;

mod freeze;

pub fn vote_witnesses(matches: &ArgMatches) -> Result<(), Error> {
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

pub fn create_witness(matches: &ArgMatches) -> Result<(), Error> {
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

pub fn update_witness(matches: &ArgMatches) -> Result<(), Error> {
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

pub fn create_proposal(matches: &ArgMatches) -> Result<(), Error> {
    let sender = matches
        .value_of("SENDER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong from address format"))?;

    let params = matches
        .values_of("PARAMS")
        .expect("required in cli.yml; qed")
        .map(|pair| {
            if let [key, val] = &pair.split('=').collect::<Vec<_>>()[..] {
                Ok((key.parse::<i64>()?, val.parse::<i64>()?))
            } else {
                Err(Error::Runtime("malformed key=value PARAMS"))
            }
        });

    let create_contract = ProposalCreateContract {
        owner_address: sender.as_bytes().to_owned(),
        parameters: params.collect::<Result<HashMap<i64, i64>, Error>>()?,
        ..Default::default()
    };

    trx::TransactionHandler::handle(create_contract, matches).run()
}

pub fn approve_proposal(approve: bool, matches: &ArgMatches) -> Result<(), Error> {
    let sender = matches
        .value_of("SENDER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong from address format"))?;
    let id = matches.value_of("ID").expect("required in cli.yml; qed");

    let approve_contract = ProposalApproveContract {
        owner_address: sender.as_bytes().to_owned(),
        proposal_id: id.parse()?,
        is_add_approval: approve,
        ..Default::default()
    };
    trx::TransactionHandler::handle(approve_contract, matches).run()
}

pub fn delete_proposal(matches: &ArgMatches) -> Result<(), Error> {
    let sender = matches
        .value_of("SENDER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong from address format"))?;
    let id = matches.value_of("ID").expect("required in cli.yml; qed");

    let delete_contract = ProposalDeleteContract {
        owner_address: sender.as_bytes().to_owned(),
        proposal_id: id.parse()?,
        ..Default::default()
    };
    trx::TransactionHandler::handle(delete_contract, matches).run()
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("vote_witness", Some(arg_matches)) => vote_witnesses(arg_matches),
        ("create_witness", Some(arg_matches)) => create_witness(arg_matches),
        ("update_witness", Some(arg_matches)) => update_witness(arg_matches),
        ("withdraw_reward", Some(arg_matches)) => withdraw_reward(arg_matches),
        ("create_proposal", Some(arg_matches)) => create_proposal(arg_matches),
        ("approve_proposal", Some(arg_matches)) => approve_proposal(true, arg_matches),
        ("disapprove_proposal", Some(arg_matches)) => approve_proposal(false, arg_matches),
        ("delete_proposal", Some(arg_matches)) => delete_proposal(arg_matches),
        ("freeze", Some(arg_matches)) => freeze::freeze(arg_matches),
        ("unfreeze", Some(arg_matches)) => freeze::unfreeze(arg_matches),

        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
