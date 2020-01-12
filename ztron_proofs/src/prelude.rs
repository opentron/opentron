use bellman::groth16::{prepare_verifying_key, Parameters, PreparedVerifyingKey};
use pairing::bls12_381::Bls12;
use std::io::BufReader;

use crate::hashreader;

pub fn load_parameters() -> (
    Parameters<Bls12>,
    PreparedVerifyingKey<Bls12>,
    Parameters<Bls12>,
    PreparedVerifyingKey<Bls12>,
) {
    // Load from each of the paths
    let spend_raw = include_bytes!("../../ztron-params/sapling-spend.params");
    let output_raw = include_bytes!("../../ztron-params/sapling-output.params");

    let mut spend_r = hashreader::HashReader::new(BufReader::with_capacity(1024 * 1024, &spend_raw[..]));
    let mut output_r = hashreader::HashReader::new(BufReader::with_capacity(1024 * 1024, &output_raw[..]));

    // Deserialize params
    let spend_params =
        Parameters::<Bls12>::read(&mut spend_r, false).expect("couldn't deserialize Sapling spend parameters file");
    let output_params =
        Parameters::<Bls12>::read(&mut output_r, false).expect("couldn't deserialize Sapling spend parameters file");

    // Prepare verifying keys
    let spend_vk = prepare_verifying_key(&spend_params.vk);
    let output_vk = prepare_verifying_key(&output_params.vk);

    (spend_params, spend_vk, output_params, output_vk)
}
