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
            Some("nile") => "47.252.3.238:50051",
            Some("testnet") => "47.252.87.110:50051",
            Some("dappchain") => "47.90.245.159:50051",
            Some("dappchain-testnet") => "47.252.85.90:50051",
            _ => unreachable!(),
        }
    }

    match matches.subcommand() {
        ("get", Some(arg_matches)) => commands::get::main(arg_matches),
        ("list", Some(arg_matches)) => commands::list::main(arg_matches),
        ("set", Some(arg_matches)) => commands::set::main(arg_matches),
        ("system", Some(arg_matches)) => commands::system::main(arg_matches),
        ("transfer", Some(arg_matches)) => commands::transfer::main(arg_matches),
        ("sign", Some(arg_matches)) => commands::sign::main(arg_matches),
        ("freeze", Some(arg_matches)) => commands::freeze::freeze_main(arg_matches),
        ("unfreeze", Some(arg_matches)) => commands::freeze::unfreeze_main(arg_matches),
        ("call", Some(arg_matches)) => commands::call::main(arg_matches),
        ("wallet", Some(arg_matches)) => commands::wallet::main(arg_matches),
        ("shielded", Some(arg_matches)) => commands::shielded::main(arg_matches),
        _ => unreachable!("handled by cli.yml; qed"),
    }
}
