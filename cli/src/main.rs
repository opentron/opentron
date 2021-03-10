use chrono::Utc;
use crypto::sha256;
use keys::{Address, KeyPair};
use prost::Message;
use proto::chain::transaction::Raw as TransactionRaw;
use proto::contract as contract_pb;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let from: Address = "TJrsCAaTePcnB2UWkqA53nF2dnyVovotrx".parse()?;
    let kp = KeyPair::from_private("d9c97ae310817434c1901b7eba568ed04c52ccc14cf365f52c78f08664a8e63b".parse()?)?;

    let to: Address = "TJRabPrwbZy45sbavfcjinPJC18kjpRTv8".parse()?;

    let ref_block_hash = hex::decode("000000000002927b159add8337b8beaf8ac82a9635dacbab2c1aaef90e3fd5d7")?;

    let transfer = contract_pb::TransferContract {
        owner_address: from.as_bytes().to_vec(),
        to_address: to.as_bytes().to_vec(),
        amount: 10_000_000,
    };

    let raw = TransactionRaw {
        contract: Some(transfer.into()),
        ref_block_bytes: ref_block_hash[6..8].to_vec(),
        ref_block_hash: ref_block_hash[8..16].to_vec(),
        expiration: Utc::now().timestamp_millis() + 60_000,
        ..Default::default()
    };
    let mut buf = Vec::with_capacity(255);

    raw.encode(&mut buf)?;

    println!("=> {}", hex::encode(&buf));
    println!("=> size {}", buf.len());
    println!("=> txn hash {:?}", sha256(&buf));

    let sig = kp.private().sign(&buf)?;
    println!("=> sig {}", hex::encode(sig.as_bytes()));

    Ok(())
}
