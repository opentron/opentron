use clap::ArgMatches;
use hex::FromHex;
use keys::Address;
use proto::api::{BytesMessage, EmptyMessage, NumberMessage};
use proto::api_grpc::Wallet;
use proto::core::Account;
use serde_json::json;

use crate::error::Error;
use crate::utils::client;
use crate::utils::jsont;

mod contract;

fn node_info() -> Result<(), Error> {
    let (_, payload, _) = client::GRPC_CLIENT
        .get_node_info(Default::default(), EmptyMessage::new())
        .wait()?;
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn get_block(matches: &ArgMatches) -> Result<(), Error> {
    let mut block = match matches.value_of("BLOCK") {
        Some(id) if id.starts_with("0000") => {
            let mut req = BytesMessage::new();
            req.value = Vec::from_hex(id)?;
            let (_, payload, _) = client::GRPC_CLIENT.get_block_by_id(Default::default(), req).wait()?;
            serde_json::to_value(&payload)?
        }
        Some(num) => {
            let mut req = NumberMessage::new();
            req.num = num.parse()?;
            let (_, payload, _) = client::GRPC_CLIENT.get_block_by_num2(Default::default(), req).wait()?;
            serde_json::to_value(&payload)?
        }
        None => {
            let (_, payload, _) = client::GRPC_CLIENT
                .get_now_block(Default::default(), EmptyMessage::new())
                .wait()?;
            serde_json::to_value(&payload)?
        }
    };
    if block["block_header"].is_null() {
        return Err(Error::Runtime("block not found on chain"));
    }

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

    let (_, payload, _) = client::GRPC_CLIENT
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

/// Get account infomation.
fn get_account(name: &str) -> Result<(), Error> {
    let mut req = Account::new();
    let addr = name.parse::<Address>()?;
    req.set_address(addr.as_bytes().to_owned());
    // FIXME: account name not supported
    // req.set_account_name(name.as_bytes().to_owned());

    let (_, payload, _) = client::GRPC_CLIENT.get_account(Default::default(), req).wait()?;
    if payload.get_address().is_empty() {
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Err(Error::Runtime("account not found on chain"));
    }

    let mut account = serde_json::to_value(&payload)?;
    jsont::fix_account(&mut account);

    println!("{}", serde_json::to_string_pretty(&account)?);
    eprintln!("! Address(Base58Check) = {:}", addr);
    Ok(())
}

/// Get account permission info.
fn get_account_permission(name: &str) -> Result<(), Error> {
    let mut req = Account::new();
    let addr = name.parse::<Address>()?;
    req.set_address(addr.as_bytes().to_owned());

    let (_, payload, _) = client::GRPC_CLIENT.get_account(Default::default(), req).wait()?;
    if payload.get_address().is_empty() {
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Err(Error::Runtime("account not found on chain"));
    }

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
    let addr = name.parse::<Address>()?;
    req.set_address(addr.as_bytes().to_owned());

    let (_, payload, _) = client::GRPC_CLIENT
        .get_account_resource(Default::default(), req)
        .wait()?;

    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn get_proposal_by_id(id: &str) -> Result<(), Error> {
    // NOTE: id should be encoded to 8 bytes as i64
    let mut req = BytesMessage::new();
    let id_hex = format!("{:016x}", id.parse::<i64>()?);
    req.set_value(Vec::from_hex(id_hex)?);

    let (_, payload, _) = client::GRPC_CLIENT.get_proposal_by_id(Default::default(), req).wait()?;
    if payload.get_proposal_id() == 0 {
        return Err(Error::Runtime("proposal not found on chain"));
    }

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

fn get_asset_by_id(id: &str) -> Result<(), Error> {
    // NOTE: id should be encoded to 8 bytes as i64
    let mut req = BytesMessage::new();
    req.set_value(id.as_bytes().to_owned());

    let (_, payload, _) = client::GRPC_CLIENT
        .get_asset_issue_by_id(Default::default(), req)
        .wait()?;
    if payload.get_id().is_empty() {
        return Err(Error::Runtime("asset not found"));
    }
    let mut asset = serde_json::to_value(&payload)?;
    jsont::fix_asset_issue_contract(&mut asset);
    println!("{}", serde_json::to_string_pretty(&asset)?);
    Ok(())
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("node", _) => node_info(),
        ("block", Some(arg_matches)) => get_block(arg_matches),
        ("transaction", Some(tr_matches)) => {
            let id = tr_matches.value_of("ID").expect("required in cli.yml; qed");
            get_transaction(id)
        }
        ("transaction_info", Some(tr_matches)) => {
            let id = tr_matches.value_of("ID").expect("required in cli.yml; qed");
            get_transaction_info(id)
        }
        ("account", Some(arg_matches)) => {
            let name = arg_matches.value_of("NAME").expect("required is cli.yml; qed");
            get_account(name)
        }
        ("account_permission", Some(arg_matches)) => {
            let name = arg_matches.value_of("NAME").expect("required is cli.yml; qed");
            get_account_permission(name)
        }
        ("account_resource", Some(arg_matches)) => {
            let name = arg_matches.value_of("NAME").expect("required is cli.yml; qed");
            get_account_resource(name)
        }
        ("contract", Some(arg_matches)) => {
            let addr = arg_matches.value_of("ADDR").expect("required is cli.yml; qed");
            contract::run(addr)
        }
        ("proposal", Some(arg_matches)) => {
            let id = arg_matches.value_of("ID").expect("required in cli.yml; qed");
            get_proposal_by_id(&id)
        }
        ("asset", Some(arg_matches)) => {
            let id = arg_matches.value_of("ID").expect("required in cli.yml; qed");
            get_asset_by_id(&id)
        }
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
