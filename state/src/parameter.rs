use config::ChainParameterConfig;

pub use proto2::state::ChainParameter;

pub fn default_parameters() -> impl IntoIterator<Item = (ChainParameter, i64)> {
    use ChainParameter::*;

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
        // = TotalEnergyLimit
        // (TotalEnergyCurrentLimit, 50_000_000_000),
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
    use ChainParameter::*;

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
        (EnergyFee, config.energy_fee),
        (WitnessCreateFee, 9999_000_000),
        (AccountCreateFee, 100_000),
        (AccountPermissionUpdateFee, 100_000_000),
        (AssetIssueFee, 1024_000_000),
        (ExchangeCreateFee, 1024_000_000),
        (MultisigFee, 1_000_000),
        (CreateNewAccountFeeInSystemContract, 0),
        (CreateNewAccountBandwidthRate, 1),
        (TotalEnergyLimit, 50_000_000_000),
        // Same as TotalEnergyLimit,
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
