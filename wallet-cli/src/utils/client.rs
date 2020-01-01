use grpc::ClientStub;
use proto::api::{BlockExtention, NumberMessage};
use proto::api_grpc::{Wallet, WalletClient};
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

pub fn get_latest_block(client: &WalletClient) -> Result<BlockExtention, Error> {
    let req = NumberMessage {
        num: 1,
        ..Default::default()
    };
    let (_, resp, _) = client.get_block_by_latest_num2(Default::default(), req).wait()?;
    resp.block
        .into_iter()
        .next()
        .ok_or(Error::Runtime("no latest block retrieved"))
}
