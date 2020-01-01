use clap::ArgMatches;
use keys::{Address, KeyPair, Private, Public};
use std::convert::TryFrom;
use tokio::runtime::Builder;
use tonic::Request;
use walletd::api::local_wallet_client::LocalWalletClient;
use walletd::api::sign_digest_request::PrivateKeyOf;
use walletd::api::{
    CreateKeyRequest, CreateKeyResponse, CreateRequest, ListKeysRequest, ListKeysResponse, LockRequest, OpenRequest,
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

async fn lock_wallet(name: &str) -> Result<(), Error> {
    let mut wallet_client = LocalWalletClient::connect(WALLETD_RPC_URL).await?;

    let request = Request::new(LockRequest { name: name.into() });
    let response = wallet_client.lock(request).await?;

    let status: StatusResponse = response.into_inner();
    println!("{:?}", &status);
    Ok(())
}

async fn unlock_wallet(name: &str, password: &str) -> Result<(), Error> {
    let mut wallet_client = LocalWalletClient::connect(WALLETD_RPC_URL).await?;

    let request = Request::new(UnlockRequest {
        name: name.into(),
        password: password.into(),
    });
    let response = wallet_client.unlock(request).await?;

    let status: StatusResponse = response.into_inner();
    println!("{:?}", &status);
    Ok(())
}

async fn create_key_in_wallet(name: &str) -> Result<(), Error> {
    let mut wallet_client = LocalWalletClient::connect(WALLETD_RPC_URL).await?;

    let request = Request::new(CreateKeyRequest { name: name.into() });
    let response = wallet_client.create_key(request).await?;

    let reply: CreateKeyResponse = response.into_inner();
    if reply.code == 200 {
        let kp_pb = reply.key_pair.as_ref().expect("won't fail; qed");
        let kp = KeyPair::from_private(Private::try_from(&kp_pb.private)?)?;

        println!("Address: {:}", kp.address());
        println!("Public:  {:}", kp.public());
        println!("Private: {:}", kp.private());
    } else {
        println!("{:?}", &reply);
    }
    Ok(())
}

async fn list_keys_in_wallet(name: &str) -> Result<(), Error> {
    let mut wallet_client = LocalWalletClient::connect(WALLETD_RPC_URL).await?;

    let request = Request::new(ListKeysRequest { name: name.into() });
    let response = wallet_client.list_keys(request).await?;
    let reply: ListKeysResponse = response.into_inner();
    if reply.code == 200 {
        for raw_key in reply.public_keys {
            let pub_key = Public::try_from(raw_key)?;
            println!("Address:  {:}", Address::from_public(&pub_key));
            println!(" Public: {:}", pub_key);
        }
    } else {
        println!("{:?}", &reply);
    }
    Ok(())
}

async fn sign_digest_via_address(digest: &[u8], address: &Address) -> Result<Vec<u8>, Error> {
    let mut wallet_client = LocalWalletClient::connect(WALLETD_RPC_URL).await?;

    let request = Request::new(SignDigestRequest {
        name: "default".to_owned(), // TODO: refine wallet name handling
        digest: digest.to_owned(),
        private_key_of: Some(PrivateKeyOf::RawAddress(address.as_ref().to_owned())),
    });

    let response = wallet_client.sign_digest(request).await?;
    let reply: SignDigestResponse = response.into_inner();
    if reply.code == 200 {
        Ok(reply.signature)
    } else {
        println!("{:?}", &reply);
        Err(Error::Runtime("fail to sign digest"))
    }
}

// NOTE: each impl Trait is a different type, so, await is required
async fn run<'a>(wallet_name: &str, matches: &'a ArgMatches<'a>) -> Result<(), Error> {
    match matches.subcommand() {
        ("create", Some(arg_matches)) => {
            let password = arg_matches.value_of("password").expect("required in cli.yml; qed");
            create_wallet(wallet_name, password).await
        }
        ("open", _) => open_wallet(wallet_name).await,
        ("lock", _) => lock_wallet(wallet_name).await,
        ("unlock", Some(arg_matches)) => {
            let password = arg_matches.value_of("password").expect("required in cli.yml; qed");
            unlock_wallet(wallet_name, password).await
        }
        ("create_key", _) => create_key_in_wallet(wallet_name).await,
        ("keys", _) => list_keys_in_wallet(wallet_name).await,
        _ => unimplemented!(),
    }
}

pub fn main(wallet_name: &str, matches: &ArgMatches) -> Result<(), Error> {
    let fut = run(wallet_name, matches);
    let mut rt = Builder::new().basic_scheduler().enable_all().build().unwrap();
    rt.block_on(fut)
}

pub fn sign_digest(digest: &[u8], address: &Address) -> Result<Vec<u8>, Error> {
    let fut = sign_digest_via_address(digest, address);
    let mut rt = Builder::new().basic_scheduler().enable_all().build().unwrap();
    rt.block_on(fut)
}
