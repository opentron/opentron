#[macro_use]
extern crate clap;
extern crate futures;
extern crate grpc;
extern crate hex;
extern crate proto;
extern crate serde_json;

mod commands;

fn main() -> Result<(), String> {
    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml).get_matches();

    // println!("DEBUG matches: {:?}", matches);
    // println!("");

    match matches.subcommand() {
        ("get", Some(import_matches)) => commands::get::main(import_matches),
        _ => Err("error parsing command line".to_owned()),
    }
}
