use clap::ArgMatches;
use futures::executor;
use hex::ToHex;
use keys::KeyPair;
use proto::api::EmptyMessage;
use serde_json::json;
use ztron_primitives::prelude::generate_zkey_pair;

use crate::error::Error;
use crate::utils::client;
use crate::utils::jsont;

fn create_key() -> Result<(), Error> {
    let kp = KeyPair::generate();
    let address = kp.address();

    println!("Address(Base58): {:}", address);
    println!("Address(hex):    {:}", address.encode_hex::<String>());
    println!("Public:          {:}", kp.public());
    println!("Private:         {:}", kp.private());
    Ok(())
}

pub fn create_zkey(matches: &ArgMatches) -> Result<(), Error> {
    if matches.is_present("offline") {
        return create_zkey_offline();
    }

    let payload = executor::block_on(
        client::GRPC_CLIENT
            .get_new_shielded_address(Default::default(), EmptyMessage::new())
            .drop_metadata(),
    )?;
    let mut addr_info = serde_json::to_value(&payload)?;

    // sk: spending key => ask, nsk, ovk
    // ask: spend authorizing key, 256 => ak
    // nsk: proof authorizing key, 256 => nk
    // ovk: outgoing viewing key, 256
    // ivk: incoming viewing key, 256 => pkD
    // d: diversifier, 11bytes
    // pkD: the public key of the address, g_d^ivk
    // pkD + d => z-addr
    for key in &["sk", "ask", "nsk", "ovk", "ak", "nk", "ivk", "d", "pkD"] {
        addr_info[key] = json!(jsont::bytes_to_hex_string(&addr_info[key]));
    }
    println!("{}", serde_json::to_string_pretty(&addr_info)?);
    Ok(())
}

pub fn create_zkey_offline() -> Result<(), Error> {
    let (addr, sk, esk, fvk) = generate_zkey_pair();

    let ask = esk.ask.to_bytes();
    let nsk = esk.nsk.to_bytes();
    let ovk = esk.ovk.as_bytes();

    let ak = fvk.vk.ak.to_bytes();
    let nk = fvk.vk.nk.to_bytes();
    let ivk = fvk.vk.ivk().to_bytes();

    let pk_d = addr.pk_d().to_bytes();
    let d = addr.diversifier().as_bytes();

    println!("d = {}", d.encode_hex::<String>());
    println!("sk  => {}", sk.encode_hex::<String>());
    println!("pk_d = {}", pk_d.encode_hex::<String>());
    println!("address = {}", addr);

    println!("ask => {}", ask.encode_hex::<String>());
    println!("nsk => {}", nsk.encode_hex::<String>());
    println!("ovk => {}", ovk.encode_hex::<String>());

    println!("ak  = {}", ak.encode_hex::<String>());
    println!("nk  = {}", nk.encode_hex::<String>());
    println!("ivk = {}", ivk.encode_hex::<String>());

    Ok(())
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("key", _) => create_key(),
        ("zkey", Some(arg_matches)) => create_zkey(arg_matches),
        _ => unreachable!("checked by cli.yml; qed"),
    }
}
