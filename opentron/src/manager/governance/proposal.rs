//! Proposal controller and validator.

use ::keys::Address;
use constants::block_version::BlockVersion;
use log::{debug, info};
use proto2::state::{proposal::State as ProposalState, Proposal};
use state::keys;
use state::keys::ChainParameter;

use super::super::controllers::ForkController;
use super::super::Manager;

/// Proposal controller to handle proposals during maintenance.
pub struct ProposalController<'m> {
    manager: &'m mut Manager,
}

impl ProposalController<'_> {
    pub fn new<'a>(manager: &'a mut Manager) -> ProposalController<'a> {
        ProposalController { manager }
    }

    pub fn process_proposals(&mut self) -> Result<(), String> {
        let latest_proposal_id = self.manager.state_db.must_get(&keys::DynamicProperty::LatestProposalId);
        if latest_proposal_id == 0 {
            debug!("no proposal yet");
            return Ok(());
        }

        // NOTE: proposals are handled in reverse order
        for proposal_id in (1..=latest_proposal_id).rev() {
            let proposal = self.manager.state_db.must_get(&keys::Proposal(proposal_id));

            if proposal.is_processed() {
                debug!("proposal #{} is processed", proposal_id);
                // NOTE: proposal number less than or equal to this is already processed.
                return Ok(());
            }

            if proposal.is_cancelled() {
                debug!("proposal #{} is cancelled", proposal_id);
                continue;
            }

            let current_maintenance_time = self
                .manager
                .state_db
                .must_get(&keys::DynamicProperty::NextMaintenanceTime);
            if proposal.expiration_time <= current_maintenance_time {
                info!(
                    "proposal #{} expired, counting votes of active witnesses...",
                    proposal_id
                );
                self.process_proposal(proposal)?;
                continue;
            }

            debug!("proposal #{} is active", proposal_id);
        }
        Ok(())
    }

    fn process_proposal(&mut self, mut proposal: Proposal) -> Result<(), String> {
        let active_witnesses = self.manager.get_active_witnesses();
        if active_witnesses.len() != constants::MAX_NUM_OF_ACTIVE_WITNESSES {
            info!("current number of active witnesses: {}", active_witnesses.len());
        }
        let approval_count = proposal
            .approver_addresses
            .iter()
            .filter(|addr| active_witnesses.contains(Address::from_bytes(addr)))
            .count();

        // 70% approvals
        if approval_count >= active_witnesses.len() * constants::SOLID_THRESHOLD_PERCENT / 100 {
            info!(
                "proposal #{} passed, parameters: {:?}",
                proposal.proposal_id, proposal.parameters
            );
            // set dynamic parameters
            for (&param, &value) in proposal.parameters.iter() {
                self.manager
                    .state_db
                    .put_key(keys::ChainParameter::from_i32(param as i32).unwrap(), value)
                    .map_err(|_| "db insert error")?;
            }
            proposal.state = ProposalState::Approved as i32;
            self.manager
                .state_db
                .put_key(keys::Proposal(proposal.proposal_id), proposal)
                .map_err(|_| "db insert error")?;
        } else {
            // disapprove
            info!(
                "proposasl #{} did not reach enough approval, disapproved",
                proposal.proposal_id
            );
            proposal.state = ProposalState::Disapproved as i32;
            self.manager
                .state_db
                .put_key(keys::Proposal(proposal.proposal_id), proposal)
                .map_err(|_| "db insert error")?;
        }
        Ok(())
    }
}

/// Proposal validator.
pub struct ProposalUtil<'m> {
    manager: &'m Manager,
}

impl ProposalUtil<'_> {
    pub fn new<'a>(manager: &'a Manager) -> ProposalUtil<'a> {
        ProposalUtil { manager }
    }

    pub fn validate(self, key: i64, value: i64) -> Result<(), String> {
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
            // NOTE: In nile's branch, this is wrongly marked as 4.0.
            //
            // See-also: https://github.com/tronprotocol/java-tron/pull/3372
            #[cfg(feature = "nile")]
            AllowShieldedTransaction => {
                self.require_version(BlockVersion::Odyssey3_7)?;
                self.accept_true(value)?;
                self.require_proposal(ChainParameter::AllowSameTokenName)
            }
            #[allow(unreachable_patterns)]
            _ => Err("unknown proposal parameter".into()),
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
