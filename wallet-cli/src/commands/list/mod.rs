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

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("node", _) => list_nodes(),
        ("witness", _) => list_witnesses(),
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
