//! JSON transformations

use hex::{FromHex, ToHex};
use proto::core::{
    AccountCreateContract, AccountPermissionUpdateContract, AccountUpdateContract, AssetIssueContract,
    ClearABIContract, CreateSmartContract, ExchangeInjectContract, FreezeBalanceContract,
    ParticipateAssetIssueContract, ProposalApproveContract, ProposalCreateContract, ProposalDeleteContract,
    ShieldedTransferContract, TransferAssetContract, TransferContract, TriggerSmartContract, UnfreezeAssetContract,
    UnfreezeBalanceContract, UpdateAssetContract, UpdateEnergyLimitContract, UpdateSettingContract,
    VoteWitnessContract, WithdrawBalanceContract, WitnessCreateContract, WitnessUpdateContract,
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
    let buf = val
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap() as u8)
        .collect::<Vec<_>>();
    String::from_utf8_lossy(&buf).into()
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
            fix_asset_issue_contract(&mut contract);
            contract
        }
        Some("UpdateSettingContract") => {
            let pb: UpdateSettingContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract["contract_address"] = json!(bytes_to_hex_string(&contract["contract_address"]));
            contract
        }
        Some("UpdateEnergyLimitContract") => {
            let pb: UpdateEnergyLimitContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract["contract_address"] = json!(bytes_to_hex_string(&contract["contract_address"]));
            contract
        }
        Some("ClearABIContract") => {
            let pb: ClearABIContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract["contract_address"] = json!(bytes_to_hex_string(&contract["contract_address"]));
            contract
        }
        Some("UpdateAssetContract") => {
            let pb: UpdateAssetContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract["description"] = json!(bytes_to_string(&contract["description"]));
            contract["url"] = json!(bytes_to_string(&contract["url"]));
            contract
        }
        Some("ParticipateAssetIssueContract") => {
            let pb: ParticipateAssetIssueContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract["to_address"] = json!(bytes_to_hex_string(&contract["to_address"]));
            contract["asset_name"] = json!(bytes_to_string(&contract["asset_name"]));
            contract
        }
        Some("UnfreezeAssetContract") => {
            let pb: UnfreezeAssetContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract
        }
        Some("UnfreezeBalanceContract") => {
            let pb: UnfreezeBalanceContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract["receiver_address"] = json!(bytes_to_hex_string(&contract["receiver_address"]));
            contract
        }
        Some("AccountCreateContract") => {
            let pb: AccountCreateContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract["account_address"] = json!(bytes_to_hex_string(&contract["account_address"]));
            contract
        }
        Some("ExchangeInjectContract") => {
            let pb: ExchangeInjectContract = protobuf::parse_from_bytes(&raw_pb)?;
            let mut contract = serde_json::to_value(&pb)?;
            contract["owner_address"] = json!(bytes_to_hex_string(&contract["owner_address"]));
            contract["token_id"] = json!(bytes_to_string(&contract["token_id"]));
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
    transaction["data"] = json!(bytes_to_hex_string(&transaction["data"]));
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
    if transaction_ext["result"]["message"].is_array() {
        transaction_ext["result"]["message"] = json!(bytes_to_string(&transaction_ext["result"]["message"]));
    }
    if transaction_ext["constant_result"].is_array() {
        transaction_ext["constant_result"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .map(|res| *res = json!(bytes_to_hex_string(res)))
            .last();
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
    account["account_id"] = json!(bytes_to_string(&account["account_id"]));
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

// pb: AssetIssueContract
pub fn fix_asset_issue_contract(asset: &mut serde_json::Value) {
    asset["abbr"] = json!(bytes_to_string(&asset["abbr"]));
    asset["description"] = json!(bytes_to_string(&asset["description"]));
    asset["name"] = json!(bytes_to_string(&asset["name"]));
    asset["url"] = json!(bytes_to_string(&asset["url"]));
    asset["owner_address"] = json!(bytes_to_hex_string(&asset["owner_address"]));
}

// pb: IncrementalMerkleVoucherInfo
#[allow(dead_code)]
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

// pb: TransactionInfo
pub fn fix_transaction_info(info: &mut serde_json::Value) {
    info["id"] = json!(bytes_to_hex_string(&info["id"]));
    info["contract_address"] = json!(bytes_to_hex_string(&info["contract_address"]));
    info["resMessage"] = json!(bytes_to_string(&info["resMessage"]));
    info["contractResult"] = json!(info["contractResult"]
        .as_array()
        .unwrap()
        .iter()
        .map(bytes_to_hex_string)
        .collect::<Vec<_>>());
    info["internal_transactions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|trx| {
            trx["caller_address"] = json!(bytes_to_hex_string(&trx["caller_address"]));
            trx["hash"] = json!(bytes_to_hex_string(&trx["hash"]));
            trx["note"] = json!(bytes_to_string(&trx["note"]));
            // NOTE: the ugly camEl_case naming
            trx["transferTo_address"] = json!(bytes_to_hex_string(&trx["transferTo_address"]));
        })
        .last();
    info["log"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|log| {
            log["address"] = json!(bytes_to_hex_string(&log["address"]));
            log["data"] = json!(bytes_to_hex_string(&log["data"]));
            log["topics"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .map(|t| {
                    *t = json!(bytes_to_hex_string(t));
                })
                .last();
        })
        .last();
}

// pb: Block / BlockExtention
pub fn fix_block(block: &mut serde_json::Value) -> Result<(), Error> {
    if block["blockid"].is_array() {
        block["blockid"] = json!(bytes_to_hex_string(&block["blockid"]));
    }

    for key in &["parentHash", "txTrieRoot", "witness_address", "accountStateRoot"] {
        block["block_header"]["raw_data"][key] = json!(bytes_to_hex_string(&block["block_header"]["raw_data"][key]));
    }
    block["block_header"]["witness_signature"] =
        json!(bytes_to_hex_string(&block["block_header"]["witness_signature"]));

    block["transactions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|mut transaction| {
            // NOTE: structual difference of get_block requests
            if transaction["txid"].is_array() {
                transaction["txid"] = json!(bytes_to_hex_string(&transaction["txid"]));
                transaction = &mut transaction["transaction"];
            }
            fix_transaction(transaction)?;
            Ok(())
        })
        .collect::<Result<Vec<_>, Error>>()
        .map(|_| ())
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
