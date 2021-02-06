# Release Versions of Java-Tron

ref: `block_version.rs`.

```rust
/// Block versions. These versions match version names on github release page(or PR numbers).
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum BlockVersion {
    /// Also applies to all blocks before around #2300000.
    Genesis = 0,
    /// First introduce block_version, PR #1290.
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
    /// - AllowTvmTransferTrc10Upgrade
    ///   - CALLTOKEN, TOKENBALANCE, CALLTOKENVALUE, CALLTOKENID
    ///
    /// Renamed: ENERGY_LIMIT
    Odyssey3_2 = 5,
    /// - Deprecates `TotalEnergyLimit`
    /// - Add `TotalCurrentEnergyLimit`
    Odyssey3_2_2 = 6,
    /// - AllowMultisig - introduce user permission
    /// - AllowAdaptiveEnergy
    /// - UpdateAccountPermissionFee
    /// - MultisigFee
    Odyssey3_5 = 7,
    /// - AllowProtoFilterNum
    //    - useless
    /// - AllowAccountStateRoot
    //    - never used, but there're some blocks generated filled with the field
    /// - AllowTvmConstantinopleUpgrade
    ///   - ClearABI
    ///   - reject delegate resource to contract address
    ///   - Save runtime code from deploy code
    ///   - SHL, SHR, SAR, fake CREATE2, EXTCODEHASH
    ///   - Introduce TransferException
    Odyssey3_6_0 = 8,
    /// - AllowTvmSolidity059Upgrade
    ///   - create account while transfer TRX/TRC10
    ///   - batchvalidatesig, validatemultisig
    ///   - ISCONTRACT
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
    /// - AllowPbft
    /// - AllowTvmIstanbulUpgrade
    /// - AllowMarketTransaction
    ///   - MarketSellFee
    ///   - MarketCancelFee
    GreatVoyage4_1_0 = 19,
    /// - MaxFeeLimit
    /// - AllowTransactionFeePool
    /// - AllowBlackholeOptimization
    GreatVoyage4_1_2 = 20,
}
```