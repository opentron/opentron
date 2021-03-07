use crate::chain::ContractType;
use prost::Message;
use prost_types::Any;

pub trait ContractExt: Message + Default + Sized {
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

/// Impl ContractExt for builtin contract protobufs.
macro_rules! impl_contract_ext_for {
    ($contract_ty:ident) => {
        impl ContractExt for $crate::contract::$contract_ty {
            fn owner_address(&self) -> &[u8] {
                &self.owner_address
            }
            fn type_code(&self) -> ContractType {
                ContractType::$contract_ty
            }
        }
    };
    ($contract_ty:ident, $type_name:expr) => {
        impl ContractExt for $crate::contract::$contract_ty {
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

#[cfg(feature = "nile")]
impl ContractExt for ::proto::contract::ShieldedTransferContract {
    fn owner_address(&self) -> &[u8] {
        &[]
    }
    fn type_code(&self) -> ContractType {
        ContractType::ShieldedTransferContract
    }
}
