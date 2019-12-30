use rand::rngs::OsRng;
use secp256k1::key;
use secp256k1::Secp256k1;
use std::convert::TryFrom;
use std::fmt;

use crate::address::Address;
use crate::error::Error;
use crate::private::Private;
use crate::public::Public;

#[derive(Debug, Hash)]
pub struct KeyPair {
    private: Private,
    public: Public,
}

impl KeyPair {
    /// Returns private part of the keypair
    pub fn private(&self) -> &Private {
        &self.private
    }

    /// Returns public part of the keypair
    pub fn public(&self) -> &Public {
        &self.public
    }

    /// Returns public part of the keypair converted into Address
    pub fn address(&self) -> Address {
        Address::from_public(&self.public)
    }

    pub fn from_private(private: Private) -> Result<Self, Error> {
        let public = Public::from_private(&private)?;
        Ok(KeyPair { private, public })
    }

    fn from_keypair(sec: key::SecretKey, publ: key::PublicKey) -> Self {
        let secp = Secp256k1::new();

        let mut pub_key = [0u8; 64];
        pub_key[..].copy_from_slice(&publ.serialize_vec(&secp, /* compressed */ false)[1..]);

        KeyPair {
            private: Private::try_from(&sec[..]).expect("won't fail; qed"),
            public: Public::from(pub_key),
        }
    }

    pub fn generate() -> Self {
        let mut rng = OsRng;
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rng).expect("generate keypair");
        KeyPair::from_keypair(secret_key, public_key)
    }
}

impl fmt::Display for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "private: {:}", &self.private)?;
        writeln!(f, "public:  {:}", &self.public)?;
        write!(f, "address: {:}", &Address::from_public(&self.public))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generate() {
        let key_pair = KeyPair::generate();

        println!("keypair =>\n{:}", key_pair);
        assert_eq!(&Public::from_private(key_pair.private()).unwrap(), key_pair.public());
    }
}
