use clap::ArgMatches;
use proto::api::EmptyMessage;
use proto::api::PrivateParameters;
use proto::api_grpc::Wallet;
// use proto::core::{OutputPoint, OutputPointInfo, };
use hex::{FromHex, ToHex};
use proto::api::{Note, ReceiveNote};
use protobuf::Message;
use serde_json::json;

use crate::error::Error;
use crate::utils::client::new_grpc_client;
use crate::utils::jsont;

pub fn new_shielded_address() -> Result<(), Error> {
    let (_, payload, _) = new_grpc_client()?
        .get_new_shielded_address(Default::default(), EmptyMessage::new())
        .wait()?;
    let mut addr_info = serde_json::to_value(&payload)?;

    for key in &["sk", "ask", "nsk", "ovk", "ak", "nk", "ivk", "d", "pkD"] {
        addr_info[key] = json!(jsont::bytes_to_hex_string(&addr_info[key]));
    }
    println!("{}", serde_json::to_string_pretty(&addr_info)?);
    Ok(())
}

pub fn debug() -> Result<(), Error> {
    /* input shielded
    let mut req = OutputPointInfo::new();
    let (_, payload, _) = new_grpc_client()?.get_merkle_tree_voucher_info(Default::default(), req).wait()?;
    */
    let grpc_client = new_grpc_client()?;

    let mut params = PrivateParameters::new();
    // TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu
    // eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
    params.set_transparent_from_address(vec![
        65, 70, 162, 62, 37, 223, 154, 15, 108, 24, 114, 157, 218, 154, 209, 175, 59, 106, 19, 17, 96,
    ]);
    params.set_from_amount(10_000_000);

    let memo = "are you joking";
    let mut note = Note::new();

    note.set_payment_address(
        "ztron1ze4ytt0pz9t6lafnhptnxted323z2rhtwjvhdq7h3vk3pv9e0ask3j30sn3j93ehx35u7ku7q0d".to_owned(),
    );
    note.set_value(6_000_000);
    note.set_memo(memo.as_bytes().to_owned());

    let (_, rcm_msg, _) = grpc_client.get_rcm(Default::default(), EmptyMessage::new()).wait()?;
    eprintln!("rcm = {:?}", rcm_msg.value.encode_hex::<String>());

    note.set_rcm(rcm_msg.value);

    let recv_note = ReceiveNote {
        note: Some(note).into(),
        ..Default::default()
    };

    params.set_shielded_receives(vec![recv_note].into());

    // when input is transparent.
    params.set_ovk(Vec::from_hex(
        "030c8c2bc59fb3eb8afb047a8ea4b028743d23e7d38c6fa30908358431e2314d",
    )?);

    let (_, transaction_ext, _) = grpc_client
        .create_shielded_transaction(Default::default(), params)
        .wait()?;

    let mut json = serde_json::to_value(&transaction_ext)?;
    jsont::fix_transaction_ext(&mut json);

    if json["result"]["result"].as_bool().unwrap() {
        json["transaction"]["raw_data_hex"] = json!(transaction_ext
            .get_transaction()
            .get_raw_data()
            .write_to_bytes()?
            .encode_hex::<String>());

        println!("{}", serde_json::to_string_pretty(&json["transaction"])?);
        Ok(())
    } else {
        eprintln!("{}", serde_json::to_string_pretty(&json)?);
        Err(Error::Runtime("can not create transaction"))
    }
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("new_address", _) => new_shielded_address(),
        ("debug", _) => debug(),
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
