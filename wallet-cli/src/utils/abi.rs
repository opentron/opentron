//! ABI related utilities

use ethabi::param_type::{ParamType, Reader};
use ethabi::token::{LenientTokenizer, StrictTokenizer, Token, Tokenizer};
use ethabi::{decode, encode};
use hex::{FromHex, ToHex};
use keys::Address;
use proto::core::SmartContract_ABI_Entry as AbiEntry;
use proto::core::SmartContract_ABI_Entry_StateMutabilityType as StateMutabilityType;
use std::fmt::Write as FmtWrite;

use crate::error::Error;
use crate::utils::crypto;

#[inline]
/// Hash code of a contract method.
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

pub fn decode_params(types: &[&str], data: &str) -> Result<String, Error> {
    let types: Vec<ParamType> = types.iter().map(|s| Reader::read(s)).collect::<Result<_, _>>()?;
    let data: Vec<u8> = Vec::from_hex(data)?;
    let tokens = decode(&types, &data)?;

    assert_eq!(types.len(), tokens.len());

    let result = types
        .iter()
        .zip(tokens.iter())
        .map(|(ty, tok)| format!("{}: {}", ty, pformat_abi_token(tok)))
        .collect::<Vec<String>>()
        .join("\n");
    Ok(result)
}

fn parse_tokens(params: &[(ParamType, &str)], lenient: bool) -> Result<Vec<Token>, Error> {
    params
        .iter()
        .map(|&(ref param, value)| match lenient {
            true => LenientTokenizer::tokenize(param, value),
            false => StrictTokenizer::tokenize(param, value),
        })
        .collect::<Result<_, _>>()
        .map_err(From::from)
}

fn pformat_abi_token(tok: &Token) -> String {
    match tok {
        Token::Address(raw) => Address::from_tvm_bytes(raw.as_ref()).to_string(),
        Token::String(s) => format!("{:?}", s),
        Token::Uint(val) => val.to_string(),
        Token::Bool(val) => val.to_string(),
        Token::Array(val) => format!("[{}]", val.iter().map(pformat_abi_token).collect::<Vec<_>>().join(", ")),
        Token::Bytes(val) => val.encode_hex::<String>(),
        ref t => format!("{:?}", t),
    }
}

pub fn entry_to_method_name(entry: &AbiEntry) -> String {
    format!(
        "{}({})",
        entry.get_name(),
        entry
            .get_inputs()
            .iter()
            .map(|arg| arg.get_field_type().to_owned())
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub fn entry_to_method_name_pretty(entry: &AbiEntry) -> Result<String, Error> {
    let mut pretty = match entry.get_field_type() {
        ::proto::core::SmartContract_ABI_Entry_EntryType::Function => "function".to_owned(),
        ::proto::core::SmartContract_ABI_Entry_EntryType::Event => "event".to_owned(),
        ::proto::core::SmartContract_ABI_Entry_EntryType::Constructor => "constructor".to_owned(),
        ::proto::core::SmartContract_ABI_Entry_EntryType::Fallback => {
            if entry.get_payable() {
                "function".to_owned()
            } else {
                "".to_owned()
            }
        }
        _ => "".to_owned(),
    };
    write!(pretty, " {:}", entry.get_name())?;
    write!(
        pretty,
        "({})",
        entry
            .get_inputs()
            .iter()
            .map(|arg| if arg.get_name().is_empty() {
                arg.get_field_type().to_owned()
            } else {
                format!("{:} {:}", arg.get_field_type(), arg.get_name())
            })
            .collect::<Vec<_>>()
            .join(", ")
    )?;
    if entry.payable {
        write!(pretty, " payable")?;
    }
    if entry.get_stateMutability() == StateMutabilityType::View {
        write!(pretty, " view")?;
    }

    if !entry.get_outputs().is_empty() {
        write!(
            pretty,
            " returns ({})",
            entry
                .get_outputs()
                .iter()
                .map(|arg| arg.get_field_type().to_owned())
                .collect::<Vec<_>>()
                .join(", "),
        )?;
    }
    Ok(pretty)
}

pub fn entry_to_output_types(entry: &AbiEntry) -> Vec<&str> {
    entry
        .get_outputs()
        .iter()
        .map(|arg| arg.get_field_type())
        .collect::<Vec<_>>()
}

pub fn entry_to_input_types(entry: &AbiEntry) -> Vec<&str> {
    entry
        .get_inputs()
        .iter()
        .map(|arg| arg.get_field_type())
        .collect::<Vec<_>>()
}

#[test]
fn test_encode_params() {
    assert_eq!(
        encode_params(&["address"], &["407d73d8a49eeb85d32cf465507dd71d507100c1".to_owned()]).unwrap(),
        vec![0u8]
    );
}
