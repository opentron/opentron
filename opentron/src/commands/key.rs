use clap::ArgMatches;
use keys::{Address, KeyPair};

type Error = Box<dyn std::error::Error>;

pub fn main(matches: &ArgMatches<'_>) -> Result<(), Error> {
    match matches.subcommand() {
        ("generate", _) => generate_key(),
        ("inspect", Some(arg_matches)) => inspect_key(arg_matches),
        // ("generate-genesis-key", _) => unimplemented!(),
        _ => {
            eprintln!("{}", matches.usage());
            Ok(())
        }
    }
}

fn generate_key() -> Result<(), Error> {
    let kp = KeyPair::generate();
    let address = kp.address();

    println!("Public:  {:}", kp.public());
    println!("Private: {:}", kp.private());
    pprint_address(&address);
    Ok(())
}

fn inspect_key(matches: &ArgMatches<'_>) -> Result<(), Error> {
    let address = match matches.value_of("ADDRESS") {
        Some(raw_addr) => raw_addr.parse()?,
        _ if matches.is_present("private") => {
            let priv_key: keys::Private = matches.value_of("private").unwrap().parse()?;
            let kp = KeyPair::from_private(priv_key)?;
            println!("Public:  {:}", kp.public());
            println!("Private: {:}", kp.private());
            kp.address()
        }
        _ if matches.is_present("public") => {
            let pub_key: keys::Public = matches.value_of("public").unwrap().parse()?;
            println!("Public: {:}", pub_key);
            Address::from_public(&pub_key)
        }
        _ => {
            eprintln!("{}", matches.usage());
            return Ok(());
        }
    };

    pprint_address(&address);
    Ok(())
}

fn pprint_address(address: &Address) {
    println!("Address:");
    println!(" - Base58: {:}", address);
    println!(" - HEX:    {:}", address.to_hex_address());
    println!(" - ETH:    {:}", address.to_eth_address());
}
