//* Chain constants.

use block_version::BlockVersion;

pub mod block_version;

/// Current block version for produced block.
pub const CURRENT_BLOCK_VERSION: BlockVersion = BlockVersion::GreatVoyage4_0_1;

// * Block Produce.

/// Will postpone txns if block size exceeds 2MiB.
/// So in theory, max txns/block is around 7700, max tps is around 2500.
pub const MAX_BLOCK_SIZE: usize = 2_000_000;

// 3s, in ms.
pub const BLOCK_PRODUCING_INTERVAL: i64 = 3_000;

// 50%
pub const BLOCK_PRODUCE_TIMEOUT_PERCENT: i64 = 50;

/// Max block size in channel protocol handler.
pub const MAX_ACCEPTABLE_BLOCK_SIZE: usize = MAX_BLOCK_SIZE + 1000;

pub const FREE_BANDWIDTH: i64 = 5000;

//* Witness and block producing.
pub const MAX_NUM_OF_ACTIVE_WITNESSES: usize = 27;
pub const MAX_NUM_OF_STANDBY_WITNESSES: usize = 127;

// 27 * 70% = 18.9, so a solid block is one verified by 19 witnesses.
pub const SOLID_THRESHOLD_PERCENT: usize = 70;

pub const NUM_OF_SKIPPED_SLOTS_IN_MAINTENANCE: usize = 2;

// An SR should produce this much blocks then next.
pub const NUM_OF_CONSECUTIVE_BLOCKS_PER_ROUND: usize = 1;

/// Renamed: WitnessAllowanceFrozenTime
pub const NUM_OF_FROZEN_DAYS_FOR_WITNESS_ALLOWANCE: i64 = 1;

/// Percent of block reward paid to witness.
pub const DEFAULT_BROKERAGE_RATE: i32 = 20;

/// Renamed: BLOCK_FILLED_SLOTS_NUMBER
pub const NUM_OF_BLOCK_FILLED_SLOTS: usize = 128;

//* Transactions

/// 500KB
pub const MAX_TRANSACTION_SIZE: usize = 500 * 1024;

pub const MAX_TRANSACTION_RESULT_SIZE: usize = 64;

/// Max number of votes in a `VoteWitness` is 30.
pub const MAX_NUM_OF_VOTES: usize = 30;

/// 1d, in ms.
pub const MAX_TRANSACTION_EXPIRATION: i64 = 24 * 60 * 60 * 1_000;

pub const DEFAULT_ORIGIN_ENERGY_LIMIT: usize = 10_000_000;

pub const MAX_NUM_OF_FROZEN_DAYS_FOR_RESOURCE: i64 = 3;
pub const MIN_NUM_OF_FROZEN_DAYS_FOR_RESOURCE: i64 = 3;

/// Max number of `FronzenSupply` in AssetIssue.
pub const MAX_NUM_OF_FROZEN_SUPPLIES_IN_ASSET_ISSUE: usize = 10;

pub const MAX_NUM_OF_FROZEN_DAYS_IN_ASSET_ISSUE: i64 = 3652;
pub const MIN_NUM_OF_FROZEN_DAYS_IN_ASSET_ISSUE: i64 = 1;

/// Renamed: OneDayNetLimit, restrict both free_asset_bandwidth_limit and public_free_asset_bandwidth_limit.
pub const MAX_FREE_BANDWIDTH_IN_ASSET_ISSUE: i64 = 57_600_000_000;

// Renamed: ExchangeBalanceLimit
pub const MAX_EXCHANGE_BALANCE: usize = 1_000_000_000_000_000;

// 1d, in ms.
pub const RESOURCE_WINDOW_SIZE: i64 = 24 * 3600 * 1000;
/// Precision used in resource calculation.
pub const RESOURCE_PRECISION: i64 = 1_000_000;

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

// * Account

pub const MAX_NUM_OF_ACTIVE_PERMISSIONS: usize = 8;

// Renamed: TotalSignNum
pub const MAX_NUM_OF_KEYS_IN_PERMISSION: usize = 5;
