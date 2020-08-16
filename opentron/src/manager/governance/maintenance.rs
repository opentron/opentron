use std::collections::HashMap;

use ::keys::Address;
use chain::IndexedBlock;
use chrono::Utc;
use log::{debug, info};
use proto2::state::Witness;
use state::keys;

use super::super::Manager;

/// Massive things done during maintenance.
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
                self.do_maintenance()?;
            } else {
                // init schedule on first non-genesis block.
                self.update_witness_schedule();
            }
            // updateNextMaintenanceTime
            self.increase_next_maintenance_time(next_maintenance_time, block.timestamp())?;
            let elapsed = (Utc::now().timestamp_nanos() - self.manager.maintenance_started_at) as f64 / 1_000_000.0;
            info!(
                "maintenance finished for block #{} total_time={}ms",
                block.number(),
                elapsed
            );
        }
        self.manager
            .state_db
            .put_key(keys::DynamicProperty::IsMaintenance, is_maintenance as _)
            .unwrap();
        Ok(())
    }

    fn do_maintenance(&mut self) -> Result<(), String> {
        // 0: default (unremoved)
        // 1: remove now
        // -1: removed
        if self.manager.state_db.must_get(&keys::ChainParameter::RemovePowerOfGr) == 1 {
            self.remove_power_of_gr()?;
        }

        // TODO: only count new votes of last epoch
        let votes = self.count_votes()?;
        if !votes.is_empty() {
            let old_active_witnesses = self.manager.get_active_witnesses();

            // FIXME: handle votes, unvotes
            let mut witnesses: HashMap<Address, Witness> = HashMap::new();
            for (&wit_addr, &vote_count) in votes.iter() {
                let mut wit = witnesses
                    .entry(wit_addr)
                    .or_insert_with(|| self.manager.state_db.must_get(&keys::Witness(wit_addr)));
                wit.vote_count = vote_count;
            }

            for (wit_addr, wit) in witnesses.into_iter() {
                // debug!("witness {} vote = {}", wit_addr, wit.vote_count);
                self.manager
                    .state_db
                    .put_key(keys::Witness(wit_addr), wit)
                    .map_err(|_| "db insert error")?;
            }

            self.update_witness_schedule();

            let new_active_witnesses = self.manager.get_active_witnesses();

            if old_active_witnesses != new_active_witnesses {
                for (idx, (old_wit_addr, new_wit_addr)) in
                    old_active_witnesses.iter().zip(new_active_witnesses.iter()).enumerate()
                {
                    if old_wit_addr != new_wit_addr {
                        debug!("active witness #{} change {} => {}", idx, old_wit_addr, new_wit_addr);
                    }
                }
            }

            // lagacy incentiveManager.reward(newWitnessAddressList)
            // Only when AllowChangeDelegation = false
            if self
                .manager
                .state_db
                .must_get(&keys::ChainParameter::AllowChangeDelegation) ==
                0
            {
                self.legacy_reward_standby_witnesses()
            }
            // TODO: update Witness.is_active
        }

        Ok(())
    }

    fn count_votes(&self) -> Result<HashMap<Address, i64>, String> {
        let mut votes: HashMap<Address, i64> = HashMap::new();

        let mut num_of_vote_entries = 0;
        {
            let votes = &mut votes;
            let num_of_vote_entries = &mut num_of_vote_entries;
            // value is a &state_pb::Votes
            self.manager.state_db.for_each(move |_key: &keys::Votes, value| {
                *num_of_vote_entries += 1;
                for vote in value.votes.iter() {
                    *votes.entry(*Address::from_bytes(&vote.vote_address)).or_default() += vote.vote_count;
                }
            });
        }
        if num_of_vote_entries > 0 {
            debug!("number of vote entries: {}", num_of_vote_entries);
        }

        if !votes.is_empty() && self.manager.state_db.must_get(&keys::ChainParameter::RemovePowerOfGr) != -1 {
            unimplemented!("TODO: the voting logic requires refactor");
        }

        Ok(votes)
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
            self.manager
                .state_db
                .put_key(keys::Witness(addr), witness)
                .map_err(|_| "insert db error")?;
        }
        self.manager
            .state_db
            .put_key(keys::ChainParameter::RemovePowerOfGr, -1)
            .map_err(|_| "insert db error")?;
        Ok(())
    }

    // DposService.updateWitness
    fn update_witness_schedule(&mut self) {
        let mut wit_votes: Vec<(Address, i64)> = Vec::new();
        {
            let wit_votes = &mut wit_votes;
            self.manager.state_db.for_each(move |key: &keys::Witness, value| {
                wit_votes.push((key.0, value.vote_count));
            });
        }

        // NOTE: This is different from java-tron. In OpenTron, raw address is used as final fallback sorting key.
        wit_votes.sort_by_cached_key(|&(addr, vote_count)| {
            (
                vote_count,
                java_bytestring_hash_code(addr.as_bytes()),
                addr.as_bytes().to_vec(),
            )
        });
        wit_votes.reverse();
        if wit_votes.len() > constants::MAX_NUM_OF_STANDBY_WITNESSES {
            let _ = wit_votes.split_off(constants::MAX_NUM_OF_STANDBY_WITNESSES);
        }
        let sched = wit_votes
            .iter()
            .map(|&(addr, _votes)| (addr, constants::DEFAULT_BROKERAGE_RATE))
            .collect();
        self.manager.state_db.put_key(keys::WitnessSchedule, sched).unwrap();
    }

    /// `IncentiveManager.reward`, only when `AllowChangeDelegation = false`.
    fn legacy_reward_standby_witnesses(&mut self) {
        unimplemented!()
    }
}

/// `hashCode()` for `com.google.protobuf.ByteString`.
///
/// NOTE: This is a really bad design mistake in java-tron, and is still vulnerable.
/// One must not depend on hash of object for stable sorting order.
/// And the serialized object must be used for last sorting key.
///
/// NOTE: Java has no unsigned integer/byte support. So `byte` is `i8`.
///
/// See-also: https://github.com/protocolbuffers/protobuf/blob/v3.4.0/java/core/src/main/java/com/google/protobuf/Internal.java
///
/// See-also: https://docs.oracle.com/javase/tutorial/java/nutsandbolts/datatypes.html
///
/// Original logic from com.google.protobuf.ByteString, in Rust:
///
/// ```rust
/// fn partial_hash(bs: &[u8], mut h: i32, offset: i32, length: i32) -> i32 {
///    for i in offset..(offset + length) {
///        h = h.wrapping_mul(31).wrapping_add(bs[i as usize] as i8 as i32);
///    }
///    h
/// }
/// let h = partial_hash(bs, bs.len() as i32, 0, bs.len() as i32);
/// if h == 0 {
///    1
/// } else {
///    h
/// }
/// ```
fn java_bytestring_hash_code(bs: &[u8]) -> i32 {
    match bs
        .iter()
        .fold(bs.len() as i32, |h, &b| h.wrapping_mul(31).wrapping_add(b as i8 as i32))
    {
        0 => 1,
        h => h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_bytestring_hash_code() {
        assert_eq!(java_bytestring_hash_code(&[]), 1);
        assert_eq!(java_bytestring_hash_code(&[0x23]), 66);
        assert_eq!(java_bytestring_hash_code(&[0x23, 0x66]), 3109);
        assert_eq!(java_bytestring_hash_code(&hex::decode("41f5").unwrap()), 3926);
        assert_eq!(
            java_bytestring_hash_code(&hex::decode("41f57bbf6b0c6530eea1f3c5718ebb0c4cdbde2c79").unwrap()),
            -797585552
        );
    }
}
