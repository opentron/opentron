use clap::ArgMatches;
use hex::{FromHex, ToHex};
use proto::api::{BytesMessage, EmptyMessage, NumberMessage};
use proto::api_grpc::{Wallet, WalletClient};
use serde_json::json;
use std::net::ToSocketAddrs;
use std::sync::Arc;

use grpc::ClientStub;

const RPC_HOST: &str = "grpc.trongrid.io:50051";

fn new_grpc_client() -> WalletClient {
    let host = RPC_HOST
        .to_socket_addrs()
        .expect("resolve host")
        .next()
        .expect("host resolve result");

    let grpc_client = Arc::new(
        grpc::Client::new_plain(&host.ip().to_string(), host.port(), Default::default()).expect("grpc client"),
    );
    WalletClient::with_client(grpc_client)
}

fn block_info() {
    let client = new_grpc_client();

    let req = EmptyMessage::new();
    let resp = client.get_node_info(Default::default(), req);

    let (_, payload, _) = resp.wait().expect("grpc request");

    println!("{}", serde_json::to_string_pretty(&payload).expect("resp json parse"));
}

fn get_block(id_or_num: &str) {
    let client = new_grpc_client();

    if id_or_num.starts_with("0000") {
        let mut req = BytesMessage::new();
        req.value = Vec::from_hex(id_or_num).expect("hex bytes parse");
        let resp = client.get_block_by_id(Default::default(), req);

        let (_, payload, _) = resp.wait().expect("grpc request");
        //println!("{:?}", payload);
        println!("{}", serde_json::to_string_pretty(&payload).expect("resp json parse"));
    } else {
        let mut req = NumberMessage::new();
        req.num = id_or_num.parse().expect("block number format");
        let resp = client.get_block_by_num2(Default::default(), req);

        let (_, payload, _) = resp.wait().expect("grpc request");
        // println!("{:?}", payload);
        println!("{}", serde_json::to_string_pretty(&payload).expect("resp json parse"));
    }
}

fn get_transaction(id: &str) {
    let client = new_grpc_client();

    let mut req = BytesMessage::new();
    req.value = Vec::from_hex(id).expect("hex bytes parse");
    let resp = client.get_transaction_by_id(Default::default(), req);

    let (_, payload, _) = resp.wait().expect("grpc request");
    println!("{}", serde_json::to_string_pretty(&payload).expect("resp json parse"));
}

fn get_transaction_info(id: &str) {
    let client = new_grpc_client();

    let mut req = BytesMessage::new();
    req.value = Vec::from_hex(id).expect("hex bytes parse");
    let resp = client.get_transaction_info_by_id(Default::default(), req);

    let (_, payload, _) = resp.wait().expect("grpc request");
    // serde_json::to_string_pretty(&payload).expect("resp json parse")
    let json = serde_json::to_value(&payload).expect("resp json parse");
    let result = json!({
        "id": json!(json_bytes_to_hex_string(&json["id"])),
        "fee": json["fee"],
        "blockNumber": json["blockNumber"],
        "blockTimeStamp": json["blockTimeStamp"],
        "contractResult": json!(
            json["contractResult"]
                .as_array()
                .unwrap()
                .iter()
                .map(json_bytes_to_hex_string)
                .collect::<Vec<_>>()
        ),
        "contract_address": json!(json_bytes_to_hex_string(&json["contract_address"])),
        "receipt": json["receipt"],
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

fn json_bytes_to_hex_string(val: &serde_json::Value) -> String {
    val.as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as u8)
        .collect::<Vec<_>>()
        .encode_hex()
}

pub fn main(matches: &ArgMatches) -> Result<(), String> {
    match matches.subcommand() {
        ("info", _) => {
            block_info();
            Ok(())
        }
        ("block", Some(block_matches)) => {
            let block = block_matches
                .value_of("BLOCK")
                .expect("block is required in cli.yml; qed");
            get_block(block);
            Ok(())
        }
        ("transaction", Some(tr_matches)) => {
            let id = tr_matches
                .value_of("ID")
                .expect("transaction is required in cli.yml; qed");
            get_transaction(id);
            Ok(())
        }
        ("transaction_info", Some(tr_matches)) => {
            let id = tr_matches
                .value_of("ID")
                .expect("transaction is required in cli.yml; qed");
            get_transaction_info(id);
            Ok(())
        }
        _ => Err("error parsing command line".to_owned()),
    }
}
