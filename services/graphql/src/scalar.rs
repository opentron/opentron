use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use primitive_types::H256;

/// Bytes32 is a 32 byte binary string, represented as 0x-prefixed hexadecimal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bytes32(pub H256);

impl From<H256> for Bytes32 {
    fn from(value: H256) -> Self {
        Bytes32(value)
    }
}

#[Scalar]
impl ScalarType for Bytes32 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(value) => {
                if value.starts_with("0x") {
                    Ok(Bytes32(H256::from_slice(&hex::decode(&value[2..])?)))
                } else {
                    Ok(Bytes32(H256::from_slice(&hex::decode(value)?)))
                }
            }
            Value::Number(value) => value
                .as_u64()
                .map(H256::from_low_u64_be)
                .map(Bytes32)
                .ok_or(InputValueError::custom("invalid number type")),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(hex::encode(&self.0))
    }
}

/// Address is a 20 byte Ethereum address, represented as 0x-prefixed hexadecimal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Address(pub keys::Address);

impl From<keys::Address> for Address {
    fn from(value: keys::Address) -> Address {
        Address(value)
    }
}

impl Default for Address {
    fn default() -> Address {
        Address(keys::Address::default())
    }
}

#[Scalar]
impl ScalarType for Address {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(value) => Ok(value.parse().map(Address)?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}

/// Bytes is an arbitrary length binary string, represented as 0x-prefixed hexadecimal.
/// An empty byte string is represented as '0x'. Byte strings must have an even number of hexadecimal nybbles.
pub struct Bytes(pub Vec<u8>);

#[Scalar]
impl ScalarType for Bytes {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(value) => {
                if value.starts_with("0x") {
                    Ok(hex::decode(&value[2..]).map(Bytes)?)
                } else {
                    Ok(hex::decode(value).map(Bytes)?)
                }
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(hex::encode(&self.0))
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(value: Bytes) -> Vec<u8> {
        value.0
    }
}

// /// BigInt is a large integer. Input is accepted as either a JSON number or as a string.
// /// Strings may be either decimal or 0x-prefixed hexadecimal. Output values are all
// /// 0x-prefixed hexadecimal.
// pub struct BigInt(BigInt);

/// Long is a 64 bit unsigned integer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Long(pub i64);

impl From<i64> for Long {
    fn from(value: i64) -> Self {
        Long(value)
    }
}

#[Scalar]
impl ScalarType for Long {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(value) => Ok(value.parse().map(Long)?),
            Value::Number(value) => value
                .as_i64()
                .map(Long)
                .ok_or(InputValueError::custom("invalid number type")),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}
