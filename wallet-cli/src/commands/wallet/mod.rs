use clap::ArgMatches;
use tokio::runtime::Builder;
use tonic::Request;
use walletd::api::local_wallet_client::LocalWalletClient;
use walletd::api::{OpenRequest, StatusResponse};

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

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    let fut = match matches.subcommand() {
        ("create", _) => unimplemented!(),
        ("open", Some(arg_matches)) => {
            let name = arg_matches.value_of("name").unwrap_or("default");
            open_wallet(name)
        }
        _ => unimplemented!(),
    };

    let mut rt = Builder::new().basic_scheduler().enable_all().build().unwrap();
    rt.block_on(fut)
}
