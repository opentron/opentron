use clap::ArgMatches;
use hex::ToHex;
use keys::{Address, Private};
use proto::api_grpc::Wallet;
use proto::core::{
    Transaction, Transaction_Contract as Contract, Transaction_Contract_ContractType as ContractType,
    Transaction_raw as TransactionRaw, TransferContract,
};
use protobuf::well_known_types::Any;
use protobuf::Message;
use serde_json::json;

use crate::commands::wallet::sign_digest;
use crate::error::Error;
use crate::utils::client;
use crate::utils::crypto;
use crate::utils::jsont;
use crate::utils::trx;

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

    let client = client::new_grpc_client()?;

    let trx_contract = TransferContract {
        owner_address: sender.to_bytes().to_owned(),
        to_address: recipient.to_bytes().to_owned(),
        amount: trx::parse_amount(amount, true)?,
        ..Default::default()
    };

    // packing contract
    let mut any = Any::new();
    any.set_type_url("type.googleapis.com/protocol.TransferContract".to_owned());
    any.set_value(trx_contract.write_to_bytes()?);

    let mut contract = Contract::new();
    contract.set_field_type(ContractType::TransferContract);
    contract.set_parameter(any);
    if let Some(val) = matches.value_of("permission-id") {
        contract.set_Permission_id(val.parse()?);
    }

    let mut raw = TransactionRaw::new();
    raw.set_contract(vec![contract].into());
    raw.set_data(memo.into());

    let expiration = matches
        .value_of("expiration")
        .expect("has default value in cli.yml; qed")
        .parse::<i64>()?;
    raw.set_expiration(trx::timestamp_millis() + 1000 * expiration);

    // fill ref_block info
    let ref_block = client::get_latest_block(&client)?;
    let ref_block_number = ref_block.get_block_header().get_raw_data().number;
    raw.set_ref_block_bytes(vec![
        ((ref_block_number & 0xff00) >> 8) as u8,
        (ref_block_number & 0xff) as u8,
    ]);
    raw.set_ref_block_hash(ref_block.blockid[8..16].to_owned());
    raw.set_timestamp(trx::timestamp_millis());

    let txid = crypto::sha256(&raw.write_to_bytes()?);

    // signature
    let mut signatures: Vec<Vec<u8>> = Vec::new();
    if !matches.is_present("skip-sign") {
        let signature = if let Some(raw_addr) = matches.value_of("account") {
            let addr = raw_addr.parse::<Address>()?;
            eprintln!("! Signing using wallet key from --account {:}", addr);
            sign_digest(&txid, &addr)?
        } else if let Some(raw_key) = matches.value_of("private-key") {
            eprintln!("! Signing using raw private key from --private-key");
            let priv_key = raw_key.parse::<Private>()?;
            priv_key.sign_digest(&txid)?[..].to_owned()
        } else {
            eprintln!("! Signing using wallet key {:}", sender);
            sign_digest(&txid, &sender)?
        };
        signatures.push(signature);
    }

    let mut req = Transaction::new();
    req.set_raw_data(raw);
    req.set_signature(signatures.into());

    eprintln!("sender:    {:}", sender);
    eprintln!("recipient: {:}", recipient);
    eprintln!("TX: {:}", txid.encode_hex::<String>());

    // skip-sign implies dont-broadcast
    if matches.is_present("skip-sign") || matches.is_present("dont-broadcast") {
        let mut json = serde_json::to_value(&req)?;
        jsont::fix_transaction(&mut json)?;
        json["raw_data_hex"] = json!(req.get_raw_data().write_to_bytes()?.encode_hex::<String>());
        json["txID"] = json!(txid.encode_hex::<String>());
        println!("{:}", serde_json::to_string_pretty(&json)?);
    } else {
        let (_, payload, _) = client.broadcast_transaction(Default::default(), req).wait()?;

        let mut result = serde_json::to_value(&payload)?;
        jsont::fix_api_return(&mut result);

        eprintln!("got => {:}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}
