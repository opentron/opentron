use clap::ArgMatches;
use hex::ToHex;
use keys::Address;
use proto::api_grpc::Wallet;
use proto::core::{
    AccountPermissionUpdateContract, Transaction, Transaction_Contract as Contract,
    Transaction_Contract_ContractType as ContractType, Transaction_raw as TransactionRaw,
};
use protobuf::well_known_types::Any;
use protobuf::Message;
use serde_json::json;
use std::io;
use std::io::Read;

use crate::commands::wallet::sign_digest;
use crate::error::Error;
use crate::utils::client;
use crate::utils::crypto;
use crate::utils::jsont;
use crate::utils::trx;

/// Set account permission info.
fn set_account_permission(name: &str, permission: &str) -> Result<(), Error> {
    let grpc_client = client::new_grpc_client()?;

    let addr = name.parse::<Address>()?;

    let mut permission_info: serde_json::Value = if permission == "-" {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        serde_json::from_str(&buffer)?
    } else {
        serde_json::from_str(permission)?
    };

    // convert from hex byte repr to Vec<u8>
    jsont::revert_permission_info(&mut permission_info);
    permission_info["owner_address"] = json!(addr.as_ref().to_owned());

    let perm_contract: AccountPermissionUpdateContract = serde_json::from_value(permission_info)?;

    // packing contract
    let mut any = Any::new();
    any.set_type_url("type.googleapis.com/protocol.AccountPermissionUpdateContract".to_owned());
    any.set_value(perm_contract.write_to_bytes()?);

    let mut contract = Contract::new();
    contract.set_field_type(ContractType::AccountPermissionUpdateContract);
    contract.set_parameter(any);

    let mut raw = TransactionRaw::new();
    raw.set_contract(vec![contract].into());
    raw.set_expiration(trx::timestamp_millis() + 1000 * 60); // 1min

    // fill ref_block info
    let ref_block = client::get_latest_block(&grpc_client)?;
    let ref_block_number = ref_block.get_block_header().get_raw_data().number;
    raw.set_ref_block_bytes(vec![
        ((ref_block_number & 0xff00) >> 8) as u8,
        (ref_block_number & 0xff) as u8,
    ]);
    raw.set_ref_block_hash(ref_block.blockid[8..16].to_owned());
    raw.set_timestamp(trx::timestamp_millis());

    // signature
    let txid = crypto::sha256(&raw.write_to_bytes()?);
    println!("TX: {:}", txid.encode_hex::<String>());

    println!("... Signing using wallet {:}", addr);
    let signature = sign_digest(&txid, &addr)?;

    let mut req = Transaction::new();
    req.set_raw_data(raw);
    req.set_signature(vec![signature].into());

    println!("sender:    {:}", addr);

    let (_, payload, _) = grpc_client.broadcast_transaction(Default::default(), req).wait()?;

    let mut result = serde_json::to_value(&payload)?;

    if !result["message"].is_null() {
        result["message"] = json!(jsont::bytes_to_string(&result["message"]));
    }

    println!("got => {:}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("account_permission", Some(arg_matches)) => {
            let name = arg_matches
                .value_of("NAME")
                .expect("account name is required is cli.yml; qed");
            let permission = arg_matches.value_of("PERMISSION").expect("required in cli.yml; qed");
            set_account_permission(name, permission)
        }
        _ => unimplemented!(),
    }
}
