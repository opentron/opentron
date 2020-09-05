/// ForkPolicies for different versions.
#[derive(Debug, PartialEq, Eq)]
pub enum ForkPolicy {
    // only used in then ENERGY_LIMIT fork
    AtBlock { block_number: i64 },
    // passOld
    Old,
    // passNew(>4.0.0)
    New { timestamp: i64, minimum_upgraded: usize },
}

/// Block versions. These versions match version names on github release page(or PR numbers).
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
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
    // NOTE: This version has only non-core API changes. Should not be a version fork.
    /// On Nile testnet, this is the `AllowShieldedTransaction` fork.
    ///
    /// See-also: https://github.com/tronprotocol/java-tron/pull/3372
    ///
    /// See-also: block 1628391 of Nile testnet
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

impl BlockVersion {
    pub fn fork_policy(&self) -> ForkPolicy {
        match *self {
            BlockVersion::Odyssey3_2 => ForkPolicy::AtBlock { block_number: 4727890 },
            BlockVersion::GreatVoyage4_0_1 => ForkPolicy::New {
                // GMT 2020-08-07 06:00:00
                timestamp: 1596780000_000,
                minimum_upgraded: 22,
            },
            _ => ForkPolicy::Old,
        }
    }

    /// The ENERGY_LIMIT fork at block #4727890.
    #[allow(non_snake_case)]
    pub const fn ENERGY_LIMIT() -> BlockVersion {
        BlockVersion::Odyssey3_2
    }
}
