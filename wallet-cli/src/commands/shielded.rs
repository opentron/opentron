use clap::ArgMatches;
use hex::{FromHex, ToHex};
use keys::Address;
use proto::api::EmptyMessage;
use proto::api::PrivateParameters;
use proto::api::{Note, ReceiveNote, SpendNote};
use proto::api_grpc::{Wallet, WalletClient};
use proto::core::{OutputPoint, OutputPointInfo};
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

    // sk: spending key
    // ivk: incoming viewing key
    // d: diversifier
    for key in &["sk", "ask", "nsk", "ovk", "ak", "nk", "ivk", "d", "pkD"] {
        addr_info[key] = json!(jsont::bytes_to_hex_string(&addr_info[key]));
    }
    println!("{}", serde_json::to_string_pretty(&addr_info)?);
    Ok(())
}

pub fn debug_zaddr_to_zaddr() -> Result<(), Error> {
    Ok(())
}

pub fn debug_zaddr_to_taddr() -> Result<(), Error> {
    let grpc_client = new_grpc_client()?;

    // # Step 1: GetMerkleTreeVoucherInfo
    let mut out_point = OutputPoint::new();
    // TX hash of the transaction
    out_point.set_hash(Vec::from_hex(
        "e4c77bf9caf8e94cb2fa6e37bd58db92dba2cbb3ab2e8f13fa4b8803f40fbf4a"
    )?);
    out_point.set_index(0); // transaction index, normally 0

    let mut req_info = OutputPointInfo::new();
    req_info.set_out_points(vec![out_point].into());
    req_info.set_block_num(1); // seemed useless, 0 or 1

    let (_, mut voucher_info, _) = grpc_client
        .get_merkle_tree_voucher_info(Default::default(), req_info)
        .wait()?;
    let mut info = serde_json::to_value(&voucher_info)?;

    jsont::fix_voucher_info(&mut info);
    // num of vouchers = num of out_points

    // # Step 2: CreateShieldedTransaction
    let mut params = PrivateParameters::new();
    // shielded input
    let mut note = Note::new();
    note.set_value(190_000_000);
    note.set_payment_address(
        "ztron1ze4ytt0pz9t6lafnhptnxted323z2rhtwjvhdq7h3vk3pv9e0ask3j30sn3j93ehx35u7ku7q0d".to_owned()
    );
    note.set_rcm(Vec::from_hex(
        "16f3cdb3baf8f24026b3a447a165a404020bfe19cd32eef7d891de657bc90601"
    )?); // 0c

    let mut spend_node = SpendNote::new();
    spend_node.set_note(note);

    spend_node.set_alpha(get_rcm(&grpc_client)?);

    spend_node.set_voucher(voucher_info.take_vouchers().into_iter().next().unwrap());
    spend_node.set_path(voucher_info.take_paths().into_iter().next().unwrap());

    params.set_shielded_spends(vec![spend_node].into());

    // from address info
    params.set_ask(Vec::from_hex(
        "8c893dfa38956290f2a1df9e6019b4a6c5f670613583948d8d975dcbccf03407"
    )?);
    params.set_nsk(Vec::from_hex(
        "560832b298c76f021126b35bfdd3d4bb62ec0d632029674b3e9157f1bff6b208"
    )?);
    // ? ovk
    params.set_ovk(Vec::from_hex(
        "034484bed6abcd44ca9a8af1dd64c8b66d70a0a92471dc24b87b5bfdba8f0ef9"
    )?);

    let taddr: Address = "TQHAvs2ZFTbsd93ycTfw1Wuf1e4WsPZWCp".parse()?;
    params.set_transparent_to_address(taddr.as_ref().to_owned());
    // from amount - 10_000_000
    params.set_to_amount(180_000_000);

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

pub fn debug_taddr_to_zaddr() -> Result<(), Error> {
    // input shielded
    // let mut req = OutputPointInfo::new();
    // let (_, payload, _) = new_grpc_client()?.get_merkle_tree_voucher_info(Default::default(), req).wait()?;
    let grpc_client = new_grpc_client()?;

    let mut params = PrivateParameters::new();

    let taddr: Address = "TGQgfK497YXmjdgvun9Bg5Zu3xE15v17cu".parse()?;
    params.set_transparent_from_address(taddr.as_ref().to_owned());
    // NOTE: current FEE = 10_000000, and amount > FEE
    params.set_from_amount(200_000_000);

    let memo = "are you joking";
    let mut note = Note::new();

    note.set_payment_address(
        "ztron1ze4ytt0pz9t6lafnhptnxted323z2rhtwjvhdq7h3vk3pv9e0ask3j30sn3j93ehx35u7ku7q0d".to_owned()
    );
    // = amount - FEE
    note.set_value(190_000_000);
    note.set_memo(memo.as_bytes().to_owned());

    let (_, rcm_msg, _) = grpc_client.get_rcm(Default::default(), EmptyMessage::new()).wait()?;
    eprintln!("rcm = {:?}", rcm_msg.value.encode_hex::<String>());

    note.set_rcm(rcm_msg.value); // random 32-bytes value

    let recv_note = ReceiveNote {
        note: Some(note).into(),
        ..Default::default()
    };

    params.set_shielded_receives(vec![recv_note].into());

    // when input is transparent.
    params.set_ovk(Vec::from_hex(
        "030c8c2bc59fb3eb8afb047a8ea4b028743d23e7d38c6fa30908358431e2314d"
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
        ("debug", _) => debug_zaddr_to_taddr(),
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}

#[inline]
fn get_rcm(client: &WalletClient) -> Result<Vec<u8>, Error> {
    let (_, mut payload, _) = client.get_rcm(Default::default(), EmptyMessage::new()).wait()?;
    Ok(payload.take_value())
}
