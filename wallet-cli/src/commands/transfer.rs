use clap::ArgMatches;
use proto::api::{BytesMessage, EmptyMessage, NumberMessage, Return_response_code as ResponseCode};
use proto::api_grpc::Wallet;
use proto::core::{Transaction, Transaction_Contract_ContractType as ContractType, Transaction_raw as TransactionRaw};
use serde_json::json;

use crate::utils::client::new_grpc_client;
use crate::utils::jsont;

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
        result["message"] = json!(jsont::bytes_to_hex_string(&result["message"]));
    }

    println!("got => {:}", serde_json::to_string_pretty(&result).unwrap());
    Ok(())
}
