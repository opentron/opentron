use clap::ArgMatches;
use hex::ToHex;
use keys::KeyPair;

use crate::error::Error;

fn create_key() -> Result<(), Error> {
    let kp = KeyPair::generate();
    let address = kp.address();

    println!("Address(Base58): {:}", address);
    println!("Address(hex):    {:}", address.encode_hex::<String>());
    println!("Public:          {:}", kp.public());
    println!("Private:         {:}", kp.private());
    Ok(())
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("key", _) => create_key(),
        _ => unreachable!("checked by cli.yml; qed"),
    }
}
