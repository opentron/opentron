use hex::{FromHex, ToHex};
use keys::{Address, KeyPair, Private, Public, Signature, ZAddress, ZKey};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use config::determine_config_directory;
pub use error::Error;

mod config;
mod crypto;
mod error;

const WALLET_FILENAME_EXTENSION: &'static str = ".wallet";
const WALLET_FILE_VERSION: &'static str = "v1";

type ZSecretKey = [u8; 32];

/// Local wallet implementaion
#[derive(Debug)]
pub struct Wallet {
    name: String,
    wallet_path: PathBuf,
    locked: bool,
    keys: HashSet<Public>,
    // when unlocked
    crypto_key: Option<Vec<u8>>,
    keypairs: Option<Vec<KeyPair>>,
    zaddrs: HashMap<ZAddress, Option<ZSecretKey>>,
}

impl Wallet {
    pub fn create(name: &str, password: &str) -> Result<Self, Error> {
        if name.is_empty() || !name.chars().all(|c| c.is_ascii() && (c.is_alphanumeric() || c == '-')) {
            return Err(Error::Runtime("invalid wallet name"));
        }

        let config_dir = determine_config_directory();
        let wallet_file = config_dir.join(format!("{:}{:}", name, WALLET_FILENAME_EXTENSION));
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        if wallet_file.exists() {
            return Err(Error::Runtime("wallet already exists, use open"));
        }

        let mut file = File::create(&wallet_file)?;
        let json = json!({
            "version": WALLET_FILE_VERSION.to_owned(),
            "salt": random_salt(),
            "checksum": "",
            "keys": {},
            "zkeys": {},
        });
        file.write_all(serde_json::to_string_pretty(&json)?.as_bytes())?;

        let value: serde_json::Value = serde_json::from_str(&fs::read_to_string(&wallet_file)?)?;

        let mut w = Wallet {
            name: name.to_owned(),
            wallet_path: wallet_file,
            locked: true,
            keys: json_to_keys(&value)?,
            crypto_key: None,
            keypairs: None,
            zaddrs: json_to_zkeys(&value)?,
        };
        w.set_password(password)?;
        Ok(w)
    }

    pub fn open(name: &str) -> Result<Self, Error> {
        let config_dir = determine_config_directory();
        let wallet_file = config_dir.join(format!("{:}{:}", name, WALLET_FILENAME_EXTENSION));
        let value: serde_json::Value = serde_json::from_str(&fs::read_to_string(&wallet_file)?)?;
        Ok(Wallet {
            name: name.to_owned(),
            wallet_path: wallet_file,
            locked: true,
            keys: json_to_keys(&value)?,
            crypto_key: None,
            keypairs: None,
            zaddrs: json_to_zkeys(&value)?,
        })
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn wallet_file(&self) -> PathBuf {
        let config_dir = determine_config_directory();
        config_dir.join(format!("{:}{:}", self.name, WALLET_FILENAME_EXTENSION))
    }

    pub fn keys(&self) -> impl Iterator<Item = &Public> {
        self.keys.iter()
    }

    fn is_new(&self) -> bool {
        self.keys.len() == 0
    }

    pub fn lock(&mut self) -> Result<(), Error> {
        if self.locked {
            Err(Error::Runtime("unable to lock a locked wallet"))
        } else {
            self.keypairs = None;
            self.crypto_key = None;
            self.locked = true;
            Ok(())
        }
    }

    pub fn unlock(&mut self, password: &str) -> Result<(), Error> {
        self.check_password(password).and_then(|verified| {
            if verified {
                Ok(())
            } else {
                Err(Error::Runtime("invalid password for wallet"))
            }
        })?;

        let wallet_json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&self.wallet_path)?)?;

        self.crypto_key = Some((&self.get_crypto_key(&wallet_json, password)?[..]).to_owned());

        let decrypt_key = self.crypto_key.as_ref().expect("won't fail; qed");

        let kps = decrypt_wallet_json_to_keypairs(&wallet_json, decrypt_key)?;
        self.keypairs = Some(kps);

        self.zaddrs = decrypt_wallet_json_to_zkeys(&wallet_json, decrypt_key)?;

        self.locked = false;
        Ok(())
    }

    // NOTE: does not support change password for now
    pub fn set_password(&mut self, password: &str) -> Result<(), Error> {
        if self.is_locked() && !self.is_new() {
            Err(Error::Runtime("wallet is locked"))
        } else if password.len() < 8 {
            Err(Error::Runtime("password should be at least 8 chars"))
        } else {
            let mut wallet_json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&self.wallet_path)?)?;
            let salt = random_salt();

            let mut raw = password.as_bytes().to_owned();
            raw.extend_from_slice(salt.as_bytes());

            let checksum = crypto::sha512(&raw);

            wallet_json["salt"] = json!(salt);
            wallet_json["checksum"] = json!((&checksum[..]).encode_hex::<String>());

            self.sync_json_to_wallet_file(&wallet_json)?;
            let _ = self.lock();

            Ok(())
        }
    }

    pub fn check_password(&self, password: &str) -> Result<bool, Error> {
        let wallet_json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&self.wallet_path)?)?;
        let salt = wallet_json["salt"]
            .as_str()
            .ok_or(Error::Runtime("malformed json without a salt field"))?;
        let checksum = wallet_json["checksum"]
            .as_str()
            .ok_or(Error::Runtime("malformed json without a checksum field"))
            .and_then(|c| Vec::from_hex(c).map_err(|_| Error::Runtime("parse checksum failed")))?;

        let mut raw = password.as_bytes().to_owned();
        raw.extend_from_slice(salt.as_bytes());

        let calculated_checksum = crypto::sha512(&raw);

        Ok(&calculated_checksum[..] == &checksum[..])
    }

    pub fn import_key(&mut self, private: Private) -> Result<(), Error> {
        if self.is_locked() {
            Err(Error::Runtime("unable to import key to a locked wallet"))
        } else {
            let kp = KeyPair::from_private(private)?;

            if self.keys.contains(kp.public()) {
                return Err(Error::Runtime("key already in wallet"));
            }

            self.keys.insert(kp.public().clone());
            let mut kps = self.keypairs.take().unwrap_or_default();
            kps.push(kp);
            self.keypairs = Some(kps);

            self.sync_to_wallet_file()?;
            Ok(())
        }
    }

    pub fn remove_key(&mut self, key: &str) -> Result<(), Error> {
        if self.is_locked() {
            return Err(Error::Runtime("unable to remove key on a locked wallet"));
        }
        if let Ok(public) = key.parse::<Public>() {
            if self.keys.contains(&public) {
                self.keys.remove(&public);
                let kps = self.keypairs.take().expect("won't fail; qed");
                self.keypairs = Some(kps.into_iter().filter(|kp| kp.public() != &public).collect::<Vec<_>>());
                self.sync_to_wallet_file()?;
                return Ok(());
            }
        }
        if let Ok(addr) = key.parse::<Address>() {
            if let Some(public) = self.keys.iter().find(|x| &Address::from_public(x) == &addr).clone() {
                let public = public.clone();
                self.keys.remove(&public);
                let kps = self.keypairs.take().expect("won't fail; qed");
                self.keypairs = Some(kps.into_iter().filter(|kp| kp.public() != &public).collect::<Vec<_>>());
                self.sync_to_wallet_file()?;
                return Ok(());
            }
        }
        Err(Error::Runtime("key not in wallet"))
    }

    pub fn create_key(&mut self) -> Result<KeyPair, Error> {
        let kp = KeyPair::generate();
        self.import_key(kp.private().clone())?;
        Ok(kp)
    }

    pub fn sign_digest(&self, digest: &[u8], public: &Public) -> Result<Signature, Error> {
        self.get_private_key(public)
            .and_then(|private| private.sign_digest(digest).map_err(Error::from))
    }

    pub fn get_private_key(&self, public: &Public) -> Result<&Private, Error> {
        if self.is_locked() {
            return Err(Error::Runtime("unable to sign on a locked wallet"));
        }
        if !self.keys.contains(public) {
            return Err(Error::Runtime("key not in wallet"));
        }
        Ok(self
            .keypairs
            .as_ref()
            .unwrap()
            .iter()
            .find(|kp| kp.public() == public)
            .map(|kp| kp.private())
            .unwrap())
    }

    pub fn get_public_key(&self, address: &Address) -> Result<&Public, Error> {
        self.keys
            .iter()
            .find(|x| &Address::from_public(x) == address)
            .ok_or(Error::Runtime("matching public key not found"))
    }

    pub fn import_zkey(&mut self, addr: ZAddress, sk: ZSecretKey) -> Result<(), Error> {
        if self.is_locked() {
            return Err(Error::Runtime("unable to import key to a locked wallet"));
        }
        self.zaddrs.insert(addr, Some(sk));
        self.sync_to_wallet_file()?;
        Ok(())
    }

    pub fn create_zkey(&mut self) -> Result<(ZAddress, ZSecretKey), Error> {
        let zkey = ZKey::generate();
        self.import_zkey(zkey.payment_address().clone(), zkey.sk().clone())?;
        Ok((zkey.payment_address().clone(), zkey.sk().clone()))
    }

    pub fn list_zkeys(&self) -> Result<Vec<ZAddress>, Error> {
        Ok(self.zaddrs.keys().cloned().collect())
    }

    fn sync_to_wallet_file(&self) -> Result<(), Error> {
        assert!(!self.is_locked(), "unreachable condition");

        let mut all_keys = json!({});
        self.keypairs.as_ref().map(|kps| {
            kps.iter()
                .map(|kp| all_keys[kp.public().encode_hex::<String>()] = json!(kp.private().encode_hex::<String>()))
                .collect::<()>()
        });

        let mut wallet_json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&self.wallet_path)?)?;
        let encrypt_key = self.crypto_key.as_ref().expect("won't fail; qed");

        wallet_json["keys"] = encrypt_keypairs_to_json(self.keypairs.as_ref().unwrap(), &encrypt_key)?;
        wallet_json["zkeys"] = encrypt_zkeys_to_json(&self.zaddrs, &encrypt_key)?;
        self.sync_json_to_wallet_file(&wallet_json)?;
        Ok(())
    }

    fn sync_json_to_wallet_file(&self, wallet_json: &serde_json::Value) -> Result<(), Error> {
        let mut file = File::create(&self.wallet_path)?;
        file.write_all(serde_json::to_string_pretty(wallet_json)?.as_bytes())?;
        Ok(())
    }

    fn get_crypto_key(&self, wallet_json: &serde_json::Value, password: &str) -> Result<[u8; 64], Error> {
        let salt = wallet_json["salt"].as_str().ok_or(Error::Runtime("malformed json"))?;

        let mut raw = salt.as_bytes().to_owned();
        raw.extend_from_slice(password.as_bytes());

        Ok(crypto::sha512(&raw))
    }

    // list_keys()
    // list_public_keys()
}

fn json_to_keys(val: &serde_json::Value) -> Result<HashSet<Public>, Error> {
    if val["version"] == json!(WALLET_FILE_VERSION.to_owned()) {
        val["keys"]
            .as_object()
            .ok_or(Error::Runtime("malformed json"))
            .and_then(|obj| obj.keys().map(|k| k.parse::<Public>().map_err(Error::from)).collect())
    } else {
        Err(Error::Runtime("malformed json"))
    }
}

fn json_to_zkeys(val: &serde_json::Value) -> Result<HashMap<ZAddress, Option<ZSecretKey>>, Error> {
    if val["version"] != json!(WALLET_FILE_VERSION.to_owned()) {
        return Err(Error::Runtime("malformed json, version not supported"));
    }
    if val["zkeys"].is_null() {
        return Ok(Default::default());
    }

    val["zkeys"]
        .as_object()
        .ok_or(Error::Runtime("malformed json"))
        .and_then(|obj| {
            obj.keys()
                .map(|k| {
                    k.parse::<ZAddress>()
                        .and_then(|addr| Ok((addr, None)))
                        .map_err(Error::from)
                })
                .collect()
        })
}

fn encrypt_keypairs_to_json(keypairs: &Vec<KeyPair>, encrypt_key: &[u8]) -> Result<serde_json::Value, Error> {
    let mut result = json!({});
    keypairs
        .iter()
        .map(|kp| {
            let pubkey = kp.public().encode_hex::<String>();
            let privkey = kp.private();
            let eprivkey = crypto::aes_encrypt(encrypt_key, privkey.as_bytes())?;
            result[pubkey] = json!(eprivkey.encode_hex::<String>());
            Ok(())
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(result)
}

fn encrypt_zkeys_to_json(
    zkeys: &HashMap<ZAddress, Option<ZSecretKey>>,
    encrypt_key: &[u8],
) -> Result<serde_json::Value, Error> {
    let mut result = json!({});
    zkeys
        .iter()
        .map(|(addr, sk)| {
            let zaddr = addr.to_string();
            let sk = sk.expect("unlocked");
            let esk = crypto::aes_encrypt(encrypt_key, &sk[..])?;
            result[zaddr] = json!(esk.encode_hex::<String>());
            Ok(())
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(result)
}

fn decrypt_wallet_json_to_keypairs(val: &serde_json::Value, decrypt_key: &[u8]) -> Result<Vec<KeyPair>, Error> {
    if val["version"] != json!(WALLET_FILE_VERSION.to_owned()) {
        return Err(Error::Runtime("malformed json"));
    }
    let kps = val["keys"]
        .as_object()
        .ok_or(Error::Runtime("malformed json"))
        .and_then(|obj| {
            obj.iter()
                .map(|(_pubkey, eprivkey)| {
                    let cipher = eprivkey
                        .as_str()
                        .ok_or(Error::Runtime("malformed json"))
                        .and_then(|s| Vec::from_hex(s).map_err(Error::from))?;
                    let privkey = crypto::aes_decrypt(decrypt_key, &cipher)?;
                    KeyPair::from_private(Private::try_from(privkey)?).map_err(Error::from)
                })
                .collect::<Result<Vec<KeyPair>, Error>>()
        })?;
    Ok(kps)
}

fn decrypt_wallet_json_to_zkeys(
    val: &serde_json::Value,
    decrypt_key: &[u8],
) -> Result<HashMap<ZAddress, Option<ZSecretKey>>, Error> {
    if val["version"] != json!(WALLET_FILE_VERSION.to_owned()) {
        return Err(Error::Runtime("malformed json"));
    }
    if val["zkeys"].is_null() {
        return Ok(Default::default());
    }

    let zkeys = val["zkeys"]
        .as_object()
        .ok_or(Error::Runtime("malformed json"))
        .and_then(|obj| {
            obj.iter()
                .map(|(addr, esk)| {
                    let cipher = esk
                        .as_str()
                        .ok_or(Error::Runtime("malformed json"))
                        .and_then(|s| Vec::from_hex(s).map_err(Error::from))?;
                    let sk_raw = crypto::aes_decrypt(decrypt_key, &cipher)?;
                    let mut sk = ZSecretKey::default();
                    sk.copy_from_slice(&sk_raw);
                    Ok((addr.parse()?, Some(sk)))
                })
                .collect::<Result<HashMap<ZAddress, Option<ZSecretKey>>, Error>>()
        })?;
    Ok(zkeys)
}

#[inline]
fn random_salt() -> String {
    let rng = thread_rng();
    rng.sample_iter(Alphanumeric).take(16).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wallet_basic_op_works() {
        let mut w = Wallet::create("test-only", "88888888").unwrap();
        // let mut w = Wallet::open("test-only").unwrap();

        assert!(w.check_password("88888888").unwrap());
        assert!(!w.check_password("68754321").unwrap());

        assert!(w.is_locked());
        assert!(w.unlock("88888888").is_ok());

        w.import_key(
            "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                .parse()
                .unwrap(),
        )
        .expect("import key");

        w.create_zkey().expect("create zkey");

        // println!("{:?}", w.keypairs);
        // println!("{:?}", w.zaddrs);

        let _zaddr = w.zaddrs.keys().next().unwrap();
        assert!(fs::remove_file(w.wallet_file()).is_ok());
    }
}
