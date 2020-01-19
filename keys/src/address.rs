use base58::{FromBase58, ToBase58};
use hex::FromHex;
use sha2::{Digest, Sha256};
use sha3::Keccak256;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr; // .parse

use crate::error::Error;
use crate::public::Public;
use crate::private::Private;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Address([u8; 21]);

impl Address {
    pub fn from_public(public: &Public) -> Address {
        let mut raw = [0x41; 21];

        let mut hasher = Keccak256::new();
        hasher.input(public);
        let digest = hasher.result();

        raw[1..21].copy_from_slice(&digest[digest.len() - 20..]);

        Address(raw)
    }

    pub fn from_private(private: &Private) -> Address {
        Address::from_public(&Public::from_private(private).expect("public from private; qed"))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_tvm_bytes(&self) -> &[u8] {
        &self.0[1..]
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        b58encode_check(&self.0).fmt(f)
    }
}

impl TryFrom<&[u8]> for Address {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 21 {
            Err(Error::InvalidAddress)
        } else {
            let mut raw = [0u8; 21];
            raw[..21].copy_from_slice(value);
            Ok(Address(raw))
        }
    }
}

impl TryFrom<Vec<u8>> for Address {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl TryFrom<&Vec<u8>> for Address {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl FromHex for Address {
    type Error = Error;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        Address::try_from(hex.as_ref())
    }
}

impl FromStr for Address {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if s.len() == 34 && s.as_bytes()[0] == b'T' {
            b58decode_check(s).and_then(Address::try_from)
        } else if s.len() == 42 && s.starts_with("41") {
            Vec::from_hex(s)
                .map_err(|_| Error::InvalidAddress)
                .and_then(Address::try_from)
        } else if s.len() == 44 && (s.starts_with("0x") || s.starts_with("0X")) {
            Vec::from_hex(&s.as_bytes()[2..])
                .map_err(|_| Error::InvalidAddress)
                .and_then(Address::try_from)
        } else {
            Err(Error::InvalidAddress)
        }
    }
}

// NOTE: AsRef<[u8]> implies ToHex
impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

fn b58encode_check<T: AsRef<[u8]>>(raw: T) -> String {
    let mut hasher = Sha256::new();
    hasher.input(raw.as_ref());
    let digest1 = hasher.result();

    let mut hasher = Sha256::new();
    hasher.input(&digest1);
    let digest = hasher.result();

    let mut raw = raw.as_ref().to_owned();
    raw.extend(&digest[..4]);
    raw.to_base58()
}

// FIXME: better isolated to a crate
fn b58decode_check(s: &str) -> Result<Vec<u8>, Error> {
    let mut result = s.from_base58().map_err(|_| Error::InvalidAddress)?;

    let check = result.split_off(result.len() - 4);

    let mut hasher = Sha256::new();
    hasher.input(&result);
    let digest1 = hasher.result();

    let mut hasher = Sha256::new();
    hasher.input(&digest1);
    let digest = hasher.result();

    if check != &digest[..4] {
        Err(Error::InvalidChecksum)
    } else {
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address() {
        let addr = Address([
            65, 150, 163, 186, 206, 90, 218, 207, 99, 126, 183, 204, 121, 213, 120, 127, 66, 71, 218, 75, 190,
        ]);

        assert_eq!("TPhiVyQZ5xyvVK2KS2LTke8YvXJU5wxnbN", format!("{:}", addr));
        assert_eq!(addr, "TPhiVyQZ5xyvVK2KS2LTke8YvXJU5wxnbN".parse().expect("parse error"));
        assert_eq!(
            addr,
            "4196a3bace5adacf637eb7cc79d5787f4247da4bbe"
                .parse()
                .expect("parse error")
        );

        assert_eq!(
            addr.as_ref().encode_hex::<String>(),
            "4196a3bace5adacf637eb7cc79d5787f4247da4bbe"
        )
    }

    #[test]
    fn test_address_from_public() {
        let public = Public::from_hex("56f19ba7de92264d94f9b6600ec05c16c0b25a064e2ee1cf5bf0dd9661d04515c99c3a6b42b2c574232a5b951bf57cf706bbfd36377b406f9313772f65612cd0").unwrap();

        let addr = Address::from_public(&public);
        assert_eq!(addr.to_string(), "TQHAvs2ZFTbsd93ycTfw1Wuf1e4WsPZWCp");
    }
}
