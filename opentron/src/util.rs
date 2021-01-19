use byteorder::{ByteOrder, BE};
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
struct Ip {
    origin: String,
}

pub fn get_my_ip() -> Result<String, Box<dyn Error>> {
    let ip = reqwest::blocking::get("http://httpbin.org/ip")?.json::<Ip>()?;
    Ok(ip.origin)
}

pub fn block_hash_to_number(hash: &[u8]) -> i64 {
    BE::read_u64(&hash[..8]) as _
}
