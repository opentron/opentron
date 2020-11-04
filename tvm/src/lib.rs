use primitive_types::H160;

pub use evm::executor::StackExecutor;
pub use evm::{Config, Context, ExitError, ExitFatal, ExitReason, ExitSucceed, Runtime};

use self::backend::Backend;

pub mod backend;
pub mod precompile;

/// Handle TVM upgrades.
#[derive(Debug, Clone, Default)]
pub struct TvmUpgrade {
    /// AllowTvmTransferTrc10Upgrade
    pub asset_transfer: bool,
    /// AllowTvmConstantinopleUpgrade
    pub constantinople: bool,
    /// AllowTvmSolidity059Upgrade, has batchvalidatesign, validatemultisign precompile.
    pub solidity059: bool,
    /// AllowTvmShieldedUpgrade, a precompile only upgrade.
    pub shielded: bool,
    /// AllowTvmStakeUpgrade
    pub stake: bool,
    /// AllowTvmIstanbulUpgrade
    pub istanbul: bool,
    /// AllowTvmAssetIssueUpgrade
    pub asset_issue: bool,
    /// AllowMultisig
    pub multisig: bool,
}

impl TvmUpgrade {
    fn validate(&self) -> bool {
        if self.constantinople && !self.asset_transfer {
            return false;
        }
        true
    }

    pub fn precompile(
        &self,
    ) -> fn(H160, &[u8], Option<usize>, &dyn Backend) -> Option<Result<(ExitSucceed, Vec<u8>, usize), ExitError>> {
        return self::precompile::tron_precompile;
    }

    pub fn to_tvm_config(&self) -> Config {
        if !self.validate() {
            panic!("inconsistent TVM state");
        }
        let mut config = Config::tvm();
        if self.multisig {
            config.has_buggy_origin = false;
        }
        if self.asset_transfer {
            config.allow_tvm_asset_transfer();
        }
        if self.constantinople {
            config.allow_tvm_constantinople();
        }
        if self.solidity059 {
            config.allow_tvm_solidity059();
        }
        // TODO: handle 4.1 update.
        config
    }
}
