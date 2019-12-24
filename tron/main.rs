#[macro_use]
extern crate clap;

extern crate hex;
extern crate proto;
extern crate protobuf;

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
