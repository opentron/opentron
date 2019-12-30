use aes::Aes256;
use cfb_mode::stream_cipher::{NewStreamCipher, StreamCipher};
use cfb_mode::Cfb;
use sha2::{Digest, Sha512};
use std::mem;


use crate::error::Error;

type AesCfb = Cfb<Aes256>;

// NOTE: key, sha512 key
pub fn aes_encrypt(key: &[u8], plain_text: &[u8]) -> Result<Vec<u8>, Error> {
    let iv = &key[32..48]; // BlockSize [u8; 16]
    let key = &key[..32]; //  KeySize [u8; 32]

    let mut buffer = plain_text.to_owned();

    // encrypt plaintext
    AesCfb::new_var(key, iv)
        .map_err(|e| {
            eprintln!("error => {:?}", e);
            Error::Runtime("InvalidKeyNonceLength")
        })?
        .encrypt(&mut buffer);

    Ok(buffer)
}

pub fn aes_decrypt(key: &[u8], cipher_text: &[u8]) -> Result<Vec<u8>, Error> {
    let iv = &key[32..48];
    let key = &key[..32];

    let mut buffer = cipher_text.to_owned();

    AesCfb::new_var(key, iv)
        .map_err(|_| Error::Runtime("InvalidKeyNonceLength"))?
        .decrypt(&mut buffer);

    Ok(buffer)
}

#[inline]
pub fn sha512(input: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.input(input);
    // NOTE: From<GenericArray<u8, 64>> is not impl-ed for [u8; 64]
    unsafe {
        mem::transmute(hasher.result())
    }
}
