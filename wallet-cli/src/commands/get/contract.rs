use hex::ToHex;
use keys::Address;
use proto::api::BytesMessage;
use proto::api_grpc::Wallet;
use serde_json::json;
use std::fmt::Write as FmtWrite;

use crate::error::Error;
use crate::utils::client::new_grpc_client;
use crate::utils::crypto;
use crate::utils::jsont;

pub fn run(addr: &str) -> Result<(), Error> {
    let address: Address = addr.parse()?;
    let mut req = BytesMessage::new();
    req.set_value(address.to_bytes().to_owned());

    let (_, payload, _) = new_grpc_client()?.get_contract(Default::default(), req).wait()?;
    let mut contract = serde_json::to_value(&payload)?;

    contract["contract_address"] = json!(jsont::bytes_to_hex_string(&contract["contract_address"]));
    contract["origin_address"] = json!(jsont::bytes_to_hex_string(&contract["origin_address"]));
    contract["bytecode"] = json!(jsont::bytes_to_hex_string(&contract["bytecode"]));
    contract["code_hash"] = json!(jsont::bytes_to_hex_string(&contract["code_hash"]));

    println!("{}", serde_json::to_string_pretty(&contract)?);
    pprint_abi_entries(payload.get_abi())?;
    Ok(())
}

// NOTE: is pb, it is abi.`entrys`
fn pprint_abi_entries(abi: &::proto::core::SmartContract_ABI) -> Result<(), Error> {
    for entry in abi.entrys.iter() {
        let mut pretty = match entry.get_field_type() {
            ::proto::core::SmartContract_ABI_Entry_EntryType::Function => "function".to_owned(),
            ::proto::core::SmartContract_ABI_Entry_EntryType::Event => "event".to_owned(),
            _ => continue,
        };
        write!(pretty, " {:}", entry.get_name())?;
        let mut raw = entry.get_name().to_owned();

        write!(
            raw,
            "({})",
            entry
                .get_inputs()
                .iter()
                .map(|arg| arg.get_field_type().to_owned())
                .collect::<Vec<_>>()
                .join(",")
        )?;

        write!(
            pretty,
            "({})",
            entry
                .get_inputs()
                .iter()
                .map(|arg| format!("{:} {:}", arg.get_field_type(), arg.get_name()))
                .collect::<Vec<_>>()
                .join(", ")
        )?;

        if entry.payable {
            write!(pretty, " payable")?;
        }

        if !entry.get_outputs().is_empty() {
            write!(
                pretty,
                " returns ({})",
                entry
                    .get_outputs()
                    .iter()
                    .map(|arg| arg.get_field_type().to_owned())
                    .collect::<Vec<_>>()
                    .join(", "),
            )?;
        }

        let fhash = abi_function_name_to_hash(&raw);
        eprintln!("{:}\n    => {:}: {:}", pretty, (&fhash[..]).encode_hex::<String>(), raw);
    }
    Ok(())
}

#[inline]
fn abi_function_name_to_hash(fname: &str) -> [u8; 4] {
    let mut hash_code = [0u8; 4];
    (&mut hash_code[..]).copy_from_slice(&crypto::keccak256(fname.as_bytes())[..4]);
    hash_code
}
