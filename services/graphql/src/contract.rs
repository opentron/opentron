use std::convert::TryInto;

use chrono::{DateTime, TimeZone, Utc};
use primitive_types::H256;
use proto::chain::transaction::Contract as ContractPb;
use proto::common::Permission as PermissionPb;
use async_graphql::{Enum, SimpleObject, Union};

use super::scalar::{Address, Bytes, Bytes32, Long};


#[derive(Enum, Clone, Copy, PartialEq, Eq)]
pub enum AccountType {
    Normal = 0,
    AssetIssue = 1,
    Contract = 2,
}

impl AccountType {
    pub fn from_i32(val: i32) -> Self {
        match val {
            0 => AccountType::Normal,
            1 => AccountType::AssetIssue,
            2 => AccountType::Contract,
            _ => unreachable!(),
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountCreateContract {
    pub owner_address: Address,
    pub account_address: Address,
    pub r#type: AccountType,
}

#[derive(SimpleObject)]
pub struct TransferContract {
    pub owner_address: Address,
    pub to_address: Address,
    pub amount: Long,
}

#[derive(SimpleObject)]
pub struct TransferAssetContract {
    pub owner_address: Address,
    pub to_address: Address,
    /// after ALLOW_SAME_TOKEN_NAME
    pub token_id: Option<i64>,
    /// before ALLOW_SAME_TOKEN_NAME
    pub token_name: Option<String>,
    pub amount: Long,
}

#[derive(SimpleObject)]
pub struct Vote {
    vote_address: Address,
    count: Long,
}

#[derive(SimpleObject)]
pub struct VoteWitnessContract {
    owner_address: Address,
    votes: Vec<Vote>,
}

#[derive(SimpleObject)]
pub struct WitnessCreateContract {
    owner_address: Address,
    url: String,
}

#[derive(SimpleObject)]
pub struct FrozenSupply {
    frozen_amount: Long,
    frozen_days: i32,
}

#[derive(SimpleObject)]
pub struct AssetIssueContract {
    owner_address: Address,
    name: String,
    abbr: String,
    total_supply: Long,
    frozen_supply: Vec<FrozenSupply>,
    num: i32,
    trx_num: i32,
    precision: i32,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    description: String,
    url: String,
    /// FreeAssetNetLimit
    free_bandwidth: Long,
    /// PublicFreeAssetNetLimit
    public_free_bandwidth: Long,
}

#[derive(SimpleObject)]
pub struct WitnessUpdateContract {
    owner_address: Address,
    update_url: String,
}

#[derive(SimpleObject)]
pub struct ParticipateAssetIssueContract {
    owner_address: Address,
    to_address: Address,
    /// after ALLOW_SAME_TOKEN_NAME
    token_id: Option<i32>,
    /// before ALLOW_SAME_TOKEN_NAME
    token_name: Option<String>,
    amount: Long,
}

/// Set account name.
#[derive(SimpleObject)]
pub struct AccountUpdateContract {
    owner_address: Address,
    account_name: String,
}

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
pub enum ResourceCode {
    Bandwidth,
    Energy,
}

#[derive(SimpleObject)]
pub struct FreezeBalanceContract {
    owner_address: Address,
    receiver_address: Option<Address>,
    resource: ResourceCode,
    frozen_balance: Long,
    frozen_duration: i32,
}

#[derive(SimpleObject)]
pub struct UnfreezeBalanceContract {
    owner_address: Address,
    receiver_address: Option<Address>,
    resource: ResourceCode,
}

#[derive(SimpleObject)]
pub struct WithdrawBalanceContract {
    owner_address: Address,
}

#[derive(SimpleObject)]
pub struct UnfreezeAssetContract {
    owner_address: Address,
}

#[derive(SimpleObject)]
pub struct UpdateAssetContract {
    owner_address: Address,
    description: String,
    url: String,
    new_limit: Long,
    new_public_limit: Long,
}

#[derive(SimpleObject)]
pub struct Parameter {
    key: i32,
    value: Long, // max: 9007199254740992
}

#[derive(SimpleObject)]
pub struct ProposalCreateContract {
    owner_address: Address,
    parameters: Vec<Parameter>,
}

#[derive(SimpleObject)]
pub struct ProposalApproveContract {
    owner_address: Address,
    proposal_id: i32,
    is_approve: bool,
}

#[derive(SimpleObject)]
pub struct ProposalDeleteContract {
    owner_address: Address,
    proposal_id: i32,
}

#[derive(SimpleObject)]
pub struct SetAccountIdContract {
    owner_address: Address,
    account_id: String,
}

#[derive(SimpleObject)]
pub struct SmartContract {
    name: String,
    origin_address: Address,
    contract_address: Option<Address>,
    /// ABI as JSON string.
    abi: Option<String>,
    code: Bytes,
    /// Percent, 0 to 100.
    user_resource_percent: i32,
    origin_energy_limit: Long,
    code_hash: Option<Bytes32>,
    // When smart contract is created by a trigger smart contract call.
    // txn_hash: Option<Bytes32>,
}

#[derive(SimpleObject)]
pub struct CreateSmartContract {
    owner_address: Address,
    new_smart_contract: SmartContract,
    call_value: Long, // moved out from inner struct
    call_token_value: Long,
    call_token_id: i32,
}

#[derive(SimpleObject)]
pub struct TriggerSmartContract {
    owner_address: Address,
    contract_address: Address,
    data: Bytes,
    call_value: Long,
    call_token_value: Long,
    call_token_id: i32,
}

#[derive(SimpleObject)]
pub struct UpdateSettingContract {
    owner_address: Address,
    contract_address: Address,
    consume_user_energy_percent: i32,
}

// TODO: Exchange

#[derive(SimpleObject)]
pub struct UpdateEnergyLimitContract {
    owner_address: Address,
    contract_address: Address,
    origin_energy_limit: Long,
}

#[derive(SimpleObject)]
pub struct ClearABIContract {
    owner_address: Address,
    contract_address: Address,
}

#[derive(SimpleObject)]
pub struct UpdateBrokerageContract {
    owner_address: Address,
    /// Brokerage in percent, dividend payout ratio.
    brokerage: i32,
}

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
pub enum PermissionType {
    Owner = 0,
    Witness = 1,
    Active = 2,
}

impl PermissionType {
    fn from_i32(val: i32) -> Self {
        match val {
            0 => PermissionType::Owner,
            1 => PermissionType::Witness,
            2 => PermissionType::Active,
            _ => unreachable!(),
        }
    }
}

#[derive(SimpleObject)]
pub struct PermissionKey {
    address: Address,
    weight: i32,
}

#[derive(SimpleObject)]
pub struct Permission {
    r#type: PermissionType,
    id: i32,
    name: String,
    threshold: i32,
    // parent_id
    operations: Option<String>,
    keys: Vec<PermissionKey>,
}

impl From<PermissionPb> for Permission {
    fn from(perm: PermissionPb) -> Self {
        Permission {
            r#type: PermissionType::from_i32(perm.r#type),
            id: perm.id,
            name: perm.name.clone(),
            threshold: perm.threshold as _,
            operations: if !perm.operations.is_empty() {
                Some(hex::encode(&perm.operations))
            } else {
                None
            },
            keys: perm
                .keys
                .iter()
                .map(|key| PermissionKey {
                    address: Address((&key.address).try_into().unwrap()),
                    weight: key.weight as _,
                })
                .collect(),
        }
    }
}

#[derive(SimpleObject)]
pub struct AccountPermissionUpdateContract {
    owner_address: Address,
    owner: Permission,
    witness: Option<Permission>,
    actives: Vec<Permission>,
}

#[derive(Union)]
pub enum Contract {
    TransferContract(TransferContract),
    TransferAssetContract(TransferAssetContract),
    AssetIssueContract(AssetIssueContract),
    ParticipateAssetIssueContract(ParticipateAssetIssueContract),
    WitnessCreateContract(WitnessCreateContract),
    WithdrawBalanceContract(WithdrawBalanceContract),
    UpdateBrokerageContract(UpdateBrokerageContract),
    VoteWitnessContract(VoteWitnessContract),
    FreezeBalanceContract(FreezeBalanceContract),
    UnfreezeBalanceContract(UnfreezeBalanceContract),
    ProposalCreateContract(ProposalCreateContract),
    ProposalApproveContract(ProposalApproveContract),
    ProposalDeleteContract(ProposalDeleteContract),
    CreateSmartContract(CreateSmartContract),
    TriggerSmartContract(TriggerSmartContract),
    AccountCreateContract(AccountCreateContract),
    AccountUpdateContract(AccountUpdateContract),
    AccountPermissionUpdateContract(AccountPermissionUpdateContract),
    WitnessUpdateContract(WitnessUpdateContract),
    UnfreezeAssetContract(UnfreezeAssetContract),
    UpdateAssetContract(UpdateAssetContract),
    SetAccountIdContract(SetAccountIdContract),
    UpdateSettingContract(UpdateSettingContract),
    UpdateEnergyLimitContract(UpdateEnergyLimitContract),
    ClearABIContract(ClearABIContract),
    /*
    VoteAssetContract(VoteAssetContract),
    ExchangeCreateContract = 41,
    ExchangeInjectContract = 42,
    ExchangeWithdrawContract = 43,
    ExchangeTransactionContract = 44,
    ShieldedTransferContract = 51,
    */
}

impl Contract {
    pub fn owner_address(&self) -> Address {
        use self::Contract::*;
        match *self {
            TransferContract(ref inner) => inner.owner_address,
            TransferAssetContract(ref inner) => inner.owner_address,
            AssetIssueContract(ref inner) => inner.owner_address,
            ParticipateAssetIssueContract(ref inner) => inner.owner_address,
            WitnessCreateContract(ref inner) => inner.owner_address,
            WithdrawBalanceContract(ref inner) => inner.owner_address,
            UpdateBrokerageContract(ref inner) => inner.owner_address,
            VoteWitnessContract(ref inner) => inner.owner_address,
            FreezeBalanceContract(ref inner) => inner.owner_address,
            UnfreezeBalanceContract(ref inner) => inner.owner_address,
            ProposalCreateContract(ref inner) => inner.owner_address,
            ProposalApproveContract(ref inner) => inner.owner_address,
            ProposalDeleteContract(ref inner) => inner.owner_address,
            CreateSmartContract(ref inner) => inner.owner_address,
            TriggerSmartContract(ref inner) => inner.owner_address,
            AccountCreateContract(ref inner) => inner.owner_address,
            AccountUpdateContract(ref inner) => inner.owner_address,
            AccountPermissionUpdateContract(ref inner) => inner.owner_address,
            WitnessUpdateContract(ref inner) => inner.owner_address,
            UnfreezeAssetContract(ref inner) => inner.owner_address,
            UpdateAssetContract(ref inner) => inner.owner_address,
            SetAccountIdContract(ref inner) => inner.owner_address,
            UpdateSettingContract(ref inner) => inner.owner_address,
            UpdateEnergyLimitContract(ref inner) => inner.owner_address,
            ClearABIContract(ref inner) => inner.owner_address,
        }
    }

    pub fn to_address(&self) -> Option<Address> {
        use self::Contract::*;
        match *self {
            TransferContract(ref inner) => Some(inner.to_address),
            TransferAssetContract(ref inner) => Some(inner.to_address),
            TriggerSmartContract(ref inner) => Some(inner.contract_address),
            AccountCreateContract(ref inner) => Some(inner.account_address),
            _ => None,
        }
    }
}

impl From<&ContractPb> for Contract {
    fn from(pb: &ContractPb) -> Self {
        use prost::Message;
        use proto::chain::ContractType;
        use proto::contract as contract_pb;

        let raw = &pb.parameter.as_ref().unwrap().value[..];

        match ContractType::from_i32(pb.r#type) {
            Some(ContractType::TransferContract) => {
                let cntr = contract_pb::TransferContract::decode(raw).unwrap();
                let inner = TransferContract {
                    owner_address: Address(cntr.owner_address.try_into().unwrap()),
                    to_address: Address(cntr.to_address.try_into().unwrap()),
                    amount: cntr.amount.into(),
                };
                Contract::TransferContract(inner)
            }
            Some(ContractType::TransferAssetContract) => {
                let cntr = contract_pb::TransferAssetContract::decode(raw).unwrap();
                let inner = TransferAssetContract {
                    owner_address: Address(cntr.owner_address.try_into().unwrap()),
                    to_address: Address(cntr.to_address.try_into().unwrap()),
                    token_id: cntr.asset_name.parse().ok(),
                    token_name: Some(cntr.asset_name),
                    amount: cntr.amount.into(),
                };
                Contract::TransferAssetContract(inner)
            }
            Some(ContractType::AssetIssueContract) => {
                let cntr = contract_pb::AssetIssueContract::decode(raw).unwrap();
                let inner = AssetIssueContract {
                    owner_address: Address(cntr.owner_address.try_into().unwrap()),
                    name: cntr.name.clone(),
                    abbr: cntr.abbr.clone(),
                    description: hex::encode(&cntr.description),
                    url: cntr.url.clone(),
                    total_supply: cntr.total_supply.into(),
                    frozen_supply: cntr
                        .frozen_supply
                        .iter()
                        .map(|sup| FrozenSupply {
                            frozen_amount: sup.frozen_amount.into(),
                            frozen_days: sup.frozen_days as _,
                        })
                        .collect(),
                    num: cntr.num as _,
                    trx_num: cntr.trx_num as _,
                    precision: cntr.precision as _,
                    start_time: Utc.timestamp(cntr.start_time / 1_000, cntr.start_time as u32 % 1_000 * 1_000_000),
                    end_time: Utc.timestamp(cntr.end_time / 1_000, cntr.end_time as u32 % 1_000 * 1_000_000),
                    /// FreeAssetNetLimit
                    free_bandwidth: cntr.free_asset_bandwidth_limit.into(),
                    /// PublicFreeAssetNetLimit
                    public_free_bandwidth: cntr.public_free_asset_bandwidth_limit.into(),
                };
                Contract::AssetIssueContract(inner)
            }
            Some(ContractType::ParticipateAssetIssueContract) => {
                let cntr = contract_pb::ParticipateAssetIssueContract::decode(raw).unwrap();
                let inner = ParticipateAssetIssueContract {
                    owner_address: Address(cntr.owner_address.try_into().unwrap()),
                    to_address: Address(cntr.to_address.try_into().unwrap()),
                    token_id: cntr.asset_name.parse().ok(),
                    token_name: Some(cntr.asset_name),
                    amount: cntr.amount.into(),
                };
                Contract::ParticipateAssetIssueContract(inner)
            }
            Some(ContractType::FreezeBalanceContract) => {
                let cntr = contract_pb::FreezeBalanceContract::decode(raw).unwrap();
                let inner = FreezeBalanceContract {
                    owner_address: Address(cntr.owner_address.try_into().unwrap()),
                    receiver_address: cntr.receiver_address.try_into().map(Address).ok(),
                    frozen_balance: cntr.frozen_balance.into(),
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
                    owner_address: Address(cntr.owner_address.try_into().unwrap()),
                    receiver_address: cntr.receiver_address.try_into().map(Address).ok(),
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
                    owner_address: Address(cntr.owner_address.try_into().unwrap()),
                    url: String::from_utf8(cntr.url).unwrap(),
                };
                Contract::WitnessCreateContract(inner)
            }
            Some(ContractType::WithdrawBalanceContract) => {
                let cntr = contract_pb::WithdrawBalanceContract::decode(raw).unwrap();
                let inner = WithdrawBalanceContract {
                    owner_address: Address(cntr.owner_address.try_into().unwrap()),
                };
                Contract::WithdrawBalanceContract(inner)
            }
            Some(ContractType::UpdateBrokerageContract) => {
                let cntr = contract_pb::UpdateBrokerageContract::decode(raw).unwrap();
                let inner = UpdateBrokerageContract {
                    owner_address: Address(cntr.owner_address.try_into().unwrap()),
                    brokerage: cntr.brokerage as _,
                };
                Contract::UpdateBrokerageContract(inner)
            }
            Some(ContractType::VoteWitnessContract) => {
                let contract_pb::VoteWitnessContract {
                    owner_address, votes, ..
                } = contract_pb::VoteWitnessContract::decode(raw).unwrap();
                let inner = VoteWitnessContract {
                    owner_address: Address(owner_address.try_into().unwrap()),
                    votes: votes
                        .into_iter()
                        .map(|vote| Vote {
                            vote_address: Address(vote.vote_address.try_into().unwrap()),
                            count: vote.vote_count.into(),
                        })
                        .collect(),
                };
                Contract::VoteWitnessContract(inner)
            }
            Some(ContractType::CreateSmartContract) => {
                let contract_pb::CreateSmartContract {
                    owner_address,
                    new_contract,
                    call_token_value,
                    call_token_id,
                } = contract_pb::CreateSmartContract::decode(raw).unwrap();
                let new_contract = new_contract.unwrap();

                let new_smart_contract = SmartContract {
                    origin_address: Address(new_contract.origin_address.try_into().unwrap()),
                    name: new_contract.name.clone(),
                    abi: new_contract
                        .abi
                        .as_ref()
                        .map(|abi| &abi.entries)
                        .and_then(|entries| serde_json::to_string(entries).ok()),
                    code: Bytes(new_contract.bytecode.clone()),
                    user_resource_percent: new_contract.consume_user_energy_percent as _,
                    origin_energy_limit: new_contract.origin_energy_limit.into(),
                    contract_address: if !new_contract.contract_address.is_empty() {
                        Some(Address(new_contract.contract_address.try_into().unwrap()))
                    } else {
                        None
                    },
                    code_hash: if !new_contract.code_hash.is_empty() {
                        Some(H256::from_slice(&new_contract.code_hash).into())
                    } else {
                        None
                    },
                };
                let inner = CreateSmartContract {
                    owner_address: owner_address.try_into().map(Address).unwrap(),
                    new_smart_contract,
                    call_value: new_contract.call_value.into(),
                    call_token_value: call_token_value.into(),
                    call_token_id: call_token_id as _,
                };
                Contract::CreateSmartContract(inner)
            }
            Some(ContractType::TriggerSmartContract) => {
                let cntr = contract_pb::TriggerSmartContract::decode(raw).unwrap();
                let inner = TriggerSmartContract {
                    owner_address: cntr.owner_address.try_into().map(Address).unwrap(),
                    contract_address: cntr.contract_address.try_into().map(Address).unwrap(),
                    data: Bytes(cntr.data.clone()),
                    call_value: cntr.call_value.into(),
                    call_token_value: cntr.call_token_value.into(),
                    call_token_id: cntr.call_token_id as _,
                };
                Contract::TriggerSmartContract(inner)
            }
            Some(ContractType::ProposalCreateContract) => {
                let cntr = contract_pb::ProposalCreateContract::decode(raw).unwrap();
                let inner = ProposalCreateContract {
                    owner_address: cntr.owner_address.try_into().map(Address).unwrap(),
                    parameters: cntr
                        .parameters
                        .iter()
                        .map(|(&k, &v)| Parameter {
                            key: k as _,
                            value: v.into(),
                        })
                        .collect(),
                };
                Contract::ProposalCreateContract(inner)
            }
            Some(ContractType::ProposalApproveContract) => {
                let cntr = contract_pb::ProposalApproveContract::decode(raw).unwrap();
                let inner = ProposalApproveContract {
                    owner_address: cntr.owner_address.try_into().map(Address).unwrap(),
                    proposal_id: cntr.proposal_id as _,
                    is_approve: cntr.is_approval,
                };
                Contract::ProposalApproveContract(inner)
            }
            Some(ContractType::ProposalDeleteContract) => {
                let cntr = contract_pb::ProposalDeleteContract::decode(raw).unwrap();
                let inner = ProposalDeleteContract {
                    owner_address: cntr.owner_address.try_into().map(Address).unwrap(),
                    proposal_id: cntr.proposal_id as _,
                };
                Contract::ProposalDeleteContract(inner)
            }
            Some(ContractType::AccountCreateContract) => {
                let cntr = contract_pb::AccountCreateContract::decode(raw).unwrap();
                let inner = AccountCreateContract {
                    owner_address: cntr.owner_address.try_into().map(Address).unwrap(),
                    account_address: cntr.account_address.try_into().map(Address).unwrap(),
                    r#type: AccountType::from_i32(cntr.r#type),
                };
                Contract::AccountCreateContract(inner)
            }
            Some(ContractType::AccountUpdateContract) => {
                let cntr = contract_pb::AccountUpdateContract::decode(raw).unwrap();
                let inner = AccountUpdateContract {
                    owner_address: cntr.owner_address.try_into().map(Address).unwrap(),
                    account_name: cntr.account_name.clone(),
                };
                Contract::AccountUpdateContract(inner)
            }
            Some(ContractType::AccountPermissionUpdateContract) => {
                let contract_pb::AccountPermissionUpdateContract {
                    owner_address,
                    owner,
                    witness,
                    actives,
                } = contract_pb::AccountPermissionUpdateContract::decode(raw).unwrap();
                let inner = AccountPermissionUpdateContract {
                    owner_address: owner_address.try_into().map(Address).unwrap(),
                    owner: owner.map(Permission::from).unwrap(),
                    witness: witness.map(Permission::from),
                    actives: actives.into_iter().map(Permission::from).collect(),
                };
                Contract::AccountPermissionUpdateContract(inner)
            }
            Some(typ) => unimplemented!("unhandled type {:?}", typ),
            None => unreachable!(),
        }
    }
}
