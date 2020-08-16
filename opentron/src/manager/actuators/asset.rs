//! Asset related builtin contracts.

use std::convert::TryFrom;

use ::keys::Address;
use log::warn;
use proto2::chain::transaction::Result as TransactionResult;
use proto2::contract as contract_pb;
use proto2::state::{asset::FrozenSupply, Asset};
use state::keys;

use super::super::executor::TransactionContext;
use super::super::Manager;
use super::BuiltinContractExecutorExt;

impl BuiltinContractExecutorExt for contract_pb::AssetIssueContract {
    fn validate(&self, manager: &Manager, ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        let fee = self.fee(manager);
        ctx.contract_fee = fee;

        // validAssetName
        if self.name.is_empty() || self.name.len() > 32 || self.name.as_bytes().iter().any(|&b| b < 0x21 || b > 0x7e) {
            return Err("invalid asset name".into());
        }

        let allow_same_token_name = state_db.must_get(&keys::ChainParameter::AllowSameTokenName) != 0;

        if allow_same_token_name && self.name.to_lowercase() == "trx" {
            return Err("asset name cannot be 'TRX'".into());
        }

        // NOTE: The check logic is wrong, but to be compatible, just leave it.
        // The actual check logic should check `precison==0` when `!allow_same_token_name`.
        if allow_same_token_name && self.precision != 0 {
            if self.precision < 0 || self.precision > 6 {
                return Err("invalid precision, valid range is [0, 6]".into());
            }
        }

        if self.abbr.is_empty() || self.abbr.len() > 32 || self.abbr.as_bytes().iter().any(|&b| b < 0x21 || b > 0x7e) {
            return Err("invalid asset abbr".into());
        }

        // validUrl
        if self.url.is_empty() || self.url.len() > 256 {
            return Err("invalid asset url".into());
        }

        // validAssetDescription
        if self.description.len() > 200 {
            return Err("invalid asset description, too long".into());
        }

        if self.start_time == 0 {
            return Err("asset start time cannot be empty".into());
        }
        if self.end_time == 0 {
            return Err("asset end time cannot be empty".into());
        }
        if self.end_time <= self.start_time {
            return Err("asset end time should be greater than start time".into());
        }
        if self.start_time <= manager.latest_block_timestamp() {
            return Err("asset start time should be greater than latest block timestamp".into());
        }

        if !allow_same_token_name && find_asset_by_name(manager, &self.name).is_some() {
            return Err("asset name already exists".into());
        }

        if self.total_supply <= 0 {
            return Err("total supply should be greater than 0".into());
        }

        if self.trx_num <= 0 {
            return Err("trx_num should be greater than 0".into());
        }
        if self.num <= 0 {
            return Err("num should be greater than 0".into());
        }

        // NOTE: This is a design mistake. This field is used for state-db, not for sending a builtin contract.
        if self.public_free_asset_bandwidth_usage != 0 {
            return Err("do not fill public_free_asset_net_usage".into());
        }

        if self.frozen_supply.len() > constants::MAX_NUM_OF_FROZEN_SUPPLIES_IN_ASSET_ISSUE {
            return Err("frozen supply list is too long".into());
        }

        if self.free_asset_bandwidth_limit < 0 ||
            self.free_asset_bandwidth_limit >= constants::MAX_FREE_BANDWIDTH_IN_ASSET_ISSUE
        {
            return Err("invalid free_asset_bandwidth_limit".into());
        }

        if self.public_free_asset_bandwidth_limit < 0 ||
            self.public_free_asset_bandwidth_limit >= constants::MAX_FREE_BANDWIDTH_IN_ASSET_ISSUE
        {
            return Err("invalid public_free_asset_bandwidth_limit".into());
        }

        let mut remain_supply = self.total_supply;
        for frozen_supply in &self.frozen_supply {
            if frozen_supply.frozen_amount <= 0 {
                return Err("frozen amount should be greater than 0".into());
            }
            if frozen_supply.frozen_amount > remain_supply {
                return Err("total frozen amount should be less than total supply".into());
            }
            if frozen_supply.frozen_days < constants::MIN_NUM_OF_FROZEN_DAYS_IN_ASSET_ISSUE ||
                frozen_supply.frozen_days > constants::MAX_NUM_OF_FROZEN_DAYS_IN_ASSET_ISSUE
            {
                return Err(format!(
                    "frozen days should be in the range [{}, {}]",
                    constants::MIN_NUM_OF_FROZEN_DAYS_IN_ASSET_ISSUE,
                    constants::MAX_NUM_OF_FROZEN_DAYS_IN_ASSET_ISSUE
                ));
            }
            remain_supply -= frozen_supply.frozen_amount;
        }

        let maybe_acct = manager
            .state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "db query error")?;
        if maybe_acct.is_none() {
            return Err("account not exists".into());
        }
        let acct = maybe_acct.unwrap();

        if acct.issued_asset_id != 0 {
            return Err("an account can only issue one asset".into());
        }

        if acct.balance < fee {
            return Err("insufficient balance".into());
        }

        // NOTE: The `order` field is not used(commented out in java-tron).

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        const DAY_IN_MS: i64 = 86_400_000;

        let owner_address = Address::try_from(&self.owner_address).unwrap();
        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_address));

        let token_id = manager.state_db.must_get(&keys::DynamicProperty::LatestTokenId) + 1;
        owner_acct.issued_asset_id = token_id;

        let allow_same_token_name = manager.state_db.must_get(&keys::ChainParameter::AllowSameTokenName) != 0;

        // NOTE: `state::Asset` is not the same as `contract::AssetIssue`.
        let mut asset = Asset {
            id: token_id,
            owner_address: self.owner_address.clone(),
            name: self.name.clone(),
            abbr: self.abbr.clone(),
            total_supply: self.total_supply,
            frozen_supply: self
                .frozen_supply
                .iter()
                .map(|sup| FrozenSupply {
                    frozen_amount: sup.frozen_amount,
                    frozen_expiry_timestamp: self.start_time + sup.frozen_days * DAY_IN_MS,
                })
                .collect(),
            trx_num: self.trx_num,
            num: self.num,
            precision: self.precision,
            start_time: self.start_time,
            end_time: self.end_time,
            description: self.description.clone(),
            url: self.url.clone(),
            free_asset_bandwidth_limit: self.free_asset_bandwidth_limit,
            public_free_asset_bandwidth_limit: self.public_free_asset_bandwidth_limit,
            public_free_asset_bandwidth_used: 0,
            public_free_asset_bandwidth_last_timestamp: 0,
        };
        if !allow_same_token_name && asset.precision != 0 {
            warn!("BUG: disallow same token name, while precision is not 0");
            asset.precision = 0;
        }
        manager
            .state_db
            .put_key(keys::Asset(token_id), asset)
            .map_err(|_| "db insert error")?;
        manager
            .state_db
            .put_key(keys::DynamicProperty::LatestTokenId, token_id)
            .map_err(|_| "db insert error")?;

        if ctx.contract_fee != 0 {
            owner_acct.adjust_balance(-ctx.contract_fee).unwrap();
            manager.add_to_blackhole(ctx.contract_fee).unwrap();
        }
        manager
            .state_db
            .put_key(keys::Account(owner_address), owner_acct)
            .map_err(|_| "db insert error")?;

        // NOTE: `assetIssueID` of TransactionResult is not filled.
        Ok(TransactionResult::success())
    }

    fn fee(&self, manager: &Manager) -> i64 {
        manager.state_db.must_get(&keys::ChainParameter::AssetIssueFee)
    }
}

// This is a legacy feature. So no need to implement any reverse index against token names.
fn find_asset_by_name(manager: &Manager, asset_name: &str) -> Option<Asset> {
    let mut found: Option<Asset> = None;
    {
        let found = &mut found;
        manager.state_db.for_each(move |_key: &keys::Asset, value: &Asset| {
            if value.name == asset_name {
                *found = Some(value.clone());
            }
        });
    }
    found
}
