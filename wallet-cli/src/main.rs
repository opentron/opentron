use clap::load_yaml;

mod commands;
mod error;
mod utils;

use error::Error;

// FIXME: should use AppConfig, for now, use static var
static mut RPC_HOST: &str = "grpc.trongrid.io:50051";
/// Used for sun-network
static mut CHAIN_ID: Option<&str> = None;

fn main() -> Result<(), Error> {
    utils::walletd::ensure_walletd()?;

    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml).get_matches();

    unsafe {
        RPC_HOST = match (matches.value_of("network"), matches.value_of("rpc-host")) {
            (_, Some(host)) => Box::leak(host.to_owned().into_boxed_str()), // FIXME: leaks
            (Some("mainnet"), _) => "grpc.trongrid.io:50051",
            (Some("shasta"), _) => "grpc.shasta.trongrid.io:50051",
            (Some("nile"), _) => "47.252.3.238:50051",
            (Some("testnet"), _) => "47.252.87.28:50051",
            (Some("dappchain"), _) => "47.90.245.159:50051",
            (Some("dappchain-testnet"), _) => "47.252.85.90:50051",
            _ => unreachable!(),
        };
        CHAIN_ID = match matches.value_of("network") {
            Some("dappchain") => Some("41E209E4DE650F0150788E8EC5CAFA240A23EB8EB7"),
            Some("dappchain-testnet") => Some("413AF23F37DA0D48234FDD43D89931E98E1144481B"),
            _ => None,
        };
    }

    match matches.subcommand() {
        ("get", Some(arg_matches)) => commands::get::main(arg_matches),
        ("list", Some(arg_matches)) => commands::list::main(arg_matches),
        ("set", Some(arg_matches)) => commands::set::main(arg_matches),
        ("system", Some(arg_matches)) => commands::system::main(arg_matches),
        ("asset", Some(arg_matches)) => commands::asset::main(arg_matches),
        ("contract", Some(arg_matches)) => commands::contract::main(arg_matches),
        ("transfer", Some(arg_matches)) => commands::transfer::main(arg_matches),
        ("sign", Some(arg_matches)) => commands::sign::main(arg_matches),
        ("wallet", Some(arg_matches)) => commands::wallet::main(arg_matches),
        ("create", Some(arg_matches)) => commands::create::main(arg_matches),
        ("shielded", Some(arg_matches)) => commands::shielded::main(arg_matches),
        _ => unreachable!("handled by cli.yml; qed"),
    }
}
