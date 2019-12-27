use clap::ArgMatches;
use keys::{Address, Private};
use proto::api_grpc::Wallet;
use proto::core::{
    Transaction, Transaction_Contract as Contract, Transaction_Contract_ContractType as ContractType,
    Transaction_raw as TransactionRaw,
    TransferContract,
};
use serde_json::json;
use protobuf::well_known_types::Any;
use protobuf::Message;
use hex::ToHex;

use crate::utils::client::new_grpc_client;
use crate::utils::jsont;

pub fn main(matches: &ArgMatches) -> Result<(), String> {
    let sender = matches
        .value_of("SENDER")
        .ok_or("required in cli.yml; qed".to_owned())
        .and_then(|s| s.parse::<Address>().map_err(|e| e.to_string()))?;
    let recipient = matches
        .value_of("RECIPIENT")
        .ok_or("required in cli.yml; qed".to_owned())
        .and_then(|s| s.parse::<Address>().map_err(|e| e.to_string()))?;
    let amount = matches.value_of("AMOUNT").expect("required in cli.yml; qed");
    let memo = matches.value_of("MEMO").unwrap_or("");

    let client = new_grpc_client();

    let mut trx_contract = TransferContract::new();
    trx_contract.set_owner_address(sender.to_bytes().to_owned());
    trx_contract.set_to_address(recipient.to_bytes().to_owned());
    trx_contract.set_amount(amount.parse().expect("transfer amount"));

    println!("pb inner => {:?}", &trx_contract);

    let mut any = Any::new();
    any.set_type_url("type.googleapis.com/protocol.TransferContract".to_owned());
    any.set_value(trx_contract.write_to_bytes().unwrap());

    let mut contract = Contract::new();
    contract.set_field_type(ContractType::TransferContract);
    contract.set_parameter(any);

    let mut raw = TransactionRaw::new();
    raw.set_contract(vec![contract].into());
    raw.set_data(memo.into());
    // raw.set_expiration(v: i64)

    let priv_key = "d705fc17c82942f85848ab522e42d986279028d09d12ad881bdc0e1327031976"
            .parse::<Private>()
            .unwrap();
    let sign = priv_key.sign(&raw.write_to_bytes().unwrap()).map_err(|e| e.to_string())?;

    let mut req = Transaction::new();
    req.set_raw_data(raw);
    req.set_signature(vec![(&sign[..]).to_owned()].into());

    let test = req.write_to_bytes().unwrap();
    println!("send hex => {:}", test.encode_hex::<String>());
    println!("send pb => {:?}", &req);

    println!("sender:    {:}", sender);
    println!("recipient: {:}", recipient);

    let resp = client.broadcast_transaction(Default::default(), req);

    let (_, payload, _) = resp.wait().expect("grpc request");

    let mut result = serde_json::to_value(&payload).expect("resp json serilization");

    if !result["message"].is_null() {
        result["message"] = json!(jsont::bytes_to_string(&result["message"]));
    }

    println!("got => {:}", serde_json::to_string_pretty(&result).unwrap());
    Ok(())
}
