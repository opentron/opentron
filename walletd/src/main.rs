use tonic::{transport::Server, Request, Response, Status};

use api::wallet_server::{Wallet, WalletServer};
use api::{CreateRequest, StatusResponse};

pub mod api {
    tonic::include_proto!("network.tron.walletd");
}

fn main() {
    println!("Hello, world!");
}
