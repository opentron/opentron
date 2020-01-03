use clap::load_yaml;

mod commands;
mod error;
mod utils;

use error::Error;

// FIXME: should use AppConfig, for now, use static var
static mut RPC_HOST: &str = "grpc.trongrid.io:50051";

fn main() -> Result<(), Error> {
    utils::walletd::ensure_walletd()?;

    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml).get_matches();

    // FIXME: as above
    unsafe {
        RPC_HOST = match matches.value_of("network") {
            Some("mainnet") => "grpc.trongrid.io:50051",
            Some("shasta") => "grpc.shasta.trongrid.io:50051",
            Some("nile") => "47.252.19.181:50051",
            Some("testnet") => "47.252.87.110:50051",
            Some("dappchain") => "47.90.245.159:50051",
            Some("dappchain-testnet") => "47.252.85.90:50051",
            _ => unreachable!(),
        }
    }

    let wallet_name = matches.value_of("name").unwrap_or("default");

    match matches.subcommand() {
        ("get", Some(arg_matches)) => commands::get::main(arg_matches),
        ("list", Some(arg_matches)) => commands::list::main(arg_matches),
        ("set", Some(arg_matches)) => commands::set::main(arg_matches),
        ("transfer", Some(arg_matches)) => commands::transfer::main(arg_matches),
        ("sign", Some(arg_matches)) => commands::sign::main(arg_matches),
        ("wallet", Some(arg_matches)) => commands::wallet::main(wallet_name, arg_matches),
        ("shielded", Some(arg_matches)) => commands::shielded::main(arg_matches),
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
