//! Transaction actuators.

use std::convert::TryFrom;

use ::keys::Address;
use prost::Message;
use prost_types::Any;
use proto2::chain::{transaction::Result as TransactionResult, ContractType};
use state::keys;

use super::executor::TransactionContext;
use super::Manager;

mod account;
pub mod asset;
mod proposal;
mod resource;
mod smart_contract;
mod transfer;
mod witness;
#[cfg(feature = "nile")]
mod shielded;

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
    fn validate_signature(
        &self,
        permission_id: i32,
        mut recover_addrs: Vec<Address>,
        manager: &Manager,
        ctx: &mut TransactionContext,
    ) -> Result<(), String> {
        let len_of_recover_addrs = recover_addrs.len();
        recover_addrs.sort();
        recover_addrs.dedup();
        if recover_addrs.len() != len_of_recover_addrs {
            return Err("duplicate signature".into());
        }

        let owner_address = Address::try_from(self.owner_address()).map_err(|_| "invalid owner_address")?;

        let allow_multisig = manager.state_db.must_get(&keys::ChainParameter::AllowMultisig) != 0;
        if allow_multisig {
            if recover_addrs.len() > 1 {
                ctx.multisig_fee = manager.state_db.must_get(&keys::ChainParameter::MultisigFee);
            }

            let maybe_acct = manager
                .state_db
                .get(&keys::Account(owner_address))
                .map_err(|_| "db query error")?;
            if maybe_acct.is_none() {
                return Err("owner account not exists".into());
            }
            let acct = maybe_acct.unwrap();

            if permission_id == 0 {
                if let Some(owner_perm) = acct.owner_permission.as_ref() {
                    let mut total_weight = 0;
                    for rec_addr in recover_addrs {
                        if let Some(key) = owner_perm.keys.iter().find(|key| key.address == rec_addr.as_bytes()) {
                            total_weight += key.weight;
                        } else {
                            return Err(format!("signature address {} is not in permission keys", rec_addr));
                        }
                    }

                    if total_weight >= owner_perm.threshold {
                        return Ok(());
                    } else {
                        return Err("insufficient weight".into());
                    }
                }
            }

            // active permissions
            if permission_id >= 2 {
                // active perm id is counted from 2
                if let Some(active_perm) = acct.active_permissions.get(permission_id as usize - 2) {
                    let type_num = self.type_code() as i32 as usize;
                    let mask = (active_perm.operations[type_num / 8] >> (type_num % 8)) & 1;
                    if mask == 0 {
                        return Err(format!("operation bit of {:?} is disabled", self.type_code()));
                    }

                    let mut total_weight = 0;
                    for rec_addr in recover_addrs {
                        if let Some(key) = active_perm.keys.iter().find(|key| key.address == rec_addr.as_bytes()) {
                            total_weight += key.weight;
                        } else {
                            return Err(format!("{} is not in permission keys", rec_addr));
                        }
                    }

                    if total_weight >= active_perm.threshold {
                        return Ok(());
                    } else {
                        return Err("insufficient weight".into());
                    }
                }
            }
        }
        // fallback, default owner
        if permission_id == 0 && recover_addrs.len() == 1 && self.owner_address() == recover_addrs[0].as_bytes() {
            return Ok(());
        }
        // fallback, default active
        if permission_id == 2 &&
            self.type_code() != ContractType::AccountPermissionUpdateContract &&
            recover_addrs.len() == 1 &&
            self.owner_address() == recover_addrs[0].as_bytes()
        {
            return Ok(());
        }
        Err("invalid signature".into())
    }

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

#[cfg(feature="nile")]
impl BuiltinContractExt for ::proto2::contract::ShieldedTransferContract {
    fn owner_address(&self) -> &[u8] {
        &[]
    }
    fn type_code(&self) -> ContractType {
        ContractType::ShieldedTransferContract
    }
}
