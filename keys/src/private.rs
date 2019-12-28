//! Secret with different format support

use hex::{FromHex, ToHex};
use secp256k1::key::SecretKey;
use secp256k1::{Message, Secp256k1};
use sha2::{Digest, Sha256};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use crate::error::Error;
use crate::signature::Signature;

/// Private key of Secp256k1
#[derive(PartialEq, Debug)]
pub struct Private([u8; 32]);

impl Private {
    pub fn sign_digest(&self, digest: &[u8]) -> Result<Signature, Error> {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&secp, &self.0).expect("32 bytes, within curve order");
        let message = Message::from_slice(digest).map_err(|_| Error::InvalidMessage)?;
        let sig = secp
            .sign_recoverable(&message, &secret_key)
            .map_err(|_| Error::InvalidSignature)?;
        let (rec_id, data) = sig.serialize_compact(&secp);

        let mut raw = [0u8; 65];
        raw[0..64].copy_from_slice(&data[0..64]);
        raw[64] = rec_id.to_i32() as u8;
        Ok(Signature::from(raw))
    }

    pub fn sign(&self, data: &[u8]) -> Result<Signature, Error> {
        let mut hasher = Sha256::new();
        hasher.input(data);
        let digest = hasher.result();

        self.sign_digest(&digest)
    }
}

impl fmt::Display for Private {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.encode_hex::<String>().fmt(f)
    }
}

impl TryFrom<&[u8]> for Private {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 32 {
            Err(Error::InvalidPrivate)
        } else {
            let mut raw = [0u8; 32];
            raw[..32].copy_from_slice(value);
            Ok(Private(raw))
        }
    }
}

impl TryFrom<Vec<u8>> for Private {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl From<[u8; 32]> for Private {
    fn from(v: [u8; 32]) -> Self {
        Private(v)
    }
}

impl FromHex for Private {
    type Error = Error;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        Vec::from_hex(hex.as_ref())
            .map_err(|_| Error::InvalidPrivate)
            .and_then(Self::try_from)
    }
}

impl FromStr for Private {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if s.len() == 64 {
            Vec::from_hex(s)
                .map_err(|_| Error::InvalidPrivate)
                .and_then(Self::try_from)
        } else if s.len() == 66 && (s.starts_with("0x") || s.starts_with("0X")) {
            Vec::from_hex(&s.as_bytes()[2..])
                .map_err(|_| Error::InvalidPrivate)
                .and_then(Self::try_from)
        } else {
            Err(Error::InvalidPrivate)
        }
    }
}

// NOTE: AsRef<[u8]> implies ToHex
impl AsRef<[u8]> for Private {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_sign() {
        let raw = Vec::from_hex(
            "0a0246742208f6a72da6712ec2a340d0fecbabf42d5a66080112620a2d747970652\
             e676f6f676c65617069732e636f6d2f70726f746f636f6c2e5472616e7366657243\
             6f6e747261637412310a15419cf784b4cc7531f1598c4c322de9afdc597fe760121\
             541340967e825557559dc46bbf0eabe5ccf99fd134e18e80770cab0c8abf42d",
        )
        .unwrap();
        let priv_key: Private = "d705fc17c82942f85848ab522e42d986279028d09d12ad881bdc0e1327031976"
            .parse()
            .unwrap();

        let sign = priv_key.sign(&raw).unwrap();
        let sign2 = Signature::from_hex(
            "27ca15976a62ae3677d85f90e20d69d313ada17dba2a869fab3e3a10794f0ed62a6\
             7a711c6106de265adca72c95138be04f40e55d1c2ee76d5fa730f18ed790c01",
        )
        .unwrap();
        assert_eq!(sign, sign2);
    }
}
