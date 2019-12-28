use clap::load_yaml;

mod commands;
mod error;
mod utils;

use error::Error;

fn main() -> Result<(), String> {
    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml).get_matches();

    let ret = match matches.subcommand() {
        ("get", Some(import_matches)) => commands::get::main(import_matches),
        ("transfer", Some(arg_matches)) => commands::transfer::main(arg_matches),
        _ => {
            println!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    };
    ret.map_err(|e| e.to_string())
}
