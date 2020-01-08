//! JSON transformations

use hex::{FromHex, ToHex};
use proto::core::{
    AccountPermissionUpdateContract, AccountUpdateContract, CreateSmartContract, FreezeBalanceContract,
    ProposalApproveContract, ProposalCreateContract, ProposalDeleteContract, ShieldedTransferContract,
    TransferAssetContract, TransferContract, TriggerSmartContract, VoteWitnessContract, WithdrawBalanceContract,
    WitnessCreateContract, WitnessUpdateContract, AssetIssueContract
};
use serde_json::json;

use crate::error::Error;

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

// pb: TransferContract
pub fn fix_transfer_contract(val: &mut serde_json::Value) {
    val["owner_address"] = json!(bytes_to_hex_string(&val["owner_address"]));
    val["to_address"] = json!(bytes_to_hex_string(&val["to_address"]));
}

// pb: TransferAssetContract
pub fn fix_transfer_asset_contract(val: &mut serde_json::Value) {
    val["owner_address"] = json!(bytes_to_hex_string(&val["owner_address"]));
    val["to_address"] = json!(bytes_to_hex_string(&val["to_address"]));
    val["asset_name"] = json!(bytes_to_string(&val["asset_name"]));
}

// pb: TriggerSmartContract
pub fn fix_trigger_smart_contract(val: &mut serde_json::Value) {
    val["owner_address"] = json!(bytes_to_hex_string(&val["owner_address"]));
    val["contract_address"] = json!(bytes_to_hex_string(&val["contract_address"]));
    val["data"] = json!(bytes_to_hex_string(&val["data"]));
}

// pb: AccountPermissionUpdateContract
pub fn fix_account_permission_update_contract(val: &mut serde_json::Value) {
    val["owner_address"] = json!(bytes_to_hex_string(&val["owner_address"]));
    val["actives"]
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
    if !val["owner"].is_null() {
        val["owner"]["keys"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .map(|key| {
                key["address"] = json!(bytes_to_hex_string(&key["address"]));
            })
            .last();
    }
    if !val["witness"].is_null() {
        val["witness"]["keys"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .map(|key| {
                key["address"] = json!(bytes_to_hex_string(&key["address"]));
            })
            .last();
    }
}

// pb: CreateSmartContract
pub fn fix_create_smart_contract(val: &mut serde_json::Value) {
    val["owner_address"] = json!(bytes_to_hex_string(&val["owner_address"]));
    let contract = &mut val["new_contract"];
    contract["bytecode"] = json!(bytes_to_hex_string(&contract["bytecode"]));
    contract["origin_address"] = json!(bytes_to_hex_string(&contract["origin_address"]));
}

// pb: ShieldedTransferContract
pub fn fix_shielded_transfer_contract(val: &mut serde_json::Value) {
    val["transparent_from_address"] = json!(bytes_to_hex_string(&val["transparent_from_address"]));
    val["transparent_to_address"] = json!(bytes_to_hex_string(&val["transparent_to_address"]));
    val["binding_signature"] = json!(bytes_to_hex_string(&val["binding_signature"]));

    val["receive_description"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|v| {
            for k in &[
                "value_commitment",
                "note_commitment",
                "epk",
                "c_enc",
                "c_out",
                "zkproof",
            ] {
                v[k] = json!(bytes_to_hex_string(&v[k]));
            }
        })
        .last();

    val["spend_description"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|v| {
            for k in &[
                "value_commitment",
                "anchor",
                "nullifier",
                "rk",
                "zkproof",
                "spend_authority_signature",
            ] {
                v[k] = json!(bytes_to_hex_string(&v[k]));
            }
        })
        .last();
}

// pb: VoteWitnessContract
pub fn fix_vote_witness_contract(val: &mut serde_json::Value) {
    val["owner_address"] = json!(bytes_to_hex_string(&val["owner_address"]));
    val["votes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|vote| {
            vote["vote_address"] = json!(bytes_to_hex_string(&vote["vote_address"]));
        })
        .last();
}

// pb: FreezeBalanceContract
pub fn fix_freeze_balance_contract(val: &mut serde_json::Value) {
    val["owner_address"] = json!(bytes_to_hex_string(&val["owner_address"]));
    val["receiver_address"] = json!(bytes_to_hex_string(&val["receiver_address"]));
}

// pb: Transaction.raw
pub fn fix_transaction_raw(transaction: &mut serde_json::Value) -> Result<(), Error> {
    if transaction["contract"].as_array().unwrap().is_empty() {
        return Ok(());
    }
    let raw_pb = transaction["contract"][0]["parameter"]["value"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as u8)
        .collect::<Vec<u8>>();

    let parsed_value = match transaction["contract"][0]["field_type"].as_str() {
        Some("TransferContract") => {
            let pb: TransferContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            fix_transfer_contract(&mut contract);
            contract
        }
        Some("TransferAssetContract") => {
            let pb: TransferAssetContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            fix_transfer_asset_contract(&mut contract);
            contract
        }
        Some("TriggerSmartContract") => {
            let pb: TriggerSmartContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            fix_trigger_smart_contract(&mut contract);
            contract
        }
        Some("AccountPermissionUpdateContract") => {
            let pb: AccountPermissionUpdateContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            fix_account_permission_update_contract(&mut contract);
            contract
        }
        Some("CreateSmartContract") => {
            let pb: CreateSmartContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            fix_create_smart_contract(&mut contract);
            contract
        }
        Some("ShieldedTransferContract") => {
            let pb: ShieldedTransferContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            fix_shielded_transfer_contract(&mut contract);
            contract
        }
        Some("VoteWitnessContract") => {
            let pb: VoteWitnessContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            fix_vote_witness_contract(&mut contract);
            contract
        }
        Some("FreezeBalanceContract") => {
            let pb: FreezeBalanceContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            fix_freeze_balance_contract(&mut contract);
            contract
        }
        Some("AccountUpdateContract") => {
            let pb: AccountUpdateContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract["account_name"] = json!(bytes_to_string(&contract["account_name"]));
            contract
        }
        Some("WitnessCreateContract") => {
            let pb: WitnessCreateContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract["url"] = json!(bytes_to_string(&contract["url"]));
            contract
        }
        Some("WitnessUpdateContract") => {
            let pb: WitnessUpdateContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract["update_url"] = json!(bytes_to_string(&contract["update_url"]));
            contract
        }
        Some("WithdrawBalanceContract") => {
            let pb: WithdrawBalanceContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract
        }
        Some("ProposalCreateContract") => {
            let pb: ProposalCreateContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract
        }
        Some("ProposalApproveContract") => {
            let pb: ProposalApproveContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract
        }
        Some("ProposalDeleteContract") => {
            let pb: ProposalDeleteContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract
        }
        Some("AssetIssueContract") => {
            let pb: AssetIssueContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract["abbr"] = json!(bytes_to_string(&contract["abbr"]));
            contract["name"] = json!(bytes_to_string(&contract["name"]));
            contract["url"] = json!(bytes_to_string(&contract["url"]));
            contract["description"] = json!(bytes_to_string(&contract["description"]));
            contract
        }
        x => {
            eprintln!("unhandled contract type => {:?}", x);
            json!(raw_pb.encode_hex::<String>())
        }
    };
    transaction["contract"][0]["parameter"]["value"] = parsed_value;

    transaction["ref_block_hash"] = json!(bytes_to_hex_string(&transaction["ref_block_hash"]));
    transaction["ref_block_bytes"] = json!(bytes_to_hex_string(&transaction["ref_block_bytes"]));
    transaction["data"] = json!(bytes_to_string(&transaction["data"]));
    Ok(())
}

// pb: Transaction
pub fn fix_transaction(transaction: &mut serde_json::Value) -> Result<(), Error> {
    fix_transaction_raw(&mut transaction["raw_data"])?;
    transaction["signature"] = json!(transaction["signature"]
        .as_array()
        .unwrap()
        .iter()
        .map(|sig| json!(bytes_to_hex_string(sig)))
        .collect::<Vec<_>>());
    Ok(())
}

// pb: TransactionExtention
pub fn fix_transaction_ext(transaction_ext: &mut serde_json::Value) -> Result<(), Error> {
    if transaction_ext["result"]["result"].as_bool().unwrap() == false {
        transaction_ext["result"]["message"] = json!(bytes_to_string(&transaction_ext["result"]["message"]));
    }
    if !transaction_ext["transaction"].is_null() {
        fix_transaction(&mut transaction_ext["transaction"])?;
    }
    transaction_ext["txid"] = json!(bytes_to_hex_string(&mut transaction_ext["txid"]));
    Ok(())
}

// pb: Account
pub fn fix_account(account: &mut serde_json::Value) {
    account["address"] = json!(bytes_to_hex_string(&account["address"]));
    account["account_name"] = json!(bytes_to_string(&account["account_name"]));
    account["asset_issued_ID"] = json!(bytes_to_string(&account["asset_issued_ID"]));
    account["asset_issued_name"] = json!(bytes_to_string(&account["asset_issued_name"]));
    account["votes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|vote| vote["vote_address"] = json!(bytes_to_hex_string(&vote["vote_address"])))
        .last();
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

// pb: Return
pub fn fix_api_return(ret: &mut serde_json::Value) {
    if !ret["message"].is_null() {
        ret["message"] = json!(bytes_to_string(&ret["message"]));
    }
}

// pb: IncrementalMerkleVoucherInfo
pub fn fix_voucher_info(voucher_info: &mut serde_json::Value) {
    voucher_info["paths"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|path| *path = json!(bytes_to_hex_string(path)))
        .last();
    voucher_info["vouchers"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|voucher| {
            voucher["rt"] = json!(bytes_to_hex_string(&voucher["rt"]));
            // right or left may be null
            voucher["tree"]["left"]
                .as_object_mut()
                .map(|obj| obj["content"] = json!(bytes_to_hex_string(&obj["content"])));
            voucher["tree"]["right"]
                .as_object_mut()
                .map(|obj| obj["content"] = json!(bytes_to_hex_string(&obj["content"])));

            voucher["tree"]["parents"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .map(|p| p["content"] = json!(bytes_to_hex_string(&p["content"])))
                .last();
        })
        .last();
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
