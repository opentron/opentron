use grpc::ClientStub;
use proto::api_grpc::WalletClient;
use std::net::ToSocketAddrs;
use std::sync::Arc;

use crate::error::Error;
use crate::RPC_HOST;

pub fn new_grpc_client() -> Result<WalletClient, Error> {
    let host = unsafe { RPC_HOST }
        .to_socket_addrs()
        .map_err(Error::from)?
        .next()
        .ok_or(Error::Runtime("at least one host resolve result required"))?;

    let grpc_client = Arc::new(grpc::Client::new_plain(
        &host.ip().to_string(),
        host.port(),
        Default::default(),
    )?);
    Ok(WalletClient::with_client(grpc_client))
}
