use clap::ArgMatches;
use hex::ToHex;
use keys::Address;
use proto::api_grpc::Wallet;
use proto::core::{FreezeBalanceContract, ResourceCode, UnfreezeBalanceContract};
use protobuf::Message;
use serde_json::json;

use crate::error::Error;
use crate::utils::client;
use crate::utils::jsont;
use crate::utils::trx;

pub fn freeze_main(matches: &ArgMatches) -> Result<(), Error> {
    let from = matches
        .value_of("FROM")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong from address format"))?;
    let receiver = matches
        .value_of("RECEIVER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong receiver address format"))?;
    let amount = matches.value_of("AMOUNT").expect("required in cli.yml; qed");
    let duration = matches.value_of("duration").expect("has default in cli.yml; qed");

    // if receiver is self, receiver_address must be empty
    let freeze_contract = FreezeBalanceContract {
        owner_address: from.to_bytes().to_owned(),
        receiver_address: if receiver == from {
            vec![]
        } else {
            receiver.to_bytes().to_owned()
        },
        frozen_balance: trx::parse_amount(amount, true)?,
        frozen_duration: duration.parse()?,
        resource: if matches.is_present("energy") {
            ResourceCode::ENERGY
        } else {
            ResourceCode::BANDWIDTH
        },
        ..Default::default()
    };

    // creating transaction
    let (_, transaction_ext, _) = client::new_grpc_client()?
        .freeze_balance2(Default::default(), freeze_contract)
        .wait()?;

    let mut json = serde_json::to_value(&transaction_ext)?;
    jsont::fix_transaction_ext(&mut json)?;

    if json["result"]["result"].as_bool().unwrap_or(false) {
        json["transaction"]["raw_data_hex"] = json!(transaction_ext
            .get_transaction()
            .get_raw_data()
            .write_to_bytes()?
            .encode_hex::<String>());

        println!("{}", serde_json::to_string_pretty(&json["transaction"])?);
        Ok(())
    } else {
        eprintln!("{}", serde_json::to_string_pretty(&json)?);
        Err(Error::Runtime("can not create transaction"))
    }
}

pub fn unfreeze_main(matches: &ArgMatches) -> Result<(), Error> {
    let from = matches
        .value_of("FROM")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong from address format"))?;
    let receiver = matches
        .value_of("RECEIVER")
        .and_then(|s| s.parse::<Address>().ok())
        .ok_or(Error::Runtime("wrong receiver address format"))?;

    // if receiver is self, receiver_address must be empty
    let unfreeze_contract = UnfreezeBalanceContract {
        owner_address: from.to_bytes().to_owned(),
        receiver_address: if receiver == from {
            vec![]
        } else {
            receiver.to_bytes().to_owned()
        },
        resource: if matches.is_present("energy") {
            ResourceCode::ENERGY
        } else {
            ResourceCode::BANDWIDTH
        },
        ..Default::default()
    };

    // creating transaction
    let (_, transaction_ext, _) = client::new_grpc_client()?
        .unfreeze_balance2(Default::default(), unfreeze_contract)
        .wait()?;

    let mut json = serde_json::to_value(&transaction_ext)?;
    jsont::fix_transaction_ext(&mut json)?;

    if json["result"]["result"].as_bool().unwrap_or(false) {
        json["transaction"]["raw_data_hex"] = json!(transaction_ext
            .get_transaction()
            .get_raw_data()
            .write_to_bytes()?
            .encode_hex::<String>());

        println!("{}", serde_json::to_string_pretty(&json["transaction"])?);
        Ok(())
    } else {
        eprintln!("{}", serde_json::to_string_pretty(&json)?);
        Err(Error::Runtime("can not create transaction"))
    }
}
