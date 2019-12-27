use std::fmt;

use crate::address::Address;
use crate::private::Private;
use crate::public::Public;

#[derive(Debug)]
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
}

impl fmt::Display for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "private: {:}", &self.private)?;
        writeln!(f, "public:  {:}", &self.public)?;
        write!(f, "address: {:}", &Address::from_public(&self.public))
    }
}
