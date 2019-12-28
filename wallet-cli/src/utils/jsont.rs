//! JSON transformations

use hex::ToHex;
use serde_json::json;

pub fn bytes_to_hex_string(val: &serde_json::Value) -> String {
    val.as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as u8)
        .collect::<Vec<_>>()
        .encode_hex()
}

pub fn bytes_to_string(val: &serde_json::Value) -> String {
    val.as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as u8 as char)
        .collect::<String>()
}

pub fn fix_transaction(transaction: &mut serde_json::Value) {
    transaction["raw_data"]["contract"][0]["parameter"]["value"] = json!(bytes_to_hex_string(
        &transaction["raw_data"]["contract"][0]["parameter"]["value"]
    ));
    transaction["raw_data"]["ref_block_hash"] = json!(bytes_to_hex_string(&transaction["raw_data"]["ref_block_hash"]));
    transaction["raw_data"]["ref_block_bytes"] =
        json!(bytes_to_hex_string(&transaction["raw_data"]["ref_block_bytes"]));
    transaction["raw_data"]["data"] = json!(bytes_to_string(&transaction["raw_data"]["data"]));
    transaction["signature"] = json!(transaction["signature"]
        .as_array()
        .unwrap()
        .iter()
        .map(|sig| json!(bytes_to_hex_string(sig)))
        .collect::<Vec<_>>());
}
