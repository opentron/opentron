use daemonize::Daemonize;
use std::fs::File;
use std::sync::{Arc, RwLock};
use tonic::{transport::Server, Request, Response, Status};
use wallet::Wallet;

use api::local_wallet_server::{LocalWallet, LocalWalletServer};
use api::KeyPair;
use api::{
    CreateKeyRequest, CreateKeyResponse, CreateRequest, LockRequest, OpenRequest, StatusResponse, UnlockRequest,
};

pub mod api {
    tonic::include_proto!("network.tron.walletd");
}

#[derive(Default)]
pub struct LocalWalletService {
    wallet: Arc<RwLock<Option<Wallet>>>,
}

#[tonic::async_trait]
impl LocalWallet for LocalWalletService {
    async fn create(&self, request: Request<CreateRequest>) -> Result<Response<StatusResponse>, Status> {
        println!("INFO request {:?} {:?}", request.remote_addr(), request.get_ref());
        let name = &request.get_ref().name;
        let password = &request.get_ref().password;

        let reply = match Wallet::create(name, password) {
            Ok(wallet) => {
                let mut w = (*self.wallet).write().unwrap();
                *w = Some(wallet);
                StatusResponse {
                    code: 200,
                    message: "OK".to_owned(),
                }
            }
            Err(e) => StatusResponse {
                code: 500,
                message: format!("Can not create wallet: {:}", e),
            },
        };

        println!("DEBUG: Current Wallet {:?}", &self.wallet);
        Ok(Response::new(reply))
    }
    async fn open(&self, request: Request<OpenRequest>) -> Result<Response<StatusResponse>, Status> {
        println!("INFO request {:?} {:?}", request.remote_addr(), request.get_ref());
        let name = &request.get_ref().name;

        let reply = match Wallet::open(name) {
            Ok(wallet) => {
                let mut w = (*self.wallet).write().unwrap();
                *w = Some(wallet);
                StatusResponse {
                    code: 200,
                    message: "OK".to_owned(),
                }
            }
            Err(e) => StatusResponse {
                code: 500,
                message: format!("Can not open wallet: {:}", e),
            },
        };

        println!("DEBUG: Current Wallet {:?}", &self.wallet);
        Ok(Response::new(reply))
    }

    async fn lock(&self, request: Request<LockRequest>) -> Result<Response<StatusResponse>, Status> {
        println!("INFO request {:?} {:?}", request.remote_addr(), request.get_ref());
        // let name = &request.get_ref().name;
        let mut w = (*self.wallet).write().unwrap();
        let reply = match w.as_mut() {
            Some(wallet) => wallet
                .lock()
                .map(|_| StatusResponse {
                    code: 200,
                    message: "OK".to_owned(),
                })
                .map_err(|e| StatusResponse {
                    code: 500,
                    message: format!("Can not lock wallet: {:}", e),
                })
                .unwrap_or_else(|e| e),
            None => StatusResponse {
                code: 500,
                message: "No wallet opened".to_owned(),
            },
        };
        Ok(Response::new(reply))
    }

    async fn unlock(&self, request: Request<UnlockRequest>) -> Result<Response<StatusResponse>, Status> {
        println!("INFO request {:?} {:?}", request.remote_addr(), request.get_ref());
        // let name = &request.get_ref().name;
        let password = &request.get_ref().password;

        let mut w = (*self.wallet).write().unwrap();
        let reply = match w.as_mut() {
            Some(wallet) => wallet
                .unlock(password)
                .map(|_| StatusResponse {
                    code: 200,
                    message: "OK".to_owned(),
                })
                .map_err(|e| StatusResponse {
                    code: 500,
                    message: format!("Can not unlock wallet: {:}", e),
                })
                .unwrap_or_else(|e| e),
            None => StatusResponse {
                code: 500,
                message: "No wallet opened".to_owned(),
            },
        };
        Ok(Response::new(reply))
    }

    async fn create_key(&self, request: Request<CreateKeyRequest>) -> Result<Response<CreateKeyResponse>, Status> {
        println!("INFO request {:?} {:?}", request.remote_addr(), request.get_ref());
        let mut w = (*self.wallet).write().unwrap();
        let reply = match w.as_mut() {
            Some(wallet) => wallet
                .create_key()
                .map(|kp| CreateKeyResponse {
                    code: 200,
                    message: "OK".to_owned(),
                    key_pair: Some(KeyPair {
                        public: kp.public().as_ref().to_owned(),
                        private: kp.private().as_ref().to_owned(),
                    }),
                })
                .map_err(|e| CreateKeyResponse {
                    code: 500,
                    message: format!("Can not unlock wallet: {:}", e),
                    key_pair: None,
                })
                .unwrap_or_else(|e| e),
            None => CreateKeyResponse {
                code: 500,
                message: "No wallet opened".to_owned(),
                key_pair: None,
            },
        };

        Ok(Response::new(reply))
    }
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
