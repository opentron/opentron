use daemonize::Daemonize;
use std::fs::File;
use std::sync::{Arc, RwLock};
use tonic::{transport::Server, Request, Response, Status};
use wallet::Wallet;

use api::local_wallet_server::{LocalWallet, LocalWalletServer};
use api::{OpenRequest, StatusResponse};

pub mod api {
    tonic::include_proto!("network.tron.walletd");
}

#[derive(Default)]
pub struct LocalWalletService {
    wallet: Arc<RwLock<Option<Wallet>>>,
}

#[tonic::async_trait]
impl LocalWallet for LocalWalletService {
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
    let stdout = File::create("/tmp/walletd.out").unwrap();
    let stderr = File::create("/tmp/walletd.err").unwrap();

    let addr = "[::1]:12020".parse().unwrap();
    let service = LocalWalletService::default();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/walletd.pid")
        .stdout(stdout)
        .stderr(stderr);
    match daemonize.start() {
        Ok(_) => println!("Success, daemonized"),
        Err(e) => eprintln!("Error, {}", e),
    }

    println!("LocalWalletService listening on {}", addr);
    Server::builder()
        .add_service(LocalWalletServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
