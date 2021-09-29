//! A KeyPair type is for generating and saving private/public key pairs.
use std::fmt;

use rand::rngs::OsRng;
use libsecp256k1::{PublicKey, SecretKey};

use crate::address::Address;
use crate::error::Error;
use crate::private::Private;
use crate::public::Public;

/// A KeyPair combines a private key and its corresponding public key.
#[derive(Debug, Hash, Clone)]
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

    /// Construct key pair from private key.
    pub fn from_private(private: Private) -> Result<Self, Error> {
        let public = Public::from_private(&private)?;
        Ok(KeyPair { private, public })
    }

    fn from_keypair(sec: SecretKey, publ: PublicKey) -> Self {
        let mut pub_key = [0u8; 64];
        pub_key[..].copy_from_slice(&publ.serialize()[1..]);

        KeyPair {
            private: Private::from(sec.serialize()),
            public: Public::from(pub_key),
        }
    }

    /// Generates a new random KeyPair.
    pub fn generate() -> Self {
        let mut rng = OsRng;
        let secret_key = SecretKey::random(&mut rng);
        let public_key = PublicKey::from_secret_key(&secret_key);

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
