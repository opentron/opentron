use std::convert::TryFrom;

use ::keys::Address;
use constants::block_version::BlockVersion;
use log::info;
use proto2::chain::transaction::Result as TransactionResult;
use proto2::common::AccountType;
use proto2::contract as contract_pb;
use proto2::state::{Account, Proposal};
use state::keys;
use state::keys::ChainParameter;

use super::super::controllers::ForkController;
use super::super::executor::TransactionContext;
use super::super::Manager;
use super::BuiltinContractExecutorExt;

impl BuiltinContractExecutorExt for contract_pb::ProposalCreateContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        /* NOTE: witness implies account
        let maybe_acct = manager
            .state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "db query error")?;
        if maybe_acct.is_none() {
            return Err("account not exists".into());
        }
        */

        let maybe_wit = manager
            .state_db
            .get(&keys::Witness(owner_address))
            .map_err(|_| "db query error")?;
        if maybe_wit.is_none() {
            return Err("account is not a witness".into());
        }

        if self.parameters.is_empty() {
            return Err("empty parameter".into());
        }

        // TODO: validate parameter entry
        for (&key, &value) in self.parameters.iter() {
            ProposalUtil::new(manager).validate(key, value)?
        }
        info!("validated .. ");

        Ok(())
    }

    // TODO: for now, use String as Error type
    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_address = Address::try_from(&self.owner_address).unwrap();

        let proposal_id = manager.state_db.must_get(&keys::DynamicProperty::NextProposalId);
        let now = manager.latest_block_timestamp();
        let expiration_time = {
            let maintenance_interval = manager.state_db.must_get(&keys::ChainParameter::MaintenanceInterval);
            let current_maintenance_ts = manager.state_db.must_get(&keys::DynamicProperty::NextMaintenanceTime);
            let offset_now = now + manager.config.chain.proposal_expiration_duration;
            let round = (offset_now - current_maintenance_ts) / maintenance_interval;
            current_maintenance_ts + (round + 1) * maintenance_interval
        };

        let proposal = Proposal {
            proposal_id,
            proposer_address: owner_address.as_bytes().to_vec(),
            parameters: self.parameters.clone(),
            creation_time: now,
            expiration_time,
            ..Default::default()
        };

        info!("now => {} future => {}", now, expiration_time);
        info!("created => {:?}", proposal);

        manager
            .state_db
            .put_key(keys::Proposal(proposal_id), proposal)
            .map_err(|_| "db insert error")?;
        manager
            .state_db
            .put_key(keys::DynamicProperty::NextExchangeId, proposal_id + 1)
            .map_err(|_| "db insert error")?;

        Ok(TransactionResult::default())
    }
}

pub struct ProposalUtil<'m> {
    manager: &'m Manager,
}

impl ProposalUtil<'_> {
    pub fn new<'a>(manager: &'a Manager) -> ProposalUtil<'a> {
        ProposalUtil { manager }
    }

    fn validate(self, key: i64, value: i64) -> Result<(), String> {
        use ChainParameter::*;

        const MAX_LONG_VALUE: i64 = 100_000_000_000_000_000;

        let proposed_param = ChainParameter::from_i32(key as i32).ok_or("invalid proposal parameter")?;

        match proposed_param {
            MaintenanceInterval => {
                if value < 3 * 27 * 1_000 || value > 24 * 3600 * 1_000 {
                    return Err("invalid chain parameter, valid range is [3 * 27 * 1000, 24 * 3600 * 1000]".into());
                }
                Ok(())
            }
            WitnessCreateFee |
            AccountCreateFee |
            WitnessPayPerBlock |
            StandbyWitnessPayPerBlock |
            CreateNewAccountFeeInSystemContract |
            CreateNewAccountBandwidthRate => {
                if value < 0 || value > MAX_LONG_VALUE {
                    return Err("invalid chain parameter, valid range is [0, 100_000_000_000_000_000]".into());
                }
                Ok(())
            }
            RemovePowerOfGr => {
                if self.manager.state_db.must_get(&ChainParameter::RemovePowerOfGr) == -1 {
                    return Err("power of gr is already removed".into());
                }
                if value != 1 {
                    return Err("invalid chain parameter, the only valid value is 1".into());
                }
                Ok(())
            }
            EnergyFee | ExchangeCreateFee => Ok(()),
            MaxCpuTimeOfOneTxn => {
                if value < 10 || value > 100 {
                    return Err("invalid chain parameter, valid range is [10, 100]".into());
                }
                Ok(())
            }
            AllowTvm | AllowUpdateAccountName | AllowSameTokenName | AllowDelegateResource => {
                if value != 1 {
                    return Err("invalid chain parameter, the only valid value is 1".into());
                }
                Ok(())
            }
            TotalEnergyLimit => {
                // i.e. the ENERGY_LIMIT fork
                if !ForkController::new(self.manager).pass_version(BlockVersion::Odyssey3_2)? {
                    return Err("proposal is unavaliable for current chain version".into());
                }
                if ForkController::new(self.manager).pass_version(BlockVersion::Odyssey3_2_2)? {
                    return Err("proposal is disabled since 3.2.2".into());
                }
                if value < 0 || value > MAX_LONG_VALUE {
                    return Err("invalid chain parameter, valid range is [0, 100_000_000_000_000_000]".into());
                }
                Ok(())
            }
            AllowTvmTransferTrc10Upgrade => {
                if value != 1 {
                    return Err("invalid chain parameter, the only valid value is 1".into());
                }
                if self
                    .manager
                    .state_db
                    .must_get(&keys::ChainParameter::AllowSameTokenName) ==
                    0
                {
                    return Err("ALLOW_SAME_TOKEN_NAME is required before this proposal".into());
                }
                Ok(())
            }

            _ => unimplemented!("unhandled proposal parameter"),
        }
    }
}
