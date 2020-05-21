use chrono::{DateTime, TimeZone, Utc};
use keys::b58encode_check;
use proto2::chain::transaction::Contract as ContractPb;

#[derive(juniper::GraphQLObject)]
pub struct TransferContract {
    owner_address: String,
    to_address: String,
    amount: f64,
}

#[derive(juniper::GraphQLObject)]
pub struct TransferAssetContract {
    owner_address: String,
    to_address: String,
    /// after ALLOW_SAME_TOKEN_NAME
    token_id: Option<i32>,
    /// before ALLOW_SAME_TOKEN_NAME
    token_name: Option<String>,
    amount: f64,
}

#[derive(juniper::GraphQLObject)]
pub struct FrozenSupply {
    frozen_amount: f64,
    frozen_days: i32,
}

#[derive(juniper::GraphQLObject)]
pub struct AssetIssueContract {
    owner_address: String,
    name: String,
    abbr: String,
    total_supply: f64,
    frozen_supply: Vec<FrozenSupply>,
    num: i32,
    trx_num: i32,
    precision: i32,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    description: String,
    url: String,
    /// FreeAssetNetLimit
    free_bandwidth: f64,
    /// PublicFreeAssetNetLimit
    public_free_bandwidth: f64,
    // int64 order = 11;  // useless
    // int32 vote_score = 16;
    // public_free_asset_net_usage = 24;
    // public_latest_free_net_time = 25;
    // string id = 41;
}

#[derive(juniper::GraphQLObject)]
pub struct ParticipateAssetIssueContract {
    owner_address: String,
    to_address: String,
    /// after ALLOW_SAME_TOKEN_NAME
    token_id: Option<i32>,
    /// before ALLOW_SAME_TOKEN_NAME
    token_name: Option<String>,
    amount: f64,
}

#[derive(juniper::GraphQLObject)]
pub struct Vote {
    vote_address: String,
    count: f64,
}

#[derive(juniper::GraphQLObject)]
pub struct WitnessCreateContract {
    owner_address: String,
    url: String,
}

#[derive(juniper::GraphQLObject)]
pub struct WithdrawBalanceContract {
    owner_address: String,
}

#[derive(juniper::GraphQLObject)]
pub struct VoteWitnessContract {
    owner_address: String,
    votes: Vec<Vote>,
}

#[derive(juniper::GraphQLEnum, PartialEq, Eq)]
pub enum ResourceCode {
    Bandwidth,
    Energy,
}

#[derive(juniper::GraphQLObject)]
pub struct FreezeBalanceContract {
    owner_address: String,
    receiver_address: String,
    resource: ResourceCode,
    frozen_balance: f64,
    frozen_duration: i32,
}

#[derive(juniper::GraphQLObject)]
pub struct UnfreezeBalanceContract {
    owner_address: String,
    receiver_address: String,
    resource: ResourceCode,
}

#[derive(juniper::GraphQLObject)]
pub struct Parameter {
    key: i32,
    value: f64, // max: 9007199254740992
}

#[derive(juniper::GraphQLObject)]
pub struct ProposalCreateContract {
    owner_address: String,
    parameters: Vec<Parameter>,
}

#[derive(juniper::GraphQLObject)]
pub struct ProposalApproveContract {
    owner_address: String,
    proposal_id: i32,
    is_approve: bool,
}

#[derive(juniper::GraphQLObject)]
pub struct SmartContract {
    name: String,
    origin_address: String,
    contract_address: Option<String>,
    /// ABI as JSON string.
    abi: Option<String>,
    code: String,
    /// Percent, 0 to 100.
    user_resource_percent: i32,
    origin_energy_limit: f64,
    code_hash: Option<String>,
    // When smart contract is created by a trigger smart contract call.
    txn_id: Option<String>,
}

#[derive(juniper::GraphQLObject)]
pub struct CreateSmartContract {
    owner_address: String,
    new_smart_contract: SmartContract,
    call_value: f64, // moved from inner struct
    call_token_value: f64,
    call_token_id: i32,
}

#[derive(juniper::GraphQLObject)]
pub struct TriggerSmartContract {
    owner_address: String,
    contract_address: String,
    data: String,
    call_value: f64,
    call_token_value: f64,
    call_token_id: i32,
}

#[derive(juniper::GraphQLUnion)]
pub enum Contract {
    TransferContract(TransferContract),
    TransferAssetContract(TransferAssetContract),
    AssetIssueContract(AssetIssueContract),
    ParticipateAssetIssueContract(ParticipateAssetIssueContract),
    WitnessCreateContract(WitnessCreateContract),
    WithdrawBalanceContract(WithdrawBalanceContract),
    VoteWitnessContract(VoteWitnessContract),
    FreezeBalanceContract(FreezeBalanceContract),
    UnfreezeBalanceContract(UnfreezeBalanceContract),
    ProposalCreateContract(ProposalCreateContract),
    ProposalApproveContract(ProposalApproveContract),
    CreateSmartContract(CreateSmartContract),
    TriggerSmartContract(TriggerSmartContract),
    // AccountPermissionUpdateContract(AccountPermissionUpdateContract),
    // AccountCreateContract = 0,
    // VoteAssetContract = 3,
    //  = 6,
    // WitnessUpdateContract = 8,
    /*
    AccountUpdateContract = 10, */
    /* = 12,
    WithdrawBalanceContract = 13,
    UnfreezeAssetContract = 14,
    UpdateAssetContract = 15,
    */
    /*
    ProposalDeleteContract = 18,
    SetAccountIdContract = 19,
    UpdateSettingContract = 33,
    ExchangeCreateContract = 41,
    ExchangeInjectContract = 42,
    ExchangeWithdrawContract = 43,
    ExchangeTransactionContract = 44,
    UpdateEnergyLimitContract = 45,
    ClearABIContract = 48,
    UpdateBrokerageContract = 49,
    ShieldedTransferContract = 51,
    */
}

impl From<ContractPb> for Contract {
    fn from(pb: ContractPb) -> Self {
        use prost::Message;
        use proto2::chain::ContractType;
        use proto2::contract as contract_pb;

        let raw = &pb.parameter.as_ref().unwrap().value[..];

        match ContractType::from_i32(pb.r#type) {
            Some(ContractType::TransferContract) => {
                let cntr = contract_pb::TransferContract::decode(raw).unwrap();
                let inner = TransferContract {
                    owner_address: b58encode_check(&cntr.owner_address),
                    to_address: b58encode_check(&cntr.to_address),
                    amount: cntr.amount as _,
                };
                Contract::TransferContract(inner)
            }
            Some(ContractType::TransferAssetContract) => {
                let cntr = contract_pb::TransferAssetContract::decode(raw).unwrap();
                let inner = TransferAssetContract {
                    owner_address: b58encode_check(&cntr.owner_address),
                    to_address: b58encode_check(&cntr.to_address),
                    token_id: cntr.asset_name.parse().ok(),
                    token_name: Some(cntr.asset_name),
                    amount: cntr.amount as _,
                };
                Contract::TransferAssetContract(inner)
            }
            Some(ContractType::AssetIssueContract) => {
                let cntr = contract_pb::AssetIssueContract::decode(raw).unwrap();
                let inner = AssetIssueContract {
                    owner_address: b58encode_check(&cntr.owner_address),
                    name: cntr.name.clone(),
                    abbr: cntr.abbr.clone(),
                    description: cntr.description.clone(),
                    url: cntr.url.clone(),
                    total_supply: cntr.total_supply as _,
                    frozen_supply: cntr
                        .frozen_supply
                        .iter()
                        .map(|sup| FrozenSupply {
                            frozen_amount: sup.frozen_amount as _,
                            frozen_days: sup.frozen_days as _,
                        })
                        .collect(),
                    num: cntr.num as _,
                    trx_num: cntr.trx_num as _,
                    precision: cntr.precision as _,
                    start_time: Utc.timestamp(cntr.start_time / 1_000, cntr.start_time as u32 % 1_000 * 1_000_000),
                    end_time: Utc.timestamp(cntr.end_time / 1_000, cntr.end_time as u32 % 1_000 * 1_000_000),
                    /// FreeAssetNetLimit
                    free_bandwidth: cntr.free_asset_net_limit as _,
                    /// PublicFreeAssetNetLimit
                    public_free_bandwidth: cntr.public_free_asset_net_limit as _,
                };
                Contract::AssetIssueContract(inner)
            }
            Some(ContractType::ParticipateAssetIssueContract) => {
                let cntr = contract_pb::ParticipateAssetIssueContract::decode(raw).unwrap();
                let inner = ParticipateAssetIssueContract {
                    owner_address: b58encode_check(&cntr.owner_address),
                    to_address: b58encode_check(&cntr.to_address),
                    token_id: cntr.asset_name.parse().ok(),
                    token_name: Some(cntr.asset_name),
                    amount: cntr.amount as _,
                };
                Contract::ParticipateAssetIssueContract(inner)
            }
            Some(ContractType::FreezeBalanceContract) => {
                let cntr = contract_pb::FreezeBalanceContract::decode(raw).unwrap();
                let inner = FreezeBalanceContract {
                    owner_address: b58encode_check(&cntr.owner_address),
                    receiver_address: b58encode_check(&cntr.receiver_address),
                    frozen_balance: cntr.frozen_balance as _,
                    frozen_duration: cntr.frozen_balance as _,
                    resource: if cntr.resource == 0 {
                        ResourceCode::Bandwidth
                    } else {
                        ResourceCode::Energy
                    },
                };
                Contract::FreezeBalanceContract(inner)
            }
            Some(ContractType::UnfreezeBalanceContract) => {
                let cntr = contract_pb::UnfreezeBalanceContract::decode(raw).unwrap();
                let inner = UnfreezeBalanceContract {
                    owner_address: b58encode_check(&cntr.owner_address),
                    receiver_address: b58encode_check(&cntr.receiver_address),
                    resource: if cntr.resource == 0 {
                        ResourceCode::Bandwidth
                    } else {
                        ResourceCode::Energy
                    },
                };
                Contract::UnfreezeBalanceContract(inner)
            }
            Some(ContractType::WitnessCreateContract) => {
                let cntr = contract_pb::WitnessCreateContract::decode(raw).unwrap();
                let inner = WitnessCreateContract {
                    owner_address: b58encode_check(&cntr.owner_address),
                    url: String::from_utf8(cntr.url).unwrap(),
                };
                Contract::WitnessCreateContract(inner)
            }
            Some(ContractType::WithdrawBalanceContract) => {
                let cntr = contract_pb::WithdrawBalanceContract::decode(raw).unwrap();
                let inner = WithdrawBalanceContract {
                    owner_address: b58encode_check(&cntr.owner_address),
                };
                Contract::WithdrawBalanceContract(inner)
            }
            Some(ContractType::VoteWitnessContract) => {
                let cntr = contract_pb::VoteWitnessContract::decode(raw).unwrap();
                let inner = VoteWitnessContract {
                    owner_address: b58encode_check(&cntr.owner_address),
                    votes: cntr
                        .votes
                        .iter()
                        .map(|vote| Vote {
                            vote_address: b58encode_check(&vote.vote_address),
                            count: vote.vote_count as _,
                        })
                        .collect(),
                };
                Contract::VoteWitnessContract(inner)
            }
            Some(ContractType::CreateSmartContract) => {
                let cntr = contract_pb::CreateSmartContract::decode(raw).unwrap();
                let smart_cntr = cntr.new_contract.as_ref().unwrap();

                let new_smart_contract = SmartContract {
                    origin_address: b58encode_check(&smart_cntr.origin_address),
                    name: smart_cntr.name.clone(),
                    abi: smart_cntr
                        .abi
                        .as_ref()
                        .map(|abi| &abi.entries)
                        .and_then(|entries| serde_json::to_string(entries).ok()),
                    code: hex::encode(&smart_cntr.bytecode),
                    user_resource_percent: smart_cntr.consume_user_resource_percent as _,
                    origin_energy_limit: smart_cntr.origin_energy_limit as _,
                    contract_address: if !smart_cntr.contract_address.is_empty() {
                        Some(b58encode_check(&smart_cntr.contract_address))
                    } else {
                        None
                    },
                    code_hash: if !smart_cntr.code_hash.is_empty() {
                        Some(hex::encode(&smart_cntr.code_hash))
                    } else {
                        None
                    },
                    txn_id: if !smart_cntr.txn_id.is_empty() {
                        Some(hex::encode(&smart_cntr.txn_id))
                    } else {
                        None
                    },
                };
                let inner = CreateSmartContract {
                    owner_address: b58encode_check(&cntr.owner_address),
                    new_smart_contract,
                    call_value: smart_cntr.call_value as _,
                    call_token_value: cntr.call_token_value as _,
                    call_token_id: cntr.call_token_id as _,
                };
                Contract::CreateSmartContract(inner)
            }
            Some(ContractType::TriggerSmartContract) => {
                let cntr = contract_pb::TriggerSmartContract::decode(raw).unwrap();
                let inner = TriggerSmartContract {
                    owner_address: b58encode_check(&cntr.owner_address),
                    contract_address: b58encode_check(&cntr.contract_address),
                    call_value: cntr.call_value as _,
                    data: hex::encode(&cntr.data),
                    call_token_value: cntr.call_token_value as _,
                    call_token_id: cntr.call_token_id as _,
                };
                Contract::TriggerSmartContract(inner)
            }
            Some(ContractType::ProposalCreateContract) => {
                let cntr = contract_pb::ProposalCreateContract::decode(raw).unwrap();
                let inner = ProposalCreateContract {
                    owner_address: b58encode_check(&cntr.owner_address),
                    parameters: cntr
                        .parameters
                        .iter()
                        .map(|(&k, &v)| Parameter {
                            key: k as _,
                            value: v as _,
                        })
                        .collect(),
                };
                Contract::ProposalCreateContract(inner)
            }
            Some(ContractType::ProposalApproveContract) => {
                let cntr = contract_pb::ProposalApproveContract::decode(raw).unwrap();
                let inner = ProposalApproveContract {
                    owner_address: b58encode_check(&cntr.owner_address),
                    proposal_id: cntr.proposal_id as _,
                    is_approve: cntr.is_approval,
                };
                Contract::ProposalApproveContract(inner)
            }
            Some(typ) => unimplemented!("unhandled type {:?}", typ),
            None => unreachable!(),
        }
    }
}
