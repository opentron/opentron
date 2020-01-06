//! ABI related utilities

use ethabi::encode;
use ethabi::param_type::{ParamType, Reader};
use ethabi::token::{LenientTokenizer, StrictTokenizer, Token, Tokenizer};

use crate::error::Error;
use crate::utils::crypto;

#[inline]
pub fn fnhash(fname: &str) -> [u8; 4] {
    let mut hash_code = [0u8; 4];
    (&mut hash_code[..]).copy_from_slice(&crypto::keccak256(fname.as_bytes())[..4]);
    hash_code
}

// ref: https://github.com/paritytech/ethabi/blob/master/cli/src/main.rs
pub fn encode_params(types: &[&str], values: &[String]) -> Result<Vec<u8>, Error> {
    assert_eq!(types.len(), values.len());

    let types: Vec<ParamType> = types.iter().map(|s| Reader::read(s)).collect::<Result<_, _>>()?;
    let params: Vec<_> = types.into_iter().zip(values.iter().map(|v| v as &str)).collect();

    let tokens = parse_tokens(&params, true)?;
    let result = encode(&tokens);

    Ok(result.to_vec())
}

fn parse_tokens(params: &[(ParamType, &str)], lenient: bool) -> Result<Vec<Token>, Error> {
    params
        .iter()
        .map(|&(ref param, value)| match lenient {
            true => LenientTokenizer::tokenize(param, value),
            false => StrictTokenizer::tokenize(param, value)
        })
        .collect::<Result<_, _>>()
        .map_err(From::from)
}

#[test]
fn test_encode_params() {
    assert_eq!(
        encode_params(&["address"], &["407d73d8a49eeb85d32cf465507dd71d507100c1"]).unwrap(),
        vec![0u8]
    );
}
