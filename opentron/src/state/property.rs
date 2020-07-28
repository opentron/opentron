#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DynamicProperty {
    /// For migration.
    DbVersion,

    // * Next Global IDs
    /// 1000001
    NextTokenId,
    /// 1
    NextProposalId,
    /// 1
    NextExchangeId,

    // * Latest Block
    LatestBlockTimestamp,
    LatestBlockNumber,
    LatestBlockHash,
    LatestSolidBlockNumber,

    // StateFlag, is in maintenance?
    // TODO fill slots
    // BlockFilledSlotsIndex // BLOCK_FILLED_SLOTS_NUMBER???

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
    GlobalFreeBandwidthLastUsedTimestamp,
    //
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
