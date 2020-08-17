//! Transaction actuators.

use prost::Message;
use prost_types::Any;
use proto2::chain::{transaction::Result as TransactionResult, ContractType};

use super::executor::TransactionContext;
use super::Manager;

mod account;
mod asset;
mod proposal;
mod resource;
mod transfer;
mod witness;

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
        unimplemented!("TODO: support builtin contract type {:?}", self.type_code())
    }

    /// Extra fee paid for specific type of builtin contract. Like asset issue, account permission update.
    #[inline]
    fn fee(&self, _manager: &Manager) -> i64 {
        0
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
