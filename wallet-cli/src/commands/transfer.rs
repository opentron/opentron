use chrono::Utc;
use clap::ArgMatches;
use hex::ToHex;
use keys::{Address, Private};
use proto::api::{BlockExtention, NumberMessage};
use proto::api_grpc::{Wallet, WalletClient};
use proto::core::{
    Transaction, Transaction_Contract as Contract, Transaction_Contract_ContractType as ContractType,
    Transaction_raw as TransactionRaw, TransferContract,
};
use protobuf::well_known_types::Any;
use protobuf::Message;
use serde_json::json;

use crate::error::Error;
use crate::utils::client::new_grpc_client;
use crate::utils::crypto;
use crate::utils::jsont;

fn timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}

fn get_latest_block(client: &WalletClient) -> Result<BlockExtention, Error> {
    let mut req = NumberMessage::new();
    req.set_num(1);
    let (_, resp, _) = client.get_block_by_latest_num2(Default::default(), req).wait()?;
    resp.block
        .into_iter()
        .next()
        .ok_or(Error::Runtime("no latest block retrieved"))
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    let sender = matches
        .value_of("SENDER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong sender address format"))?;
    let recipient = matches
        .value_of("RECIPIENT")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong recipient address format"))?;
    let amount = matches.value_of("AMOUNT").expect("required in cli.yml; qed");
    let memo = matches.value_of("MEMO").unwrap_or("");

    let priv_key = matches
        .value_of("priv-key")
        .and_then(|k| k.parse::<Private>().ok())
        .ok_or(Error::Runtime("private key(--priv-key) required"))?;

    let client = new_grpc_client()?;

    let mut trx_contract = TransferContract::new();
    trx_contract.set_owner_address(sender.to_bytes().to_owned());
    trx_contract.set_to_address(recipient.to_bytes().to_owned());
    trx_contract.set_amount(amount.parse()?);

    let mut any = Any::new();
    any.set_type_url("type.googleapis.com/protocol.TransferContract".to_owned());
    any.set_value(trx_contract.write_to_bytes()?);

    let mut contract = Contract::new();
    contract.set_field_type(ContractType::TransferContract);
    contract.set_parameter(any);

    let mut raw = TransactionRaw::new();
    raw.set_contract(vec![contract].into());
    raw.set_data(memo.into());
    raw.set_expiration(timestamp_millis() + 1000 * 60); // 1min

    // fill ref_block info
    let ref_block = get_latest_block(&client)?;
    let ref_block_number = ref_block.get_block_header().get_raw_data().number;
    raw.set_ref_block_bytes(vec![
        ((ref_block_number & 0xff00) >> 8) as u8,
        (ref_block_number & 0xff) as u8,
    ]);
    raw.set_ref_block_hash(ref_block.blockid[8..16].to_owned());
    raw.set_timestamp(timestamp_millis());

    // signature
    println!("TX: {:}", crypto::sha256(&raw.write_to_bytes()?).encode_hex::<String>());
    let sign = priv_key.sign(&raw.write_to_bytes()?)?;

    let mut req = Transaction::new();
    req.set_raw_data(raw);
    req.set_signature(vec![(&sign[..]).to_owned()].into());

    println!("sender:    {:}", sender);
    println!("recipient: {:}", recipient);

    let resp = client.broadcast_transaction(Default::default(), req);
    let (_, payload, _) = resp.wait()?;

    let mut result = serde_json::to_value(&payload)?;

    if !result["message"].is_null() {
        result["message"] = json!(jsont::bytes_to_string(&result["message"]));
    }

    println!("got => {:}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
