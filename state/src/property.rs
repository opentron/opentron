use std::collections::HashMap;

use super::ChainParameter;

/// Used for DB migrations. Corresponding key is `DynamicProperty::DbVersion`.
const CURRENT_DB_VERSION: i64 = 1;

/// Dynamic properties of a living chain.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DynamicProperty {
    /// For migration.
    DbVersion,

    // * Global IDs
    /// 1000000 (start, never used)
    LatestTokenId,
    /// 0, starts from 1
    LatestProposalId,
    /// 1
    NextExchangeId,

    // * Latest Block
    LatestBlockTimestamp,
    LatestBlockNumber,
    // LatestBlockHash,
    LatestSolidBlockNumber,

    IsMaintenance,
    NextMaintenanceTime,
    HasNewVotesInCurrentEpoch,

    // StateFlag, is in maintenance?
    // TODO fill slots
    BlockFilledSlotsIndex,

    // * Bandwidth
    /// Renamed: TotalNetWeight
    TotalBandwidthWeight,
    /// Renamed: TotalNetLimit
    ///
    /// Default: 43_200_000_000
    TotalBandwidthLimit,

    // * Adaptive Energy
    /// Accumulator frozen energy.
    TotalEnergyWeight,
    /// Default: getTotalEnergyLimit() / 14400
    /// Calculated when active. so = 6250000
    TotalEnergyTargetLimit,
    /// Default: 0
    TotalEnergyAverageUsage,
    TotalEnergyAverageTime,
    // ChainParameter::TotalEnergyCurrentLimit = getTotalEnergyLimit()
    // ChainParameter::TotalEnergyLimit = 90000000000

    // * Global Free Bandwidth ('public' is ambiguous)
    /// Renamed: PublicNetLimit = 14_400_000_000
    GlobalFreeBandwidthLimit,
    /// Renamed: PublicNetUsage = 0
    GlobalFreeBandwidthUsed,
    /// Renamed: PublicNetTime = 0
    GlobalFreeBandwidthLatestSlot,
    // * Unused and deprecated
    // ! Why a block scoped variable is saved to store?
    // BlockEnergyUsage

    // ! accumulator, unused
    // TotalTransactionCost
    // TotalCreateWitnessCost
    // TotalCreateWitnessCost
    // TotalCreateAccountCost

    // ! storage, unused
    // TotalStoragePool
    // TotalStorageTax
    // TotalStorageReserved
    // StorageExchangeTaxRate

    // ! should be a calculated/cached property
    // Default: 7fff1fc0037e0000000000000000000000000000000000000000000000000000
    // DefaultPermissionMask,
    // DefaultPermissionMask without UpdateAccountPermission
    //
    // Default: 7fff1fc0033e0000000000000000000000000000000000000000000000000000
    // ActivePermissionMask,

    // Unused in mainnet: TotalShieldedPoolValue
}

impl DynamicProperty {
    pub fn default_properties() -> impl IntoIterator<Item = (DynamicProperty, i64)> {
        use self::DynamicProperty::*;

        return vec![
            (DbVersion, CURRENT_DB_VERSION),
            (LatestTokenId, 1000000),
            (LatestProposalId, 0),
            (NextExchangeId, 1),
            // LatestBlockTimestamp,
            // will be overwriten when apply genesis block
            (LatestBlockNumber, -1),
            (LatestSolidBlockNumber, -1),
            // * maintenance
            (IsMaintenance, 0),
            // FIXME: should be after genesis timestamp
            (NextMaintenanceTime, 0),
            (HasNewVotesInCurrentEpoch, 0),
            (BlockFilledSlotsIndex, 0),
            // * bandwidth
            (TotalBandwidthWeight, 0),
            (TotalBandwidthLimit, 43_200_000_000),
            (TotalEnergyWeight, 0),
            (GlobalFreeBandwidthLimit, 14_400_000_000),
            (GlobalFreeBandwidthUsed, 0),
            (GlobalFreeBandwidthLatestSlot, 0),
            // Default: ChainParameter::TotalEnergyLimit / 14400, when accessed
            // (TotalEnergyTargetLimit, 90_000_000_000 / 14400)
            (TotalEnergyAverageUsage, 0),
            (TotalEnergyAverageTime, 0),
        ];
    }

    pub fn initial_value_hook(
        &self,
        params: &HashMap<ChainParameter, i64>,
        _props: &HashMap<DynamicProperty, i64>,
    ) -> Option<i64> {
        match *self {
            DynamicProperty::TotalEnergyTargetLimit => Some(params[&ChainParameter::TotalEnergyLimit] / 14400),
            _ => None,
        }
    }
}
