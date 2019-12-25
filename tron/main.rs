#[macro_use]
extern crate clap;

extern crate base58;
extern crate hex;
extern crate proto;
extern crate protobuf;
extern crate sha2;
extern crate serde_json;

mod commands;

fn main() -> Result<(), String> {
    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml).get_matches();

    // println!("matches: {:?}", matches);

    match matches.subcommand() {
        ("hello", Some(import_matches)) => commands::hello::run(import_matches),
        _ => unreachable!("subcommand required"),
    }
}
