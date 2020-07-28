pub const MAX_NUM_OF_ACTIVE_WITNESSES: usize = 27;
pub const MAX_NUM_OF_STANDBY_WITNESSES: usize = 127;

// 27 * 70% = 18.9, so a solid block is one verified by 19 witnesses.
pub const SOLID_THRESHOLD_PERCENT: usize = 70;

/// Will postpone txns if block size exceeds 2MiB.
/// So in theory, max txns/block is around 7700, max tps is around 2500.
pub const MAX_BLOCK_SIZE: usize = 2_000_000;

/// Max block size in channel protocol handler.
pub const MAX_ACCEPTABLE_BLOCK_SIZE: usize = MAX_BLOCK_SIZE + 1000;

pub const CURRENT_BLOCK_VERSION: BlockVersion = BlockVersion::GreatVoyage4_0_0;

pub const NUM_OF_SKIPPED_SLOTS_IN_MAINTENANCE: usize = 2;

/// Max number of votes in a `VoteWitness` is 30.
pub const MAX_NUM_OF_VOTES: usize = 30;

/// 500KB
pub const MAX_TRANSACTION_SIZE: usize = 500 * 1024;

pub const MAX_TRANSACTION_RESULT_SIZE: usize = 64;

/// 1d, in ms.
pub const MAX_TRANSACTION_EXPIRATION: usize = 24 * 60 * 60 * 1_000;

pub const DEFAULT_ORIGIN_ENERGY_LIMIT: usize = 10_000_000;

/// Block versions. These versions match version names on github release page(or PR numbers).
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum BlockVersion {
    /// Also applies to all blocks before around #2300000.
    Genesis = 0,
    Unknown1290 = 1,
    Unknown1442 = 2,
    Unknown1485 = 3,
    /// - Support Of Resource Delegation.
    /// - UpdateEnergyLimitContract
    /// - TotalEnergyLimit
    ///
    /// Renamed: ENERGY_LIMIT
    Odyssey3_2 = 5,
    /// - Deprecates `TotalEnergyLimit`
    /// - Add `TotalCurrentEnergyLimit`
    Odyssey3_2_2 = 6,
    /// - AllowMultisig
    /// - AllowAdaptiveEnergy
    /// - UpdateAccountPermissionFee
    /// - MultisigFee
    Odyssey3_5 = 7,
    /// - AllowProtoFilterNum
    /// - AllowAccountStateRoot
    /// - AllowTvmConstantinopleUpgrade
    Odyssey3_6_0 = 8,
    /// - AllowTvmSolidity059Upgrade
    /// - AdaptiveResourceLimitTargetRatio
    /// - AdaptiveResourceLimitMultiplier
    /// - AllowChangeDelegation
    /// - StandbyWitnessPayPerBlock
    Odyssey3_6_5 = 9,
    /// - ForbidTransferToContract
    Odyssey3_6_6 = 10,
    /// - AllowTvmShieldedUpgrade
    GreatVoyage4_0_0 = 16,
}
