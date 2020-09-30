//! The address type and decode/encode functions.
use std::fmt;
use std::str::FromStr;

use base58::{FromBase58, ToBase58};
use digest::Digest;
use hex::FromHex;
use sha2::Sha256;
use sha3::Keccak256;
use std::convert::TryFrom;

use crate::error::Error;
use crate::private::Private;
use crate::public::Public;

/// The mainnet uses 0x41('A') as address type prefix.
const ADDRESS_TYPE_PREFIX: u8 = 0x41;

/// Address of Tron, saved in 21-byte format.
#[derive(PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct Address([u8; 21]);

impl Address {
    /// Address of a public key.
    pub fn from_public(public: &Public) -> Address {
        let mut hasher = Keccak256::new();
        hasher.update(public);
        let digest = hasher.finalize();

        let mut raw = [ADDRESS_TYPE_PREFIX; 21];
        raw[1..21].copy_from_slice(&digest[digest.len() - 20..]);

        Address(raw)
    }

    /// Address of a private key.
    pub fn from_private(private: &Private) -> Address {
        Address::from_public(&Public::from_private(private).expect("public from private; qed"))
    }

    /// As raw 21-byte address.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// As 20-byte address that compatiable with Ethereum.
    pub fn as_tvm_bytes(&self) -> &[u8] {
        &self.0[1..]
    }

    /// Address from 20-byte address that compatiable with Ethereum.
    pub fn from_tvm_bytes(raw: &[u8]) -> Self {
        assert!(raw.len() == 20);

        let mut inner = [ADDRESS_TYPE_PREFIX; 21];
        inner[1..21].copy_from_slice(raw);
        Address(inner)
    }

    /// Address rom raw 21-byte.
    pub fn from_bytes(raw: &[u8]) -> &Address {
        assert!(raw.len() == 21);

        unsafe { std::mem::transmute(&raw[0]) }
    }
}

impl Default for Address {
    fn default() -> Self {
        Address([0x41, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        b58encode_check(&self.0).fmt(f)
    }
}

impl ::std::fmt::Debug for Address {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_tuple("Address").field(&self.to_string()).finish()
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

impl TryFrom<&str> for Address {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Address::from_str(value)
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
        if s.len() == 34 {
            // T-address, aka. base58check address.
            b58decode_check(s).and_then(Address::try_from)
        } else if s.len() == 42 && s[..2] == hex::encode(&[ADDRESS_TYPE_PREFIX]) {
            // 41-address, aka. hex address.
            Vec::from_hex(s)
                .map_err(|_| Error::InvalidAddress)
                .and_then(Address::try_from)
        } else if s.len() == 42 && s.starts_with("0x") {
            // 0x-address, aka. eth address
            Vec::from_hex(&s.as_bytes()[2..])
                .map_err(|_| Error::InvalidAddress)
                .map(|bs| Address::from_tvm_bytes(&bs))
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

/// Base58check encode.
pub fn b58encode_check<T: AsRef<[u8]>>(raw: T) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_ref());
    let digest1 = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(&digest1);
    let digest = hasher.finalize();

    let mut raw = raw.as_ref().to_owned();
    raw.extend(&digest[..4]);
    raw.to_base58()
}

/// Base58check decode.
pub fn b58decode_check(s: &str) -> Result<Vec<u8>, Error> {
    let mut result = s.from_base58().map_err(|_| Error::InvalidAddress)?;

    let check = result.split_off(result.len() - 4);

    let mut hasher = Sha256::new();
    hasher.update(&result);
    let digest1 = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(&digest1);
    let digest = hasher.finalize();

    if check != &digest[..4] {
        Err(Error::InvalidChecksum)
    } else {
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex::ToHex;

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
            addr.as_bytes().encode_hex::<String>(),
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
