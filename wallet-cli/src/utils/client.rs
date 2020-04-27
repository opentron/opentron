use grpc::ClientStubExt;
use lazy_static::lazy_static;
use proto::api_grpc::WalletClient;
use std::net::ToSocketAddrs;

use crate::error::Error;
use crate::RPC_ADDR;

lazy_static! {
    pub static ref GRPC_CLIENT: WalletClient = {
        let addr = unsafe { RPC_ADDR }
            .to_socket_addrs()
            .ok()
            .and_then(|mut addrs| addrs.next())
            .expect("can not resolve rpc host");
        WalletClient::new_plain(&addr.ip().to_string(), addr.port(), Default::default())
            .expect("can not create gRPC client")
    };
}

pub fn new_grpc_client(host: &str) -> Result<WalletClient, Error> {
    let host = host
        .to_socket_addrs()
        .ok()
        .and_then(|mut addrs| addrs.next())
        .ok_or(Error::Runtime("can not resolve address"))?;
    Ok(WalletClient::new_plain(
        &host.ip().to_string(),
        host.port(),
        Default::default(),
    )?)
}
