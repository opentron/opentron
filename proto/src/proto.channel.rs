#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HandshakeHello {
    #[prost(message, optional, tag="1")]
    pub from: ::core::option::Option<super::common::Endpoint>,
    #[prost(int32, tag="2")]
    pub version: i32,
    #[prost(int64, tag="3")]
    pub timestamp: i64,
    /// number=0
    #[prost(message, optional, tag="4")]
    pub genesis_block_id: ::core::option::Option<super::common::BlockId>,
    #[prost(message, optional, tag="5")]
    pub solid_block_id: ::core::option::Option<super::common::BlockId>,
    #[prost(message, optional, tag="6")]
    pub head_block_id: ::core::option::Option<super::common::BlockId>,
    #[prost(bytes="vec", tag="7")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="8")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HandshakeDisconnect {
    #[prost(enumeration="ReasonCode", tag="1")]
    pub reason: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChainInventory {
    #[prost(message, repeated, tag="1")]
    pub ids: ::prost::alloc::vec::Vec<super::common::BlockId>,
    #[prost(int64, tag="2")]
    pub remain_num: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockInventory {
    #[prost(message, repeated, tag="1")]
    pub ids: ::prost::alloc::vec::Vec<super::common::BlockId>,
    #[prost(enumeration="block_inventory::Type", tag="2")]
    pub r#type: i32,
}
/// Nested message and enum types in `BlockInventory`.
pub mod block_inventory {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        Sync = 0,
        /// unused
        Advtise = 1,
        /// unused
        Fetch = 2,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Inventory {
    #[prost(enumeration="inventory::Type", tag="1")]
    pub r#type: i32,
    #[prost(bytes="vec", repeated, tag="2")]
    pub ids: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
/// Nested message and enum types in `Inventory`.
pub mod inventory {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        Trx = 0,
        Block = 1,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transactions {
    #[prost(message, repeated, tag="1")]
    pub transactions: ::prost::alloc::vec::Vec<super::chain::Transaction>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ReasonCode {
    Requested = 0,
    BadProtocol = 2,
    TooManyPeers = 4,
    DuplicatePeer = 5,
    IncompatibleProtocol = 6,
    NullIdentity = 7,
    PeerQuiting = 8,
    UnexpectedIdentity = 9,
    LocalIdentity = 10,
    PingTimeout = 11,
    UserReason = 16,
    Reset = 17,
    SyncFail = 18,
    FetchFail = 19,
    BadTx = 20,
    BadBlock = 21,
    Forked = 22,
    Unlinkable = 23,
    IncompatibleVersion = 24,
    IncompatibleChain = 25,
    TimeOut = 32,
    ConnectFail = 33,
    TooManyPeersWithSameIp = 34,
    Unknown = 255,
}
