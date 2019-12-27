use clap::ArgMatches;
use grpc::ClientStub;
use proto::api::{BytesMessage, EmptyMessage, NumberMessage, Return_response_code as ResponseCode};
use proto::api_grpc::{Wallet, WalletClient};
use proto::core::{Transaction, Transaction_Contract_ContractType as ContractType, Transaction_raw as TransactionRaw};
use serde_json::json;
use std::net::ToSocketAddrs;
use std::sync::Arc;

const RPC_HOST: &str = "grpc.shasta.trongrid.io:50051";

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

fn json_bytes_to_string(val: &serde_json::Value) -> String {
    val.as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as u8 as char)
        .collect::<String>()
}

pub fn main(matches: &ArgMatches) -> Result<(), String> {
    let sender = matches.value_of("SENDER").expect("required in cli.yml; qed");
    let recipient = matches.value_of("RECIPIENT").expect("required in cli.yml; qed");
    let amount = matches.value_of("AMOUNT").expect("required in cli.yml; qed");

    let memo = matches.value_of("MEMO").unwrap_or("");

    let client = new_grpc_client();
    let mut req = Transaction::new();

    let resp = client.broadcast_transaction(Default::default(), req);

    let (_, payload, _) = resp.wait().expect("grpc request");

    let mut result = serde_json::to_value(&payload).expect("resp json serilization");

    if !result["message"].is_null() {
        result["message"] = json!(json_bytes_to_string(&result["message"]));
    }

    println!("got => {:}", serde_json::to_string_pretty(&result).unwrap());
    Ok(())
}
