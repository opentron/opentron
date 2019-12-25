use clap::ArgMatches;
use proto::api::{EmptyMessage, NumberMessage, BytesMessage};
use proto::api_grpc::{Wallet, WalletClient};
use std::net::ToSocketAddrs;
use std::sync::Arc;
use hex::FromHex;

use grpc::ClientStub;

const RPC_HOST: &str = "grpc.trongrid.io:50051";

fn new_grpc_client() -> WalletClient {
    let host = RPC_HOST
        .to_socket_addrs()
        .expect("resolve host")
        .next()
        .expect("non-empty resolve list");

    let grpc_client = Arc::new(
        grpc::Client::new_plain(&host.ip().to_string(), host.port(), Default::default()).expect("grpc client"),
    );
    WalletClient::with_client(grpc_client)
}

fn block_info() {
    let client = new_grpc_client();

    let req = EmptyMessage::new();
    let resp = client.get_node_info(Default::default(), req);

    let (_, payload, _) = resp.wait().expect("request ok");

    println!(
        "{}",
        serde_json::to_string_pretty(&payload).expect("resp json parse ok")
    );
}

fn get_block(id_or_num: &str) {
    let client = new_grpc_client();

    if id_or_num.starts_with("0000") {
        let mut req = BytesMessage::new();
        req.value = Vec::from_hex(id_or_num).expect("hex bytes parse ok");
        let resp = client.get_block_by_id(Default::default(), req);

        let (_, payload, _) = resp.wait().expect("request ok");
        //println!("{:?}", payload);
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).expect("resp json parse ok")
        );
    } else {
        let mut req = NumberMessage::new();
        req.num = id_or_num.parse().expect("number format ok");
        let resp = client.get_block_by_num2(Default::default(), req);

        let (_, payload, _) = resp.wait().expect("request ok");
        // println!("{:?}", payload);
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).expect("resp json parse ok")
        );
    }

}

pub fn main(matches: &ArgMatches) -> Result<(), String> {
    match matches.subcommand() {
        ("info", _) => {
            block_info();
            Ok(())
        }
        ("block", Some(block_matches)) => {
            let block = block_matches.value_of("BLOCK").expect("block is required in cli.yml; qed");
            get_block(block);
            Ok(())
        },
        _ => Err("error parsing command line".to_owned()),
    }
}
