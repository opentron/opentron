//! Key Errors.
use std::fmt;

use secp256k1::Error as Secp256k1Error;

/// Key Errors.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Public key format error.
    InvalidPublic,
    /// Digest data format error.
    InvalidMessage,
    /// Signature data format error.
    InvalidSignature,
    /// Invalid checksum of base58check.
    InvalidChecksum,
    /// Private key format error.
    InvalidPrivate,
    /// Invalid address format.
    InvalidAddress,
    /// Unable to generate a key pair.
    FailedKeyGeneration,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            Error::InvalidPublic => "Invalid Public",
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

impl std::error::Error for Error {}

impl From<Secp256k1Error> for Error {
    fn from(e: Secp256k1Error) -> Self {
        match e {
            Secp256k1Error::InvalidPublicKey => Error::InvalidPublic,
            Secp256k1Error::InvalidSecretKey => Error::InvalidPrivate,
            Secp256k1Error::InvalidMessage => Error::InvalidMessage,
            _ => Error::InvalidSignature,
        }
    }
}
