use clap::load_yaml;

mod commands;

fn main() -> Result<(), String> {
    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml).get_matches();

    // println!("matches: {:?}", matches);

    match matches.subcommand() {
        ("hello", Some(arg_matches)) => commands::hello::run(arg_matches),
        ("vanity", Some(arg_matches)) => commands::hello::run_vanity(arg_matches),
        _ => unreachable!("subcommand required"),
    }
}
