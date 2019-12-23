extern crate proto;
extern crate hex;
extern crate protobuf;


use proto::core::{Transaction_raw};
use protobuf::{parse_from_bytes};

fn main() {
    let input = "0a029f9122082fc729fb70fb41514098d7d790f32d5a860108041281010a30747970652e676f6f676c65617069732e636f6d2f70726f746f636f6c2e566f74655769746e657373436f6e7472616374124d0a1541340967e825557559dc46bbf0eabe5ccf99fd134e12190a1541f16412b9a17ee9408646e2a21e16478f72ed1e95100312190a1541f1a0466076c57c9f6d07decc86021ddbf8bae0b2100570c392d490f32d";

    let raw =  hex::decode(input).expect("hex decode ok");
    let tx = parse_from_bytes::<Transaction_raw>(&raw).expect("parse ok");

    println!("{:?}", tx);
    println!("Hello, world!");
}
