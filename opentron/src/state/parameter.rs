use crate::config::ChainParameterConfig;

/// Chain parameters, known as proposals, can be changed via proposal.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ChainParameter {
    // Core chain parameters.
    /// Renamed: `MaintenanceTimeInterval`
    ///
    /// Default: 6h, 21600000 in ms
    ///
    /// Range: [3 * 27 * 1000, 24 * 3600 * 1000] (27x3s ~ 1d)
    MaintenanceInterval = 0,
    /// Max execution duration of a TVM transaction(smart contract creation and triggering).
    ///
    /// Default: 50, in ms
    ///
    /// Renamed: `MaxCpuTimeOfOneTx`
    ///
    /// Range: [10, 100]
    MaxCpuTimeOfOneTxn = 13,
    /// Remove votes from geneses GRs(guard representative).
    ///
    /// Renamed: `RemoveThePowerOfTheGr`
    ///
    /// Default: 0, unremoved
    ///
    /// Mainnet: -1
    ///
    /// Note: 1 to remove, -1 denotes already removed
    RemovePowerOfGr = 10,

    // New features and fixes of new versions.
    /// Allow update an accunt's name; Allow duplicate names.
    ///
    /// Default: 0
    ///
    /// Mainnet: 0
    AllowUpdateAccountName = 14,
    /// Default: config, 0
    AllowSameTokenName = 15,
    /// Default: config, 0
    AllowDelegateResource = 16,
    /// Enabled: 3.5
    ///
    /// Renamed: `AllowMultiSign`
    ///
    /// Default: config, 0
    AllowMultisig = 20,
    /// Enabled: 3.6
    ///
    /// Default: 0
    ///
    /// Mainnet: 0
    AllowAccountStateRoot = 25,
    /// This enables TVM and allows creation of smart contracts.
    ///
    /// Renamed: `AllowCreationOfContracts`
    ///
    /// Default: config, 0
    AllowTvm = 9,
    /// Enabled: 3.6.6
    ///
    /// Requires: `AllowTvm`
    ///
    /// Default: 0
    ///
    /// Mainnet: 0 (not passed)
    ForbidTransferToContract = 35,
    /// Add `UpdateBrokerageContract`, something to do with witness reward. Decentralized vote dividend.
    /// TODO: bad naming
    ///
    /// Renamed: `ChangeDelegation`
    ///
    /// Enabled: 3.6.5
    AllowChangeDelegation = 30,

    // Fees, in SUN.
    /// Renamed: `TransactionFee`
    ///
    /// Default: 10
    BandwidthFee = 3,
    /// Default: 100
    ///
    /// Mainnet: 10
    EnergyFee = 11,
    /// Renamed: `AccountUpgradeCost`
    ///
    /// Default: 9999_000_000
    WitnessCreateFee = 1,
    /// Used in `CreateAccount`, `Transfer`, `TransferAsset`.
    ///
    /// Renamed: `CreateAccountFee`
    ///
    /// Default: 100_000
    AccountCreateFee = 2,
    /// Default: 1024_000_000
    AssetIssueFee = 4,
    /// Default: 1024_000_000
    ExchangeCreateFee = 12,
    /// Renamed: `UpdateAccountPermissionFee`
    ///
    /// Enabled: 3.5
    ///
    /// Range: [0, 100_000_000000]
    ///
    /// Default: 100_000_000
    AccountPermissionUpdateFee = 22,
    /// Renamed: `MultiSignFee`
    ///
    /// Enabled: 3.5
    ///
    /// Default: 1_000_000
    ///
    /// Range: [0, 100_000_000000]
    MultisigFee = 23,
    /// Used in `CreateAccount`, `Transfer`, `TransferAsset`.
    ///
    /// Default: 0
    CreateNewAccountFeeInSystemContract = 7,
    /// Default: 1
    CreateNewAccountBandwidthRate = 8,

    // Adaptive energy.
    /// Enabled by EnergyLimit update, Deprecated by 3.2.2.
    ///
    /// Default: 50_000_000_000
    TotalEnergyLimit = 17,
    /// Enabled: 3.2.2
    ///
    /// Default: 50_000_000_000
    ///
    /// Mainnet: 90_000_000_000
    TotalEnergyCurrentLimit = 19,
    /// Default: config, 0
    ///
    /// Mainnet: 0
    AllowAdaptiveEnergy = 21,
    // derived from above
    // TotalEnergyTargetLimit = 6250000
    // TotalEnergyAverageUsage = 0
    /// Enabled: 3.6.5
    ///
    /// Default: 14400 (24 * 60 * 10)
    ///
    /// Mainnet: 10
    AdaptiveResourceLimitTargetRatio = 33,
    /// Enabled: 3.6.5
    ///
    /// Default: 1000
    AdaptiveResourceLimitMultiplier = 29,

    // Witness rewards. Standby witness = 27 active witnesses + 100 partner witnesses.
    /// Default: 32_000_000
    ///
    /// Mainnet: 16_000_000
    WitnessPayPerBlock = 5,
    /// In maintenance cycle, after new witness list is updated, All standby witnesses will be rewarded.
    /// Actual pay amount is divided by the total vote weight. Only when `AllowChangeDelegation` = false.
    ///
    /// Renamed: `WitnessStandbyAllowance`
    ///
    /// Requires: !`AllowChangeDelegation`
    ///
    /// Default: 115_200_000_000 = 16_000_000 * (21600_000 / 3_000)
    StandbyWitnessAllowance = 6,
    /// Each block, amount of TRX reward is paid to standby witness, actual pay amount is according to the vote weight.
    ///
    /// Renamed: `Witness127PayPerBlock`
    ///
    /// Requires: `AllowChangeDelegation`
    ///
    /// Enabled: 3.6.5
    ///
    /// Default: 16_000_000
    ///
    /// Mainnet: 160_000_000
    StandbyWitnessPayPerBlock = 31,

    // TVM updates
    /// TVM v3.2 update. CALLTOKEN, TOKENBALANCE, CALLTOKENVALUE, CALLTOKENID.
    ///
    /// Requires: `AllowSameTokenName`
    ///
    /// Default: config, 0
    AllowTvmTransferTrc10Upgrade = 18,
    /// TVM with shift instructions, CREATE2, EXTCODEHASH. ClearABI, forbid delegate resource to contract.
    ///
    /// Renamed: `AllowTvmConstantinople`
    ///
    /// Enabled: 3.6
    ///
    /// Requires: `AllowTvmTransferTrc10`
    ///
    /// Default: config, 0
    AllowTvmConstantinopleUpgrade = 26,
    /// TVM with `batchvalidatesign`, `validatemultisign`, `iscontract` support.
    ///
    /// Renamed: `AllowTvmSolidity059`
    ///
    /// Enabled: 3.6.5
    ///
    /// Requires: `AllowTvm`
    ///
    /// Default: config, 0
    AllowTvmSolidity059Upgrade = 32,
    /// TVM with librustzcash Shielded precompiles upgrade.
    ///
    /// Renamed: `AllowShieldedTRC20Transaction`
    ///
    /// Enabled: 4.0
    ///
    /// Default: config, 0
    AllowTvmShieldedUpgrade = 39,

    /// Useless.
    ///
    /// Enabled: 3.6
    ///
    /// Default: 0
    AllowProtoFilterNum = 24,
}

impl ChainParameter {
    pub fn to_i32(&self) -> i32 {
        // ProposalType
        use self::ChainParameter::*;

        match *self {
            MaintenanceInterval => 0,
            MaxCpuTimeOfOneTxn => 13,
            RemovePowerOfGr => 10,

            AllowUpdateAccountName => 14,
            AllowSameTokenName => 15,
            AllowDelegateResource => 16,
            AllowMultisig => 20,
            AllowAccountStateRoot => 25,
            ForbidTransferToContract => 35,
            AllowChangeDelegation => 30,

            BandwidthFee => 3,
            EnergyFee => 11,
            WitnessCreateFee => 1,
            AccountCreateFee => 2,
            AccountPermissionUpdateFee => 22,
            AssetIssueFee => 4,
            ExchangeCreateFee => 12,
            MultisigFee => 23,

            CreateNewAccountFeeInSystemContract => 7,
            CreateNewAccountBandwidthRate => 8,

            TotalEnergyLimit => 17,
            TotalEnergyCurrentLimit => 19,
            AllowAdaptiveEnergy => 21,
            AdaptiveResourceLimitTargetRatio => 33,
            AdaptiveResourceLimitMultiplier => 29,

            WitnessPayPerBlock => 5,
            StandbyWitnessAllowance => 6,
            StandbyWitnessPayPerBlock => 31,

            AllowTvm => 9,
            AllowTvmTransferTrc10Upgrade => 18,
            AllowTvmConstantinopleUpgrade => 26,
            AllowTvmSolidity059Upgrade => 32,
            AllowTvmShieldedUpgrade => 39,

            AllowProtoFilterNum => 24,
        }
    }

    pub fn default_parameters() -> impl IntoIterator<Item = (ChainParameter, i64)> {
        use self::ChainParameter::*;

        return vec![
            (MaintenanceInterval, 21600_000),
            (MaxCpuTimeOfOneTxn, 50),
            (RemovePowerOfGr, 0),
            (AllowUpdateAccountName, 0),
            (AllowSameTokenName, 0),
            (AllowDelegateResource, 0),
            (AllowMultisig, 0),
            (AllowAccountStateRoot, 0),
            (AllowChangeDelegation, 0),
            (AllowTvm, 0),
            (ForbidTransferToContract, 0),
            (BandwidthFee, 10),
            (EnergyFee, 100),
            (WitnessCreateFee, 9999_000_000),
            (AccountCreateFee, 100_000),
            (AccountPermissionUpdateFee, 100_000_000),
            (AssetIssueFee, 1024_000_000),
            (ExchangeCreateFee, 1024_000_000),
            (MultisigFee, 1_000_000),
            (CreateNewAccountFeeInSystemContract, 0),
            (CreateNewAccountBandwidthRate, 1),
            (TotalEnergyLimit, 50_000_000_000),
            (TotalEnergyCurrentLimit, 50_000_000_000),
            (AllowAdaptiveEnergy, 0),
            (AdaptiveResourceLimitTargetRatio, 14400),
            (AdaptiveResourceLimitMultiplier, 1_000),
            (WitnessPayPerBlock, 32_000_000),
            (StandbyWitnessAllowance, 115_200_000_000),
            (StandbyWitnessPayPerBlock, 16_000_000),
            (AllowTvmTransferTrc10Upgrade, 0),
            (AllowTvmConstantinopleUpgrade, 0),
            (AllowTvmSolidity059Upgrade, 0),
            (AllowTvmShieldedUpgrade, 0),
            (AllowProtoFilterNum, 0),
        ];
    }

    pub fn default_parameters_from_config(
        config: &ChainParameterConfig,
    ) -> impl IntoIterator<Item = (ChainParameter, i64)> {
        use self::ChainParameter::*;

        return vec![
            (MaintenanceInterval, config.maintenance_interval),
            (MaxCpuTimeOfOneTxn, 50),
            (RemovePowerOfGr, 0),
            (AllowUpdateAccountName, 0),
            (AllowSameTokenName, config.allow_duplicate_asset_names as i64),
            (AllowDelegateResource, config.allow_delegate_resource as i64),
            (AllowMultisig, config.allow_multisig as i64),
            (AllowAccountStateRoot, 0),
            (AllowChangeDelegation, 0),
            (AllowTvm, config.allow_tvm as i64),
            (ForbidTransferToContract, 0),
            (BandwidthFee, 10),
            (EnergyFee, config.energy_fee.unwrap_or(100)),
            (WitnessCreateFee, 9999_000_000),
            (AccountCreateFee, 100_000),
            (AccountPermissionUpdateFee, 100_000_000),
            (AssetIssueFee, 1024_000_000),
            (ExchangeCreateFee, 1024_000_000),
            (MultisigFee, 1_000_000),
            (CreateNewAccountFeeInSystemContract, 0),
            (CreateNewAccountBandwidthRate, 1),
            (TotalEnergyLimit, 50_000_000_000),
            (TotalEnergyCurrentLimit, 50_000_000_000),
            (AllowAdaptiveEnergy, config.allow_adaptive_energy as i64),
            (AdaptiveResourceLimitTargetRatio, 14400),
            (AdaptiveResourceLimitMultiplier, 1_000),
            (WitnessPayPerBlock, 32_000_000),
            (StandbyWitnessAllowance, 115_200_000_000),
            (StandbyWitnessPayPerBlock, 16_000_000),
            (
                AllowTvmTransferTrc10Upgrade,
                config.allow_tvm_transfer_trc10_upgrade as i64,
            ),
            (
                AllowTvmConstantinopleUpgrade,
                config.allow_tvm_constantinople_upgrade as i64,
            ),
            (AllowTvmSolidity059Upgrade, config.allow_tvm_solidity_059_upgrade as i64),
            (AllowTvmShieldedUpgrade, config.allow_tvm_shielded_upgrade as i64),
            (AllowProtoFilterNum, 0),
        ];
    }
}
