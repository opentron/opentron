//! Sign a transaction, for multisig or save for broadcast later

use clap::ArgMatches;
use hex::{FromHex, ToHex};
use keys::{Address, Private, Public};
use proto::api_grpc::Wallet;
use proto::core::{Transaction, Transaction_raw as TransactionRaw};
use protobuf::Message;
use serde_json::json;
use sha3::{Digest, Keccak256};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::commands::wallet::sign_digest;
use crate::error::Error;
use crate::utils::client;
use crate::utils::crypto;
use crate::utils::jsont;
use crate::utils::trx;
use crate::CHAIN_ID;

// Message Signature
//
// HEADER + length in digits(normally a '32') + message
// keccak256
// then sign using ethereum method, i.e. adding 27 to recovery id

const TRX_MESSAGE_HEADER: &[u8; 22] = b"\x19TRON Signed Message:\n";

pub fn sign_message(matches: &ArgMatches) -> Result<(), Error> {
    let message = matches.value_of("TRANSACTION").expect("required in cli.yml; qed");

    if !matches.is_present("account") && !matches.is_present("private-key") {
        return Err(Error::Runtime("-k/-K is required for sign a message"));
    }
    if matches.is_present("broadcast") || matches.is_present("skip-sign") {
        return Err(Error::Runtime("-b/-s is not required for sign a message"));
    }

    let origin_message = if message.starts_with("0x") {
        hex::decode(&message[2..])?
    } else {
        message.to_owned().into_bytes()
    };
    if origin_message.len() != 32 {
        eprintln!("!! Warning: message is not 32 bytes long")
    }

    let mut raw_message = TRX_MESSAGE_HEADER.to_vec();
    raw_message.extend(origin_message.len().to_string().into_bytes());

    eprintln!("! Raw message header => {:?}", String::from_utf8_lossy(&raw_message));
    eprintln!("! Hex message body   => {}", hex::encode(&origin_message));

    raw_message.extend(origin_message);

    let mut hasher = Keccak256::new();
    hasher.input(&raw_message);
    let digest = hasher.result();

    assert_eq!(digest.len(), 32);

    let mut signature = if let Some(raw_key) = matches.value_of("private-key") {
        eprintln!("! Signing using raw private key from --private-key");
        let priv_key = raw_key.parse::<Private>()?;
        priv_key.sign_digest(&digest)?[..].to_owned()
    } else {
        let owner_address = matches
            .value_of("account")
            .and_then(|addr| addr.parse().ok())
            .ok_or(Error::Runtime("can not determine owner address for signing"))?;
        eprintln!("! Signing using wallet key {:}", owner_address);
        sign_digest(&digest, &owner_address)?
    };

    // yep, the magic
    signature[64] += 27;

    println!("! Signature = {}", hex::encode(signature));

    Ok(())
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    let trx = matches.value_of("TRANSACTION").expect("required in cli.yml; qed");

    let trx_raw: String = match trx {
        fname if Path::new(fname).exists() => fs::read_to_string(Path::new(fname))?,
        "-" => {
            eprintln!("Loading transaction from STDIN...");
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
        // assume json
        _ if trx.trim_start().starts_with("{") => trx.to_owned(),
        _ => {
            return sign_message(matches);
        }
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
    jsont::fix_transaction_raw(&mut trx_json)?;
    eprintln!("{:}", serde_json::to_string_pretty(&trx_json)?);

    // signature
    let txid = crypto::sha256(&raw.write_to_bytes()?);
    let digest = if let Some(chain_id) = unsafe { CHAIN_ID } {
        let mut raw = (&txid[..]).to_owned();
        raw.extend(Vec::from_hex(chain_id)?);
        crypto::sha256(&raw)
    } else {
        txid
    };

    if !signatures.is_empty() {
        eprintln!("! Already signed by:");
        for sig in &signatures {
            let public = Public::recover_digest(&digest[..], &FromHex::from_hex(sig)?)?;
            eprintln!("  {}", Address::from_public(&public));
        }
    }

    if !matches.is_present("skip-sign") {
        let signature = if let Some(raw_key) = matches.value_of("private-key") {
            eprintln!("! Signing using raw private key from --private-key");
            let priv_key = raw_key.parse::<Private>()?;
            priv_key.sign_digest(&digest)?[..].to_owned()
        } else {
            let owner_address = matches
                .value_of("account")
                .and_then(|addr| addr.parse().ok())
                .or_else(|| trx::extract_owner_address_from_parameter(raw.contract[0].get_parameter()).ok())
                .ok_or(Error::Runtime("can not determine owner address for signing"))?;
            eprintln!("! Signing using wallet key {:}", owner_address);
            sign_digest(&digest, &owner_address)?
        };

        let sig_hex = signature.encode_hex::<String>();
        if signatures.contains(&sig_hex) {
            return Err(Error::Runtime("already signed by this key"));
        } else {
            signatures.push(sig_hex);
        }
    }

    let ret = json!({
        "raw_data": trx_json,
        "raw_data_hex": json!(raw_data.encode_hex::<String>()),
        "signature": json!(signatures),
        "txID": json!(txid.encode_hex::<String>()),
    });

    println!("{:}", serde_json::to_string_pretty(&ret)?);

    if matches.is_present("broadcast") {
        eprintln!("! Broadcasting transaction ...");
        let mut req = Transaction::new();
        req.set_raw_data(raw);
        req.set_signature(
            signatures
                .into_iter()
                .map(|sig| Vec::from_hex(sig).unwrap())
                .collect::<Vec<_>>()
                .into(),
        );

        let (_, payload, _) = client::GRPC_CLIENT
            .broadcast_transaction(Default::default(), req)
            .wait()?;
        let mut result = serde_json::to_value(&payload)?;
        jsont::fix_api_return(&mut result);
        eprintln!("got => {:}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}
