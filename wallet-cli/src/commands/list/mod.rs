use clap::ArgMatches;
use proto::api::EmptyMessage;
use proto::api_grpc::Wallet;
use serde_json::json;

use crate::error::Error;
use crate::utils::client::new_grpc_client;
use crate::utils::jsont;

fn list_nodes() -> Result<(), Error> {
    let client = new_grpc_client()?;
    let req = EmptyMessage::new();
    let (_, payload, _) = client.list_nodes(Default::default(), req).wait()?;

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
    let client = new_grpc_client()?;
    let req = EmptyMessage::new();
    let (_, payload, _) = client.list_witnesses(Default::default(), req).wait()?;
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
    let client = new_grpc_client()?;

    let req = EmptyMessage::new();
    let (_, payload, _) = client.get_asset_issue_list(Default::default(), req).wait()?;
    let mut assets = serde_json::to_value(&payload)?;

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

    println!("{}", serde_json::to_string_pretty(&assets["assetIssue"])?);
    Ok(())
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("node", _) => list_nodes(),
        ("witness", _) => list_witnesses(),
        ("asset", _) => list_assets(),
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
