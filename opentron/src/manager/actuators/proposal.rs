use std::convert::TryFrom;

use ::keys::Address;
use constants::block_version::BlockVersion;
use log::info;
use proto2::chain::transaction::Result as TransactionResult;
use proto2::contract as contract_pb;
use proto2::state::{proposal::State as ProposalState, Proposal};
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

        for (&key, &value) in self.parameters.iter() {
            ProposalUtil::new(manager).validate(key, value)?
        }
        info!("validated .. ");

        Ok(())
    }

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
            .put_key(keys::DynamicProperty::NextProposalId, proposal_id + 1)
            .map_err(|_| "db insert error")?;

        Ok(TransactionResult::default())
    }
}

impl BuiltinContractExecutorExt for contract_pb::ProposalApproveContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        // NOTE: witness implies account, so no need to check account
        let maybe_wit = manager
            .state_db
            .get(&keys::Witness(owner_address))
            .map_err(|_| "db query error")?;
        if maybe_wit.is_none() {
            return Err("account is not a witness".into());
        }

        let next_proposal_id = manager.state_db.must_get(&keys::DynamicProperty::NextProposalId);
        if self.proposal_id >= next_proposal_id {
            return Err("proposal does not exist".into());
        }

        let maybe_proposal = manager
            .state_db
            .get(&keys::Proposal(self.proposal_id))
            .map_err(|_| "db query error")?;
        if let Some(proposal) = maybe_proposal {
            if manager.latest_block_timestamp() >= proposal.expiration_time {
                return Err("proposal has expired".into());
            }
            if proposal.state == ProposalState::Cancelled as i32 {
                return Err("proposal is cancelled".into());
            }
            if !self.is_approval && !proposal.approver_addresses.contains(&self.owner_address.to_vec()) {
                return Err("cannot disapprove without former approval".into());
            }
        } else {
            return Err("proposal does not exist".into());
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_address = Address::try_from(&self.owner_address).unwrap();

        let mut proposal = manager.state_db.must_get(&keys::Proposal(self.proposal_id));

        if self.is_approval {
            proposal.approver_addresses.push(owner_address.as_bytes().to_vec());
        } else {
            proposal.approver_addresses = proposal
                .approver_addresses
                .into_iter()
                .filter(|addr| &addr[..] != owner_address.as_bytes())
                .collect();
        }
        manager
            .state_db
            .put_key(keys::Proposal(self.proposal_id), proposal)
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

        let proposed_param = ChainParameter::from_i32(key as i32).ok_or("invalid proposal parameter")?;

        match proposed_param {
            MaintenanceInterval => self.accept_range_value(value, 3 * 27 * 1_000, 24 * 3600 * 1_000),
            WitnessCreateFee |
            AccountCreateFee |
            BandwidthFee |
            AssetIssueFee |
            WitnessPayPerBlock |
            StandbyWitnessAllowance |
            CreateNewAccountFeeInSystemContract |
            CreateNewAccountBandwidthRate => self.accept_long_value(value),
            RemovePowerOfGr => {
                if self.manager.state_db.must_get(&ChainParameter::RemovePowerOfGr) == -1 {
                    return Err("power of gr is already removed".into());
                }
                self.accept_true(value)
            }
            EnergyFee | ExchangeCreateFee => Ok(()),
            MaxCpuTimeOfOneTxn => self.accept_range_value(value, 10, 100),
            AllowTvm | AllowUpdateAccountName | AllowSameTokenName | AllowDelegateResource => self.accept_true(value),
            TotalEnergyLimit => {
                // i.e. the ENERGY_LIMIT fork
                self.require_version(BlockVersion::Odyssey3_2)?;
                if ForkController::new(self.manager).pass_version(BlockVersion::Odyssey3_2_2)? {
                    return Err("proposal is disabled since 3.2.2".into());
                }
                self.accept_long_value(value)
            }
            AllowTvmTransferTrc10Upgrade => {
                self.accept_true(value)?;
                self.require_proposal(ChainParameter::AllowSameTokenName)
            }
            TotalEnergyCurrentLimit => {
                self.require_version(BlockVersion::Odyssey3_2_2)?;
                self.accept_long_value(value)
            }
            AllowMultisig | AllowAdaptiveEnergy => {
                self.require_version(BlockVersion::Odyssey3_5)?;
                self.accept_true(value)
            }
            AccountPermissionUpdateFee | MultisigFee => {
                self.require_version(BlockVersion::Odyssey3_5)?;
                self.accept_range_value(value, 0, 100_000_000_000)
            }
            AllowProtoFilterNum | AllowAccountStateRoot => {
                self.require_version(BlockVersion::Odyssey3_6_0)?;
                self.accept_bool(value)
            }
            AllowTvmConstantinopleUpgrade => {
                self.require_version(BlockVersion::Odyssey3_6_0)?;
                self.accept_true(value)?;
                self.require_proposal(ChainParameter::AllowTvmTransferTrc10Upgrade)
            }
            AllowTvmSolidity059Upgrade => {
                self.require_version(BlockVersion::Odyssey3_6_5)?;
                self.accept_true(value)?;
                self.require_proposal(ChainParameter::AllowTvm)
            }
            AdaptiveResourceLimitTargetRatio => {
                self.require_version(BlockVersion::Odyssey3_6_5)?;
                self.accept_range_value(value, 1, 1_000)
            }
            AdaptiveResourceLimitMultiplier => {
                self.require_version(BlockVersion::Odyssey3_6_5)?;
                self.accept_range_value(value, 1, 10_000)
            }
            AllowChangeDelegation => {
                self.require_version(BlockVersion::Odyssey3_6_5)?;
                self.accept_bool(value)
            }
            StandbyWitnessPayPerBlock => {
                self.require_version(BlockVersion::Odyssey3_6_5)?;
                self.accept_long_value(value)
            }
            ForbidTransferToContract => {
                self.require_version(BlockVersion::Odyssey3_6_6)?;
                self.accept_true(value)?;
                self.require_proposal(ChainParameter::AllowTvm)
            }
            AllowTvmShieldedUpgrade => {
                self.require_version(BlockVersion::GreatVoyage4_0_1)?;
                self.accept_bool(value)
            }
        }
    }

    fn require_version(&self, version: BlockVersion) -> Result<(), String> {
        if !ForkController::new(self.manager).pass_version(version)? {
            return Err("proposal is unavaliable for current chain version".into());
        }
        Ok(())
    }

    fn require_proposal(&self, parameter: ChainParameter) -> Result<(), String> {
        if self.manager.state_db.must_get(&parameter) == 0 {
            return Err(format!("{:?} is required before this proposal", parameter));
        }
        Ok(())
    }

    fn accept_long_value(&self, value: i64) -> Result<(), String> {
        const MAX_LONG_VALUE: i64 = 100_000_000_000_000_000;

        self.accept_range_value(value, 0, MAX_LONG_VALUE)
    }

    fn accept_range_value(&self, value: i64, start: i64, end: i64) -> Result<(), String> {
        if value < start || value > end {
            return Err(format!("invalid chain parameter, valid range is [{}, {}]", start, end));
        }
        Ok(())
    }

    fn accept_true(&self, value: i64) -> Result<(), String> {
        if value != 1 {
            return Err("invalid chain parameter, the only valid value is 1".into());
        }
        Ok(())
    }

    fn accept_bool(&self, value: i64) -> Result<(), String> {
        if value != 0 && value != 1 {
            return Err("invalid chain parameter, valid values are 0 and 1".into());
        }
        Ok(())
    }
}
