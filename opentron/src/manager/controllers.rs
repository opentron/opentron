use constants::block_version::{BlockVersion, ForkPolicy};
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
/// FIXME: If there's inconsistent when an SR downgrades it's node version?
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
