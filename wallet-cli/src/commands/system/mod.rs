//! Commands for system contracts.

use clap::ArgMatches;
use itertools::Itertools;
use keys::Address;
use proto::core::{VoteWitnessContract, VoteWitnessContract_Vote as Vote};

use crate::error::Error;
use crate::utils::trx;

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
                        vote_address: addr.parse::<Address>()?.as_ref().to_owned(),
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
        owner_address: sender.as_ref().to_owned(),
        votes: votes.into(),
        ..Default::default()
    };

    trx::TransactionHandler::handle(vote_contract, matches).run()
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("vote_witness", Some(arg_matches)) => vote_witnesses(arg_matches),
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
