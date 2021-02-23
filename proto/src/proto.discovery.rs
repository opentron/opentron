#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ping {
    #[prost(message, optional, tag="1")]
    pub from: ::core::option::Option<super::common::Endpoint>,
    #[prost(message, optional, tag="2")]
    pub to: ::core::option::Option<super::common::Endpoint>,
    #[prost(int32, tag="3")]
    pub version: i32,
    #[prost(int64, tag="4")]
    pub timestamp: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pong {
    #[prost(message, optional, tag="1")]
    pub from: ::core::option::Option<super::common::Endpoint>,
    #[prost(int32, tag="2")]
    pub echo_version: i32,
    #[prost(int64, tag="3")]
    pub timestamp: i64,
}
/// renamed: FindNeighbours
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FindPeers {
    #[prost(message, optional, tag="1")]
    pub from: ::core::option::Option<super::common::Endpoint>,
    #[prost(bytes="vec", tag="2")]
    pub target_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="3")]
    pub timestamp: i64,
}
/// renamed: Neighbours
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Peers {
    #[prost(message, optional, tag="1")]
    pub from: ::core::option::Option<super::common::Endpoint>,
    #[prost(message, repeated, tag="2")]
    pub peers: ::prost::alloc::vec::Vec<super::common::Endpoint>,
    #[prost(int64, tag="3")]
    pub timestamp: i64,
}
