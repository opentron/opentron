//! Helpers for transaction.

use chrono::Utc;
use clap::ArgMatches;
use hex::ToHex;
use keys::{Address, Private};
use proto::api::NumberMessage;
use proto::api_grpc::Wallet;
use proto::core::{
    AccountCreateContract, AccountPermissionUpdateContract, AccountUpdateContract, AssetIssueContract,
    ClearABIContract, CreateSmartContract, ExchangeCreateContract, ExchangeInjectContract, ExchangeTransactionContract,
    ExchangeWithdrawContract, FreezeBalanceContract, ParticipateAssetIssueContract, ProposalApproveContract,
    ProposalCreateContract, ProposalDeleteContract, SetAccountIdContract, ShieldedTransferContract,
    TransferAssetContract, TransferContract, TriggerSmartContract, UnfreezeAssetContract, UnfreezeBalanceContract,
    UpdateAssetContract, UpdateBrokerageContract, UpdateEnergyLimitContract, UpdateSettingContract, VoteAssetContract,
    VoteWitnessContract, WithdrawBalanceContract, WitnessCreateContract, WitnessUpdateContract,
};
use proto::core::{
    Transaction, Transaction_Contract as Contract, Transaction_Contract_ContractType as ContractType,
    Transaction_raw as TransactionRaw,
};
use protobuf::well_known_types::Any;
use protobuf::{parse_from_bytes, Message};
use serde_json::json;
use std::convert::TryFrom;

use crate::commands::wallet::sign_digest;
use crate::error::Error;
use crate::utils::client;
use crate::utils::crypto;
use crate::utils::jsont;

pub fn timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn extract_owner_address_from_parameter(any: &Any) -> Result<Address, Error> {
    match any.get_type_url() {
        "type.googleapis.com/protocol.TransferContract" => Ok(Address::try_from(
            parse_from_bytes::<TransferContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.ShieldedTransferContract" => {
            let pb = parse_from_bytes::<ShieldedTransferContract>(any.get_value())?;
            if !pb.get_transparent_from_address().is_empty() {
                Ok(Address::try_from(pb.get_transparent_from_address())?)
            } else {
                Err(Error::Runtime(
                    "can not extract sender address from ShieldedTransferContract. Use -k/-K instead",
                ))
            }
        }
        "type.googleapis.com/protocol.CreateSmartContract" => Ok(Address::try_from(
            parse_from_bytes::<CreateSmartContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.TriggerSmartContract" => Ok(Address::try_from(
            parse_from_bytes::<TriggerSmartContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.FreezeBalanceContract" => Ok(Address::try_from(
            parse_from_bytes::<FreezeBalanceContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.UnfreezeBalanceContract" => Ok(Address::try_from(
            parse_from_bytes::<UnfreezeBalanceContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.AccountPermissionUpdateContract" => Ok(Address::try_from(
            parse_from_bytes::<AccountPermissionUpdateContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.VoteWitnessContract" => Ok(Address::try_from(
            parse_from_bytes::<VoteWitnessContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.AccountUpdateContract" => Ok(Address::try_from(
            parse_from_bytes::<AccountUpdateContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.WitnessCreateContract" => Ok(Address::try_from(
            parse_from_bytes::<WitnessCreateContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.WitnessUpdateContract" => Ok(Address::try_from(
            parse_from_bytes::<WitnessUpdateContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.WithdrawBalanceContract" => Ok(Address::try_from(
            parse_from_bytes::<WithdrawBalanceContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.ProposalCreateContract" => Ok(Address::try_from(
            parse_from_bytes::<ProposalCreateContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.ProposalApproveContract" => Ok(Address::try_from(
            parse_from_bytes::<ProposalApproveContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.ProposalDeleteContract" => Ok(Address::try_from(
            parse_from_bytes::<ProposalDeleteContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.AssetIssueContract" => Ok(Address::try_from(
            parse_from_bytes::<AssetIssueContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.TransferAssetContract" => Ok(Address::try_from(
            parse_from_bytes::<TransferAssetContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.UpdateSettingContract" => Ok(Address::try_from(
            parse_from_bytes::<UpdateSettingContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.UpdateEnergyLimitContract" => Ok(Address::try_from(
            parse_from_bytes::<UpdateEnergyLimitContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.ClearABIContract" => Ok(Address::try_from(
            parse_from_bytes::<ClearABIContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.UpdateAssetContract" => Ok(Address::try_from(
            parse_from_bytes::<UpdateAssetContract>(any.get_value())?.get_owner_address(),
        )?),
        "type.googleapis.com/protocol.ParticipateAssetIssueContract" => Ok(Address::try_from(
            parse_from_bytes::<ParticipateAssetIssueContract>(any.get_value())?.get_owner_address(),
        )?),
        _ => unimplemented!(),
    }
}

/// Parse command line amount to amount in pb.
#[inline]
pub fn parse_amount_without_surfix(amount: &str) -> Result<i64, Error> {
    if amount.is_empty() {
        return Err(Error::Runtime("can not parse empty amount"));
    }
    Ok(amount.replace("_", "").parse()?)
}

/// Parse command line amount to amount in pb.
pub fn parse_amount(amount: &str, allow_unit: bool) -> Result<i64, Error> {
    // NOTE: simple parse, buggy but works
    if amount.is_empty() {
        return Err(Error::Runtime("can not parse empty amount"));
    }
    if allow_unit {
        let length = amount.as_bytes().len();
        // FIXME: allow other unit
        if amount.ends_with("TRX") || amount.ends_with("TRZ") {
            String::from_utf8_lossy(&amount.as_bytes()[..length - 3])
                .replace("_", "")
                .parse::<i64>()
                .map(|v| v * 1_000_000)
                .map_err(Error::from)
        } else if amount.ends_with("SUN") {
            Ok(String::from_utf8_lossy(&amount.as_bytes()[..length - 3])
                .replace("_", "")
                .parse()?)
        } else {
            Ok(amount.replace("_", "").parse()?)
        }
    } else {
        Ok(amount.replace("_", "").parse()?)
    }
}

pub struct TransactionHandler<'a, C> {
    contract: C,
    arg_matches: &'a ArgMatches<'a>,
    raw_trx_fn: Option<Box<dyn FnMut(&mut TransactionRaw) -> () + 'static>>,
}

impl<'a, C: ContractPbExt> TransactionHandler<'a, C> {
    pub fn handle(contract: C, matches: &'a ArgMatches<'a>) -> Self {
        TransactionHandler {
            contract,
            arg_matches: matches,
            raw_trx_fn: None,
        }
    }

    pub fn map_raw_transaction<F>(&mut self, f: F) -> &mut Self
    where
        F: FnMut(&mut TransactionRaw) -> () + 'static,
    {
        self.raw_trx_fn = Some(Box::new(f));
        self
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let matches = self.arg_matches;

        // packing contract
        let any = self.contract.as_google_any()?;

        let mut contract = Contract::new();
        contract.set_field_type(self.contract.contract_type());
        contract.set_parameter(any);
        if let Some(val) = matches.value_of("permission-id") {
            contract.set_Permission_id(val.parse()?);
        }

        let mut raw = TransactionRaw::new();
        raw.set_contract(vec![contract].into());
        if let Some(f) = self.raw_trx_fn.as_mut() {
            f(&mut raw);
        }

        let expiration = matches.value_of("expiration").unwrap_or("60").parse::<i64>()?;
        raw.set_expiration(timestamp_millis() + 1000 * expiration);

        let grpc_client = client::new_grpc_client()?;

        // fill ref_block info
        let ref_block = match matches.value_of("ref-block") {
            Some(num) => {
                let mut req = NumberMessage::new();
                req.set_num(num.parse()?);
                let (_, block, _) = grpc_client.get_block_by_num2(Default::default(), req).wait()?;
                block
            }
            None => {
                let (_, block, _) = grpc_client
                    .get_now_block2(Default::default(), Default::default())
                    .wait()?;
                block
            }
        };
        let ref_block_number = ref_block.get_block_header().get_raw_data().number;
        raw.set_ref_block_bytes(vec![
            ((ref_block_number & 0xff00) >> 8) as u8,
            (ref_block_number & 0xff) as u8,
        ]);
        raw.set_ref_block_hash(ref_block.blockid[8..16].to_owned());

        raw.set_timestamp(timestamp_millis());

        // signature
        let txid = crypto::sha256(&raw.write_to_bytes()?);
        let mut signatures: Vec<Vec<u8>> = Vec::new();
        if !matches.is_present("skip-sign") {
            let signature = if let Some(raw_addr) = matches.value_of("account") {
                let addr = raw_addr.parse::<Address>()?;
                eprintln!("! Signing using wallet key from --account {:}", addr);
                sign_digest(&txid, &addr)?
            } else if let Some(raw_key) = matches.value_of("private-key") {
                eprintln!("! Signing using raw private key from --private-key");
                let priv_key = raw_key.parse::<Private>()?;
                priv_key.sign_digest(&txid)?[..].to_owned()
            } else {
                let owner_address = extract_owner_address_from_parameter(raw.contract[0].get_parameter())?;
                eprintln!("! Signing using wallet key {:}", owner_address);
                sign_digest(&txid, &owner_address)?
            };
            signatures.push(signature);
        }

        let mut req = Transaction::new();
        req.set_raw_data(raw);
        req.set_signature(signatures.into());

        eprintln!("TX: {:}", txid.encode_hex::<String>());

        // skip-sign implies dont-broadcast
        if matches.is_present("skip-sign") || matches.is_present("dont-broadcast") {
            let mut json = serde_json::to_value(&req)?;
            jsont::fix_transaction(&mut json)?;
            json["raw_data_hex"] = json!(req.get_raw_data().write_to_bytes()?.encode_hex::<String>());
            json["txID"] = json!(txid.encode_hex::<String>());
            println!("{:}", serde_json::to_string_pretty(&json)?);

            Ok(())
        } else {
            let (_, payload, _) = grpc_client.broadcast_transaction(Default::default(), req).wait()?;
            let mut result = serde_json::to_value(&payload)?;
            jsont::fix_api_return(&mut result);
            eprintln!("got => {:}", serde_json::to_string_pretty(&result)?);

            if result["result"].as_bool().unwrap_or(false) {
                Ok(())
            } else {
                Err(Error::Runtime("broadcast transaction failed!"))
            }
        }
    }
}

/// Helper trait for packing contract.
pub trait ContractPbExt: Message {
    fn contract_type(&self) -> ContractType;

    /// Convert Pb to protobuf::well_known_types::Any
    fn as_google_any(&self) -> Result<Any, protobuf::ProtobufError> {
        Ok(Any {
            type_url: format!("type.googleapis.com/protocol.{:?}", self.contract_type()),
            value: self.write_to_bytes()?,
            ..Default::default()
        })
    }
}

macro_rules! impl_contract_pb_ext_for {
    ($contract_ty:ident) => {
        impl ContractPbExt for $contract_ty {
            fn contract_type(&self) -> ContractType {
                ContractType::$contract_ty
            }
        }
    };
}

impl_contract_pb_ext_for!(AccountCreateContract);
impl_contract_pb_ext_for!(TransferContract);
impl_contract_pb_ext_for!(TransferAssetContract);
impl_contract_pb_ext_for!(VoteAssetContract);
impl_contract_pb_ext_for!(VoteWitnessContract);
impl_contract_pb_ext_for!(WitnessCreateContract);
impl_contract_pb_ext_for!(AssetIssueContract);
impl_contract_pb_ext_for!(WitnessUpdateContract);
impl_contract_pb_ext_for!(ParticipateAssetIssueContract);
impl_contract_pb_ext_for!(AccountUpdateContract);
impl_contract_pb_ext_for!(FreezeBalanceContract);
impl_contract_pb_ext_for!(UnfreezeBalanceContract);
impl_contract_pb_ext_for!(WithdrawBalanceContract);
impl_contract_pb_ext_for!(UnfreezeAssetContract);
impl_contract_pb_ext_for!(UpdateAssetContract);
impl_contract_pb_ext_for!(ProposalCreateContract);
impl_contract_pb_ext_for!(ProposalApproveContract);
impl_contract_pb_ext_for!(ProposalDeleteContract);
impl_contract_pb_ext_for!(SetAccountIdContract);
impl_contract_pb_ext_for!(CreateSmartContract);
impl_contract_pb_ext_for!(TriggerSmartContract);
impl_contract_pb_ext_for!(UpdateSettingContract);
impl_contract_pb_ext_for!(ExchangeCreateContract);
impl_contract_pb_ext_for!(ExchangeInjectContract);
impl_contract_pb_ext_for!(ExchangeWithdrawContract);
impl_contract_pb_ext_for!(ExchangeTransactionContract);
impl_contract_pb_ext_for!(UpdateEnergyLimitContract);
impl_contract_pb_ext_for!(AccountPermissionUpdateContract);
impl_contract_pb_ext_for!(ClearABIContract);
impl_contract_pb_ext_for!(UpdateBrokerageContract);
impl_contract_pb_ext_for!(ShieldedTransferContract);
