use clap::ArgMatches;
use hex::{FromHex};
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
    let client = new_grpc_client()?;

    let req = EmptyMessage::new();
    let (_, payload, _) = client.get_node_info(Default::default(), req).wait()?;
    println!("{}", serde_json::to_string_pretty(&payload).expect("resp json parse"));
    Ok(())
}

fn get_block(id_or_num: &str) -> Result<(), Error> {
    let client = new_grpc_client()?;

    let mut block = {
        if id_or_num.starts_with("0000") {
            let mut req = BytesMessage::new();
            req.value = Vec::from_hex(id_or_num).expect("hex bytes parse");
            let resp = client.get_block_by_id(Default::default(), req);

            let (_, payload, _) = resp.wait().expect("grpc request");
            serde_json::to_value(&payload).expect("pd json")
        } else {
            let mut req = NumberMessage::new();
            req.num = id_or_num.parse().expect("block number format");
            let resp = client.get_block_by_num2(Default::default(), req);

            let (_, payload, _) = resp.wait().expect("grpc request");
            serde_json::to_value(&payload).expect("pd json")
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
            jsont::fix_transaction(transaction);
        })
        .last();

    println!("{:}", serde_json::to_string_pretty(&block).expect("pb json"));
    Ok(())
}

fn get_transaction(id: &str) -> Result<(), Error> {
    let client = new_grpc_client()?;

    let mut req = BytesMessage::new();
    req.value = Vec::from_hex(id).expect("hex bytes parse");
    let resp = client.get_transaction_by_id(Default::default(), req);

    let (_, payload, _) = resp.wait().expect("grpc request");

    let mut transaction = serde_json::to_value(&payload).expect("resp json serilization");

    jsont::fix_transaction(&mut transaction);

    println!("{}", serde_json::to_string_pretty(&transaction).unwrap());
    Ok(())
}

fn get_transaction_info(id: &str) -> Result<(), Error> {
    let client = new_grpc_client()?;

    let mut req = BytesMessage::new();
    req.value = Vec::from_hex(id).expect("hex bytes parse");
    let resp = client.get_transaction_info_by_id(Default::default(), req);

    let (_, payload, _) = resp.wait().expect("grpc request");
    // serde_json::to_string_pretty(&payload).expect("resp json parse")
    let json = serde_json::to_value(&payload).expect("resp json serilization");
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
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
    Ok(())
}

/// Get account infomation.
fn get_account(name: &str) -> Result<(), Error> {
    let client = new_grpc_client()?;

    let mut req = Account::new();
    let addr = name.parse::<Address>().expect("addr format");
    req.set_address(addr.to_bytes().to_owned());
    // FIXME: account name not supported
    // req.set_account_name(name.as_bytes().to_owned());

    let resp = client.get_account(Default::default(), req);

    let (_, payload, _) = resp.wait().expect("grpc request");
    let mut account = serde_json::to_value(&payload).expect("resp json serilization");

    // first byte of address
    if account["address"][0].is_null() {
        eprintln!("error: not found!");
        println!("{}", serde_json::to_string_pretty(&payload).expect("resp json parse"));
        return Err(Error::Runtime("address not found"));
    }

    account["address"] = json!(jsont::bytes_to_hex_string(&account["address"]));
    account["account_name"] = json!(jsont::bytes_to_string(&account["account_name"]));
    // NOTE: one can remove owner_permission by setting null
    if !account["owner_permission"].is_null() {
        account["owner_permission"]["keys"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .map(|key| {
                key["address"] = json!(jsont::bytes_to_hex_string(&key["address"]));
            })
            .last();
    }

    account["active_permission"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|perm| {
            perm["keys"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .map(|key| {
                    key["address"] = json!(jsont::bytes_to_hex_string(&key["address"]));
                })
                .last();
            perm["operations"] = json!(jsont::bytes_to_hex_string(&perm["operations"]));
        })
        .last();
    // TODO: witness_permission

    println!("{}", serde_json::to_string_pretty(&account).expect("resp json parse"));
    Ok(())
}

/// Get account energy and bandwidth infomation.
fn get_account_resource(name: &str) -> Result<(), Error> {
    let client = new_grpc_client()?;

    let mut req = Account::new();
    let addr = name.parse::<Address>().expect("addr format");
    req.set_address(addr.to_bytes().to_owned());

    let (_, payload, _) = client
        .get_account_resource(Default::default(), req)
        .wait()
        .expect("grpc request");

    println!("{}", serde_json::to_string_pretty(&payload).expect("resp json parse"));
    Ok(())
}

fn get_asset_list() -> Result<(), Error> {
    let client = new_grpc_client()?;

    let req = EmptyMessage::new();
    let resp = client.get_asset_issue_list(Default::default(), req);

    let (_, payload, _) = resp.wait().expect("grpc request");

    let mut assets = serde_json::to_value(&payload).expect("resp json parse");

    assets["assetIssue"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|asset| {
            asset["abbr"] = json!(jsont::bytes_to_string(&asset["abbr"]));
            asset["description"] = json!(jsont::bytes_to_string(&asset["description"]));
            asset["name"] = json!(jsont::bytes_to_string(&asset["name"]));
            asset["url"] = json!(jsont::bytes_to_string(&asset["url"]));
            asset["owner_address"] = json!(jsont::bytes_to_hex_string(&asset["owner_address"]));
        })
        .last();

    println!(
        "{}",
        serde_json::to_string_pretty(&assets["assetIssue"]).expect("resp json parse")
    );
    Ok(())
}


pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("node", _) => node_info(),
        ("block", Some(block_matches)) => {
            let block = block_matches
                .value_of("BLOCK")
                .expect("block is required in cli.yml; qed");
            get_block(block)
        }
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
        ("asset", _) => get_asset_list(),
        _ => Err(Error::Runtime("error parsing command line")),
    }
}
