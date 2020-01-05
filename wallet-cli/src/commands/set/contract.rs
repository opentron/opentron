use clap::ArgMatches;
use hex::{FromHex, ToHex};
use keys::Address;
use proto::api_grpc::Wallet;
use proto::core::{
    CreateSmartContract, SmartContract, SmartContract_ABI as Abi, SmartContract_ABI_Entry as AbiEntry,
    SmartContract_ABI_Entry_EntryType as AbiEntryType, SmartContract_ABI_Entry_Param as AbiEntryParam,
    SmartContract_ABI_Entry_StateMutabilityType as AbiEntryStateMutabilityType,
};
use protobuf::Message;
use serde_json::json;
use std::fs;
use std::path::Path;

use crate::error::Error;
use crate::utils::client;
use crate::utils::jsont;

#[inline]
fn translate_state_mutablility(val: &serde_json::Value) -> AbiEntryStateMutabilityType {
    match val.as_str().unwrap_or("") {
        "view" | "View" => AbiEntryStateMutabilityType::View,
        "nonpayable" | "Nonpayable" => AbiEntryStateMutabilityType::Nonpayable,
        x => {
            println!("unknown => {}", x);
            unimplemented!()
        }
    }
}

#[inline]
fn translate_abi_type(val: &serde_json::Value) -> AbiEntryType {
    match val.as_str().unwrap_or("") {
        "function" | "Function" => AbiEntryType::Function,
        "event" | "Event" => AbiEntryType::Event,
        _ => unimplemented!(),
    }
}

#[inline]
fn translate_abi_entry_params(val: &serde_json::Value) -> Vec<AbiEntryParam> {
    val.as_array()
        .unwrap()
        .iter()
        .map(|param| AbiEntryParam {
            indexed: param["indexed"].as_bool().unwrap_or(false),
            name: param["name"].as_str().unwrap_or("").to_owned(),
            field_type: param["type"].as_str().unwrap_or("").to_owned(),
            ..Default::default()
        })
        .collect()
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

pub fn run(matches: &ArgMatches) -> Result<(), Error> {
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

    let bytecode: Vec<u8> = match matches.value_of("code") {
        Some(fname) if Path::new(fname).exists() => {
            let bytecode_hex = fs::read_to_string(fname)?;
            Vec::from_hex(bytecode_hex)?
        }
        Some(bytecode_hex) => {
            Vec::from_hex(bytecode_hex).map_err(|_| Error::Runtime("can not determine bytecode format"))?
        }
        _ => unreachable!("required in cli.yml; qed"),
    };

    let mut new_contract = SmartContract::new();
    new_contract.set_bytecode(bytecode);
    new_contract.set_abi(abi);

    new_contract.set_origin_address(owner_address.as_ref().to_owned());
    if let Some(name) = matches.value_of("name") {
        new_contract.set_name(name.to_owned());
    }

    let percent = matches
        .value_of("user-resource-percent")
        .expect("required in cli.yml; qed")
        .parse()?;
    new_contract.set_consume_user_resource_percent(percent);

    let mut req = CreateSmartContract::new();
    req.set_owner_address(owner_address.as_ref().to_owned());
    req.set_new_contract(new_contract);

    // creating transaction
    let (_, transaction_ext, _) = client::new_grpc_client()?
        .deploy_contract(Default::default(), req)
        .wait()?;

    let mut json = serde_json::to_value(&transaction_ext)?;
    jsont::fix_transaction_ext(&mut json)?;

    if json["result"]["result"].as_bool().unwrap_or(false) {
        json["transaction"]["raw_data_hex"] = json!(transaction_ext
            .get_transaction()
            .get_raw_data()
            .write_to_bytes()?
            .encode_hex::<String>());

        println!("{}", serde_json::to_string_pretty(&json["transaction"])?);
        Ok(())
    } else {
        eprintln!("{}", serde_json::to_string_pretty(&json)?);
        Err(Error::Runtime("can not create transaction"))
    }
}
