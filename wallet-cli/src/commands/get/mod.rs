use chrono::{Local, TimeZone};
use clap::ArgMatches;
use hex::FromHex;
use keys::Address;
use proto::api::{BytesMessage, EmptyMessage, NumberMessage};
use proto::api_grpc::Wallet;
use proto::core::Account;
use serde_json::json;
use std::collections::HashSet;

use crate::error::Error;
use crate::utils::client;
use crate::utils::jsont;
use crate::utils::trx;

mod contract;
mod transaction;

fn node_info() -> Result<(), Error> {
    let (_, payload, _) = client::GRPC_CLIENT
        .get_node_info(Default::default(), EmptyMessage::new())
        .wait()?;
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn visit_node(ip: &str, edges: &mut HashSet<(String, String)>) -> Result<(), Error> {
    let mut stack = vec![ip.to_owned()];
    let mut visited = HashSet::new();

    while let Some(self_ip) = stack.pop() {
        visited.insert(self_ip.clone());

        eprintln!("({})visiting ... {}", edges.len(), self_ip);
        if let Ok(grpc_client) = client::new_grpc_client(&format!("{}:50051", self_ip)) {
            if let Ok((_, node_info, _)) = grpc_client
                .get_node_info(Default::default(), EmptyMessage::new())
                .wait()
            {
                eprintln!(
                    "p2p version: {}, node version: {}",
                    node_info.get_configNodeInfo().get_p2pVersion(),
                    node_info.get_configNodeInfo().get_codeVersion()
                );
                for peer in node_info.get_peerInfoList() {
                    let peer_ip = peer.get_host();
                    let edge = (self_ip.to_owned(), peer_ip.to_owned());
                    if !edges.contains(&edge) {
                        edges.insert(edge);
                    }
                    if !visited.contains(peer_ip) {
                        stack.push(peer_ip.to_owned());
                    }
                }
            }
        }
    }
    Ok(())
}

fn get_node_graph() -> Result<(), Error> {
    let mut edges: HashSet<(String, String)> = HashSet::new();
    let (_, node_info, _) = client::GRPC_CLIENT
        .get_node_info(Default::default(), EmptyMessage::new())
        .wait()?;

    for peer in node_info.get_peerInfoList() {
        let ip = peer.get_host();
        let _ = visit_node(ip, &mut edges);
    }
    println!("digraph G {{");
    for (from, to) in edges {
        println!("  {:?} -> {:?};", from, to);
    }
    println!("}}");
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

    jsont::fix_block(&mut block)?;

    println!("{:}", serde_json::to_string_pretty(&block)?);
    eprintln!("! Block Number: {}", block["block_header"]["raw_data"]["number"]);
    eprintln!(
        "! Number of Transactions: {}",
        block["transactions"].as_array().unwrap().len()
    );
    eprintln!(
        "! Generated At: {}",
        Local.timestamp(
            block["block_header"]["raw_data"]["timestamp"].as_i64().unwrap() / 1_000,
            0
        )
    );
    let _ = block["block_header"]["raw_data"]["witness_address"]
        .as_str()
        .unwrap()
        .parse::<Address>()
        .map(|addr| {
            eprintln!("! Witness: {}", addr);
        });

    Ok(())
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

    eprintln!(
        "! Type = {:?}{}",
        payload.field_type,
        if payload.is_witness { " | Witness" } else { "" }
    );
    eprintln!("! Address(Base58Check) = {:}", addr);
    eprintln!("! Created At: {}", Local.timestamp(payload.create_time / 1_000, 0));

    if payload.balance != 0 {
        eprintln!(
            "! Balance = {}",
            trx::format_amount_with_surfix(payload.balance, "TRX", 6)
        );
    }
    if payload.allowance != 0 {
        eprintln!(
            "! Unwithdrawn SR Reward = {}",
            trx::format_amount_with_surfix(payload.allowance, "TRX", 6)
        );
    }

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
    if payload.get_freeNetLimit() == 0 {
        return Err(Error::Runtime("account not found on chain"));
    }
    eprintln!("! Free Bandwith Usage: {}/{}", payload.freeNetUsed, payload.freeNetLimit);
    eprintln!(
        "! Energy By Freezing    1_TRX = {:.5}",
        payload.TotalEnergyLimit as f64 / payload.TotalEnergyWeight as f64
    );
    eprintln!(
        "! Bandwidth By Freezing 1_TRX = {:.5}",
        payload.TotalNetLimit as f64 / payload.TotalNetWeight as f64
    );
    Ok(())
}

fn get_proposal_by_id(id: &str) -> Result<(), Error> {
    // NOTE: id should be encoded to 8 bytes as i64
    let mut req = BytesMessage::new();
    req.set_value((id.parse::<i64>()?.to_be_bytes()[..]).to_owned());

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

fn get_reward_info(addr: &str) -> Result<(), Error> {
    let addr = addr.parse::<Address>()?;
    let mut req = BytesMessage::new();
    req.set_value(addr.as_bytes().to_owned());

    let (_, val, _) = client::GRPC_CLIENT.get_reward_info(Default::default(), req).wait()?;
    println!("value = {}", val.get_num());
    Ok(())
}

fn get_brokerage_info(addr: &str) -> Result<(), Error> {
    let addr = addr.parse::<Address>()?;
    let mut req = BytesMessage::new();
    req.set_value(addr.as_bytes().to_owned());

    let (_, val, _) = client::GRPC_CLIENT.get_brokerage_info(Default::default(), req).wait()?;
    println!("sharing percent = {}%", 100 - val.get_num());
    println!("kept percent    = {}%", val.get_num());
    Ok(())
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("node", _) => node_info(),
        ("node_graph", _) => get_node_graph(),
        ("block", Some(arg_matches)) => get_block(arg_matches),
        ("transaction", Some(tr_matches)) => {
            let id = tr_matches.value_of("ID").expect("required in cli.yml; qed");
            transaction::get_transaction(id)
        }
        ("transaction_info", Some(tr_matches)) => {
            let id = tr_matches.value_of("ID").expect("required in cli.yml; qed");
            transaction::get_transaction_info(id)
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
        ("reward", Some(arg_matches)) => {
            let addr = arg_matches.value_of("ADDR").expect("required in cli.yml; qed");
            get_reward_info(&addr)
        }
        ("brokerage", Some(arg_matches)) => {
            let addr = arg_matches.value_of("ADDR").expect("required in cli.yml; qed");
            get_brokerage_info(&addr)
        }
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
