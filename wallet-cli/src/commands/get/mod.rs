use clap::ArgMatches;
use proto::api::EmptyMessage;
use proto::api_grpc::{Wallet, WalletClient};
use std::net::ToSocketAddrs;
use std::sync::Arc;

use grpc::ClientStub;

const RPC_HOST: &str = "grpc.trongrid.io:50051";

fn block_info() {
    let host = RPC_HOST
        .to_socket_addrs()
        .expect("resolve host")
        .next()
        .expect("non-empty resolve list");

    let grpc_client = Arc::new(
        grpc::Client::new_plain(&host.ip().to_string(), host.port(), Default::default()).expect("grpc client"),
    );
    let client = WalletClient::with_client(grpc_client);

    let req = EmptyMessage::new();
    let resp = client.get_node_info(Default::default(), req);

    let (_, payload, _) = resp.wait().expect("request ok");

    println!(
        "{}",
        serde_json::to_string_pretty(&payload).expect("resp json parse ok")
    );
}

pub fn main(matches: &ArgMatches) -> Result<(), String> {
    match matches.subcommand() {
        ("info", _) => {
            block_info();
            Ok(())
        }
        _ => Err("error parsing command line".to_owned()),
    }
}
