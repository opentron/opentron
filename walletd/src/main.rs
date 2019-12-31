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
        println!("Got a request from {:?} {:?}", request.remote_addr(), request.get_ref());
        println!("Current Wallet {:?}", &self.wallet);
        let name = &request.get_ref().name;

        {
            let mut w = (*self.wallet).write().unwrap();
            let wallet = Wallet::new(name).map_err(error_to_status)?;
            *w = Some(wallet);
        }

        let reply = StatusResponse {
            code: 200,
            message: "OK".to_owned(),
        };
        Ok(Response::new(reply))
    }
}

fn error_to_status(err: wallet::Error) -> Status {
    Status::unknown(format!("{:?}", err))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = File::create("/tmp/walletd.out").unwrap();
    let stderr = File::create("/tmp/walletd.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/walletd.pid")
        .stdout(stdout)
        .stderr(stderr);
    /*
    match daemonize.start() {
        Ok(_) => println!("Success, daemonized"),
        Err(e) => eprintln!("Error, {}", e),
    }
    */

    let addr = "[::1]:8888".parse().unwrap();
    let service = LocalWalletService::default();

    println!("LocalWalletService listening on {}", addr);
    Server::builder()
        .add_service(LocalWalletServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
