//! Keys of state db. Due to name conflicts, all types here should be used with mod prefix.
use std::borrow::Cow;
use std::convert::TryFrom;

use byteorder::{ByteOrder, BE};
use bytes::BytesMut;
use keys::Address;
use primitive_types::H256;
use prost::Message;
use proto2::state as pb;

pub use super::parameter::ChainParameter;
pub use super::property::DynamicProperty;

/// Should be used to get database key associated with given value.
pub trait Key<T>: Sized {
    /// The db key associated with this value.
    type Target: AsRef<[u8]>;
    const COL: usize = 0;

    /// Returns db key.
    fn key(&self) -> Self::Target;

    /// Returns db value.
    fn value<'a>(val: &'a T) -> Cow<'a, [u8]>;

    /// Parse db value.
    fn parse_value(raw: &[u8]) -> T;

    /// Parse db key.
    fn parse_key(_raw: &[u8]) -> Self {
        unreachable!("key parsing is not implemented")
    }
}

impl Key<i64> for ChainParameter {
    type Target = Vec<u8>;
    const COL: usize = super::db::COL_DEFAULT;

    fn key(&self) -> Self::Target {
        let mut raw = [b'p'; 9];
        BE::write_u64(&mut raw[1..], *self as u64);
        raw.to_vec()
    }

    fn value(val: &i64) -> Cow<[u8]> {
        Cow::Owned(val.to_be_bytes().to_vec())
    }

    fn parse_value(raw: &[u8]) -> i64 {
        BE::read_u64(raw) as _
    }
}

impl Key<i64> for DynamicProperty {
    type Target = String;
    const COL: usize = super::db::COL_DEFAULT;

    fn key(&self) -> Self::Target {
        format!("k{:?}", self)
    }

    fn value(val: &i64) -> Cow<[u8]> {
        Cow::Owned(val.to_be_bytes().to_vec())
    }

    fn parse_value(raw: &[u8]) -> i64 {
        BE::read_u64(raw) as _
    }
}

#[derive(Debug)]
pub struct WitnessSchedule;

// `<<Address, number_of_votes, brokerage>>`
impl Key<Vec<(Address, i64, u8)>> for WitnessSchedule {
    type Target = &'static str;
    const COL: usize = super::db::COL_DEFAULT;

    // Same as DynamicProperty
    fn key(&self) -> Self::Target {
        "kWitnessSchedule"
    }

    fn value(val: &Vec<(Address, i64, u8)>) -> Cow<[u8]> {
        val.iter()
            .map(|(ref addr, num_votes, brokerage)| {
                [addr.as_bytes(), &num_votes.to_be_bytes()[..], &[*brokerage]].concat()
            })
            .collect::<Vec<_>>()
            .concat()
            .into()
    }

    fn parse_value(raw: &[u8]) -> Vec<(Address, i64, u8)> {
        if raw.len() % (21 + 1 + 8) != 0 {
            panic!("malformed kWitnessSchedule");
        }
        raw.chunks(30)
            .map(|wit| {
                let mut raw_num = [0u8; 8];
                raw_num.copy_from_slice(&wit[21..29]);
                (Address::try_from(&wit[..21]).unwrap(), i64::from_be_bytes(raw_num), wit[2])
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct LatestBlockHash;

impl Key<H256> for LatestBlockHash {
    type Target = &'static str;
    const COL: usize = super::db::COL_DEFAULT;

    // Same as DynamicProperty
    fn key(&self) -> Self::Target {
        "kLatestBlockHash"
    }

    fn value(val: &H256) -> Cow<[u8]> {
        val.as_bytes().into()
    }

    fn parse_value(raw: &[u8]) -> H256 {
        if raw.len() != 32 {
            panic!("malformed kLatestBlockHash");
        }
        H256::from_slice(raw)
    }
}

#[derive(Debug)]
pub struct BlockFilledSlots;

// Value is 0-1 vec.
impl Key<Vec<u8>> for BlockFilledSlots {
    type Target = &'static str;
    const COL: usize = super::db::COL_DEFAULT;

    // Same as DynamicProperty
    fn key(&self) -> Self::Target {
        "ksaveBlockFilledSlots"
    }

    fn value(val: &Vec<u8>) -> Cow<[u8]> {
        (&val[..]).into()
    }

    fn parse_value(raw: &[u8]) -> Vec<u8> {
        raw.to_vec()
    }
}

#[derive(Debug)]
pub struct Witness(pub Address);

impl Key<pb::Witness> for Witness {
    type Target = Vec<u8>;
    const COL: usize = super::db::COL_WITNESS;

    fn key(&self) -> Self::Target {
        self.0.as_bytes().to_vec()
    }

    fn value(val: &pb::Witness) -> Cow<[u8]> {
        let mut buf = BytesMut::with_capacity(val.encoded_len());
        val.encode(&mut buf).unwrap();
        Cow::from(buf.to_vec())
    }

    fn parse_value(raw: &[u8]) -> pb::Witness {
        pb::Witness::decode(raw).unwrap()
    }

    fn parse_key(raw: &[u8]) -> Self {
        Witness(*Address::from_bytes(raw))
    }
}

#[derive(Debug)]
pub struct Account(pub Address);

impl Key<pb::Account> for Account {
    type Target = Vec<u8>;
    const COL: usize = super::db::COL_ACCOUNT;

    fn key(&self) -> Self::Target {
        self.0.as_bytes().to_vec()
    }

    fn value(val: &pb::Account) -> Cow<[u8]> {
        let mut buf = BytesMut::with_capacity(val.encoded_len());
        val.encode(&mut buf).unwrap();
        Cow::from(buf.to_vec())
    }

    fn parse_value(raw: &[u8]) -> pb::Account {
        pb::Account::decode(raw).unwrap()
    }

    fn parse_key(raw: &[u8]) -> Self {
        Account(*Address::from_bytes(raw))
    }
}

#[derive(Debug)]
pub struct AccountIndex(pub String);

impl Key<Address> for AccountIndex {
    type Target = Vec<u8>;
    const COL: usize = super::db::COL_ACCOUNT_INDEX;

    fn key(&self) -> Self::Target {
        self.0.as_bytes().to_owned()
    }

    fn value(val: &Address) -> Cow<[u8]> {
        Cow::from(val.as_bytes())
    }

    fn parse_value(raw: &[u8]) -> Address {
        *Address::from_bytes(raw)
    }
}

/// Resource delegation, from_address, to_address.
#[derive(Debug)]
pub struct ResourceDelegation(pub Address, pub Address);

impl Key<pb::ResourceDelegation> for ResourceDelegation {
    type Target = Vec<u8>;
    const COL: usize = super::db::COL_RESOURCE_DELEGATION;

    fn key(&self) -> Self::Target {
        [self.0.as_bytes(), self.1.as_bytes()].concat()
    }

    fn value(val: &pb::ResourceDelegation) -> Cow<[u8]> {
        let mut buf = BytesMut::with_capacity(val.encoded_len());
        val.encode(&mut buf).unwrap();
        Cow::from(buf.to_vec())
    }

    fn parse_value(raw: &[u8]) -> pb::ResourceDelegation {
        pb::ResourceDelegation::decode(raw).unwrap()
    }
}

/// Reverse index for resource delegation info, to_address.
#[derive(Debug)]
pub struct ResourceDelegationIndex(pub Address);

impl Key<Vec<Address>> for ResourceDelegationIndex {
    type Target = Vec<u8>;
    const COL: usize = super::db::COL_RESOURCE_DELEGATION_INDEX;

    fn key(&self) -> Self::Target {
        self.0.as_bytes().to_vec()
    }

    fn value(val: &Vec<Address>) -> Cow<[u8]> {
        val.iter()
            .map(|addr| addr.as_bytes())
            .collect::<Vec<_>>()
            .concat()
            .into()
    }

    fn parse_value(raw: &[u8]) -> Vec<Address> {
        if raw.len() % 21 != 0 {
            panic!("malformed ResourceDelegationIndex db")
        }
        raw.chunks(21)
            .map(Address::try_from)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }
}

// TODO: Votes should be stored with epoch round as prefix, which simplifies maintenance round.
/// `Votes: <<Address>> => [Vote]`
#[derive(Debug)]
pub struct Votes(pub Address);

impl Key<pb::Votes> for Votes {
    type Target = Vec<u8>;
    const COL: usize = super::db::COL_VOTES;

    fn key(&self) -> Self::Target {
        self.0.as_bytes().to_vec()
    }

    fn value(val: &pb::Votes) -> Cow<[u8]> {
        let mut buf = BytesMut::with_capacity(val.encoded_len());
        val.encode(&mut buf).unwrap();
        Cow::from(buf.to_vec())
    }

    fn parse_value(raw: &[u8]) -> pb::Votes {
        pb::Votes::decode(raw).unwrap()
    }

    fn parse_key(raw: &[u8]) -> Self {
        Votes(*Address::from_bytes(raw))
    }
}

#[derive(Debug)]
pub struct Contract(Address);

impl Key<pb::SmartContract> for Contract {
    type Target = Vec<u8>;
    const COL: usize = super::db::COL_CONTRACT;

    fn key(&self) -> Self::Target {
        self.0.as_bytes().to_vec()
    }

    fn value(val: &pb::SmartContract) -> Cow<[u8]> {
        let mut buf = BytesMut::with_capacity(val.encoded_len());
        val.encode(&mut buf).unwrap();
        Cow::from(buf.to_vec())
    }

    fn parse_value(raw: &[u8]) -> pb::SmartContract {
        pb::SmartContract::decode(raw).unwrap()
    }
}

#[derive(Debug)]
pub struct ContractCode(Address);

impl Key<Vec<u8>> for ContractCode {
    type Target = Vec<u8>;
    const COL: usize = super::db::COL_CONTRACT_CODE;

    fn key(&self) -> Self::Target {
        self.0.as_bytes().to_vec()
    }

    fn value(val: &Vec<u8>) -> Cow<[u8]> {
        Cow::Borrowed(val)
    }

    fn parse_value(raw: &[u8]) -> Vec<u8> {
        raw.to_vec()
    }
}

#[derive(Debug)]
pub struct ContractStorage(Address, H256);

impl Key<H256> for ContractStorage {
    type Target = Vec<u8>;
    const COL: usize = super::db::COL_CONTRACT_STORAGE;

    fn key(&self) -> Self::Target {
        [self.0.as_bytes(), self.1.as_bytes()].concat()
    }

    fn value(val: &H256) -> Cow<[u8]> {
        Cow::Borrowed(val.as_bytes())
    }

    fn parse_value(raw: &[u8]) -> H256 {
        H256::from_slice(raw)
    }
}

#[derive(Debug)]
pub struct Proposal(pub i64);

impl Key<pb::Proposal> for Proposal {
    type Target = Vec<u8>;
    const COL: usize = super::db::COL_PROPOSAL;

    fn key(&self) -> Self::Target {
        (self.0 as u64).to_be_bytes().to_vec()
    }

    fn value(val: &pb::Proposal) -> Cow<[u8]> {
        let mut buf = BytesMut::with_capacity(val.encoded_len());
        val.encode(&mut buf).unwrap();
        Cow::from(buf.to_vec())
    }

    fn parse_value(raw: &[u8]) -> pb::Proposal {
        pb::Proposal::decode(raw).unwrap()
    }
}

/// TRC10.
#[derive(Debug)]
pub struct Asset(pub i64);

impl Key<pb::Asset> for Asset {
    type Target = Vec<u8>;
    const COL: usize = super::db::COL_ASSET;

    fn key(&self) -> Self::Target {
        (self.0 as u64).to_be_bytes().to_vec()
    }

    fn value(val: &pb::Asset) -> Cow<[u8]> {
        let mut buf = BytesMut::with_capacity(val.encoded_len());
        val.encode(&mut buf).unwrap();
        Cow::from(buf.to_vec())
    }

    fn parse_value(raw: &[u8]) -> pb::Asset {
        pb::Asset::decode(raw).unwrap()
    }

    fn parse_key(raw: &[u8]) -> Self {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(raw);
        Asset(u64::from_be_bytes(bytes) as i64)
    }
}

#[derive(Debug)]
pub struct TransactionReceipt(pub H256);

impl Key<pb::TransactionReceipt> for TransactionReceipt {
    type Target = Vec<u8>;
    const COL: usize = super::db::COL_TRANSACTION_RECEIPT;

    fn key(&self) -> Self::Target {
        self.0.as_bytes().to_vec()
    }

    fn value(val: &pb::TransactionReceipt) -> Cow<[u8]> {
        let mut buf = BytesMut::with_capacity(val.encoded_len());
        val.encode(&mut buf).unwrap();
        Cow::from(buf.to_vec())
    }

    fn parse_value(raw: &[u8]) -> pb::TransactionReceipt {
        pb::TransactionReceipt::decode(raw).unwrap()
    }
}

#[derive(Debug)]
pub struct InternalTransaction(H256);

impl Key<pb::InternalTransaction> for InternalTransaction {
    type Target = Vec<u8>;
    const COL: usize = super::db::COL_INTERNAL_TRANSACTION;

    fn key(&self) -> Self::Target {
        self.0.as_bytes().to_vec()
    }

    fn value(val: &pb::InternalTransaction) -> Cow<[u8]> {
        let mut buf = BytesMut::with_capacity(val.encoded_len());
        val.encode(&mut buf).unwrap();
        Cow::from(buf.to_vec())
    }

    fn parse_value(raw: &[u8]) -> pb::InternalTransaction {
        pb::InternalTransaction::decode(raw).unwrap()
    }
}

#[derive(Debug)]
pub struct TransactionLog(Address, H256);

impl Key<pb::TransactionLog> for TransactionLog {
    type Target = Vec<u8>;
    const COL: usize = super::db::COL_TRANSACTION_LOG;

    fn key(&self) -> Self::Target {
        self.0.as_bytes().to_vec()
    }

    fn value(val: &pb::TransactionLog) -> Cow<[u8]> {
        let mut buf = BytesMut::with_capacity(val.encoded_len());
        val.encode(&mut buf).unwrap();
        Cow::from(buf.to_vec())
    }

    fn parse_value(raw: &[u8]) -> pb::TransactionLog {
        pb::TransactionLog::decode(raw).unwrap()
    }
}
