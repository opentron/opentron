use ::keys::Address;
use chain::IndexedBlock;
use log::debug;
use state::keys;

use super::super::Manager;

/// Massive thing done during maintenance.
pub struct MaintenanceManager<'m> {
    manager: &'m mut Manager,
}

impl MaintenanceManager<'_> {
    pub fn new<'a>(manager: &'a mut Manager) -> MaintenanceManager<'a> {
        MaintenanceManager { manager }
    }

    pub fn apply_block(mut self, block: &IndexedBlock) -> Result<(), String> {
        let next_maintenance_time = self
            .manager
            .state_db
            .must_get(&keys::DynamicProperty::NextMaintenanceTime);

        let is_maintenance = next_maintenance_time <= block.timestamp();

        if is_maintenance {
            if block.number() != 1 {
                self.do_maintenance(block)?;
            }
            // updateNextMaintenanceTime
            self.increase_next_maintenance_time(next_maintenance_time, block.timestamp())?;
        }
        self.manager
            .state_db
            .put_key(keys::DynamicProperty::IsMaintenance, is_maintenance as _)
            .unwrap();
        Ok(())
    }

    fn do_maintenance(&mut self, block: &IndexedBlock) -> Result<(), String> {
        // 0: default
        // 1: remove now
        // -1: removed
        if self.manager.state_db.must_get(&keys::ChainParameter::RemovePowerOfGr) == 1 {
            self.remove_power_of_gr()?;
        }

        let _votes = self.count_votes();

        // TODO: handle vote
        // unimplemented!()
        debug!("TODO: handle do_maintenance(votes counting) at block #{}", block.number());
        Ok(())
    }

    fn count_votes(&self) -> Vec<(Address, i64)> {
        use std::collections::HashMap;

        let mut votes: HashMap<Address, i64> = HashMap::new();

        // value is a &state_pb::Votes
        {
            let votes = &mut votes;
            self.manager.state_db.for_each(move |key: &keys::Votes, value| {
                debug!("got votes key => {:?} {:?}", key, value);
                for vote in value.votes.iter() {
                    *votes.entry(*Address::from_bytes(&vote.vote_address)).or_default() += vote.vote_count;
                }
            });
        }
        // TODO: unimplemented!()
        vec![]
    }

    // in DynamicPropertiesStore.java
    fn increase_next_maintenance_time(
        &mut self,
        current_next_maintenance_time: i64,
        block_ts: i64,
    ) -> Result<(), String> {
        let maintenance_interval = self
            .manager
            .state_db
            .must_get(&keys::ChainParameter::MaintenanceInterval);

        let round = (block_ts - current_next_maintenance_time) / maintenance_interval;
        let next_maintenance_time = current_next_maintenance_time + (round + 1) * maintenance_interval;
        self.manager
            .state_db
            .put_key(keys::DynamicProperty::NextMaintenanceTime, next_maintenance_time)
            .unwrap();
        Ok(())
    }

    // Remove vote counts in genesis config.
    fn remove_power_of_gr(&mut self) -> Result<(), String> {
        for gr_wit in &self.manager.genesis_config.witnesses {
            let addr = gr_wit.address.parse::<Address>().expect("address format error");

            let mut witness = self.manager.state_db.must_get(&keys::Witness(addr));
            witness.vote_count -= gr_wit.votes;
            self.manager.state_db.put_key(keys::Witness(addr), witness).map_err(|_| "insert db error")?;
        }
        self.manager.state_db.put_key(keys::ChainParameter::RemovePowerOfGr, -1).map_err(|_| "insert db error")?;
        Ok(())
    }
}
