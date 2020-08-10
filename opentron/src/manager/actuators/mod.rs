//! Transaction actuators.

use std::convert::TryFrom;

use ::keys::Address;
use prost::Message;
use prost_types::Any;
use proto2::chain::{transaction::Result as TransactionResult, ContractType};
use proto2::contract as contract_pb;
use state::keys;

use super::executor::TransactionContext;
use super::Manager;

mod transfer;

pub trait BuiltinContractExt: Message + Default + Sized {
    fn owner_address(&self) -> &[u8];

    fn type_code(&self) -> ContractType;

    fn from_any(any: &Any) -> Option<Self> {
        Self::decode(&any.value[..]).ok()
    }

    fn to_any(&self) -> Option<Any> {
        let mut buf = Vec::with_capacity(255);
        self.encode(&mut buf).ok()?;
        Some(Any {
            type_url: format!("type.googleapis.com/protocol.{:?}", self.type_code()),
            value: buf,
        })
    }
}

pub trait BuiltinContractExecutorExt: BuiltinContractExt {
    fn validate(&self, _manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        Ok(())
    }

    // TODO: for now, use String as Error type
    fn execute(&self, _manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        unimplemented!()
    }

    /// Extra fee paid for specific type of builtin contract. Like asset issue, account permission update.
    #[inline]
    fn fee(&self) -> i64 {
        0
    }
}


impl BuiltinContractExecutorExt for contract_pb::ProposalCreateContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        let maybe_acct = manager.state_db.get(&keys::Account(owner_address)).map_err(|_| "db query error")?;
        if maybe_acct.is_none() {
            return Err("account not exists".into());
        }

        let maybe_wit = manager.state_db.get(&keys::Witness(owner_address)).map_err(|_| "db query error")?;
        if maybe_wit.is_none() {
            return Err("account is not a witness".into());
        }

        if self.parameters.is_empty() {
            return Err("empty parameter".into());
        }

        // TODO: validate parameter entry


        Ok(())
    }

    // TODO: for now, use String as Error type
    fn execute(&self, _manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        unimplemented!()
    }
}




/// Impl BuiltinContractExt for builtin contract protobufs.
macro_rules! impl_contract_ext_for {
    ($contract_ty:ident) => {
        impl BuiltinContractExt for ::proto2::contract::$contract_ty {
            fn owner_address(&self) -> &[u8] {
                &self.owner_address
            }
            fn type_code(&self) -> ContractType {
                ContractType::$contract_ty
            }
        }
    };
    ($contract_ty:ident, $type_name:expr) => {
        impl BuiltinContractExt for ::proto2::contract::$contract_ty {
            fn owner_address(&self) -> &[u8] {
                &self.owner_address
            }
            fn type_code(&self) -> ContractType {
                ContractType::$contract_ty
            }
            fn to_any(&self) -> Option<Any> {
                let mut buf = Vec::with_capacity(255);
                self.encode(&mut buf).ok()?;
                Some(Any {
                    type_url: format!("type.googleapis.com/protocol.{:?}", $type_name),
                    value: buf,
                })
            }
        }
    };
}

impl_contract_ext_for!(AccountCreateContract);
impl_contract_ext_for!(AccountUpdateContract);
impl_contract_ext_for!(SetAccountIdContract);
impl_contract_ext_for!(AccountPermissionUpdateContract);
impl_contract_ext_for!(TransferContract);
impl_contract_ext_for!(TransferAssetContract);
impl_contract_ext_for!(AssetIssueContract);
impl_contract_ext_for!(ParticipateAssetIssueContract);
// NOTE: VoteAssetContract is not used in java-tron.
// impl_contract_ext_for!(VoteAssetContract);
impl_contract_ext_for!(UpdateAssetContract);
impl_contract_ext_for!(UnfreezeAssetContract);
impl_contract_ext_for!(WitnessCreateContract);
impl_contract_ext_for!(WitnessUpdateContract);
impl_contract_ext_for!(UpdateBrokerageContract);
impl_contract_ext_for!(VoteWitnessContract);
impl_contract_ext_for!(WithdrawBalanceContract);
impl_contract_ext_for!(CreateSmartContract);
impl_contract_ext_for!(TriggerSmartContract);
impl_contract_ext_for!(UpdateSettingContract);
impl_contract_ext_for!(UpdateEnergyLimitContract);
// prost will rename enum variant to CamelCase.
impl_contract_ext_for!(ClearAbiContract, "ClearABIContract");
impl_contract_ext_for!(FreezeBalanceContract);
impl_contract_ext_for!(UnfreezeBalanceContract);
impl_contract_ext_for!(ProposalCreateContract);
impl_contract_ext_for!(ProposalApproveContract);
impl_contract_ext_for!(ProposalDeleteContract);
impl_contract_ext_for!(ExchangeCreateContract);
impl_contract_ext_for!(ExchangeInjectContract);
impl_contract_ext_for!(ExchangeWithdrawContract);
impl_contract_ext_for!(ExchangeTransactionContract);
