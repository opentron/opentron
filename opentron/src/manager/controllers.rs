use ::keys::Address;
use constants::block_version::{BlockVersion, ForkPolicy};
use log::{debug, info};
use proto2::state::{proposal::State as ProposalState, Proposal};
use state::keys;

use super::Manager;

/// Handle block version upgrade.
///
/// NOTE: The implementation is very different from java-tron.
///
/// In java-tron, version fork is handle by `ForkUtils.java` and `ForkController.java`.
/// Each version in saved in DB as a bit vector, each bit represents an SR's block version status(downgrade or upgrade).
/// `ForkController` accepts block and saves version status.
///
/// Here, OpenTron saves an SR's block version status in Witness as `latest_block_version`, updated as recieving blocks.
///
/// FIXME: If there's inconsistent when an SR downgrades it's block version?
pub struct ForkController<'m> {
    manager: &'m Manager,
}

impl ForkController<'_> {
    pub fn new<'a>(manager: &'a Manager) -> ForkController<'a> {
        ForkController { manager }
    }

    pub fn pass_version(&self, version: BlockVersion) -> Result<bool, String> {
        match version.fork_policy() {
            ForkPolicy::AtBlock { block_number } => Ok(self.manager.latest_block_number() >= block_number),
            ForkPolicy::Old => {
                let active_wit_addrs = self.manager.get_active_witnesses();
                let all_passed = active_wit_addrs
                    .into_iter()
                    .map(|addr| self.manager.state_db.must_get(&keys::Witness(addr)))
                    .all(|wit| wit.latest_block_version >= version as _);
                Ok(all_passed)
            }
            ForkPolicy::New {
                timestamp,
                minimum_upgraded,
            } => {
                let maintenance_interval = self
                    .manager
                    .state_db
                    .must_get(&keys::ChainParameter::MaintenanceInterval);
                let hard_fork_ts = ((timestamp - 1) / maintenance_interval + 1) * maintenance_interval;

                if self.manager.latest_block_number() < hard_fork_ts {
                    return Ok(false);
                }

                let active_wit_addrs = self.manager.get_active_witnesses();
                let num_passed = active_wit_addrs
                    .into_iter()
                    .map(|addr| self.manager.state_db.must_get(&keys::Witness(addr)))
                    .map(|wit| wit.latest_block_version >= version as _)
                    .count();
                Ok(num_passed >= minimum_upgraded)
            }
        }
    }
}

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
                info!("proposal #{} expired, counting votes...", proposal_id);
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
