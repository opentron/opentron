//! JSON transformations

use hex::{FromHex, ToHex};
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

pub fn fix_account(account: &mut serde_json::Value) {
    account["address"] = json!(bytes_to_hex_string(&account["address"]));
    account["account_name"] = json!(bytes_to_string(&account["account_name"]));
    // NOTE: one can remove owner_permission by setting null
    if !account["owner_permission"].is_null() {
        account["owner_permission"]["keys"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .map(|key| {
                key["address"] = json!(bytes_to_hex_string(&key["address"]));
            })
            .last();
    }
    account["active_permission"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|perm| {
            perm["keys"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .map(|key| {
                    key["address"] = json!(bytes_to_hex_string(&key["address"]));
                })
                .last();
            perm["operations"] = json!(bytes_to_hex_string(&perm["operations"]));
        })
        .last();
    if !account["witness_permission"].is_null() {
        account["witness_permission"]["keys"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .map(|key| {
                key["address"] = json!(bytes_to_hex_string(&key["address"]));
            })
            .last();
    }
}

// revert for serializing to pb
pub fn revert_permission_info(permission: &mut serde_json::Value) {
    if !permission["owner"].is_null() {
        permission["owner"]["keys"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .map(|key| key["address"] = json!(Vec::from_hex(key["address"].as_str().unwrap()).unwrap()))
            .last();
    }
    permission["actives"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|perm| {
            perm["keys"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .map(|key| {
                    key["address"] = json!(Vec::from_hex(key["address"].as_str().unwrap()).unwrap());
                })
                .last();
            perm["operations"] = json!(Vec::from_hex(perm["operations"].as_str().unwrap()).unwrap());
        })
        .last();
}
