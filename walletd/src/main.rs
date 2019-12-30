use tonic::{transport::Server, Request, Response, Status};

use api::wallet_server::{Wallet, WalletServer};
use api::{OpenRequest, StatusResponse};

pub mod api {
    tonic::include_proto!("network.tron.walletd");
}

#[derive(Default)]
pub struct LocalWalletService;

#[tonic::async_trait]
impl Wallet for LocalWalletService {
    async fn open(&self, request: Request<OpenRequest>) -> Result<Response<StatusResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = StatusResponse {
            code: 200,
            message: "OK".to_owned(),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let service = LocalWalletService::default();

    println!("LocalWalletService listening on {}", addr);

    Server::builder()
        .add_service(WalletServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
