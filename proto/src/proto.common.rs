// for discover, handshake, channel

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Endpoint {
    /// type changed: bytes
    #[prost(string, tag="1")]
    pub address: ::prost::alloc::string::String,
    #[prost(int32, tag="2")]
    pub port: i32,
    #[prost(bytes="vec", tag="3")]
    pub node_id: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockId {
    #[prost(bytes="vec", tag="1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="2")]
    pub number: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Vote {
    #[prost(bytes="vec", tag="1")]
    pub vote_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="2")]
    pub vote_count: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SmartContract {
    #[prost(bytes="vec", tag="1")]
    pub origin_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub contract_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="3")]
    pub abi: ::core::option::Option<smart_contract::Abi>,
    #[prost(bytes="vec", tag="4")]
    pub bytecode: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="5")]
    pub call_value: i64,
    #[prost(int64, tag="6")]
    pub consume_user_energy_percent: i64,
    #[prost(string, tag="7")]
    pub name: ::prost::alloc::string::String,
    #[prost(int64, tag="8")]
    pub origin_energy_limit: i64,
    #[prost(bytes="vec", tag="9")]
    pub code_hash: ::prost::alloc::vec::Vec<u8>,
    /// When smart contract is created by a trigger smart contract call.
    /// renamed: trx_hash
    #[prost(bytes="vec", tag="10")]
    pub txn_hash: ::prost::alloc::vec::Vec<u8>,
}
/// Nested message and enum types in `SmartContract`.
pub mod smart_contract {
    #[derive(serde::Serialize)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Abi {
        /// renamed: entrys
        #[prost(message, repeated, tag="1")]
        pub entries: ::prost::alloc::vec::Vec<abi::Entry>,
    }
    /// Nested message and enum types in `ABI`.
    pub mod abi {
        #[derive(serde::Serialize)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Param {
            /// This will cause the respective arguments to be searched for.
            /// If arrays (including string and bytes) are used as indexed arguments,
            /// the Keccak-256 hash of it is stored as topic instead.
            #[prost(bool, tag="1")]
            pub indexed: bool,
            #[prost(string, tag="2")]
            pub name: ::prost::alloc::string::String,
            /// SolidityType type = 3;
            #[prost(string, tag="3")]
            pub r#type: ::prost::alloc::string::String,
        }
        #[derive(serde::Serialize)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Entry {
            /// The event was declared as `anonymous`
            #[prost(bool, tag="1")]
            pub anonymous: bool,
            /// Replaced by view and pure.
            #[prost(bool, tag="2")]
            pub constant: bool,
            #[prost(string, tag="3")]
            pub name: ::prost::alloc::string::String,
            #[prost(message, repeated, tag="4")]
            pub inputs: ::prost::alloc::vec::Vec<Param>,
            #[prost(message, repeated, tag="5")]
            pub outputs: ::prost::alloc::vec::Vec<Param>,
            #[prost(enumeration="EntryType", tag="6")]
            pub r#type: i32,
            #[prost(bool, tag="7")]
            pub payable: bool,
            #[prost(enumeration="StateMutabilityType", tag="8")]
            pub state_mutability: i32,
        }
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
        #[repr(i32)]
        pub enum EntryType {
            UnknownEntryType = 0,
            Constructor = 1,
            Function = 2,
            Event = 3,
            /// Fallback functions are executed whenever a particular contract receives
            /// plain Ether without any other data associated with the transaction.
            Fallback = 4,
            /// Added in 4.1.2, for Solidity 0.6.0
            Receive = 5,
        }
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
        #[repr(i32)]
        pub enum StateMutabilityType {
            UnknownStateMutabilityType = 0,
            /// With pure you cannot access the contract storage.
            /// e.g. utility libraries.
            Pure = 1,
            /// With view you cannot modify the contract storage, but you can access the storage.
            /// e.g. contract getters.
            View = 2,
            Nonpayable = 3,
            Payable = 4,
        }
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Permission {
    #[prost(enumeration="permission::PermissionType", tag="1")]
    pub r#type: i32,
    /// Owner id=0, Witness id=1, Active id starts by 2
    #[prost(int32, tag="2")]
    pub id: i32,
    #[prost(string, tag="3")]
    pub name: ::prost::alloc::string::String,
    #[prost(int64, tag="4")]
    pub threshold: i64,
    #[prost(int32, tag="5")]
    pub parent_id: i32,
    /// 1 bit for 1 contract type
    #[prost(bytes="vec", tag="6")]
    pub operations: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, repeated, tag="7")]
    pub keys: ::prost::alloc::vec::Vec<permission::Key>,
}
/// Nested message and enum types in `Permission`.
pub mod permission {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Key {
        #[prost(bytes="vec", tag="1")]
        pub address: ::prost::alloc::vec::Vec<u8>,
        #[prost(int64, tag="2")]
        pub weight: i64,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum PermissionType {
        Owner = 0,
        Witness = 1,
        Active = 2,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ResourceCode {
    Bandwidth = 0,
    Energy = 1,
}
// for contract

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum AccountType {
    Normal = 0,
    AssetIssue = 1,
    Contract = 2,
}
