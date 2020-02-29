use clap::ArgMatches;
use keys::Address;
use proto::core::{ProposalApproveContract, ProposalCreateContract, ProposalDeleteContract};
use std::collections::HashMap;

use crate::error::Error;
use crate::utils::trx;

pub fn create(matches: &ArgMatches) -> Result<(), Error> {
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

pub fn approve(approve: bool, matches: &ArgMatches) -> Result<(), Error> {
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

pub fn delete(matches: &ArgMatches) -> Result<(), Error> {
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
