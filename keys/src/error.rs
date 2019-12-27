use secp256k1::Error as Secp256k1Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidPublic,
    InvalidSecret,
    InvalidMessage,
    InvalidSignature,
    InvalidChecksum,
    InvalidPrivate,
    InvalidAddress,
    FailedKeyGeneration,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            Error::InvalidPublic => "Invalid Public",
            Error::InvalidSecret => "Invalid Secret",
            Error::InvalidMessage => "Invalid Message",
            Error::InvalidSignature => "Invalid Signature",
            Error::InvalidChecksum => "Invalid Checksum",
            Error::InvalidPrivate => "Invalid Private",
            Error::InvalidAddress => "Invalid Address",
            Error::FailedKeyGeneration => "Key generation failed",
        };

        msg.fmt(f)
    }
}

impl From<Secp256k1Error> for Error {
    fn from(e: Secp256k1Error) -> Self {
        match e {
            Secp256k1Error::InvalidPublicKey => Error::InvalidPublic,
            Secp256k1Error::InvalidSecretKey => Error::InvalidSecret,
            Secp256k1Error::InvalidMessage => Error::InvalidMessage,
            _ => Error::InvalidSignature,
        }
    }
}
