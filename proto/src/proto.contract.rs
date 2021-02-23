// # Account

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountCreateContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub account_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(enumeration="super::common::AccountType", tag="3")]
    pub r#type: i32,
}
/// Update account name. Account name is not unique now.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountUpdateContract {
    /// max len = 200
    /// changed: bytes
    #[prost(string, tag="1")]
    pub account_name: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="2")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
}
/// Set account id if the account has no id. Account id is unique and case insensitive.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetAccountIdContract {
    #[prost(bytes="vec", tag="1")]
    pub account_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountPermissionUpdateContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    /// permission_id = 0
    #[prost(message, optional, tag="2")]
    pub owner: ::core::option::Option<super::common::Permission>,
    /// permission_id = 1
    #[prost(message, optional, tag="3")]
    pub witness: ::core::option::Option<super::common::Permission>,
    /// permission_id = 2, 3, 4, ...
    #[prost(message, repeated, tag="4")]
    pub actives: ::prost::alloc::vec::Vec<super::common::Permission>,
}
// # TRX transfer

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransferContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub to_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="3")]
    pub amount: i64,
}
// # TRC10

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransferAssetContract {
    /// this field is token name before the proposal ALLOW_SAME_TOKEN_NAME's
    /// activation, otherwise it is token id and token should be in string format.
    /// used as asset id str as bytes, LEN = 7 bytes: 100xx
    /// changed: bytes
    #[prost(bytes="vec", tag="2")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="3")]
    pub to_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="1")]
    pub asset_name: ::prost::alloc::string::String,
    #[prost(int64, tag="4")]
    pub amount: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetIssueContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    /// changed: bytes
    #[prost(string, tag="2")]
    pub name: ::prost::alloc::string::String,
    /// changed: bytes
    #[prost(string, tag="3")]
    pub abbr: ::prost::alloc::string::String,
    #[prost(int64, tag="4")]
    pub total_supply: i64,
    #[prost(message, repeated, tag="5")]
    pub frozen_supply: ::prost::alloc::vec::Vec<asset_issue_contract::FrozenSupply>,
    #[prost(int32, tag="6")]
    pub trx_num: i32,
    #[prost(int32, tag="7")]
    pub precision: i32,
    #[prost(int32, tag="8")]
    pub num: i32,
    #[prost(int64, tag="9")]
    pub start_time: i64,
    #[prost(int64, tag="10")]
    pub end_time: i64,
    /// NOTE: might be illegal utf8 bytes
    #[prost(bytes="vec", tag="20")]
    pub description: ::prost::alloc::vec::Vec<u8>,
    /// changed: bytes
    #[prost(string, tag="21")]
    pub url: ::prost::alloc::string::String,
    /// NOTE: rename net to bandwidth
    #[prost(int64, tag="22")]
    pub free_asset_bandwidth_limit: i64,
    #[prost(int64, tag="23")]
    pub public_free_asset_bandwidth_limit: i64,
    #[prost(int64, tag="24")]
    pub public_free_asset_bandwidth_usage: i64,
    /// useless, and not checked
    #[prost(int64, tag="25")]
    pub obsolete_public_latest_free_net_time: i64,
    #[prost(string, tag="41")]
    pub obsolete_id: ::prost::alloc::string::String,
    #[prost(int64, tag="11")]
    pub obsolete_order: i64,
    #[prost(int32, tag="16")]
    pub obsolete_vote_score: i32,
}
/// Nested message and enum types in `AssetIssueContract`.
pub mod asset_issue_contract {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FrozenSupply {
        #[prost(int64, tag="1")]
        pub frozen_amount: i64,
        #[prost(int64, tag="2")]
        pub frozen_days: i64,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ParticipateAssetIssueContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub to_address: ::prost::alloc::vec::Vec<u8>,
    /// this field is token name before the proposal ALLOW_SAME_TOKEN_NAME's
    /// activation, otherwise it is token id and token should be in string format.
    ///
    /// changed: bytes
    #[prost(string, tag="3")]
    pub asset_name: ::prost::alloc::string::String,
    /// the amount of drops
    #[prost(int64, tag="4")]
    pub amount: i64,
}
// NOTE: unused
//message VoteAssetContract {
//bytes owner_address = 1;
//repeated bytes vote_address = 2;
//// renamed: support
//bool is_support = 3;
//int32 count = 5;
//}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateAssetContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub description: ::prost::alloc::vec::Vec<u8>,
    /// changed: bytes
    #[prost(string, tag="3")]
    pub url: ::prost::alloc::string::String,
    #[prost(int64, tag="4")]
    pub new_limit: i64,
    #[prost(int64, tag="5")]
    pub new_public_limit: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnfreezeAssetContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
}
// # Witness

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WitnessCreateContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub url: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WitnessUpdateContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="12")]
    pub update_url: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateBrokerageContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    /// in percent
    #[prost(int32, tag="2")]
    pub brokerage: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteWitnessContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, repeated, tag="2")]
    pub votes: ::prost::alloc::vec::Vec<super::common::Vote>,
    /// not used
    #[prost(bool, tag="3")]
    pub is_support: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WithdrawBalanceContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
}
// # Smart Contract

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateSmartContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="2")]
    pub new_contract: ::core::option::Option<super::common::SmartContract>,
    #[prost(int64, tag="3")]
    pub call_token_value: i64,
    #[prost(int64, tag="4")]
    pub call_token_id: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TriggerSmartContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="3")]
    pub call_value: i64,
    #[prost(bytes="vec", tag="4")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="5")]
    pub call_token_value: i64,
    #[prost(int64, tag="6")]
    pub call_token_id: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateSettingContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="3")]
    pub consume_user_energy_percent: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateEnergyLimitContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="3")]
    pub origin_energy_limit: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClearAbiContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub contract_address: ::prost::alloc::vec::Vec<u8>,
}
// # Resource

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FreezeBalanceContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="2")]
    pub frozen_balance: i64,
    #[prost(int64, tag="3")]
    pub frozen_duration: i64,
    #[prost(enumeration="super::common::ResourceCode", tag="10")]
    pub resource: i32,
    #[prost(bytes="vec", tag="15")]
    pub receiver_address: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnfreezeBalanceContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(enumeration="super::common::ResourceCode", tag="10")]
    pub resource: i32,
    #[prost(bytes="vec", tag="15")]
    pub receiver_address: ::prost::alloc::vec::Vec<u8>,
}
// # Proposal

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalCreateContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(map="int64, int64", tag="2")]
    pub parameters: ::std::collections::HashMap<i64, i64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalApproveContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="2")]
    pub proposal_id: i64,
    /// renamed: is_add_approval
    #[prost(bool, tag="3")]
    pub is_approval: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProposalDeleteContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="2")]
    pub proposal_id: i64,
}
// # Exchange

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExchangeCreateContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="2")]
    pub first_token_id: ::prost::alloc::string::String,
    #[prost(int64, tag="3")]
    pub first_token_balance: i64,
    #[prost(string, tag="4")]
    pub second_token_id: ::prost::alloc::string::String,
    #[prost(int64, tag="5")]
    pub second_token_balance: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExchangeInjectContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="2")]
    pub exchange_id: i64,
    #[prost(string, tag="3")]
    pub token_id: ::prost::alloc::string::String,
    #[prost(int64, tag="4")]
    pub quant: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExchangeWithdrawContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="2")]
    pub exchange_id: i64,
    #[prost(string, tag="3")]
    pub token_id: ::prost::alloc::string::String,
    #[prost(int64, tag="4")]
    pub quant: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExchangeTransactionContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="2")]
    pub exchange_id: i64,
    #[prost(string, tag="3")]
    pub token_id: ::prost::alloc::string::String,
    #[prost(int64, tag="4")]
    pub quant: i64,
    #[prost(int64, tag="5")]
    pub expected: i64,
}
//
//message BuyStorageContract {
//bytes owner_address = 1;
//int64 quant = 2; // trx quantity for buy storage (in sun)
//}
//
//message BuyStorageBytesContract {
//bytes owner_address = 1;
//int64 bytes = 2; // storage bytes for buy
//}
//
//message SellStorageContract {
//bytes owner_address = 1;
//int64 storage_bytes = 2;
//}

/// NOTE: This is only used in nile testnet.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShieldedTransferContract {
    /// transparent address
    #[prost(bytes="vec", tag="1")]
    pub transparent_from_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="2")]
    pub from_amount: i64,
    /// changed: SpendDescription, ignore inner pb.
    #[prost(bytes="vec", repeated, tag="3")]
    pub spend_description: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    /// changed: ReceiveDescription, ignore inner pb.
    #[prost(bytes="vec", repeated, tag="4")]
    pub receive_description: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(bytes="vec", tag="5")]
    pub binding_signature: ::prost::alloc::vec::Vec<u8>,
    /// transparent address
    #[prost(bytes="vec", tag="6")]
    pub transparent_to_address: ::prost::alloc::vec::Vec<u8>,
    /// the amount to transparent to_address
    #[prost(int64, tag="7")]
    pub to_amount: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MarketSellAssetContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub sell_token_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="3")]
    pub sell_token_quantity: i64,
    #[prost(bytes="vec", tag="4")]
    pub buy_token_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="5")]
    pub buy_token_quantity: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MarketCancelOrderContract {
    #[prost(bytes="vec", tag="1")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub order_id: ::prost::alloc::vec::Vec<u8>,
}
