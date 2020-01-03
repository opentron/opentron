//! Helpers for transaction.

use chrono::Utc;
use keys::Address;
use proto::core::TransferContract;
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
        _ => unimplemented!(),
    }
}
