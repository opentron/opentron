use chrono::{Local, TimeZone};
use clap::ArgMatches;
use futures::executor;
use hex::FromHex;
use keys::Address;
use proto::api::{BytesMessage, EmptyMessage, NumberMessage};
use proto::core::Account;
use serde_json::json;
use std::collections::HashSet;

use crate::error::Error;
use crate::utils::client;
use crate::utils::jsont;
use crate::utils::trx;

mod contract;
mod transaction;

fn node_info() -> Result<(), Error> {
    let payload = executor::block_on(
        client::GRPC_CLIENT
            .get_node_info(Default::default(), EmptyMessage::new())
            .drop_metadata(),
    )?;
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn visit_node(ip: &str, edges: &mut HashSet<(String, String)>) -> Result<(), Error> {
    let mut stack = vec![ip.to_owned()];
    let mut visited = HashSet::new();

    while let Some(self_ip) = stack.pop() {
        visited.insert(self_ip.clone());

        eprintln!("({})visiting ... {}", edges.len(), self_ip);
        if let Ok(grpc_client) = client::new_grpc_client(&format!("{}:50051", self_ip)) {
            if let Ok(node_info) = executor::block_on(
                grpc_client
                    .get_node_info(Default::default(), EmptyMessage::new())
                    .drop_metadata(),
            ) {
                eprintln!(
                    "p2p version: {}, node version: {}",
                    node_info.get_configNodeInfo().get_p2pVersion(),
                    node_info.get_configNodeInfo().get_codeVersion()
                );
                for peer in node_info.get_peerInfoList() {
                    let peer_ip = peer.get_host();
                    let edge = (self_ip.to_owned(), peer_ip.to_owned());
                    if !edges.contains(&edge) {
                        edges.insert(edge);
                    }
                    if !visited.contains(peer_ip) {
                        stack.push(peer_ip.to_owned());
                    }
                }
            }
        }
    }
    Ok(())
}

fn get_node_graph() -> Result<(), Error> {
    let mut edges: HashSet<(String, String)> = HashSet::new();
    let node_info = executor::block_on(
        client::GRPC_CLIENT
            .get_node_info(Default::default(), EmptyMessage::new())
            .drop_metadata(),
    )?;

    for peer in node_info.get_peerInfoList() {
        let ip = peer.get_host();
        let _ = visit_node(ip, &mut edges);
    }
    println!("digraph G {{");
    for (from, to) in edges {
        println!("  {:?} -> {:?};", from, to);
    }
    println!("}}");
    Ok(())
}

fn dump_merkle_tree() -> Result<(), Error> {
    use crate::utils::crypto;
    use protobuf::Message;

    let block_nums = [
        1102553, 1103364, 1103650, 1104274, 1104326, 1104948, 1105494, 1106300, 1106888, 1107730, 1110468, 1110780,
        1110832, 1111066, 1111222, 1111430, 1111508, 1111818, 1111896, 1111922, 1111966, 1112021, 1112026, 1112052,
        1112078, 1112099, 1112104, 1112122, 1112130, 1112156, 1112564, 1112668, 1112754, 1113222, 1114106, 1114124,
        1114205, 1114444, 1114522, 1114548, 1114938, 1115536, 1115640, 1115788, 1115822, 1116264, 1116282, 1116308,
        1116386, 1116706, 1116732, 1116758, 1116984, 1117018, 1117460, 1117668, 1117694, 1117928, 1118050, 1118292,
        1118370, 1118604, 1118758, 1118810, 1118914, 1118992, 1119408, 1119460, 1119486, 1119642, 1119660, 1119668,
        1119772, 1120032, 1120084, 1120136, 1120180, 1120344, 1120370, 1120388, 1120422, 1121124, 1121228, 1121402,
        1121696, 1121852, 1122736, 1123022, 1123568, 1123638, 1123750, 1124166, 1124756, 1124990, 1125336, 1125518,
        1125750, 1125846, 1126192, 1126314, 1126340, 1126582, 1127128, 1127536, 1128272, 1129000, 1129858, 1129910,
        1129962, 1130118, 1130690, 1131028, 1131626, 1132922, 1132941, 1132976, 1133084, 1134406, 1135135, 1135513,
        1135675, 1135702, 1135837, 1135918, 1135972, 9023535, 9031292, 9031302, 9031395, 9047637, 9050591, 9088413,
        9117054, 9144428, 9258022, 9314226, 9342309, 9402274, 9545005, 9601119, 9601144, 9601555, 9746423, 9859519,
        9859543, 9916886, 9916906, 9946923, 9946945, 9972961, 9972983, 10344002, 10430101, 10451465, 10451490,
        10451737, 10451798, 10451980, 10452060, 10452149, 10452186, 10452823, 10453179, 10466915, 10466960, 10467186,
        10467229, 10467356, 10467394, 10469444, 10469508, 10472308, 10474132, 10474645, 10475107, 10476216, 10476312,
        10476409, 10476461, 10476746, 10477778, 10479675, 10479693, 10479731, 10479775, 10479798, 10479820, 10479893,
        10479904, 10479914, 10479920, 10480305, 10480321, 10480335, 10480342, 10480383, 10480470, 10480508, 10480520,
        10480564, 10480574, 10480741, 10481131, 10481484, 10495287, 10495427, 10495547, 10496286, 10496340, 10496614,
        10496627, 10496675, 10496705, 10496720, 10496760, 10496828, 10498246, 10498464, 10498612, 10498687, 10498733,
        10498801, 10501834, 10503636, 10544991, 10545010, 10596876, 11229571, 11229645, 11229672, 11240120, 12606097,
        14734523, 14734681, 14848103, 14848129, 14877094, 14877125, 14877762, 14905850, 14905872, 15137348, 15137376,
        15302325, 15767553, 15969715, 16258004, 16258011, 16346033, 16346068, 16487094, 16487116, 16515875, 16515951,
        16660568, 16660633, 16746113, 16746179, 16861899, 16862200, 16948617, 16948646, 16977391, 17839835, 17839980,
        17840259, 17897094, 17897144, 17897176, 17897366, 17953720, 17953734, 17984748, 18011991, 18213657, 18214173,
    ];

    for &num in block_nums.iter() {
        let mut req = NumberMessage::new();
        req.num = num;
        let block = executor::block_on(
            client::GRPC_CLIENT
                .get_block_by_num2(Default::default(), req)
                .drop_metadata(),
        )?;

        for (i, txn_ex) in block.get_transactions().iter().enumerate() {
            let txn = txn_ex.get_transaction();
            let raw = txn.write_to_bytes()?;
            let txn_merkle_node = crypto::sha256(&raw);
            let txn_hash = crypto::sha256(&txn.get_raw_data().write_to_bytes()?);

            /*
                        if txn.get_ret().len() > 1 {
                            println!("# ! len(Transaction.ret) = {}", txn.get_ret().len());
                            print!("# ! ret[1] = {:?}", txn.get_ret()[1]);
                            println!(
                                r#"
            [[merkle-tree-patch]]
            # block => {} txn => {}
            txn = "{}"
            tree-node-hash = "{}"
            type = "extra-ret"
            "#,
                                num,
                                i,
                                hex::encode(txn_hash),
                                hex::encode(txn_merkle_node)
                            );
                        }
                        if let Some(ref ufields) = txn.get_raw_data().unknown_fields.fields {
                            print!("# ! malformed raw_data {:?}", ufields);
                            println!(
                                r#"
            [[merkle-tree-patch]]
            # block => {} txn => {}
            txn = "{}"
            tree-node-hash = "{}"
            type = "malformed-raw-data"
            "#,
                                num,
                                i,
                                hex::encode(txn_hash),
                                hex::encode(txn_merkle_node)
                            );
                        } */
            if let Some(ret) = txn.get_ret().get(0) {
                if let Some(ref ufields) = ret.unknown_fields.fields {
                    print!("# ! malformed ret {:?}", ufields);
                    println!(
                        r#"
[[merkle-tree-patch]]
# block => {} txn => {}
txn = "{}"
tree-node-hash = "{}"
type = "malformed-ret"
"#,
                        num,
                        i,
                        hex::encode(txn_hash),
                        hex::encode(txn_merkle_node)
                    );
                }
            }
            if let Some(ref ufields) = txn.unknown_fields.fields {
                println!("# ! malformed txn {:?}", ufields);
                print!("# ! malformed raw_data {:?}", txn.get_raw_data().unknown_fields.fields);
                println!(
                    r#"
[[merkle-tree-patch]]
# block => {} txn => {}
txn = "{}"
tree-node-hash = "{}"
type = "malformed-txn"
"#,
                    num,
                    i,
                    hex::encode(txn_hash),
                    hex::encode(txn_merkle_node)
                );
            }
        }
    }
    Ok(())
}

fn get_merkle_tree(matches: &ArgMatches) -> Result<(), Error> {
    use crate::utils::crypto;
    use protobuf::Message;

    if !matches.is_present("BLOCK") {
        return dump_merkle_tree();
    }

    let num = matches.value_of("BLOCK").unwrap().parse()?;
    let mut req = NumberMessage::new();
    req.num = num;
    let block = executor::block_on(
        client::GRPC_CLIENT
            .get_block_by_num2(Default::default(), req)
            .drop_metadata(),
    )?;

    for (i, txn_ex) in block.get_transactions().iter().enumerate() {
        let txn = txn_ex.get_transaction();
        let raw = txn.write_to_bytes()?;
        let txn_merkle_node = crypto::sha256(&raw);
        let txn_hash = crypto::sha256(&txn.get_raw_data().write_to_bytes()?);
        println!(
            "{:4}  {} txn={}",
            i,
            hex::encode(txn_merkle_node),
            hex::encode(txn_hash)
        );
        if txn.get_ret().len() > 1 {
            eprintln!("      ! len(Transaction.ret) = {}", txn.get_ret().len());
        }
        for ret in txn.get_ret().iter() {
            if let Some(ref ufields) = ret.unknown_fields.fields {
                eprintln!("      ! malformed ret {:?}", ufields);
            }
        }
        if let Some(ref ufields) = txn.get_raw_data().unknown_fields.fields {
            eprintln!("      ! malformed raw_data {:?}", ufields);
        }
        if let Some(ref ufields) = txn.unknown_fields.fields {
            eprintln!("      ! malformed txn {:?}", ufields);
        }
    }

    Ok(())
}

fn get_block(matches: &ArgMatches) -> Result<(), Error> {
    let mut block = match matches.value_of("BLOCK") {
        Some(id) if id.starts_with("0000") => {
            let mut req = BytesMessage::new();
            req.value = Vec::from_hex(id)?;
            let payload = executor::block_on(
                client::GRPC_CLIENT
                    .get_block_by_id(Default::default(), req)
                    .drop_metadata(),
            )?;
            serde_json::to_value(&payload)?
        }
        Some(num) => {
            let mut req = NumberMessage::new();
            req.num = num.parse()?;
            let payload = executor::block_on(
                client::GRPC_CLIENT
                    .get_block_by_num2(Default::default(), req)
                    .drop_metadata(),
            )?;
            serde_json::to_value(&payload)?
        }
        None => {
            let payload = executor::block_on(
                client::GRPC_CLIENT
                    .get_now_block(Default::default(), EmptyMessage::new())
                    .drop_metadata(),
            )?;
            serde_json::to_value(&payload)?
        }
    };
    if block["block_header"].is_null() {
        return Err(Error::Runtime("block not found on chain"));
    }

    jsont::fix_block(&mut block)?;

    println!("{:}", serde_json::to_string_pretty(&block)?);
    eprintln!("! Block Number: {}", block["block_header"]["raw_data"]["number"]);
    eprintln!(
        "! Number of Transactions: {}",
        block["transactions"].as_array().unwrap().len()
    );
    eprintln!(
        "! Generated At: {}",
        Local.timestamp(
            block["block_header"]["raw_data"]["timestamp"].as_i64().unwrap() / 1_000,
            0
        )
    );
    let _ = block["block_header"]["raw_data"]["witness_address"]
        .as_str()
        .unwrap()
        .parse::<Address>()
        .map(|addr| {
            eprintln!("! Witness: {}", addr);
        });

    Ok(())
}

/// Get account infomation.
fn get_account(name: &str) -> Result<(), Error> {
    let mut req = Account::new();
    let addr = name.parse::<Address>()?;
    req.set_address(addr.as_bytes().to_owned());
    // FIXME: account name not supported
    // req.set_account_name(name.as_bytes().to_owned());

    let payload = executor::block_on(client::GRPC_CLIENT.get_account(Default::default(), req).drop_metadata())?;
    if payload.get_address().is_empty() {
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Err(Error::Runtime("account not found on chain"));
    }

    let mut account = serde_json::to_value(&payload)?;
    jsont::fix_account(&mut account);

    println!("{}", serde_json::to_string_pretty(&account)?);

    eprintln!(
        "! Type = {:?}{}",
        payload.field_type,
        if payload.is_witness { " | Witness" } else { "" }
    );
    eprintln!("! Address(Base58Check) = {:}", addr);
    eprintln!("! Created At: {}", Local.timestamp(payload.create_time / 1_000, 0));

    if payload.balance != 0 {
        eprintln!(
            "! Balance = {}",
            trx::format_amount_with_surfix(payload.balance, "TRX", 6)
        );
    }
    if payload.allowance != 0 {
        eprintln!(
            "! Unwithdrawn SR Reward = {}",
            trx::format_amount_with_surfix(payload.allowance, "TRX", 6)
        );
    }

    Ok(())
}

/// Get account permission info.
fn get_account_permission(name: &str) -> Result<(), Error> {
    let mut req = Account::new();
    let addr = name.parse::<Address>()?;
    req.set_address(addr.as_bytes().to_owned());

    let payload = executor::block_on(client::GRPC_CLIENT.get_account(Default::default(), req).drop_metadata())?;
    if payload.get_address().is_empty() {
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Err(Error::Runtime("account not found on chain"));
    }

    let mut account = serde_json::to_value(&payload)?;
    jsont::fix_account(&mut account);
    let permission_info = json!({
        "owner": account["owner_permission"],
        "witness": account["witness_permission"],
        "actives": account["active_permission"],
    });

    println!("{}", serde_json::to_string_pretty(&permission_info)?);
    Ok(())
}

/// Get account energy and bandwidth infomation.
fn get_account_resource(name: &str) -> Result<(), Error> {
    let mut req = Account::new();
    let addr = name.parse::<Address>()?;
    req.set_address(addr.as_bytes().to_owned());

    let payload = executor::block_on(
        client::GRPC_CLIENT
            .get_account_resource(Default::default(), req)
            .drop_metadata(),
    )?;

    println!("{}", serde_json::to_string_pretty(&payload)?);
    if payload.get_freeNetLimit() == 0 {
        return Err(Error::Runtime("account not found on chain"));
    }
    eprintln!(
        "! Free Bandwith Usage: {}/{}",
        payload.freeNetUsed, payload.freeNetLimit
    );
    if payload.EnergyLimit > 0 {
        eprintln!("! Energy Usage: {}/{}", payload.EnergyUsed, payload.EnergyLimit);
    }
    eprintln!(
        "! Energy By Freezing    1_TRX = {:.5}",
        payload.TotalEnergyLimit as f64 / payload.TotalEnergyWeight as f64
    );
    eprintln!(
        "! Bandwidth By Freezing 1_TRX = {:.5}",
        payload.TotalNetLimit as f64 / payload.TotalNetWeight as f64
    );
    Ok(())
}

fn get_proposal_by_id(id: &str) -> Result<(), Error> {
    // NOTE: id should be encoded to 8 bytes as i64
    let mut req = BytesMessage::new();
    req.set_value((id.parse::<i64>()?.to_be_bytes()[..]).to_owned());

    let payload = executor::block_on(
        client::GRPC_CLIENT
            .get_proposal_by_id(Default::default(), req)
            .drop_metadata(),
    )?;
    if payload.get_proposal_id() == 0 {
        return Err(Error::Runtime("proposal not found on chain"));
    }

    let mut proposal = serde_json::to_value(&payload)?;
    proposal["proposer_address"] = json!(jsont::bytes_to_hex_string(&proposal["proposer_address"]));
    proposal["approvals"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .map(|addr| *addr = json!(jsont::bytes_to_hex_string(addr)))
        .last();

    println!("{}", serde_json::to_string_pretty(&proposal)?);
    Ok(())
}

fn get_asset_by_id(id: &str) -> Result<(), Error> {
    let mut req = BytesMessage::new();
    req.set_value(id.as_bytes().to_owned());

    let payload = executor::block_on(
        client::GRPC_CLIENT
            .get_asset_issue_by_id(Default::default(), req)
            .drop_metadata(),
    )?;
    if payload.get_id().is_empty() {
        return Err(Error::Runtime("asset not found"));
    }
    let mut asset = serde_json::to_value(&payload)?;
    jsont::fix_asset_issue_contract(&mut asset);
    println!("{}", serde_json::to_string_pretty(&asset)?);
    Ok(())
}

fn get_reward_info(addr: &str) -> Result<(), Error> {
    let addr = addr.parse::<Address>()?;
    let mut req = BytesMessage::new();
    req.set_value(addr.as_bytes().to_owned());

    let payload = executor::block_on(
        client::GRPC_CLIENT
            .get_reward_info(Default::default(), req)
            .drop_metadata(),
    )?;
    println!("value = {}", payload.get_num());
    Ok(())
}

fn get_brokerage_info(addr: &str) -> Result<(), Error> {
    let addr = addr.parse::<Address>()?;
    let mut req = BytesMessage::new();
    req.set_value(addr.as_bytes().to_owned());

    let payload = executor::block_on(
        client::GRPC_CLIENT
            .get_brokerage_info(Default::default(), req)
            .drop_metadata(),
    )?;
    println!("sharing percent = {}%", 100 - payload.get_num());
    println!("kept percent    = {}%", payload.get_num());
    Ok(())
}

pub fn main(matches: &ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("node", _) => node_info(),
        ("node_graph", _) => get_node_graph(),
        ("block", Some(arg_matches)) => get_block(arg_matches),
        ("merkle_tree", Some(arg_matches)) => get_merkle_tree(arg_matches),
        ("transaction", Some(tr_matches)) => {
            let id = tr_matches.value_of("ID").expect("required in cli.yml; qed");
            transaction::get_transaction(id)
        }
        ("transaction_info", Some(tr_matches)) => {
            let id = tr_matches.value_of("ID").expect("required in cli.yml; qed");
            transaction::get_transaction_info(id)
        }
        ("account", Some(arg_matches)) => {
            let name = arg_matches.value_of("NAME").expect("required is cli.yml; qed");
            get_account(name)
        }
        ("account_permission", Some(arg_matches)) => {
            let name = arg_matches.value_of("NAME").expect("required is cli.yml; qed");
            get_account_permission(name)
        }
        ("account_resource", Some(arg_matches)) => {
            let name = arg_matches.value_of("NAME").expect("required is cli.yml; qed");
            get_account_resource(name)
        }
        ("contract", Some(arg_matches)) => {
            let addr = arg_matches.value_of("ADDR").expect("required is cli.yml; qed");
            contract::run(addr)
        }
        ("proposal", Some(arg_matches)) => {
            let id = arg_matches.value_of("ID").expect("required in cli.yml; qed");
            get_proposal_by_id(&id)
        }
        ("asset", Some(arg_matches)) => {
            let id = arg_matches.value_of("ID").expect("required in cli.yml; qed");
            get_asset_by_id(&id)
        }
        ("reward", Some(arg_matches)) => {
            let addr = arg_matches.value_of("ADDR").expect("required in cli.yml; qed");
            get_reward_info(&addr)
        }
        ("brokerage", Some(arg_matches)) => {
            let addr = arg_matches.value_of("ADDR").expect("required in cli.yml; qed");
            get_brokerage_info(&addr)
        }
        _ => {
            eprintln!("{}", matches.usage());
            Err(Error::Runtime("error parsing command line"))
        }
    }
}
