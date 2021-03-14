use clap::ArgMatches;
use keys::Address;
use proto::chain::transaction::Contract;
use proto::contract as contract_pb;
use std::collections::HashMap;

use crate::{custom_error, Result};

pub fn main(matches: &ArgMatches) -> Result<Contract> {
    match matches.subcommand() {
        ("create", Some(arg_matches)) => create(arg_matches),
        ("approve", Some(arg_matches)) => approve(arg_matches),
        ("delete", Some(arg_matches)) => delete(arg_matches),
        _ => unreachable!("checked; qed"),
    }
}

fn create(matches: &ArgMatches) -> Result<Contract> {
    let from: Address = matches.value_of("SENDER").expect("required; qed").parse()?;
    let params = matches
        .values_of("PARAMS")
        .expect("required in cli.yml; qed")
        .map(|pair| {
            if let [key, val] = &pair.split('=').collect::<Vec<_>>()[..] {
                Ok((key.parse::<i64>()?, val.parse::<i64>()?))
            } else {
                Err(custom_error("malformed key=value PARAMS"))
            }
        });

    let inner = contract_pb::ProposalCreateContract {
        owner_address: from.as_bytes().into(),
        parameters: params.collect::<Result<HashMap<i64, i64>>>()?,
    };

    Ok(inner.into())
}

fn approve(matches: &ArgMatches) -> Result<Contract> {
    let from: Address = matches.value_of("SENDER").expect("required; qed").parse()?;
    let id = matches.value_of("ID").expect("required; qed");

    let is_approval = matches.value_of("approve").expect("required; qed") == "yes";

    let inner = contract_pb::ProposalApproveContract {
        owner_address: from.as_bytes().into(),
        proposal_id: id.parse()?,
        is_approval,
    };

    Ok(inner.into())
}

fn delete(matches: &ArgMatches) -> Result<Contract> {
    let from: Address = matches.value_of("SENDER").expect("required; qed").parse()?;
    let id = matches.value_of("ID").expect("required; qed");

    let inner = contract_pb::ProposalDeleteContract {
        owner_address: from.as_bytes().into(),
        proposal_id: id.parse()?,
    };

    Ok(inner.into())
}
