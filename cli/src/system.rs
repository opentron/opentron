use clap::ArgMatches;
use itertools::Itertools;
use keys::Address;
use proto::chain::transaction::Contract;
use proto::contract as contract_pb;

pub fn main(matches: &ArgMatches) -> Option<Contract> {
    match matches.subcommand() {
        ("freeze", Some(arg_matches)) => freeze(arg_matches),
        ("vote", Some(arg_matches)) => vote(arg_matches),
        _ => unimplemented!(),
    }
}

fn freeze(matches: &ArgMatches) -> Option<Contract> {
    use proto::common::ResourceCode as ResourceType;

    let from: Address = matches.value_of("SENDER")?.parse().ok()?;
    let to: Address = matches.value_of("RECIPIENT")?.parse().ok()?;
    let amount = crate::util::parse_amount_with_currency(matches.value_of("AMOUNT")?, "TRX", 6)?;

    let resource_type = match matches.value_of("type") {
        Some("bandwidth") => ResourceType::Bandwidth,
        Some("energy") => ResourceType::Energy,
        _ => unreachable!("checks values in clap; qed"),
    };

    let inner = contract_pb::FreezeBalanceContract {
        owner_address: from.as_bytes().into(),
        frozen_balance: amount,
        frozen_duration: 3,
        resource: resource_type as i32,
        receiver_address: if from == to { vec![] } else { to.as_bytes().into() },
    };

    Some(inner.into())
}

fn vote(matches: &ArgMatches) -> Option<Contract> {
    use proto::common::Vote;

    let from: Address = matches.value_of("SENDER")?.parse().ok()?;
    let votes = match matches.values_of("VOTES") {
        Some(vote_args) => vote_args
            .chunks(2)
            .into_iter()
            .map(|chunk| {
                if let &[addr, count] = &chunk.collect::<Vec<_>>()[..] {
                    Ok(Vote {
                        vote_address: addr.parse::<Address>()?.as_bytes().to_owned(),
                        vote_count: crate::util::parse_amount(count).expect("parse amount failed"),
                    })
                } else {
                    unreachable!("restricted by cli.yml; qed")
                }
            })
            .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()
            .ok()?,
        _ => vec![],
    };

    let inner = contract_pb::VoteWitnessContract {
        owner_address: from.as_bytes().into(),
        votes: votes,
        ..Default::default()
    };

    Some(inner.into())
}
