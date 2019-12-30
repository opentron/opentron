use keys::{Address, KeyPair, Private, Public};
use std::collections::hash_set::HashSet;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use serde_json::json;
use sha2::{Digest, Sha512};

mod config;
mod crypto;
mod error;

use config::determine_config_directory;
use error::Error;

const WALLET_FILENAME_EXTENSION: &'static str = ".wallet";

const EMPTY_WALLET_JSON: &'static [u8] = br#"
{
    "version": "v1",
    "salt": "",
    "checksum": "",
    "keys": {}
}
"#;

/// Local wallet implementaion
#[derive(Debug)]
pub struct Wallet {
    name: String,
    wallet_path: PathBuf,
    locked: bool,
    addrs: HashSet<Address>,
    // when unlocked
    keypairs: Option<Vec<KeyPair>>,
}

impl Wallet {
    pub fn new(name: &str) -> Result<Self, Error> {
        if !name.is_empty() && name.chars().all(|c| c.is_ascii() && c.is_alphanumeric()) {
            let config_dir = determine_config_directory();
            let wallet_file = config_dir.join(format!("{:}{:}", name, WALLET_FILENAME_EXTENSION));
            if !config_dir.exists() {
                fs::create_dir_all(&config_dir)?;
            }
            if !wallet_file.exists() {
                let mut file = File::create(&wallet_file)?;
                file.write_all(EMPTY_WALLET_JSON)?;
            }
            let value: serde_json::Value = serde_json::from_str(&fs::read_to_string(&wallet_file)?)?;

            Ok(Wallet {
                name: name.to_owned(),
                wallet_path: wallet_file,
                locked: true,
                addrs: json_to_address(&value)?,
                keypairs: None,
            })
        } else {
            Err(Error::Runtime("invalid wallet name"))
        }
    }

    pub fn lock(&mut self) {
        self.keypairs = None;
        self.locked = true;
    }

    pub fn unlock(&mut self, password: &str) {

    }

    /*
    check_password
    change_password
    list_keys()
    list_public_keys()
    get_private_key(pubkey)
    try_sign_digest(digest, pubkey)
    */
}

fn json_to_address(val: &serde_json::Value) -> Result<HashSet<Address>, Error> {
    if val["version"] == json!("v1".to_owned()) {
        val["keys"]
            .as_object()
            .ok_or(Error::Runtime("malformed json"))
            .and_then(|obj| obj.keys().map(|k| k.parse::<Address>().map_err(Error::from)).collect())
    } else {
        Err(Error::Runtime("malformed json"))
    }
}

#[test]
fn test_hello() {
    let w = Wallet::new("default");

    let key = b"very secret key.";
    let mut hasher = Sha512::new();
    hasher.input(&key[..]);

    let res_key = hasher.result();

    let message = b"Across the Great Wall, we can reach every corner in the world.";
    let res = crypto::aes_encrypt(&res_key[..], &message[..]).unwrap();

    let res = crypto::aes_decrypt(&res_key, &res).unwrap();
    assert_eq!(&message[..], &res[..]);
}
