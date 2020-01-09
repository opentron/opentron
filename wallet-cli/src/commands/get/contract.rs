use hex::ToHex;
use keys::Address;
use proto::api::BytesMessage;
use proto::api_grpc::Wallet;
use serde_json::json;
use std::fmt::Write as FmtWrite;

use crate::error::Error;
use crate::utils::abi;
use crate::utils::client::new_grpc_client;
use crate::utils::jsont;

pub fn run(addr: &str) -> Result<(), Error> {
    let address: Address = addr.parse()?;
    let mut req = BytesMessage::new();
    req.set_value(address.to_bytes().to_owned());

    let (_, payload, _) = new_grpc_client()?.get_contract(Default::default(), req).wait()?;
    if payload.get_code_hash().is_empty() {
        return Err(Error::Runtime("contract not found on chain"));
    }

    let mut contract = serde_json::to_value(&payload)?;
    contract["contract_address"] = json!(jsont::bytes_to_hex_string(&contract["contract_address"]));
    contract["origin_address"] = json!(jsont::bytes_to_hex_string(&contract["origin_address"]));
    contract["bytecode"] = json!(jsont::bytes_to_hex_string(&contract["bytecode"]));
    contract["code_hash"] = json!(jsont::bytes_to_hex_string(&contract["code_hash"]));

    println!("{}", serde_json::to_string_pretty(&contract)?);
    pprint_abi_entries(payload.get_abi())?;
    Ok(())
}

// NOTE: there is a typo in pb: abi.`entrys`
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
        let fnhash = abi::fnhash(&raw);

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

        eprintln!(
            "{:}\n    => {:}: {:}",
            pretty,
            (&fnhash[..]).encode_hex::<String>(),
            raw
        );
    }
    Ok(())
}
