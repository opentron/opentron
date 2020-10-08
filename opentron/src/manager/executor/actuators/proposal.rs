//! Proposal handling for chain governance.
use std::convert::TryFrom;

use ::keys::Address;
use proto2::chain::transaction::Result as TransactionResult;
use proto2::contract as contract_pb;
use proto2::state::{proposal::State as ProposalState, Proposal};
use state::keys;

use super::super::TransactionContext;
use super::super::super::governance::proposal::ProposalUtil;
use super::super::Manager;
use super::BuiltinContractExecutorExt;

impl BuiltinContractExecutorExt for contract_pb::ProposalCreateContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        // NOTE: witness implies account

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

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_address = Address::try_from(&self.owner_address).unwrap();

        let proposal_id = manager.state_db.must_get(&keys::DynamicProperty::LatestProposalId) + 1;
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

        manager
            .state_db
            .put_key(keys::Proposal(proposal_id), proposal)
            .map_err(|_| "db insert error")?;
        manager
            .state_db
            .put_key(keys::DynamicProperty::LatestProposalId, proposal_id)
            .map_err(|_| "db insert error")?;

        Ok(TransactionResult::success())
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

        Ok(TransactionResult::success())
    }
}

impl BuiltinContractExecutorExt for contract_pb::ProposalDeleteContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        // NOTE: Proposal creator implies a witness. No need to check others.
        let proposal = manager
            .state_db
            .get(&keys::Proposal(self.proposal_id))
            .map_err(|_| "db query error")?
            .ok_or("proposal id not found")?;

        if proposal.proposer_address != self.owner_address {
            return Err(format!(
                "proposal #{} is not proposed by {}",
                proposal.proposal_id, owner_address
            ));
        }
        // NOTE: Pending implies not-expired, not-cancelled
        if proposal.state != ProposalState::Pending as i32 {
            return Err(format!(
                "proposal #{} is not in pending state(expired or cancelled)",
                proposal.proposal_id
            ));
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let mut proposal = manager.state_db.must_get(&keys::Proposal(self.proposal_id));
        proposal.state = ProposalState::Cancelled as _;
        manager
            .state_db
            .put_key(keys::Proposal(self.proposal_id), proposal)
            .map_err(|_| "db insert error")?;

        Ok(TransactionResult::success())
    }
}
