use grpc::ClientStub;
use lazy_static::lazy_static;
use proto::api_grpc::WalletClient;
use std::net::ToSocketAddrs;
use std::sync::Arc;

use crate::RPC_HOST;

lazy_static! {
    pub static ref GRPC_CLIENT: WalletClient = {
        let host = unsafe { RPC_HOST }
            .to_socket_addrs()
            .expect("can not resolve rpc host")
            .next()
            .expect("can not resolve rpc host");

        let grpc_client = Arc::new(grpc::Client::new_plain(
            &host.ip().to_string(),
            host.port(),
            Default::default(),
        ).expect("can not create gRPC client"));
        WalletClient::with_client(grpc_client)
    };
}
