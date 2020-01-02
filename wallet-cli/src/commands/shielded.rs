use clap::ArgMatches;
use proto::api::EmptyMessage;
use proto::api_grpc::Wallet;
use serde_json::json;

use crate::error::Error;
use crate::utils::client::new_grpc_client;
use crate::utils::jsont;

pub fn new_shielded_address() -> Result<(), Error> {
    let (_, payload, _) = new_grpc_client()?
        .get_new_shielded_address(Default::default(), EmptyMessage::new())
        .wait()?;
    let mut addr_info = serde_json::to_value(&payload)?;

    for key in &["sk", "ask", "nsk", "ovk", "ak", "nk", "ivk", "d", "pkD"] {
        addr_info[key] = json!(jsont::bytes_to_hex_string(&addr_info[key]));
    }
    println!("{}", serde_json::to_string_pretty(&addr_info)?);
    Ok(())
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("new_address", _) => new_shielded_address(),
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
