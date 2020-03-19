use chrono::{Local, TimeZone};
use hex::{FromHex, ToHex};
use keys::Address;
use proto::api::BytesMessage;
use proto::api_grpc::Wallet;
use proto::core::{Transaction_Contract_ContractType as ContractType, Transaction_Result_code as ResultCode};

use crate::error::Error;
use crate::utils::abi;
use crate::utils::client;
use crate::utils::jsont;
use crate::utils::trx;

pub fn get_transaction(id: &str) -> Result<(), Error> {
    let mut req = BytesMessage::new();
    req.value = Vec::from_hex(id)?;

    let (_, payload, _) = client::GRPC_CLIENT
        .get_transaction_by_id(Default::default(), req)
        .wait()?;

    let mut transaction = serde_json::to_value(&payload)?;
    if transaction["raw_data"].is_null() {
        return Err(Error::Runtime("transaction not found"));
    }
    jsont::fix_transaction(&mut transaction)?;
    println!("{}", serde_json::to_string_pretty(&transaction).unwrap());

    eprintln!(
        "! Timestamp: {}",
        Local.timestamp(
            payload.get_raw_data().timestamp / 1_000,
            (payload.get_raw_data().timestamp % 1_000 * 1_000_000) as _
        )
    );

    let sender = trx::extract_owner_address_from_parameter(payload.get_raw_data().get_contract()[0].get_parameter())?;
    eprintln!("! Sender Address(base58check):   {}", sender);

    if payload.get_raw_data().get_contract()[0].get_field_type() == ContractType::TriggerSmartContract &&
        payload.get_ret()[0].get_ret() == ResultCode::SUCESS
    {
        let contract_address = transaction["raw_data"]["contract"][0]["parameter"]["value"]["contract_address"]
            .as_str()
            .ok_or(Error::Runtime("unreachable field"))
            .and_then(|s| s.parse::<Address>().map_err(Error::from))?;
        let data = transaction["raw_data"]["contract"][0]["parameter"]["value"]["data"]
            .as_str()
            .unwrap();
        eprintln!("! Contract Address(base58check): {}", contract_address);
        pprint_contract_call_data(&contract_address, data)?;
    }
    Ok(())
}

pub fn get_transaction_info(id: &str) -> Result<(), Error> {
    let mut req = BytesMessage::new();
    req.value = Vec::from_hex(id)?;

    let (_, payload, _) = client::GRPC_CLIENT
        .get_transaction_info_by_id(Default::default(), req)
        .wait()?;

    if !payload.get_id().is_empty() {
        let mut json = serde_json::to_value(&payload)?;
        jsont::fix_transaction_info(&mut json);
        println!("{}", serde_json::to_string_pretty(&json)?);
        Ok(())
    } else {
        Err(Error::Runtime("transaction not found"))
    }
}

fn pprint_contract_call_data(contract: &Address, data: &str) -> Result<(), Error> {
    let abi = trx::get_contract_abi(contract)?;
    let fnhash = Vec::from_hex(&data[..8])?;
    abi.iter()
        .find(|entry| abi::fnhash(&abi::entry_to_method_name(entry)) == fnhash[..])
        .ok_or(Error::Runtime("ABI not found, can not parse result"))
        .and_then(|entry| {
            eprintln!("! {}", abi::entry_to_method_name_pretty(entry)?);
            eprintln!(
                "!          {} [{}]",
                abi::entry_to_method_name(entry),
                fnhash.encode_hex::<String>(),
            );
            let types = abi::entry_to_input_types(&entry);
            let params = abi::decode_params(&types, &data[8..])?;
            if !types.is_empty() {
                eprintln!("! Arguments:");
                for (input, param) in entry.get_inputs().iter().zip(params.iter()) {
                    eprintln!("  {}: {} = {}", input.get_name(), input.get_field_type(), param);
                }
            }
            Ok(())
        })
}
