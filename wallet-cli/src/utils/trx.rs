//! Helpers for transaction.

use chrono::Utc;
use keys::Address;
use proto::core::{TransferContract, CreateSmartContract};
use protobuf::parse_from_bytes;
use protobuf::well_known_types::Any;
use std::convert::TryFrom;

use crate::error::Error;

pub fn timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn extract_owner_address_from_parameter(any: &Any) -> Result<Address, Error> {
    match any.get_type_url() {
        "type.googleapis.com/protocol.TransferContract" => Ok(Address::try_from(
            parse_from_bytes::<TransferContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.ShieldedTransferContract" => Err(Error::Runtime(
            "can not extract sender address from ShieldedTransferContract. Use -k/-K instead.",
        )),
        "type.googleapis.com/protocol.CreateSmartContract" => Ok(Address::try_from(
            parse_from_bytes::<CreateSmartContract>(any.get_value())?.get_owner_address(),
        )?),
        _ => unimplemented!(),
    }
}

/// Parse command line amount to amount in pb.
pub fn parse_amount(amount: &str, allow_unit: bool) -> Result<i64, Error> {
    // NOTE: simple parse, buggy but works
    if amount.is_empty() {
        return Err(Error::Runtime("can not parse empty amount"));
    }
    if allow_unit {
        let length = amount.as_bytes().len();
        // FIXME: allow other unit
        if amount.ends_with("TRX") || amount.ends_with("TRZ") {
            Ok(String::from_utf8_lossy(&amount.as_bytes()[..length - 3])
                .replace("_", "")
                .parse::<i64>()?
                * 1_000_000)
        } else if amount.ends_with("SUN") {
            Ok(String::from_utf8_lossy(&amount.as_bytes()[..length - 3])
                .replace("_", "")
                .parse()?)
        } else {
            Ok(amount.replace("_", "").parse()?)
        }
    } else {
        Ok(amount.replace("_", "").parse()?)
    }
}
