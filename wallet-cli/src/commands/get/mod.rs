use clap::ArgMatches;
use hex::FromHex;
use keys::Address;
use proto::api::{BytesMessage, EmptyMessage, NumberMessage};
use proto::api_grpc::Wallet;
use proto::core::Account;
use serde_json::json;

use crate::error::Error;
use crate::utils::client::new_grpc_client;
use crate::utils::jsont;

mod contract;

fn node_info() -> Result<(), Error> {
    let (_, payload, _) = new_grpc_client()?
        .get_node_info(Default::default(), EmptyMessage::new())
        .wait()?;
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn get_block(matches: &ArgMatches) -> Result<(), Error> {
    let client = new_grpc_client()?;

    let mut block = match matches.value_of("BLOCK") {
        Some(id) if id.starts_with("0000") => {
            let mut req = BytesMessage::new();
            req.value = Vec::from_hex(id)?;
            let (_, payload, _) = client.get_block_by_id(Default::default(), req).wait()?;
            serde_json::to_value(&payload)?
        }
        Some(num) => {
            let mut req = NumberMessage::new();
            req.num = num.parse()?;
            let (_, payload, _) = client.get_block_by_num2(Default::default(), req).wait()?;
            serde_json::to_value(&payload)?
        }
        None => {
            let (_, payload, _) = client.get_now_block(Default::default(), EmptyMessage::new()).wait()?;
            serde_json::to_value(&payload)?
        }
    };

    // get_block_by_id won't return blockid
    if block["blockid"].is_array() {
        block["blockid"] = json!(jsont::bytes_to_hex_string(&block["blockid"]));
    }

    for key in &["parentHash", "txTrieRoot", "witness_address"] {
        block["block_header"]["raw_data"][key] =
            json!(jsont::bytes_to_hex_string(&block["block_header"]["raw_data"][key]));
    }
    block["block_header"]["witness_signature"] =
        json!(jsont::bytes_to_hex_string(&block["block_header"]["witness_signature"]));

    block["transactions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|mut transaction| {
            // NOTE: structual difference of get_block requests
            if transaction["txid"].is_array() {
                transaction["txid"] = json!(jsont::bytes_to_hex_string(&transaction["txid"]));
                transaction = &mut transaction["transaction"];
            }
            jsont::fix_transaction(transaction)?;
            Ok(())
        })
        .collect::<Result<Vec<_>, Error>>()?;

    println!("{:}", serde_json::to_string_pretty(&block)?);
    Ok(())
}

fn get_transaction(id: &str) -> Result<(), Error> {
    let mut req = BytesMessage::new();
    req.value = Vec::from_hex(id)?;

    let (_, payload, _) = new_grpc_client()?
        .get_transaction_by_id(Default::default(), req)
        .wait()?;

    let mut transaction = serde_json::to_value(&payload)?;
    if transaction["raw_data"].is_null() {
        Err(Error::Runtime("transaction not found"))
    } else {
        jsont::fix_transaction(&mut transaction)?;
        println!("{}", serde_json::to_string_pretty(&transaction).unwrap());
        Ok(())
    }
}

fn get_transaction_info(id: &str) -> Result<(), Error> {
    let mut req = BytesMessage::new();
    req.value = Vec::from_hex(id)?;

    let (_, payload, _) = new_grpc_client()?
        .get_transaction_info_by_id(Default::default(), req)
        .wait()?;

    let json = serde_json::to_value(&payload)?;
    let result = json!({
        "id": json!(jsont::bytes_to_hex_string(&json["id"])),
        "fee": json["fee"],
        "blockNumber": json["blockNumber"],
        "blockTimeStamp": json["blockTimeStamp"],
        "contractResult": json!(
            json["contractResult"]
                .as_array()
                .unwrap()
                .iter()
                .map(jsont::bytes_to_hex_string)
                .collect::<Vec<_>>()
        ),
        "contract_address": json!(jsont::bytes_to_hex_string(&json["contract_address"])),
        "receipt": json["receipt"],
    });
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

/// Get account infomation.
fn get_account(name: &str) -> Result<(), Error> {
    let mut req = Account::new();
    let addr = name.parse::<Address>()?;
    req.set_address(addr.to_bytes().to_owned());
    // FIXME: account name not supported
    // req.set_account_name(name.as_bytes().to_owned());

    let (_, payload, _) = new_grpc_client()?.get_account(Default::default(), req).wait()?;

    let mut account = serde_json::to_value(&payload)?;

    // first byte of address
    if account["address"][0].is_null() {
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Err(Error::Runtime("account not found on chain"));
    }

    jsont::fix_account(&mut account);

    println!("{}", serde_json::to_string_pretty(&account)?);
    Ok(())
}

/// Get account permission info.
fn get_account_permission(name: &str) -> Result<(), Error> {
    let mut req = Account::new();
    let addr = name.parse::<Address>()?;
    req.set_address(addr.to_bytes().to_owned());

    let (_, payload, _) = new_grpc_client()?.get_account(Default::default(), req).wait()?;

    let mut account = serde_json::to_value(&payload)?;
    jsont::fix_account(&mut account);

    let permission_info = json!({
        "owner": account["owner_permission"],
        "witness": account["witness_permission"],
        "actives": account["active_permission"],
    });

    println!("{}", serde_json::to_string_pretty(&permission_info)?);
    Ok(())
}

/// Get account energy and bandwidth infomation.
fn get_account_resource(name: &str) -> Result<(), Error> {
    let mut req = Account::new();
    let addr = name.parse::<Address>().expect("addr format");
    req.set_address(addr.to_bytes().to_owned());

    let (_, payload, _) = new_grpc_client()?
        .get_account_resource(Default::default(), req)
        .wait()
        .expect("grpc request");

    println!("{}", serde_json::to_string_pretty(&payload).expect("resp json parse"));
    Ok(())
}

fn get_proposal_by_id(id: &str) -> Result<(), Error> {
    // NOTE: id should be encoded to 8 bytes as i64
    let mut req = BytesMessage::new();
    let id_hex = format!("{:016x}", id.parse::<i64>()?);
    req.set_value(Vec::from_hex(id_hex)?);

    let (_, payload, _) = new_grpc_client()?.get_proposal_by_id(Default::default(), req).wait()?;
    let mut proposal = serde_json::to_value(&payload)?;
    proposal["proposer_address"] = json!(jsont::bytes_to_hex_string(&proposal["proposer_address"]));
    proposal["approvals"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|addr| *addr = json!(jsont::bytes_to_hex_string(addr)))
        .last();

    println!("{}", serde_json::to_string_pretty(&proposal)?);
    Ok(())
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("node", _) => node_info(),
        ("block", Some(arg_matches)) => get_block(arg_matches),
        ("transaction", Some(tr_matches)) => {
            let id = tr_matches
                .value_of("ID")
                .expect("transaction is required in cli.yml; qed");
            get_transaction(id)
        }
        ("transaction_info", Some(tr_matches)) => {
            let id = tr_matches
                .value_of("ID")
                .expect("transaction is required in cli.yml; qed");
            get_transaction_info(id)
        }
        ("account", Some(arg_matches)) => {
            let name = arg_matches
                .value_of("NAME")
                .expect("account name is required is cli.yml; qed");
            get_account(name)
        }
        ("account_permission", Some(arg_matches)) => {
            let name = arg_matches
                .value_of("NAME")
                .expect("account name is required is cli.yml; qed");
            get_account_permission(name)
        }
        ("account_resource", Some(arg_matches)) => {
            let name = arg_matches
                .value_of("NAME")
                .expect("account name is required is cli.yml; qed");
            get_account_resource(name)
        }
        ("contract", Some(arg_matches)) => {
            let addr = arg_matches
                .value_of("ADDR")
                .expect("address is required is cli.yml; qed");
            contract::run(addr)
        }
        ("proposal", Some(arg_matches)) => {
            let id = arg_matches.value_of("ID").expect("required in cli.yml; qed");
            get_proposal_by_id(&id)
        }
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
