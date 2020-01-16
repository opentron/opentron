use clap::ArgMatches;
use hex::{FromHex, ToHex};
use keys::Address;
use proto::core::{
    ClearABIContract, CreateSmartContract, SmartContract, SmartContract_ABI as Abi,
    SmartContract_ABI_Entry as AbiEntry, SmartContract_ABI_Entry_EntryType as AbiEntryType,
    SmartContract_ABI_Entry_Param as AbiEntryParam,
    SmartContract_ABI_Entry_StateMutabilityType as AbiEntryStateMutabilityType, UpdateEnergyLimitContract,
    UpdateSettingContract,
};
use std::fs;
use std::path::Path;

use crate::error::Error;
use crate::utils::abi;
use crate::utils::trx;

mod call;

#[inline]
fn translate_state_mutablility(val: &serde_json::Value) -> AbiEntryStateMutabilityType {
    match val.as_str().unwrap_or_default() {
        "view" | "View" => AbiEntryStateMutabilityType::View,
        "nonpayable" | "Nonpayable" => AbiEntryStateMutabilityType::Nonpayable,
        "payable" | "Payable" => AbiEntryStateMutabilityType::Payable,
        "pure" | "Pure" => AbiEntryStateMutabilityType::Pure,
        "" => AbiEntryStateMutabilityType::UnknownMutabilityType,
        x => {
            println!("unknown => {:?}", x);
            unimplemented!()
        }
    }
}

#[inline]
fn translate_abi_type(val: &serde_json::Value) -> AbiEntryType {
    match val.as_str().unwrap_or("") {
        "function" | "Function" => AbiEntryType::Function,
        "event" | "Event" => AbiEntryType::Event,
        "constructor" | "Constructor" => AbiEntryType::Constructor,
        _ => unimplemented!(),
    }
}

#[inline]
fn translate_abi_entry_params(val: &serde_json::Value) -> Vec<AbiEntryParam> {
    val.as_array()
        .map(|arr| {
            arr.iter()
                .map(|param| AbiEntryParam {
                    indexed: param["indexed"].as_bool().unwrap_or(false),
                    name: param["name"].as_str().unwrap_or("").to_owned(),
                    field_type: param["type"].as_str().unwrap_or("").to_owned(),
                    ..Default::default()
                })
                .collect()
        })
        .unwrap_or_default()
}

fn json_to_abi(json: &serde_json::Value) -> Abi {
    let entries: Vec<AbiEntry> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|abi| {
            let mut entry = AbiEntry::new();
            entry.set_anonymous(abi["anonymous"].as_bool().unwrap_or(false));
            entry.set_constant(abi["constant"].as_bool().unwrap_or(false));
            entry.set_name(abi["name"].as_str().unwrap_or("").to_owned());
            entry.set_payable(abi["payable"].as_bool().unwrap_or(false));
            entry.set_stateMutability(translate_state_mutablility(&abi["stateMutability"]));
            entry.set_field_type(translate_abi_type(&abi["type"]));

            entry.set_inputs(translate_abi_entry_params(&abi["inputs"]).into());
            entry.set_outputs(translate_abi_entry_params(&abi["outputs"]).into());

            entry
        })
        .collect();

    Abi {
        entrys: entries.into(),
        ..Default::default()
    }
}

pub fn create_contract(matches: &ArgMatches) -> Result<(), Error> {
    let owner_address: Address = matches.value_of("OWNER").expect("required in cli.yml; qed").parse()?;
    let abi = match matches.value_of("abi") {
        Some(fname) if Path::new(fname).exists() => {
            let raw_json = fs::read_to_string(Path::new(fname))?;
            json_to_abi(&serde_json::from_str(&raw_json)?)
        }
        Some(raw_json) if raw_json.trim_start().starts_with("[") => json_to_abi(&serde_json::from_str(&raw_json)?),
        Some(_) => {
            return Err(Error::Runtime("can not determine ABI format"));
        }
        _ => unreachable!("required in cli.yml; qed"),
    };
    if matches.is_present("libraries") {
        return Err(Error::Runtime("--libraries unimplemented"));
    }
    let mut bytecode: Vec<u8> = match matches.value_of("code") {
        Some(fname) if Path::new(fname).exists() => {
            let bytecode_hex = fs::read_to_string(fname)?;
            Vec::from_hex(bytecode_hex)?
        }
        Some(bytecode_hex) => {
            Vec::from_hex(bytecode_hex).map_err(|_| Error::Runtime("can not determine bytecode format"))?
        }
        _ => unreachable!("required in cli.yml; qed"),
    };

    let types = abi
        .get_entrys()
        .iter()
        .find(|entry| entry.get_field_type() == AbiEntryType::Constructor)
        .map(|entry| {
            entry
                .get_inputs()
                .iter()
                .map(|param| param.get_field_type())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut data = match (matches.values_of("ARGS"), matches.value_of("data")) {
        (Some(args), None) => {
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
            abi::encode_params(&types, &values)?
        }
        (None, Some(data_hex)) => Vec::from_hex(data_hex)?,
        (None, None) => vec![],
        (_, _) => unreachable!("set conflicts in cli.yml; qed"),
    };

    bytecode.append(&mut data);

    let mut new_contract = SmartContract::new();
    new_contract.set_bytecode(bytecode);
    new_contract.set_abi(abi);

    new_contract.set_origin_address(owner_address.as_ref().to_owned());
    if let Some(name) = matches.value_of("name") {
        new_contract.set_name(name.to_owned());
    }

    let percent = matches
        .value_of("user-resource-percent")
        .expect("has default in cli.yml; qed")
        .parse()?;
    new_contract.set_consume_user_resource_percent(percent);

    if let Some(val) = matches.value_of("energy-limit") {
        new_contract.set_origin_energy_limit(val.parse()?);
    }

    let mut create_contract = CreateSmartContract::new();
    create_contract.set_owner_address(owner_address.as_ref().to_owned());
    create_contract.set_new_contract(new_contract);

    trx::TransactionHandler::handle(create_contract, matches)
        .map_raw_transaction(|raw| raw.set_fee_limit(1_000_000))
        .run()
}

pub fn update_contract_settings(matches: &ArgMatches) -> Result<(), Error> {
    let owner_address: Address = matches.value_of("OWNER").expect("required in cli.yml; qed").parse()?;
    let contract = matches
        .value_of("CONTRACT")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong contract address format"))?;

    match (
        matches.value_of("user-resource-percent"),
        matches.value_of("energy-limit"),
    ) {
        (Some(res_percent), None) => {
            let update_contract = UpdateSettingContract {
                owner_address: owner_address.to_bytes().to_owned(),
                contract_address: contract.to_bytes().to_owned(),
                consume_user_resource_percent: res_percent.parse()?,
                ..Default::default()
            };
            trx::TransactionHandler::handle(update_contract, matches).run()
        }
        (None, Some(limit)) => {
            let update_contract = UpdateEnergyLimitContract {
                owner_address: owner_address.to_bytes().to_owned(),
                contract_address: contract.to_bytes().to_owned(),
                origin_energy_limit: limit.parse()?,
                ..Default::default()
            };
            trx::TransactionHandler::handle(update_contract, matches).run()
        }
        _ => Err(Error::Runtime(
            "one of --user-resource-percent or --energy-limit required",
        )),
    }
}

pub fn clear_contract_abi(matches: &ArgMatches) -> Result<(), Error> {
    let owner_address: Address = matches.value_of("OWNER").expect("required in cli.yml; qed").parse()?;
    let contract = matches
        .value_of("CONTRACT")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong contract address format"))?;

    let clear_contract = ClearABIContract {
        owner_address: owner_address.to_bytes().to_owned(),
        contract_address: contract.to_bytes().to_owned(),
        ..Default::default()
    };
    trx::TransactionHandler::handle(clear_contract, matches).run()
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("create", Some(arg_matches)) => create_contract(arg_matches),
        ("call", Some(arg_matches)) => call::main(arg_matches),
        ("update", Some(arg_matches)) => update_contract_settings(arg_matches),
        ("clear_abi", Some(arg_matches)) => clear_contract_abi(arg_matches),
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
