use hex::ToHex;
use keys::Address;
use proto::api::BytesMessage;
use proto::api_grpc::Wallet;
use protobuf::ProtobufEnum;
use serde_json::json;

use crate::error::Error;
use crate::utils::abi;
use crate::utils::client;
use crate::utils::crypto;
use crate::utils::jsont;

pub fn run(addr: &str) -> Result<(), Error> {
    let address: Address = addr.parse()?;
    let mut req = BytesMessage::new();
    req.set_value(address.as_bytes().to_owned());

    let (_, mut payload, _) = client::GRPC_CLIENT.get_contract(Default::default(), req).wait()?;
    if payload.get_contract_address().is_empty() {
        return Err(Error::Runtime("contract not found on chain"));
    }

    let mut contract = serde_json::to_value(&payload)?;
    contract["contract_address"] = json!(jsont::bytes_to_hex_string(&contract["contract_address"]));
    contract["origin_address"] = json!(jsont::bytes_to_hex_string(&contract["origin_address"]));
    contract["bytecode"] = json!(jsont::bytes_to_hex_string(&contract["bytecode"]));
    contract["code_hash"] = json!(jsont::bytes_to_hex_string(&contract["code_hash"]));

    println!("{}", serde_json::to_string_pretty(&contract)?);
    // re-order abi entry by types
    let mut abi = payload.take_abi();
    abi.entrys.sort_by_key(|entry| entry.get_field_type().value());
    pprint_abi_entries(&abi)?;
    Ok(())
}

// NOTE: there is a typo in pb: abi.`entrys`
fn pprint_abi_entries(abi: &::proto::core::SmartContract_ABI) -> Result<(), Error> {
    for entry in abi.entrys.iter() {
        let method = abi::entry_to_method_name(entry);
        if entry.get_field_type() == ::proto::core::SmartContract_ABI_Entry_EntryType::Function {
            let fnhash = abi::fnhash(&method);
            eprintln!(
                "{:}\n    => {:}: {:}",
                abi::entry_to_method_name_pretty(entry)?,
                (&fnhash[..]).encode_hex::<String>(),
                method
            );
        } else if entry.get_field_type() == ::proto::core::SmartContract_ABI_Entry_EntryType::Event {
            // will show in `topic` field
            let event_hash = crypto::keccak256(method.as_bytes());
            eprintln!(
                "{:}\n    => {}",
                abi::entry_to_method_name_pretty(entry)?,
                hex::encode(&event_hash)
            );
        } else {
            eprintln!("{:}", abi::entry_to_method_name_pretty(entry)?,);
        }
    }
    Ok(())
}
