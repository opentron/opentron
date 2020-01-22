use daemonize::Daemonize;
use std::convert::TryFrom;
use std::fs::File;
use std::sync::{Arc, RwLock};
use tokio::runtime::Builder;
use tonic::{transport::Server, Request, Response, Status};
use wallet::Wallet;

use api::local_wallet_server::{LocalWallet, LocalWalletServer};
use api::{sign_digest_request::PrivateKeyOf, KeyPair};
use api::{
    CreateKeyRequest, CreateKeyResponse, CreateRequest, CreateZkeyRequest, CreateZkeyResponse, ImportKeyRequest,
    ImportNoteRequest, ImportZkeyRequest, ListKeysRequest, ListKeysResponse, ListNotesRequest, ListNotesResponse,
    ListPrivateKeysRequest, ListPrivateKeysResponse, ListZkeysRequest, ListZkeysResponse, LockRequest, OpenRequest,
    SignDigestRequest, SignDigestResponse, StatusResponse, UnlockRequest,
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

        Ok(Response::new(reply))
    }

    async fn lock(&self, request: Request<LockRequest>) -> Result<Response<StatusResponse>, Status> {
        println!("INFO request {:?} {:?}", request.remote_addr(), request.get_ref());
        // let name = &request.get_ref().name;
        let reply = match *(*self.wallet).write().unwrap() {
            Some(ref mut wallet) => wallet
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

        let reply = match *(*self.wallet).write().unwrap() {
            Some(ref mut wallet) => wallet
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

        let reply = match *(*self.wallet).write().unwrap() {
            Some(ref mut wallet) => wallet
                .create_key()
                .map(|kp| CreateKeyResponse {
                    code: 200,
                    message: "OK".to_owned(),
                    key_pair: Some(KeyPair {
                        public: kp.public().as_bytes().to_owned(),
                        private: kp.private().as_bytes().to_owned(),
                    }),
                })
                .map_err(|e| CreateKeyResponse {
                    code: 500,
                    message: format!("Can create key for wallet: {:}", e),
                    key_pair: None,
                })
                .unwrap_or_else(|e| e),
            None => CreateKeyResponse {
                code: 500,
                message: "No wallet opened".to_owned(),
                ..Default::default()
            },
        };

        Ok(Response::new(reply))
    }

    async fn import_key(&self, request: Request<ImportKeyRequest>) -> Result<Response<StatusResponse>, Status> {
        println!("INFO request {:?} {:?}", request.remote_addr(), request.get_ref());
        let raw_key = &request.get_ref().private_key;

        let reply = match *(*self.wallet).write().unwrap() {
            Some(ref mut wallet) => keys::Private::try_from(raw_key)
                .map_err(From::from)
                .and_then(|priv_key| wallet.import_key(priv_key))
                .map(|_| StatusResponse {
                    code: 200,
                    message: "OK".to_owned(),
                })
                .map_err(|e| StatusResponse {
                    code: 500,
                    message: format!("Can not import key to wallet: {:}", e),
                })
                .unwrap_or_else(|e| e),
            None => StatusResponse {
                code: 500,
                message: "No wallet opened".to_owned(),
            },
        };

        Ok(Response::new(reply))
    }

    async fn list_keys(&self, request: Request<ListKeysRequest>) -> Result<Response<ListKeysResponse>, Status> {
        println!("INFO request {:?} {:?}", request.remote_addr(), request.get_ref());
        let reply = match *(*self.wallet).read().unwrap() {
            Some(ref wallet) => {
                let pub_keys = wallet.keys().map(|k| k.as_bytes().to_owned()).collect::<Vec<_>>();
                ListKeysResponse {
                    code: 200,
                    message: "OK".to_owned(),
                    public_keys: pub_keys,
                }
            }
            None => ListKeysResponse {
                code: 500,
                message: "No wallet opened".to_owned(),
                ..Default::default()
            },
        };
        Ok(Response::new(reply))
    }

    async fn sign_digest(&self, request: Request<SignDigestRequest>) -> Result<Response<SignDigestResponse>, Status> {
        println!("INFO request {:?} {:?}", request.remote_addr(), request.get_ref());

        let digest = &request.get_ref().digest;
        let key_of = &request.get_ref().private_key_of.as_ref().expect("won't fail; qed");

        let reply = match *(*self.wallet).read().unwrap() {
            Some(ref wallet) => {
                let (public, address) = match key_of {
                    PrivateKeyOf::PublicKey(raw) => (Some(raw), None),
                    PrivateKeyOf::RawAddress(raw) => (None, Some(raw)),
                };

                match sign_digest_via_public_or_address(wallet, digest, public, address) {
                    Ok(signature) => SignDigestResponse {
                        code: 200,
                        message: "OK".to_owned(),
                        signature,
                    },
                    Err(e) => SignDigestResponse {
                        code: 500,
                        message: format!("{:?}", e),
                        ..Default::default()
                    },
                }
            }
            None => SignDigestResponse {
                code: 500,
                message: "No wallet opened".to_owned(),
                ..Default::default()
            },
        };
        Ok(Response::new(reply))
    }

    async fn list_private_keys(
        &self,
        _request: Request<ListPrivateKeysRequest>,
    ) -> Result<Response<ListPrivateKeysResponse>, Status> {
        unimplemented!()
    }

    async fn create_zkey(&self, request: Request<CreateZkeyRequest>) -> Result<Response<CreateZkeyResponse>, Status> {
        println!("INFO request {:?} {:?}", request.remote_addr(), request.get_ref());
        let reply = match *(*self.wallet).write().unwrap() {
            Some(ref mut wallet) => wallet
                .create_zkey()
                .map(|(addr, sk)| CreateZkeyResponse {
                    code: 200,
                    message: "OK".to_owned(),
                    address: addr.to_string(),
                    sk: sk[..].to_owned(),
                })
                .unwrap_or_else(|e| CreateZkeyResponse {
                    code: 500,
                    message: format!("Can not create zkey: {:}", e),
                    ..Default::default()
                }),
            None => CreateZkeyResponse {
                code: 500,
                message: "No wallet opened".to_owned(),
                ..Default::default()
            },
        };

        Ok(Response::new(reply))
    }

    async fn list_zkeys(&self, request: Request<ListZkeysRequest>) -> Result<Response<ListZkeysResponse>, Status> {
        println!("INFO request {:?} {:?}", request.remote_addr(), request.get_ref());

        let reply = match *(*self.wallet).read().unwrap() {
            Some(ref wallet) => {
                let addrs = wallet
                    .list_zkeys()
                    .unwrap_or_default()
                    .iter()
                    .map(|addr| addr.to_string())
                    .collect();
                ListZkeysResponse {
                    code: 200,
                    message: "OK".to_owned(),
                    addresses: addrs,
                }
            }
            None => ListZkeysResponse {
                code: 500,
                message: "No wallet opened".to_owned(),
                ..Default::default()
            },
        };
        Ok(Response::new(reply))
    }

    async fn list_notes(&self, _request: Request<ListNotesRequest>) -> Result<Response<ListNotesResponse>, Status> {
        unimplemented!()
    }

    async fn import_zkey(&self, _request: Request<ImportZkeyRequest>) -> Result<Response<StatusResponse>, Status> {
        unimplemented!()
    }

    async fn import_note(&self, _request: Request<ImportNoteRequest>) -> Result<Response<StatusResponse>, Status> {
        unimplemented!()
    }
}

fn sign_digest_via_public_or_address(
    wallet: &Wallet,
    digest: &[u8],
    public: Option<&Vec<u8>>,
    address: Option<&Vec<u8>>,
) -> Result<Vec<u8>, wallet::Error> {
    match (public, address) {
        (Some(raw), _) => {
            let pub_key = keys::Public::try_from(raw)?;
            wallet.sign_digest(digest, &pub_key).map(|s| s.as_bytes().to_owned())
        }
        (_, Some(raw)) => {
            let addr = keys::Address::try_from(raw)?;
            let pub_key = wallet.get_public_key(&addr)?;
            wallet.sign_digest(digest, &pub_key).map(|s| s.as_bytes().to_owned())
        }
        (_, _) => unreachable!(),
    }
}

// NOTE: #[tokio:main] conflicts with daemonize
async fn tokio_main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:8888".parse().unwrap();
    let service = LocalWalletService::default();

    println!("LocalWalletService listening on {}", addr);
    Server::builder()
        .add_service(LocalWalletServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = File::create("/tmp/walletd.out").unwrap();
    let stderr = File::create("/tmp/walletd.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/walletd.pid")
        .stdout(stdout)
        .stderr(stderr);
    match daemonize.start() {
        Ok(_) => println!("Success, daemonized"),
        Err(e) => eprintln!("Error, {}", e),
    }

    let fut = tokio_main();
    let mut rt = Builder::new().basic_scheduler().enable_all().build()?;
    rt.block_on(fut)
}
