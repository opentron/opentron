#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountResource {
    /// Normally free bandwidth limit is a const defined in code, 5000.
    #[prost(int64, tag="1")]
    pub free_bandwidth_used: i64,
    #[prost(int64, tag="3")]
    pub free_bandwidth_latest_slot: i64,
    #[prost(int64, tag="4")]
    pub frozen_bandwidth_used: i64,
    #[prost(int64, tag="6")]
    pub frozen_bandwidth_latest_slot: i64,
    #[prost(int64, tag="7")]
    pub energy_used: i64,
    #[prost(int64, tag="9")]
    pub energy_latest_slot: i64,
    #[prost(map="int64, int64", tag="10")]
    pub asset_bandwidth_used: ::std::collections::HashMap<i64, i64>,
    /// asset limit is saved in Asset
    #[prost(map="int64, int64", tag="12")]
    pub asset_bandwidth_latest_slot: ::std::collections::HashMap<i64, i64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PermissionKey {
    #[prost(bytes="vec", tag="1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="2")]
    pub weight: i64,
}
/// permission_id = 0
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OwnerPermission {
    #[prost(int64, tag="1")]
    pub threshold: i64,
    #[prost(message, repeated, tag="2")]
    pub keys: ::prost::alloc::vec::Vec<PermissionKey>,
}
/// permission_id = 2, 3, 4, ...
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActivePermission {
    #[prost(int64, tag="1")]
    pub threshold: i64,
    #[prost(message, repeated, tag="2")]
    pub keys: ::prost::alloc::vec::Vec<PermissionKey>,
    #[prost(bytes="vec", tag="3")]
    pub operations: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="5")]
    pub permission_name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Account {
    #[prost(enumeration="super::common::AccountType", tag="1")]
    pub r#type: i32,
    #[prost(string, tag="2")]
    pub name: ::prost::alloc::string::String,
    #[prost(int64, tag="3")]
    pub creation_time: i64,
    #[prost(int64, tag="4")]
    pub balance: i64,
    #[prost(map="int64, int64", tag="5")]
    pub token_balance: ::std::collections::HashMap<i64, i64>,
    #[prost(message, optional, tag="6")]
    pub resource: ::core::option::Option<AccountResource>,
    #[prost(int64, tag="7")]
    pub latest_operation_timestamp: i64,
    /// frozen resource
    #[prost(int64, tag="8")]
    pub frozen_amount_for_bandwidth: i64,
    #[prost(int64, tag="9")]
    pub frozen_amount_for_energy: i64,
    /// delegated in bandwidth
    #[prost(int64, tag="10")]
    pub delegated_frozen_amount_for_bandwidth: i64,
    /// delegated in energy
    #[prost(int64, tag="11")]
    pub delegated_frozen_amount_for_energy: i64,
    #[prost(int64, tag="12")]
    pub delegated_out_amount: i64,
    #[prost(int64, tag="13")]
    pub issued_asset_id: i64,
    #[prost(int64, tag="14")]
    pub latest_withdraw_timestamp: i64,
    #[prost(int64, tag="15")]
    pub allowance: i64,
    #[prost(message, optional, tag="16")]
    pub owner_permission: ::core::option::Option<OwnerPermission>,
    #[prost(message, repeated, tag="17")]
    pub active_permissions: ::prost::alloc::vec::Vec<ActivePermission>,
    /// deprecated, but still exists
    #[prost(bytes="vec", tag="20")]
    pub account_id: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Witness {
    #[prost(bytes="vec", tag="1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="2")]
    pub url: ::prost::alloc::string::String,
    /// renamed: is_jobs
    #[prost(bool, tag="3")]
    pub is_active: bool,
    #[prost(int64, tag="4")]
    pub vote_count: i64,
    #[prost(int64, tag="5")]
    pub total_produced: i64,
    #[prost(int64, tag="6")]
    pub total_missed: i64,
    #[prost(int64, tag="7")]
    pub latest_block_number: i64,
    #[prost(int64, tag="8")]
    pub latest_slot_number: i64,
    #[prost(int32, tag="9")]
    pub latest_block_version: i32,
    /// range: 0-100
    #[prost(int32, tag="10")]
    pub brokerage: i32,
    /// Witness permission, permission_id = 1
    #[prost(bytes="vec", tag="11")]
    pub signature_key: ::prost::alloc::vec::Vec<u8>,
}
/// Witness reward info of one epoch.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WitnessVoterReward {
    #[prost(int64, tag="1")]
    pub vote_count: i64,
    #[prost(int64, tag="3")]
    pub reward_amount: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceDelegation {
    #[prost(bytes="vec", tag="1")]
    pub to_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub from_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="3")]
    pub amount_for_bandwidth: i64,
    #[prost(int64, tag="4")]
    pub expiration_timestamp_for_bandwidth: i64,
    #[prost(int64, tag="5")]
    pub amount_for_energy: i64,
    #[prost(int64, tag="6")]
    pub expiration_timestamp_for_energy: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Votes {
    #[prost(message, repeated, tag="1")]
    pub votes: ::prost::alloc::vec::Vec<super::common::Vote>,
    /// used for calculating vote reward
    #[prost(int64, tag="2")]
    pub epoch: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Proposal {
    #[prost(int64, tag="1")]
    pub proposal_id: i64,
    #[prost(bytes="vec", tag="2")]
    pub proposer_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(map="int64, int64", tag="3")]
    pub parameters: ::std::collections::HashMap<i64, i64>,
    #[prost(int64, tag="4")]
    pub expiration_time: i64,
    #[prost(int64, tag="5")]
    pub creation_time: i64,
    #[prost(bytes="vec", repeated, tag="6")]
    pub approver_addresses: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(enumeration="proposal::State", tag="7")]
    pub state: i32,
}
/// Nested message and enum types in `Proposal`.
pub mod proposal {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum State {
        Pending = 0,
        Disapproved = 1,
        Approved = 2,
        Cancelled = 3,
    }
}
/// Internal transaction of smart contract call.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InternalTransaction {
    /// internalTransaction identity, the root InternalTransaction hash
    /// should equals to root transaction id.
    #[prost(bytes="vec", tag="1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    /// the one send trx, or token or call via function
    #[prost(bytes="vec", tag="2")]
    pub caller_address: ::prost::alloc::vec::Vec<u8>,
    /// the one recieve trx, or token or call via function
    #[prost(bytes="vec", tag="3")]
    pub to_address: ::prost::alloc::vec::Vec<u8>,
    /// call value
    #[prost(int64, tag="4")]
    pub call_value: i64,
    /// token id
    #[prost(int64, tag="5")]
    pub call_token_id: i64,
    /// token value
    #[prost(int64, tag="6")]
    pub call_token_value: i64,
    /// call data
    #[prost(bytes="vec", tag="8")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    /// caller function name: call, suicide
    #[prost(bytes="vec", tag="7")]
    pub note: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag="9")]
    pub accepted: bool,
}
/// renamed: AssetIssue
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Asset {
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
    pub frozen_supply: ::prost::alloc::vec::Vec<asset::FrozenSupply>,
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
    #[prost(int64, tag="22")]
    pub free_asset_bandwidth_limit: i64,
    #[prost(int64, tag="23")]
    pub public_free_asset_bandwidth_limit: i64,
    #[prost(int64, tag="24")]
    pub public_free_asset_bandwidth_used: i64,
    #[prost(int64, tag="25")]
    pub public_free_asset_bandwidth_last_slot: i64,
    /// changed: string
    #[prost(int64, tag="41")]
    pub id: i64,
}
/// Nested message and enum types in `Asset`.
pub mod asset {
    /// NOTE: This is different from contract.Asset.FrozenSupply.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FrozenSupply {
        #[prost(int64, tag="1")]
        pub frozen_amount: i64,
        #[prost(int64, tag="2")]
        pub frozen_expiry_timestamp: i64,
        #[prost(bool, tag="3")]
        pub is_unfrozen: bool,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Exchange {
    #[prost(int64, tag="1")]
    pub id: i64,
    #[prost(bytes="vec", tag="2")]
    pub owner_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="3")]
    pub creation_time: i64,
    #[prost(int64, tag="6")]
    pub first_token_id: i64,
    #[prost(int64, tag="7")]
    pub first_token_balance: i64,
    #[prost(int64, tag="8")]
    pub second_token_id: i64,
    #[prost(int64, tag="9")]
    pub second_token_balance: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionLog {
    /// contract address
    #[prost(bytes="vec", tag="1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    /// <<hash of topic signature, [indexed parameter]>>
    #[prost(bytes="vec", repeated, tag="2")]
    pub topics: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    /// <<[non-indexed parameter]>>
    #[prost(bytes="vec", tag="3")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResourceReceipt {
    #[prost(int64, tag="1")]
    pub energy_usage: i64,
    #[prost(int64, tag="2")]
    pub energy_fee: i64,
    #[prost(int64, tag="3")]
    pub origin_energy_usage: i64,
    #[prost(int64, tag="4")]
    pub energy: i64,
    #[prost(int64, tag="5")]
    pub bandwidth_usage: i64,
    /// when create a new account, usage = 0, fee = 0.1 TRX
    #[prost(int64, tag="6")]
    pub bandwidth_fee: i64,
    /// Oneof: asset issue, exchange create, witness create, account permission update.
    #[prost(int64, tag="7")]
    pub contract_fee: i64,
    #[prost(int64, tag="8")]
    pub multisig_fee: i64,
}
/// renamed: TransactionInfo
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionReceipt {
    #[prost(bytes="vec", tag="1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag="2")]
    pub success: bool,
    #[prost(int64, tag="3")]
    pub block_number: i64,
    #[prost(int64, tag="4")]
    pub block_timestamp: i64,
    #[prost(int64, tag="5")]
    pub fee: i64,
    #[prost(message, optional, tag="6")]
    pub resource_receipt: ::core::option::Option<ResourceReceipt>,
    #[prost(int64, tag="7")]
    pub asset_created_token_id: i64,
    #[prost(int64, tag="8")]
    pub withdrawal_amount: i64,
    #[prost(int64, tag="9")]
    pub unfrozen_amount: i64,
    #[prost(enumeration="super::chain::transaction::result::ContractStatus", tag="10")]
    pub vm_status: i32,
    /// VM error message
    #[prost(bytes="vec", tag="11")]
    pub vm_message: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="12")]
    pub vm_created_contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="13")]
    pub vm_result: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, repeated, tag="14")]
    pub vm_internal_transactions: ::prost::alloc::vec::Vec<InternalTransaction>,
    #[prost(message, repeated, tag="15")]
    pub vm_logs: ::prost::alloc::vec::Vec<TransactionLog>,
    #[prost(int64, tag="16")]
    pub exchange_created_exchange_id: i64,
    #[prost(int64, tag="17")]
    pub exchange_received_amount: i64,
    #[prost(int64, tag="18")]
    pub exchange_injected_amount: i64,
    #[prost(int64, tag="19")]
    pub exchange_withdrawal_amount: i64,
}
/// Chain parameters, known as proposals, can be changed via proposal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ChainParameter {
    /// # Chain core parameters.
    ///
    /// The maintenance interval of SRs.
    ///
    /// Renamed: `MaintenanceTimeInterval`
    ///
    /// Default: 6h, 21600000 in ms
    ///
    /// Range: [3 * 27 * 1000, 24 * 3600 * 1000] (27x3s ~ 1d)
    MaintenanceInterval = 0,
    /// Max execution duration of a TVM transaction(smart contract creation and triggering).
    ///
    /// Default: 50, in ms
    ///
    /// Renamed: `MaxCpuTimeOfOneTx`
    ///
    /// Range: [10, 100]
    MaxCpuTimeOfOneTxn = 13,
    /// Remove votes from geneses GRs(guard representative).
    ///
    /// Renamed: `RemoveThePowerOfTheGr`
    ///
    /// Default: 0, unremoved
    ///
    /// Mainnet: -1
    ///
    /// Note: 1 to remove, -1 denotes already removed
    RemovePowerOfGr = 10,
    /// Enabled: 4.1
    AllowPbft = 40,
    /// # New features and fixes of new versions.
    ///
    /// Allow update an accunt's name; Allow duplicate names.
    ///
    /// Default: 0
    ///
    /// Mainnet: 0
    AllowUpdateAccountName = 14,
    /// Default: config, 0
    AllowSameTokenName = 15,
    /// Default: config, 0
    AllowDelegateResource = 16,
    /// Enabled: 3.5
    ///
    /// Renamed: `AllowMultiSign`
    ///
    /// Default: config, 0
    AllowMultisig = 20,
    /// Enabled: 3.6
    ///
    /// Default: 0
    ///
    /// Mainnet: 0
    ///
    /// Note: Never appear as a proposal. Hidden in tronscan, but is avaliable as config options in java-tron.
    /// Block #8222293 has this field.
    AllowAccountStateRoot = 25,
    /// This enables TVM and allows creation of smart contracts.
    ///
    /// Renamed: `AllowCreationOfContracts`
    ///
    /// Default: config, 0
    AllowTvm = 9,
    /// Enabled: 3.6.6
    ///
    /// Requires: `AllowTvm`
    ///
    /// Default: 0
    ///
    /// Mainnet: 0 (not passed due to bad design)
    ForbidTransferToContract = 35,
    /// Add `UpdateBrokerageContract`, something to do with witness reward. Decentralized vote dividend.
    /// TODO: bad naming
    ///
    /// Renamed: `ChangeDelegation`
    ///
    /// Enabled: 3.6.5
    AllowChangeDelegation = 30,
    /// Add market transaction support.
    ///
    /// Enabled: 4.1
    AllowMarketTransaction = 44,
    /// Allow transaction fee pool.
    ///
    /// Enabled: 4.1.2
    ///
    /// Default: 0
    AllowTransactionFeePool = 48,
    /// Use BURN_TRX_AMOUNT instead of Blackhole.
    ///
    /// Enabled: 4.1.2
    ///
    /// Default: 0
    AllowBlackholeOptimization = 49,
    /// # Fees
    ///
    /// Fees, in SUN.
    /// Renamed: `TransactionFee`
    ///
    /// Default: 10
    BandwidthPrice = 3,
    /// Default: 100
    EnergyPrice = 11,
    /// Used in `CreateAccount`, `Transfer`, `TransferAsset`.
    ///
    /// Default: 0
    CreateNewAccountFeeInSystemContract = 7,
    /// Default: 1
    CreateNewAccountBandwidthRate = 8,
    /// The cost of applying to be an SR account.
    ///
    /// Renamed: `AccountUpgradeCost`
    ///
    /// Default: 9999_000_000
    WitnessCreateFee = 1,
    /// Account creation fee.
    ///
    /// Used in `CreateAccount`, `Transfer`, `TransferAsset`.
    ///
    /// Renamed: `CreateAccountFee`
    ///
    /// Default: 100_000
    AccountCreateFee = 2,
    /// Default: 1024_000_000
    AssetIssueFee = 4,
    /// Default: 1024_000_000
    ExchangeCreateFee = 12,
    /// Renamed: `UpdateAccountPermissionFee`
    ///
    /// Enabled: 3.5
    ///
    /// Range: [0, 100_000_000000]
    ///
    /// Default: 100_000_000
    AccountPermissionUpdateFee = 22,
    /// Renamed: `MultiSignFee`
    ///
    /// Enabled: 3.5
    ///
    /// Default: 1_000_000
    ///
    /// Range: [0, 100_000_000000]
    MultisigFee = 23,
    /// Market fees.
    MarketSellFee = 45,
    MarketCancelFee = 46,
    /// Max fee limit.
    ///
    /// Enabled: 4.1.2
    ///
    /// Default: 1000_000_000
    ///
    /// Range: 0, 10_000_000_000
    MaxFeeLimit = 47,
    /// # Energy / bandwidth model
    ///
    /// Adaptive energy.
    /// Enabled by EnergyLimit update, Deprecated by 3.2.2.
    ///
    /// Default: 50_000_000_000
    ///
    /// Note: uselesss now, but
    TotalEnergyLimit = 17,
    /// Enabled: 3.2.2
    ///
    /// Default: 50_000_000_000
    ///
    /// Mainnet: 90_000_000_000 (via proposal)
    TotalEnergyCurrentLimit = 19,
    /// Default: config, 0
    ///
    /// Mainnet: 0
    AllowAdaptiveEnergy = 21,
    /// derived from above
    /// TotalEnergyTargetLimit = 6250000
    /// TotalEnergyAverageUsage = 0
    /// Enabled: 3.6.5
    ///
    /// Default: 14400 (24 * 60 * 10)
    ///
    /// Mainnet: 10
    AdaptiveResourceLimitTargetRatio = 33,
    /// Enabled: 3.6.5
    ///
    /// Default: 1000
    AdaptiveResourceLimitMultiplier = 29,
    /// # Witness and rewarding
    ///
    /// Witness rewards. Standby witness = 27 active witnesses + 100 partner witnesses.
    /// SR block generation reward.
    ///
    /// Default: 32_000_000
    ///
    /// Mainnet: 16_000_000
    WitnessPayPerBlock = 5,
    /// In maintenance cycle, after new witness list is updated, All standby witnesses will be rewarded.
    /// Actual pay amount is divided by the total vote weight. Only when `AllowChangeDelegation` = false.
    ///
    /// Renamed: `WitnessStandbyAllowance`
    ///
    /// Requires: !`AllowChangeDelegation`
    ///
    /// Default: 115_200_000_000 = 16_000_000 * (21600_000 / 3_000)
    StandbyWitnessAllowance = 6,
    /// Each block, amount of TRX reward is paid to standby witness, actual pay amount is according to the vote weight.
    ///
    /// Renamed: `Witness127PayPerBlock`
    ///
    /// Requires: `AllowChangeDelegation`
    ///
    /// Enabled: 3.6.5
    ///
    /// Default: 16_000_000
    ///
    /// Mainnet: 160_000_000
    StandbyWitnessPayPerBlock = 31,
    /// # TVM updates
    ///
    /// TVM v3.2 update. CALLTOKEN, TOKENBALANCE, CALLTOKENVALUE, CALLTOKENID.
    ///
    /// Requires: `AllowSameTokenName`
    ///
    /// Default: config, 0
    AllowTvmTransferTrc10Upgrade = 18,
    /// TVM with shift instructions, CREATE2, EXTCODEHASH. ClearABI, forbid delegate resource to contract.
    ///
    /// Renamed: `AllowTvmConstantinople`
    ///
    /// Enabled: 3.6
    ///
    /// Requires: `AllowTvmTransferTrc10`
    ///
    /// Default: config, 0
    AllowTvmConstantinopleUpgrade = 26,
    /// TVM with `batchvalidatesign`, `validatemultisign`, `iscontract` support.
    ///
    /// Renamed: `AllowTvmSolidity059`
    ///
    /// Enabled: 3.6.5
    ///
    /// Requires: `AllowTvm`
    ///
    /// Default: config, 0
    AllowTvmSolidity059Upgrade = 32,
    /// TVM with librustzcash Shielded precompiles upgrade.
    ///
    /// Renamed: `AllowShieldedTRC20Transaction`
    ///
    /// Enabled: 4.0
    ///
    /// Default: config, 0
    AllowTvmShieldedUpgrade = 39,
    /// TVM 4.1 update. CHAINID, SELFBALANCE, CREATE2 support.
    ///
    /// Enabled: 4.1
    AllowTvmIstanbulUpgrade = 41,
    /// # Useless.
    ///
    /// Enabled: 3.6
    ///
    /// Default: 0
    AllowProtoFilterNum = 24,
    /// Legacy. Only available on nile testnet.
    ///
    /// Enabled: 3.7
    ///
    /// Default: 0
    AllowShieldedTransaction = 27,
}
