use clap::ArgMatches;
use hex::ToHex;
use keys::{Address, KeyPair, Private, Public};
use std::convert::TryFrom;
use tokio::runtime::Builder;
use tonic::Request;
use walletd::api::local_wallet_client::LocalWalletClient;
use walletd::api::sign_digest_request::PrivateKeyOf;
use walletd::api::{
    CreateKeyRequest, CreateKeyResponse, CreateRequest, CreateZkeyRequest, CreateZkeyResponse, ImportKeyRequest,
    ListKeysRequest, ListKeysResponse, ListZkeysRequest, ListZkeysResponse, LockRequest, OpenRequest,
    SignDigestRequest, SignDigestResponse, StatusResponse, UnlockRequest,
};

use crate::error::Error;

const WALLETD_RPC_URL: &str = "http://[::1]:8888";

async fn create_wallet(name: &str, password: &str) -> Result<(), Error> {
    let mut wallet_client = LocalWalletClient::connect(WALLETD_RPC_URL).await?;

    let request = Request::new(CreateRequest {
        name: name.into(),
        password: password.into(),
    });
    let response = wallet_client.create(request).await?;

    let status: StatusResponse = response.into_inner();
    println!("{:?}", &status);
    Ok(())
}

async fn open_wallet(name: &str) -> Result<(), Error> {
    let mut wallet_client = LocalWalletClient::connect(WALLETD_RPC_URL).await?;

    let request = Request::new(OpenRequest { name: name.into() });
    let response = wallet_client.open(request).await?;

    let status: StatusResponse = response.into_inner();
    println!("{:?}", &status);
    Ok(())
}

async fn lock_wallet() -> Result<(), Error> {
    let mut wallet_client = LocalWalletClient::connect(WALLETD_RPC_URL).await?;

    let request = Request::new(LockRequest { name: "".into() });
    let response = wallet_client.lock(request).await?;

    let status: StatusResponse = response.into_inner();
    println!("{:?}", &status);
    Ok(())
}

async fn unlock_wallet(password: &str) -> Result<(), Error> {
    let mut wallet_client = LocalWalletClient::connect(WALLETD_RPC_URL).await?;

    let request = Request::new(UnlockRequest {
        name: "".into(),
        password: password.into(),
    });
    let response = wallet_client.unlock(request).await?;

    let status: StatusResponse = response.into_inner();
    println!("{:?}", &status);
    Ok(())
}

async fn create_key_in_wallet() -> Result<(), Error> {
    let mut wallet_client = LocalWalletClient::connect(WALLETD_RPC_URL).await?;

    let request = Request::new(CreateKeyRequest { name: "".into() });
    let response = wallet_client.create_key(request).await?;

    let reply: CreateKeyResponse = response.into_inner();
    if reply.code == 200 {
        let kp_pb = reply.key_pair.as_ref().expect("won't fail; qed");
        let kp = KeyPair::from_private(Private::try_from(&kp_pb.private)?)?;

        println!("Address: {:}", kp.address());
        println!("Public:  {:}", kp.public());
        println!("Private: {:}", kp.private());
    } else {
        eprintln!("{:?}", &reply);
    }
    Ok(())
}

async fn import_key_to_wallet(private_key: &str) -> Result<(), Error> {
    let mut wallet_client = LocalWalletClient::connect(WALLETD_RPC_URL).await?;

    let private: Private = private_key.parse()?;
    println!("Importing private key for {:} ...", Address::from_private(&private));
    let request = Request::new(ImportKeyRequest {
        name: "".into(),
        private_key: private.as_bytes().to_owned(),
    });
    let response = wallet_client.import_key(request).await?;

    let status: StatusResponse = response.into_inner();
    println!("{:?}", &status);
    Ok(())
}

async fn list_keys_in_wallet() -> Result<(), Error> {
    let mut wallet_client = LocalWalletClient::connect(WALLETD_RPC_URL).await?;

    let request = Request::new(ListKeysRequest { name: "".into() });
    let response = wallet_client.list_keys(request).await?;
    let reply: ListKeysResponse = response.into_inner();
    if reply.code == 200 {
        for raw_key in reply.public_keys {
            let pub_key = Public::try_from(raw_key)?;
            let addr = Address::from_public(&pub_key);
            println!("Address(Base58): {:}", addr);
            println!("Address(hex):    {:}", addr.encode_hex::<String>());
            println!("         Public: {:}\n", pub_key);
        }
    } else {
        eprintln!("{:?}", &reply);
    }
    Ok(())
}

async fn create_zkey_in_wallet() -> Result<(), Error> {
    let mut wallet_client = LocalWalletClient::connect(WALLETD_RPC_URL).await?;

    let request = Request::new(CreateZkeyRequest {});
    let response = wallet_client.create_zkey(request).await?;

    let reply: CreateZkeyResponse = response.into_inner();
    if reply.code == 200 {
        println!("Address: {:}", reply.address);
        println!("SK:      {:}", reply.sk.encode_hex::<String>());
    } else {
        eprintln!("{:?}", &reply);
    }
    Ok(())
}

async fn list_zkeys_in_wallet() -> Result<(), Error> {
    let mut wallet_client = LocalWalletClient::connect(WALLETD_RPC_URL).await?;

    let request = Request::new(ListZkeysRequest {});
    let response = wallet_client.list_zkeys(request).await?;
    let reply: ListZkeysResponse = response.into_inner();
    if reply.code == 200 {
        for addr in reply.addresses {
            println!("Address: {:}", addr);
        }
    } else {
        eprintln!("{:?}", &reply);
    }
    Ok(())
}

async fn sign_digest_via_address(digest: &[u8], address: &Address) -> Result<Vec<u8>, Error> {
    let mut wallet_client = LocalWalletClient::connect(WALLETD_RPC_URL).await?;

    let request = Request::new(SignDigestRequest {
        name: "default".to_owned(), // TODO: refine wallet name handling
        digest: digest.to_owned(),
        private_key_of: Some(PrivateKeyOf::RawAddress(address.as_bytes().to_owned())),
    });

    let response = wallet_client.sign_digest(request).await?;
    let reply: SignDigestResponse = response.into_inner();
    if reply.code == 200 {
        Ok(reply.signature)
    } else {
        eprintln!("{:?}", &reply);
        Err(Error::Runtime("fail to sign digest"))
    }
}

// NOTE: each impl Trait is a different type, so, await is required
async fn run<'a>(matches: &'a ArgMatches<'a>) -> Result<(), Error> {
    match matches.subcommand() {
        ("create", Some(arg_matches)) => {
            let wallet_name = arg_matches.value_of("name").expect("havs default in cli.yml; qed");
            match arg_matches.value_of("password") {
                Some(password) => create_wallet(wallet_name, password).await,
                _ => {
                    let password = rpassword::prompt_password_stderr("Wallet Password:")
                        .map_err(|_| Error::Runtime("can not get password"))?;
                    let password2 = rpassword::prompt_password_stderr("Retype Password:")
                        .map_err(|_| Error::Runtime("can not get password"))?;
                    if password == password2 {
                        create_wallet(wallet_name, &password).await
                    } else {
                        Err(Error::Runtime("password mismatch"))
                    }
                }
            }
        }
        ("open", Some(arg_matches)) => {
            let wallet_name = arg_matches.value_of("name").expect("havs default in cli.yml; qed");
            open_wallet(wallet_name).await
        }
        ("lock", _) => lock_wallet().await,
        ("unlock", Some(arg_matches)) => match arg_matches.value_of("password") {
            Some(password) => unlock_wallet(password).await,
            _ => {
                let password = rpassword::prompt_password_stderr("Wallet Password:")
                    .map_err(|_| Error::Runtime("can not get password"))?;
                unlock_wallet(&password).await
            }
        },
        ("create_key", _) => create_key_in_wallet().await,
        ("import_key", Some(arg_matches)) => {
            let priv_key = arg_matches.value_of("private-key").expect("required in cli.yml; qed");
            import_key_to_wallet(priv_key).await
        }
        ("keys", _) => list_keys_in_wallet().await,
        ("create_zkey", _) => create_zkey_in_wallet().await,
        ("zkeys", _) => list_zkeys_in_wallet().await,
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("command line arguments parsing error"))
        }
    }
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    let fut = run(matches);
    let mut rt = Builder::new().basic_scheduler().enable_all().build().unwrap();
    rt.block_on(fut)
}

pub fn sign_digest(digest: &[u8], address: &Address) -> Result<Vec<u8>, Error> {
    let fut = sign_digest_via_address(digest, address);
    let mut rt = Builder::new().basic_scheduler().enable_all().build().unwrap();
    rt.block_on(fut)
}
