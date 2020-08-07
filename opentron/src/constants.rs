//* Chain parameters.

pub const CURRENT_BLOCK_VERSION: BlockVersion = BlockVersion::GreatVoyage4_0_1;

/// Will postpone txns if block size exceeds 2MiB.
/// So in theory, max txns/block is around 7700, max tps is around 2500.
pub const MAX_BLOCK_SIZE: usize = 2_000_000;

// 3s, in ms.
pub const BLOCK_PRODUCING_INTERVAL: i64 = 3_000;

/// Max block size in channel protocol handler.
pub const MAX_ACCEPTABLE_BLOCK_SIZE: usize = MAX_BLOCK_SIZE + 1000;

pub const FREE_BANDWIDTH: usize = 5000;

//* Witness and block producing.
pub const MAX_NUM_OF_ACTIVE_WITNESSES: usize = 27;
pub const MAX_NUM_OF_STANDBY_WITNESSES: usize = 127;

// 27 * 70% = 18.9, so a solid block is one verified by 19 witnesses.
pub const SOLID_THRESHOLD_PERCENT: usize = 70;

pub const NUM_OF_SKIPPED_SLOTS_IN_MAINTENANCE: i64 = 2;

/// Renamed: WitnessAllowanceFrozenTime
pub const NUM_OF_FRONZEN_DAYS_FOR_WITNESS_ALLOWANCE: usize = 1;

/// in percent
pub const DEFAULT_BROKERAGE_RATE: u8 = 20;

//* Transactions

/// 500KB
pub const MAX_TRANSACTION_SIZE: usize = 500 * 1024;

pub const MAX_TRANSACTION_RESULT_SIZE: usize = 64;

/// Max number of votes in a `VoteWitness` is 30.
pub const MAX_NUM_OF_VOTES: usize = 30;

/// 1d, in ms.
pub const MAX_TRANSACTION_EXPIRATION: usize = 24 * 60 * 60 * 1_000;

pub const DEFAULT_ORIGIN_ENERGY_LIMIT: usize = 10_000_000;

/// Renamed: TotalSignNum
pub const MAX_NUM_OF_KEYS_IN_MULTISIG: usize = 5;

pub const MAX_NUM_OF_FROZEN_DAYS_FOR_RESOURCE: usize = 3;
pub const MIN_NUM_OF_FROZEN_DAYS_FOR_RESOURCE: usize = 3;

/// Max number of `FronzenSupply` in AssetIssue.
pub const MAX_NUM_OF_FROZEN_SUPPLIES_IN_ASSET_ISSUE: usize = 10;

pub const MAX_NUM_OF_FRONZEN_DAYS_IN_ASSET_ISSUE: usize = 3652;
pub const MIN_NUM_OF_FRONZEN_DAYS_IN_ASSET_ISSUE: usize = 1;

/// Renamed: OneDayNetLimit, restrict both free_asset_net_limit and public_free_asset_net_limit.
pub const MAX_FREE_BANDWIDTH_IN_ASSET_ISSUE: usize = 57_600_000_000;

// Renamed: ExchangeBalanceLimit
pub const MAX_EXCHANGE_BALANCE: usize = 1_000_000_000_000_000;

// 1d, in ms.
pub const RESOURCE_WINDOW_SIZE: usize = 24 * 3600 * 1000;
/// Precision used in resource calculation.
pub const RESOURCE_PRECISION: usize = 1_000_000;

// * Adaptive Energy
// if TotalEnergyAverageUsage > TotalEnergyTargetLimit:
//    decrease TotalEnergyCurrentLimit to 99/100
// else
//    increase TotalEnergyCurrentLimit to 1000/999
//
// FYI: Limit / Weight = price
pub const ADAPTIVE_ENERGY_DECREASE_RATE_NUMERATOR: i64 = 99;
pub const ADAPTIVE_ENERGY_DECREASE_RATE_DENOMINATOR: i64 = 100;

pub const ADAPTIVE_ENERGY_INCREASE_RATE_NUMERATOR: i64 = 1000;
pub const ADAPTIVE_ENERGY_INCREASE_RATE_DENOMINATOR: i64 = 999;

/// Block versions. These versions match version names on github release page(or PR numbers).
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum BlockVersion {
    /// Also applies to all blocks before around #2300000.
    Genesis = 0,
    Unknown1290 = 1,
    // PR #1442
    Odyssey3_0_1 = 2,
    // PR #1485
    Odyssey3_1_0 = 3,
    /// Special Check after block #4727890 (enery.limit.block.num).
    /// When block.version == 5,
    /// it makes block use new energy to handle transaction when block number >= 4727890L.
    /// Otherwise version !=5, skip.
    ///
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
    // Note: This version has only non-core API changes.
    Odyssey3_7 = 15,
    /// Shielded TVM precompiles.
    ///
    /// - support AllowTvmShieldedUpgrade config
    GreatVoyage4_0_0 = 16,
    /// First hard fork based on timestamp. Fork at 1596780000_000, at least 22 SRs.
    ///
    /// See-also: https://github.com/tronprotocol/java-tron/pull/3314
    ///
    /// - support AllowTvmShieldedUpgrade proposal
    GreatVoyage4_0_1 = 17,
}
