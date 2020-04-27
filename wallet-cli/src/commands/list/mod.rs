use clap::ArgMatches;
use futures::executor;
use proto::api::EmptyMessage;
use serde_json::json;

use crate::error::Error;
use crate::utils::client;
use crate::utils::jsont;

fn list_nodes() -> Result<(), Error> {
    let req = EmptyMessage::new();
    let payload = executor::block_on(client::GRPC_CLIENT.list_nodes(Default::default(), req).drop_metadata())?;

    let mut nodes = serde_json::to_value(&payload)?;
    nodes["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|node| {
            node["address"]["host"] = json!(jsont::bytes_to_string(&node["address"]["host"]));
        })
        .last();
    println!("{}", serde_json::to_string_pretty(&nodes["nodes"])?);
    Ok(())
}

fn list_witnesses() -> Result<(), Error> {
    let req = EmptyMessage::new();
    let payload = executor::block_on(
        client::GRPC_CLIENT
            .list_witnesses(Default::default(), req)
            .drop_metadata(),
    )?;
    let mut witnesses = serde_json::to_value(&payload)?;
    witnesses["witnesses"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|witness| {
            witness["address"] = json!(jsont::bytes_to_hex_string(&witness["address"]));
        })
        .last();
    println!("{}", serde_json::to_string_pretty(&witnesses["witnesses"])?);
    Ok(())
}

fn list_assets() -> Result<(), Error> {
    let req = EmptyMessage::new();
    let payload = executor::block_on(
        client::GRPC_CLIENT
            .get_asset_issue_list(Default::default(), req)
            .drop_metadata(),
    )?;
    let mut assets = serde_json::to_value(&payload)?;

    assets["assetIssue"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(jsont::fix_asset_issue_contract)
        .last();

    println!("{}", serde_json::to_string_pretty(&assets["assetIssue"])?);
    Ok(())
}

pub fn list_proposals() -> Result<(), Error> {
    let payload = executor::block_on(
        client::GRPC_CLIENT
            .list_proposals(Default::default(), EmptyMessage::new())
            .drop_metadata(),
    )?;
    let mut proposals = serde_json::to_value(&payload)?;

    proposals["proposals"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|proposal| {
            proposal["proposer_address"] = json!(jsont::bytes_to_hex_string(&proposal["proposer_address"]));
            proposal["approvals"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .map(|val| {
                    *val = json!(jsont::bytes_to_hex_string(val));
                })
                .last();
        })
        .last();
    println!("{}", serde_json::to_string_pretty(&proposals["proposals"])?);
    Ok(())
}

pub fn list_parameters() -> Result<(), Error> {
    let payload = executor::block_on(
        client::GRPC_CLIENT
            .get_chain_parameters(Default::default(), EmptyMessage::new())
            .drop_metadata(),
    )?;
    let parameters = serde_json::to_value(&payload)?;
    println!("{}", serde_json::to_string_pretty(&parameters["chainParameter"])?);
    Ok(())
}

pub fn list_exchanges() -> Result<(), Error> {
    let payload = executor::block_on(
        client::GRPC_CLIENT
            .list_exchanges(Default::default(), EmptyMessage::new())
            .drop_metadata(),
    )?;
    let mut exchanges = serde_json::to_value(&payload)?;
    exchanges["exchanges"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|ex| {
            ex["creator_address"] = json!(jsont::bytes_to_hex_string(&ex["creator_address"]));
            ex["first_token_id"] = json!(jsont::bytes_to_string(&ex["first_token_id"]));
            ex["second_token_id"] = json!(jsont::bytes_to_string(&ex["second_token_id"]));
        })
        .last();
    println!("{}", serde_json::to_string_pretty(&exchanges["exchanges"])?);
    Ok(())
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("node", _) => list_nodes(),
        ("witness", _) => list_witnesses(),
        ("asset", _) => list_assets(),
        ("proposal", _) => list_proposals(),
        ("parameter", _) => list_parameters(),
        ("exchange", _) => list_exchanges(),
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
