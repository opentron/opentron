use hex::{FromHex, ToHex};
use secp256k1::key::{PublicKey, SecretKey};
use secp256k1::{Message, RecoverableSignature, RecoveryId, Secp256k1};
use sha2::{Digest, Sha256};
use std::convert::TryFrom;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use crate::error::Error;
use crate::private::Private;
use crate::signature::Signature;

/// Raw public key
#[derive(Clone)]
pub struct Public([u8; 64]);

impl Public {
    pub fn verify_digest(&self, digest: &[u8], signature: &Signature) -> Result<(), Error> {
        let secp = Secp256k1::new();

        // key format
        const TAG_PUBKEY_FULL: u8 = 4;
        let mut key = [0u8; 65];
        key[0] = TAG_PUBKEY_FULL;
        key[1..65].copy_from_slice(&self.0);
        let pub_key = PublicKey::from_slice(&secp, &key)?;

        let rsig =
            RecoverableSignature::from_compact(&secp, &signature[0..64], RecoveryId::from_i32(signature[64] as i32)?)?;
        let sig = rsig.to_standard(&secp);

        secp.verify(&Message::from_slice(digest)?, &sig, &pub_key)
            .map_err(Error::from)
    }

    pub fn verify(&self, data: &[u8], signature: &Signature) -> Result<(), Error> {
        let mut hasher = Sha256::new();
        hasher.input(data);
        let digest = hasher.result();

        self.verify_digest(&digest, signature)
    }

    pub fn recover_digest(digest: &[u8], signature: &Signature) -> Result<Public, Error> {
        let secp = Secp256k1::new();
        let rsig =
            RecoverableSignature::from_compact(&secp, &signature[0..64], RecoveryId::from_i32(signature[64] as i32)?)?;
        let pub_key = secp.recover(&Message::from_slice(digest)?, &rsig)?;

        let mut key = [0u8; 64];
        key[..].copy_from_slice(&pub_key.serialize_vec(&secp, /* compressed */ false)[1..]);

        Ok(Public(key))
    }

    pub fn recover(data: &[u8], signature: &Signature) -> Result<Public, Error> {
        let mut hasher = Sha256::new();
        hasher.input(data);
        let digest = hasher.result();

        Public::recover_digest(&digest, signature)
    }

    pub fn from_private(private: &Private) -> Result<Public, Error> {
        let secp = Secp256k1::new();

        let secret_key = SecretKey::from_slice(&secp, private.as_bytes())?;
        let pub_key = PublicKey::from_secret_key(&secp, &secret_key)?;

        let mut key = [0u8; 64];
        key[..].copy_from_slice(&pub_key.serialize_vec(&secp, /* compressed */ false)[1..]);

        Ok(Public(key))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

impl PartialEq for Public {
    fn eq(&self, other: &Self) -> bool {
        self.0[..] == other.0[..]
    }
}

impl Eq for Public {}

impl fmt::Display for Public {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (&self.0[..]).encode_hex::<String>().fmt(f)
    }
}

impl fmt::Debug for Public {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (&self.0[..]).encode_hex::<String>().fmt(f)
    }
}

// since std::array::LengthAtMost32 is required for derive Hash
impl Hash for Public {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (&self.0[..]).hash(state);
    }
}

impl TryFrom<&[u8]> for Public {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 64 {
            Err(Error::InvalidPublic)
        } else {
            let mut raw = [0u8; 64];
            raw[..64].copy_from_slice(value);
            Ok(Public(raw))
        }
    }
}

impl TryFrom<Vec<u8>> for Public {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl TryFrom<&Vec<u8>> for Public {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(&value[..])
    }
}

impl From<[u8; 64]> for Public {
    fn from(v: [u8; 64]) -> Self {
        Public(v)
    }
}

impl FromHex for Public {
    type Error = Error;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        Vec::from_hex(hex.as_ref())
            .map_err(|_| Error::InvalidPublic)
            .and_then(Self::try_from)
    }
}

impl FromStr for Public {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if s.len() == 128 {
            Vec::from_hex(s)
                .map_err(|_| Error::InvalidPublic)
                .and_then(Self::try_from)
        } else if s.len() == 128 + 2 && (s.starts_with("0x") || s.starts_with("0X")) {
            Vec::from_hex(&s.as_bytes()[2..])
                .map_err(|_| Error::InvalidPublic)
                .and_then(Self::try_from)
        } else {
            Err(Error::InvalidPublic)
        }
    }
}

impl AsRef<[u8]> for Public {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Private;

    #[test]
    fn test_public_verify() {
        let pub_raw = Vec::from_hex(
            "56f19ba7de92264d94f9b6600ec05c16c0b25a064e2ee1cf5bf0dd9661d04515c99\
             c3a6b42b2c574232a5b951bf57cf706bbfd36377b406f9313772f65612cd0",
        )
        .unwrap();
        let pub_key = Public::try_from(pub_raw).unwrap();

        let sig = Signature::from_hex(
            "27ca15976a62ae3677d85f90e20d69d313ada17dba2a869fab3e3a10794f0ed62a6\
             7a711c6106de265adca72c95138be04f40e55d1c2ee76d5fa730f18ed790c01",
        )
        .unwrap();
        let raw_data = Vec::from_hex(
            "0a0246742208f6a72da6712ec2a340d0fecbabf42d5a66080112620a2d747970652\
             e676f6f676c65617069732e636f6d2f70726f746f636f6c2e5472616e73666572436\
             f6e747261637412310a15419cf784b4cc7531f1598c4c322de9afdc597fe76012154\
             1340967e825557559dc46bbf0eabe5ccf99fd134e18e80770cab0c8abf42d",
        )
        .unwrap();
        let priv_key = "d705fc17c82942f85848ab522e42d986279028d09d12ad881bdc0e1327031976"
            .parse::<Private>()
            .unwrap();

        let sign = priv_key.sign(&raw_data).unwrap();

        println!("sign0 = {:}", sign);
        println!("sign1 = {:}", sig);

        assert!(pub_key.verify(&raw_data, &sig).is_ok());

        let rec = Public::recover(&raw_data, &sig).unwrap();
        println!("recover => {:}", rec);
        assert_eq!(rec, pub_key);

        assert_eq!(pub_key, Public::from_private(&priv_key).unwrap())
    }
}
