//! Helpers for transaction.

use chrono::Utc;
use keys::Address;
use proto::core::TransferContract;
use protobuf::parse_from_bytes;
use protobuf::well_known_types::Any;
use std::convert::TryFrom;

pub fn timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn extract_owner_address_from_parameter(any: &Any) -> Address {
    match any.get_type_url() {
        "type.googleapis.com/protocol.TransferContract" => Address::try_from(
            parse_from_bytes::<TransferContract>(any.get_value())
                .unwrap()
                .get_owner_address(),
        )
        .expect("won't fail"),
        _ => unimplemented!(),
    }
}
