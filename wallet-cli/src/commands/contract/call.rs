//! Subcommand to call a contract.

use clap::ArgMatches;
use hex::{FromHex, ToHex};
use keys::Address;
use proto::api_grpc::Wallet;
use proto::core::TriggerSmartContract;
use serde_json::json;

use crate::error::Error;
use crate::utils::abi;
use crate::utils::client;
use crate::utils::jsont;
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
                            .map(|addr| addr.as_tvm_bytes().encode_hex::<String>())
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

    let mut trigger_contract = TriggerSmartContract {
        owner_address: sender.as_bytes().to_owned(),
        contract_address: contract.as_bytes().to_owned(),
        data: data.into(),
        ..Default::default()
    };

    if let Some(value) = matches.value_of("value") {
        trigger_contract.set_call_value(trx::parse_amount_with_surfix(value, "TRX", 6)?);
    }

    if let Some(token_id) = matches.value_of("token-id") {
        let value = matches.value_of("token-value").expect("constraint in cli.yml; qed");
        trigger_contract.set_token_id(token_id.parse()?);
        trigger_contract.set_call_token_value(trx::parse_amount(value)?);
    }

    if matches.is_present("const") {
        let (_, trx_ext, _) = client::GRPC_CLIENT
            .trigger_constant_contract(Default::default(), trigger_contract)
            .wait()?;
        let mut json = serde_json::to_value(&trx_ext)?;
        jsont::fix_transaction_ext(&mut json)?;
        let ret = json!({
            "result": json["result"],
            "constant_result": json["constant_result"],
        });
        println!("{:}", serde_json::to_string_pretty(&ret)?);
        if !trx_ext.get_constant_result().is_empty() && !trx_ext.get_constant_result()[0].is_empty() {
            handle_contract_result(&contract, method, &trx_ext.get_constant_result()[0])?;
        }
        Ok(())
    } else {
        let mut handler = trx::TransactionHandler::handle(trigger_contract, matches);
        handler.map_raw_transaction(|raw| raw.set_fee_limit(1_000_000));
        handler.run()?;
        handler.watch(|info| handle_contract_result(&contract, method, &info.get_contractResult()[0]))
    }
}

#[inline]
fn extract_types(fnname: &str) -> Result<Vec<&str>, Error> {
    let start = fnname.find('(').ok_or(Error::Runtime("malformed method name"))?;
    let end = fnname.find(')').ok_or(Error::Runtime("malformed method name"))?;
    Ok(fnname[start + 1..end].split(",").filter(|ty| !ty.is_empty()).collect())
}

fn handle_contract_result(contract: &Address, method: &str, result: &[u8]) -> Result<(), Error> {
    let abi = trx::get_contract_abi(contract)?;
    abi.iter()
        .find(|entry| abi::entry_to_method_name(entry) == method)
        .ok_or(Error::Runtime("ABI not found, can not parse result"))
        .and_then(|entry| {
            let types = abi::entry_to_output_types(&entry);
            let output = abi::decode_params(&types, &result.encode_hex::<String>())?;
            eprintln!("! Parsed result:\n{:}", output);
            Ok(())
        })
}
