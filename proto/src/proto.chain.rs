// ref: https://github.com/paritytech/parity-bitcoin/blob/master/chain/README.md

/// A blockchain is a chain of blocks.
///
/// A block is a data structure with two fields:
/// - Block header: a data structure containing the block's metadata
/// - Transactions: an array (vector in rust) of transactions
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Block {
    #[prost(message, repeated, tag="1")]
    pub transactions: ::prost::alloc::vec::Vec<Transaction>,
    #[prost(message, optional, tag="2")]
    pub block_header: ::core::option::Option<BlockHeader>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockHeader {
    #[prost(message, optional, tag="1")]
    pub raw_data: ::core::option::Option<block_header::Raw>,
    #[prost(bytes="vec", tag="2")]
    pub witness_signature: ::prost::alloc::vec::Vec<u8>,
}
/// Nested message and enum types in `BlockHeader`.
pub mod block_header {
    /// renamed: raw
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Raw {
        #[prost(int64, tag="1")]
        pub timestamp: i64,
        /// renamed: txTrieRoot
        /// the root hash of merkle tree
        #[prost(bytes="vec", tag="2")]
        pub merkle_root_hash: ::prost::alloc::vec::Vec<u8>,
        /// renamed: parentHash
        #[prost(bytes="vec", tag="3")]
        pub parent_hash: ::prost::alloc::vec::Vec<u8>,
        /// bytes nonce = 5;
        /// bytes difficulty = 6;
        #[prost(int64, tag="7")]
        pub number: i64,
        /// seemed unused
        /// int64 witness_id = 8;
        #[prost(bytes="vec", tag="9")]
        pub witness_address: ::prost::alloc::vec::Vec<u8>,
        #[prost(int32, tag="10")]
        pub version: i32,
        /// renamed: accountStateRoot, First appares in block=8222293
        #[prost(bytes="vec", tag="11")]
        pub account_state_root: ::prost::alloc::vec::Vec<u8>,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    #[prost(message, optional, tag="1")]
    pub raw_data: ::core::option::Option<transaction::Raw>,
    /// normally size = 1, repeated list here for multi-sig extension
    #[prost(bytes="vec", repeated, tag="2")]
    pub signatures: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    /// renamed: ret
    /// NOTE: Many malformed transactions with len(result) = 2.
    /// NOTE: Cannot make it a non-repeated, since only last will be returned, while the first is the actual result.
    #[prost(message, repeated, tag="5")]
    pub result: ::prost::alloc::vec::Vec<transaction::Result>,
    /// NOTE: guess from wrong format
    /// in txn 17e597a68ea38205ca3d6724fc9733563d60879fc2118a52303f515fa1f36fec
    /// also, might be a default
    /// c455d5dd001ffff9216b2673095a6d9f4ff0aaadb921a7399608e7654cc3e5d9
    #[prost(oneof="transaction::ObsoleteRawWrapper", tags="6")]
    pub obsolete_raw_wrapper: ::core::option::Option<transaction::ObsoleteRawWrapper>,
    /// might be right or wrong
    /// like in 901af9115d4944a87d2923be2d67ae5e3fc0df4dcb8867bb952612a551695116
    #[prost(oneof="transaction::ObsoleteTxidWrapper", tags="7")]
    pub obsolete_txid_wrapper: ::core::option::Option<transaction::ObsoleteTxidWrapper>,
}
/// Nested message and enum types in `Transaction`.
pub mod transaction {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Contract {
        #[prost(enumeration="super::ContractType", tag="1")]
        pub r#type: i32,
        #[prost(message, optional, tag="2")]
        pub parameter: ::core::option::Option<::prost_types::Any>,
        /// NOE: Not used.
        #[prost(bytes="vec", tag="3")]
        pub provider: ::prost::alloc::vec::Vec<u8>,
        /// renamed: ContractName
        #[prost(bytes="vec", tag="4")]
        pub contract_name: ::prost::alloc::vec::Vec<u8>,
        /// renamed: Permission_id
        #[prost(int32, tag="5")]
        pub permission_id: i32,
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Result {
        /// renamed: ret
        /// NOTE: Actually meaningless in a block storage. Always = 0
        #[prost(enumeration="result::Status", tag="2")]
        pub status: i32,
        /// renamed: contractRet
        #[prost(enumeration="result::ContractStatus", tag="3")]
        pub contract_status: i32,
        /// NOTE: All following fields are not used. Some can be found in TransactionInfo.
        #[prost(int64, tag="1")]
        pub fee: i64,
        /// renamed: assetIssueID
        #[prost(string, tag="14")]
        pub asset_issue_id: ::prost::alloc::string::String,
        #[prost(int64, tag="15")]
        pub withdraw_amount: i64,
        #[prost(int64, tag="16")]
        pub unfreeze_amount: i64,
        #[prost(int64, tag="18")]
        pub exchange_received_amount: i64,
        #[prost(int64, tag="19")]
        pub exchange_inject_another_amount: i64,
        #[prost(int64, tag="20")]
        pub exchange_withdraw_another_amount: i64,
        #[prost(int64, tag="21")]
        pub exchange_id: i64,
        #[prost(int64, tag="22")]
        pub shielded_transaction_fee: i64,
        /// NOTE: 2018/08/02 block=1102553
        /// Then it was wrongly deleted, and wrongly used
        /// e.g. txn: 97d6802de90da0d7e680c184c5780c4396b45ae8df83e69e05c5ae7d3fca3987
        /// there might be empty bytes, which will be omitted by protobuf encoder
        /// `oneof` forces encoding of default empty value.
        #[prost(oneof="result::ObsoleteResourceReceiptWrapper", tags="4")]
        pub obsolete_resource_receipt_wrapper: ::core::option::Option<result::ObsoleteResourceReceiptWrapper>,
    }
    /// Nested message and enum types in `Result`.
    pub mod result {
        /// renamed: code
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
        #[repr(i32)]
        pub enum Status {
            Sucess = 0,
            Failed = 1,
        }
        /// renamed: contractResult
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
        #[repr(i32)]
        pub enum ContractStatus {
            Default = 0,
            Success = 1,
            /// Used by CreatSmartContract and TriggerSmartContract
            Revert = 2,
            IllegalOperation = 8,
            OutOfTime = 11,
            OutOfEnergy = 10,
            /// TransferException
            TransferFailed = 14,
            /// Un-caught exception
            Unknown = 13,
            /// Maybe not used
            BadJumpDestination = 3,
            OutOfMemory = 4,
            PrecompiledContract = 5,
            StackTooSmall = 6,
            StackTooLarge = 7,
            StackOverflow = 9,
            JvmStackOverFlow = 12,
        }
        /// NOTE: 2018/08/02 block=1102553
        /// Then it was wrongly deleted, and wrongly used
        /// e.g. txn: 97d6802de90da0d7e680c184c5780c4396b45ae8df83e69e05c5ae7d3fca3987
        /// there might be empty bytes, which will be omitted by protobuf encoder
        /// `oneof` forces encoding of default empty value.
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum ObsoleteResourceReceiptWrapper {
            #[prost(bytes, tag="4")]
            ObsoleteResourceReceipt(::prost::alloc::vec::Vec<u8>),
        }
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Raw {
        #[prost(bytes="vec", tag="1")]
        pub ref_block_bytes: ::prost::alloc::vec::Vec<u8>,
        #[prost(bytes="vec", tag="4")]
        pub ref_block_hash: ::prost::alloc::vec::Vec<u8>,
        /// max = now + 86400_000
        #[prost(int64, tag="8")]
        pub expiration: i64,
        /// used as transaction memo
        /// max size = 512000
        #[prost(bytes="vec", tag="10")]
        pub data: ::prost::alloc::vec::Vec<u8>,
        /// only support size = 1, repeated list here for extension
        /// changed: from repeated to optional(default for proto3)
        #[prost(message, optional, tag="11")]
        pub contract: ::core::option::Option<Contract>,
        /// unused, but is filled with creation time.
        #[prost(int64, tag="14")]
        pub timestamp: i64,
        /// max energy fee limit
        #[prost(int64, tag="18")]
        pub fee_limit: i64,
        /// unused
        #[prost(int64, tag="3")]
        pub ref_block_num: i64,
        /// unused, changed from Authority type
        #[prost(bytes="vec", repeated, tag="9")]
        pub auths: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
        /// unused
        #[prost(bytes="vec", tag="12")]
        pub scripts: ::prost::alloc::vec::Vec<u8>,
        /// in commit ae0075bd6d433f6bfb2ecbb74e5f380ee819dbc8
        /// in txn a5262325574c1cd4f0b7e0ea3d099d8546f47c72f8c165b792971f52d67d436c
        /// there might be an encoded default`0`:
        /// 2b95b265f75fd0f91c3cd39b428d104bd80c5344f3bb4d5c06eede2ff542f8a9
        #[prost(oneof="raw::ObsoleteMaxCpuUsageWrapper", tags="15")]
        pub obsolete_max_cpu_usage_wrapper: ::core::option::Option<raw::ObsoleteMaxCpuUsageWrapper>,
    }
    /// Nested message and enum types in `Raw`.
    pub mod raw {
        /// in commit ae0075bd6d433f6bfb2ecbb74e5f380ee819dbc8
        /// in txn a5262325574c1cd4f0b7e0ea3d099d8546f47c72f8c165b792971f52d67d436c
        /// there might be an encoded default`0`:
        /// 2b95b265f75fd0f91c3cd39b428d104bd80c5344f3bb4d5c06eede2ff542f8a9
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum ObsoleteMaxCpuUsageWrapper {
            #[prost(int64, tag="15")]
            ObsoleteMaxCpuUsage(i64),
        }
    }
    /// NOTE: guess from wrong format
    /// in txn 17e597a68ea38205ca3d6724fc9733563d60879fc2118a52303f515fa1f36fec
    /// also, might be a default
    /// c455d5dd001ffff9216b2673095a6d9f4ff0aaadb921a7399608e7654cc3e5d9
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum ObsoleteRawWrapper {
        #[prost(bytes, tag="6")]
        ObsoleteRaw(::prost::alloc::vec::Vec<u8>),
    }
    /// might be right or wrong
    /// like in 901af9115d4944a87d2923be2d67ae5e3fc0df4dcb8867bb952612a551695116
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum ObsoleteTxidWrapper {
        #[prost(bytes, tag="7")]
        ObsoleteTxid(::prost::alloc::vec::Vec<u8>),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ContractType {
    /// NOTE: unused or deprecated
    /// BuyStorageContract = 21;
    /// BuyStorageBytesContract = 22;
    /// SellStorageContract = 23;
    AccountCreateContract = 0,
    TransferContract = 1,
    TransferAssetContract = 2,
    /// NOTE: only used in active permission bits.
    ObsoleteVoteAssetContract = 3,
    VoteWitnessContract = 4,
    WitnessCreateContract = 5,
    AssetIssueContract = 6,
    WitnessUpdateContract = 8,
    ParticipateAssetIssueContract = 9,
    AccountUpdateContract = 10,
    FreezeBalanceContract = 11,
    UnfreezeBalanceContract = 12,
    WithdrawBalanceContract = 13,
    UnfreezeAssetContract = 14,
    UpdateAssetContract = 15,
    ProposalCreateContract = 16,
    ProposalApproveContract = 17,
    ProposalDeleteContract = 18,
    SetAccountIdContract = 19,
    /// NOTE: only used in active permission bits.
    ObsoleteCustomContract = 20,
    CreateSmartContract = 30,
    TriggerSmartContract = 31,
    /// NOTE: only used in active permission bits.
    ObsoleteGetContract = 32,
    UpdateSettingContract = 33,
    ExchangeCreateContract = 41,
    ExchangeInjectContract = 42,
    ExchangeWithdrawContract = 43,
    ExchangeTransactionContract = 44,
    UpdateEnergyLimitContract = 45,
    AccountPermissionUpdateContract = 46,
    ClearAbiContract = 48,
    UpdateBrokerageContract = 49,
    /// NOTE: Only used in active permission bits, and nile testnet.
    ShieldedTransferContract = 51,
    /// NOTE: Introduced in 4.1
    MarketSellAssetContract = 52,
    MarketCancelOrderContract = 53,
}
