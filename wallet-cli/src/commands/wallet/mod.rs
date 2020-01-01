use clap::ArgMatches;
use tokio::runtime::Builder;
use tonic::Request;
use walletd::api::local_wallet_client::LocalWalletClient;
use walletd::api::{LockRequest, OpenRequest, StatusResponse, UnlockRequest};

use crate::error::Error;

const WALLETD_RPC_URL: &str = "http://[::1]:8888";

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

// NOTE: each impl Trait is a different type, so, await is required
async fn run<'a>(wallet_name: &str, matches: &'a ArgMatches<'a>) -> Result<(), Error> {
    match matches.subcommand() {
        ("create", _) => unimplemented!(),
        ("open", _) => open_wallet(wallet_name).await,
        ("lock", _) => lock_wallet(wallet_name).await,
        ("unlock", Some(arg_matches)) => {
            let password = arg_matches.value_of("password").expect("required in cli.yml; qed");
            unlock_wallet(wallet_name, password).await
        }
        _ => unimplemented!(),
    }
}

pub fn main(wallet_name: &str, matches: &ArgMatches) -> Result<(), Error> {
    let fut = run(wallet_name, matches);
    let mut rt = Builder::new().basic_scheduler().enable_all().build().unwrap();
    rt.block_on(fut)
}
