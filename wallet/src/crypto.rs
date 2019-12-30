use aes::Aes256;
use cfb_mode::stream_cipher::{NewStreamCipher, StreamCipher};
use cfb_mode::Cfb;

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
