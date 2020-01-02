//! Sign a transaction, for multisig or save for broadcast later

use clap::ArgMatches;
use hex::{FromHex, ToHex};
use keys::{Address, Private, Public};
use proto::api_grpc::Wallet;
use proto::core::{Transaction, Transaction_raw as TransactionRaw};
use protobuf::Message;
use serde_json::json;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::commands::wallet::sign_digest;
use crate::error::Error;
use crate::utils::client::new_grpc_client;
use crate::utils::crypto;
use crate::utils::jsont;
use crate::utils::trx;

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    let broadcast_after_signing = matches.is_present("broadcast");

    let trx = matches.value_of("TRANSACTION").expect("required in cli.yml; qed");

    let trx_raw: String = match trx {
        fname if Path::new(fname).exists() => fs::read_to_string(Path::new(fname))?,
        "-" => {
            eprintln!("Loading transaction from STDIN...");
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
        _ => trx.to_owned(),
    };

    let mut signatures = Vec::new();

    let raw_data: Vec<u8> = if trx_raw.trim_start().starts_with("{") {
        let trx = serde_json::from_str::<serde_json::Value>(&trx_raw)?;
        if !trx["signature"].is_null() {
            trx["signature"]
                .as_array()
                .unwrap()
                .iter()
                .map(|sig| signatures.push(sig.as_str().expect("malformed json").to_owned()))
                .last();
        }
        Vec::from_hex(trx["raw_data_hex"].as_str().expect("raw_data_hex field required"))?
    } else {
        Vec::from_hex(&trx_raw)?
    };

    let raw = protobuf::parse_from_bytes::<TransactionRaw>(&raw_data)?;
    let mut trx_json = serde_json::to_value(&raw)?;
    jsont::fix_transaction_raw(&mut trx_json);
    eprintln!("{:}", serde_json::to_string_pretty(&trx_json)?);

    let owner_address = trx::extract_owner_address_from_parameter(raw.contract[0].get_parameter());

    // signature
    let txid = crypto::sha256(&raw.write_to_bytes()?);

    if !signatures.is_empty() {
        eprintln!("Already signed by:");
        for sig in &signatures {
            let public = Public::recover_digest(&txid[..], &FromHex::from_hex(sig)?)?;
            eprintln!("  {}", Address::from_public(&public));
        }
    }

    let signature: Vec<u8> = if let Some(raw_key) = matches.value_of("private-key") {
        let private = raw_key.parse::<Private>()?;
        eprintln!("! Signing with --private-key {:}", Address::from_private(&private));
        private.sign_digest(&txid)?[..].to_owned()
    } else {
        eprintln!("! Signing using wallet key {:}", owner_address);
        sign_digest(&txid, &owner_address)?
    };

    let sig_hex = signature.encode_hex::<String>();

    if signatures.contains(&sig_hex) {
        return Err(Error::Runtime("already signed by this key"));
    } else {
        signatures.push(sig_hex);
    }

    let ret = json!({
        "raw_data": trx_json,
        "raw_data_hex": json!(raw_data.encode_hex::<String>()),
        "signatures": json!(signatures),
        "txID": json!(txid.encode_hex::<String>()),
    });

    println!("{:}", serde_json::to_string_pretty(&ret)?);

    if broadcast_after_signing {
        eprintln!("Broadcasting transaction ...");
        let mut req = Transaction::new();
        req.set_raw_data(raw);
        req.set_signature(
            signatures
                .into_iter()
                .map(|sig| Vec::from_hex(sig).unwrap())
                .collect::<Vec<_>>()
                .into(),
        );

        let (_, payload, _) = new_grpc_client()?
            .broadcast_transaction(Default::default(), req)
            .wait()?;
        let mut result = serde_json::to_value(&payload)?;
        jsont::fix_api_return(&mut result);
        eprintln!("got => {:}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}
