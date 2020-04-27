use chrono::{Local, TimeZone};
use hex::FromHex;
use keys::Address;
use proto::api::BytesMessage;
use proto::api_grpc::Wallet;
use proto::core::{
    Transaction_Contract_ContractType as ContractType, Transaction_Result_code as ResultCode,
    Transaction_Result_contractResult as ContractResult,
};
use protobuf::Message;

use crate::error::Error;
use crate::utils::abi;
use crate::utils::client;
use crate::utils::jsont;
use crate::utils::trx;

pub fn get_transaction(id: &str) -> Result<(), Error> {
    let mut req = BytesMessage::new();
    req.value = Vec::from_hex(id)?;

    let (_, mut payload, _) = client::GRPC_CLIENT
        .get_transaction_by_id(Default::default(), req)
        .wait()?;

    let mut transaction = serde_json::to_value(&payload)?;
    if transaction["raw_data"].is_null() {
        return Err(Error::Runtime("transaction not found"));
    }
    jsont::fix_transaction(&mut transaction)?;
    println!("{}", serde_json::to_string_pretty(&transaction).unwrap());

    if !payload.get_raw_data().get_data().is_empty() {
        eprintln!(
            "! Data: {:?}",
            String::from_utf8_lossy(payload.get_raw_data().get_data())
        );
    }

    eprintln!(
        "! Timestamp: {}",
        Local.timestamp(
            payload.get_raw_data().timestamp / 1_000,
            (payload.get_raw_data().timestamp % 1_000 * 1_000_000) as _
        )
    );

    let sender = trx::extract_owner_address_from_parameter(payload.get_raw_data().get_contract()[0].get_parameter())?;
    eprintln!("! Sender Address(base58check):   {}", sender);

    if payload.get_raw_data().get_contract()[0].get_field_type() == ContractType::TriggerSmartContract
        && payload.get_ret()[0].get_ret() == ResultCode::SUCESS
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

    // NOTE: when calculating bandwidth, `Transaction.ret` must be excluded.
    payload.clear_ret();
    eprintln!(
        "! Bandwidth: {}",
        payload.compute_size() as usize + trx::MAX_RESULT_SIZE_IN_TX
    );

    Ok(())
}

pub fn get_transaction_info(id: &str) -> Result<(), Error> {
    let mut req = BytesMessage::new();
    req.value = Vec::from_hex(id)?;

    let (_, payload, _) = client::GRPC_CLIENT
        .get_transaction_info_by_id(Default::default(), req)
        .wait()?;

    if payload.get_id().is_empty() {
        return Err(Error::Runtime("transaction not found"));
    }
    let mut json = serde_json::to_value(&payload)?;
    jsont::fix_transaction_info(&mut json);

    println!("{}", serde_json::to_string_pretty(&json)?);
    if payload.get_receipt().net_usage > 0 {
        eprintln!("! Free/Frozen Bandwidth Used: {}", payload.get_receipt().net_usage);
    }
    if payload.get_receipt().net_fee > 0 {
        eprintln!(
            "! Burnt for Bandwidth: {} TRX",
            payload.get_receipt().net_fee as f64 / 1_000_000.0
        );
    }

    if payload.get_receipt().energy_usage_total > 0 {
        eprintln!("! Total Energy: {}", payload.get_receipt().energy_usage_total);
    }
    if payload.get_receipt().energy_usage > 0 {
        eprintln!("! Frozen Energy Used: {}", payload.get_receipt().energy_usage);
    }
    if payload.get_receipt().energy_fee > 0 {
        eprintln!(
            "! Burnt for Energy: {} TRX",
            payload.get_receipt().energy_fee as f64 / 1_000_000.0
        );
    }
    if payload.get_receipt().origin_energy_usage > 0 {
        eprintln!(
            "! Contract Owner's Energy Used: {}",
            payload.get_receipt().origin_energy_usage
        );
    }

    if payload.fee > 0 {
        eprintln!("! Total Fee: {} TRX", payload.fee as f64 / 1_000_000.0);
    }

    if [ContractResult::OUT_OF_TIME, ContractResult::JVM_STACK_OVER_FLOW].contains(&payload.get_receipt().result) {
        eprintln!("!! All of Fee Limit Spent!");
    }

    Ok(())
}

fn pprint_contract_call_data(contract: &Address, data: &str) -> Result<(), Error> {
    let abi = trx::get_contract_abi(contract)?;
    let fnhash = hex::decode(&data[..8])?;
    abi.iter()
        .find(|entry| abi::fnhash(&abi::entry_to_method_name(entry)) == fnhash[..])
        .ok_or(Error::Runtime("ABI not found, can not parse result"))
        .and_then(|entry| {
            eprintln!("! {}", abi::entry_to_method_name_pretty(entry)?);
            eprintln!(
                "!          {} [{}]",
                abi::entry_to_method_name(entry),
                hex::encode(fnhash)
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
