//! Subcommand to call a contract.

use clap::ArgMatches;
use hex::{FromHex, ToHex};
use keys::Address;
use proto::core::TriggerSmartContract;

use crate::error::Error;
use crate::utils::abi;
use crate::utils::trx;

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    let sender = matches
        .value_of("SENDER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong sender address format"))?;
    let contract = matches
        .value_of("CONTRACT")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong contract address format"))?;
    let method = matches.value_of("METHOD").expect("required in cli.yml; qed");

    let data = match (matches.values_of("ARGS"), matches.value_of("data")) {
        (Some(args), None) => {
            let types = extract_types(method)?;
            if matches.occurrences_of("ARGS") as usize != types.len() {
                return Err(Error::Runtime("wrong number of ARGS"));
            }
            // Fix tron base58checked addresses, remove 0x41
            let values = args
                .zip(types.iter())
                .map(|(arg, ty)| {
                    if ty == &"address" {
                        arg.parse::<Address>()
                            .map(|addr| addr.encode_hex::<String>()[2..].to_owned())
                            .map_err(Error::from)
                    } else {
                        Ok(arg.to_owned())
                    }
                })
                .collect::<Result<Vec<_>, Error>>()?;
            let mut data = (&abi::fnhash(method)[..]).to_owned();
            data.append(&mut abi::encode_params(&types, &values)?);
            eprintln!("! data = {:}", data.encode_hex::<String>());
            data
        }
        (None, Some(data_hex)) => Vec::from_hex(data_hex)?,
        // nullary call
        (None, None) => Vec::from(&abi::fnhash(method)[..]),
        (_, _) => unreachable!("set conflicts in cli.yml; qed"),
    };

    let trigger_contract = TriggerSmartContract {
        owner_address: sender.to_bytes().to_owned(),
        contract_address: contract.to_bytes().to_owned(),
        data: data.into(),
        ..Default::default()
    };

    trx::TransactionHandler::handle(trigger_contract, matches)
        .map_raw_transaction(|raw| raw.set_fee_limit(1_000_000))
        .run()
}

#[inline]
fn extract_types(fnname: &str) -> Result<Vec<&str>, Error> {
    let start = fnname.find('(').ok_or(Error::Runtime("malformed method name"))?;
    let end = fnname.find(')').ok_or(Error::Runtime("malformed method name"))?;
    Ok(fnname[start + 1..end].split(",").filter(|ty| !ty.is_empty()).collect())
}
