//! Asset related builtin contracts.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Mutex;

use ::keys::Address;
use lazy_static::lazy_static;
use log::warn;
use proto2::chain::transaction::Result as TransactionResult;
use proto2::common::AccountType;
use proto2::contract as contract_pb;
use proto2::state::{asset::FrozenSupply, Account, Asset};
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

        // NOTE: `abbr` can be empty, like asset #1000477.
        if self.abbr.len() > 32 || self.abbr.as_bytes().iter().any(|&b| b < 0x21 || b > 0x7e) {
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

        // NOTE: This is a design flaw. This field is used for state-db, not for sending a builtin contract.
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
                    is_unfrozen: false,
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
            public_free_asset_bandwidth_last_slot: 0,
        };
        if !allow_same_token_name && asset.precision != 0 {
            warn!("BUG: disallow same token name, while precision is not 0");
            asset.precision = 0;
        }
        let remain_supply = self.total_supply - self.frozen_supply.iter().map(|sup| sup.frozen_amount).sum::<i64>();
        owner_acct.token_balance.insert(token_id, remain_supply);

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

// Transfer TRC10(Asset) tokens, creating to_account when it is not on chain.
impl BuiltinContractExecutorExt for contract_pb::TransferAssetContract {
    fn validate(&self, manager: &Manager, ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;
        let to_address = Address::try_from(&self.to_address).map_err(|_| "invalid to_address")?;

        let mut fee = self.fee(manager);

        if self.amount <= 0 {
            return Err("transfer amount must be greater than 0".into());
        }

        if owner_address == to_address {
            return Err("cannot transfer to oneself".into());
        }

        let allow_same_token_name = manager.state_db.must_get(&keys::ChainParameter::AllowSameTokenName) != 0;
        let maybe_asset = if allow_same_token_name {
            let token_id = self.asset_name.parse().map_err(|_| "invalid asset name")?;
            state_db.get(&keys::Asset(token_id)).map_err(|_| "db query error")?
        } else {
            find_asset_by_name(manager, &self.asset_name)
        };
        if maybe_asset.is_none() {
            return Err(format!("asset name {} not found", self.asset_name));
        }
        let asset = maybe_asset.unwrap();

        let maybe_owner_acct = manager
            .state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "db query error")?;
        if maybe_owner_acct.is_none() {
            return Err("account not exists".into());
        }
        let owner_acct = maybe_owner_acct.unwrap();

        let token_balance = owner_acct.token_balance.get(&asset.id).copied().unwrap_or(0);
        if token_balance < self.amount {
            return Err("insufficient token balance".into());
        }

        let maybe_to_acct = state_db
            .get(&keys::Account(to_address))
            .map_err(|_| "error while querying db")?;
        if let Some(to_acct) = maybe_to_acct {
            if to_acct.r#type == AccountType::Contract as i32 &&
                state_db.must_get(&keys::ChainParameter::ForbidTransferToContract) == 1
            {
                return Err("cannot transfer to a smart contract".into());
            }

            if to_acct
                .token_balance
                .get(&asset.id)
                .copied()
                .unwrap_or(0)
                .checked_add(self.amount)
                .is_none()
            {
                return Err("math overflow".into());
            }
        } else {
            ctx.new_account_created = true;
            // NOTE: CreateNewAccountFeeInSystemContract is 0, account creation fee is handled by BandwidthProcessor.
            fee += state_db.must_get(&keys::ChainParameter::CreateNewAccountFeeInSystemContract);
        }

        if fee != 0 && owner_acct.balance < fee {
            return Err("insufficient balance".into());
        }

        ctx.contract_fee = fee;

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_address = Address::try_from(&self.owner_address).unwrap();
        let to_address = Address::try_from(&self.to_address).unwrap();

        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_address));

        let fee = ctx.contract_fee;

        let mut to_acct = manager
            .state_db
            .get(&keys::Account(to_address))
            .map_err(|e| format!("state-db error: {:?}", e))?
            .unwrap_or_else(|| Account::new(manager.latest_block_timestamp()));

        let allow_same_token_name = manager.state_db.must_get(&keys::ChainParameter::AllowSameTokenName) != 0;
        let token_id: i64 = if allow_same_token_name {
            self.asset_name.parse().map_err(|_| "invalid asset name")?
        } else {
            find_asset_by_name(manager, &self.asset_name).unwrap().id
        };

        if fee != 0 {
            owner_acct.adjust_balance(-fee).unwrap();
            manager.add_to_blackhole(fee).unwrap();
        }

        owner_acct
            .token_balance
            .entry(token_id)
            .and_modify(|bal| *bal -= self.amount);
        *to_acct.token_balance.entry(token_id).or_default() += self.amount;

        manager
            .state_db
            .put_key(keys::Account(owner_address), owner_acct)
            .map_err(|e| e.to_string())?;
        manager
            .state_db
            .put_key(keys::Account(to_address), to_acct)
            .map_err(|e| e.to_string())?;

        Ok(TransactionResult::success())
    }
}

// Participate asset issuing while asset is in issuing period. Buy new TRC10 token using TRX.
impl BuiltinContractExecutorExt for contract_pb::ParticipateAssetIssueContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;
        let to_address = Address::try_from(&self.to_address).map_err(|_| "invalid to_address")?;

        if self.amount <= 0 {
            return Err("amount must be greater than 0".into());
        }

        if owner_address == to_address {
            return Err("cannot participate asset issue of oneself".into());
        }

        let maybe_owner_acct = state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "error while querying db")?;
        if maybe_owner_acct.is_none() {
            return Err("owner account is not on chain".into());
        }
        let owner_acct = maybe_owner_acct.unwrap();

        if owner_acct.balance < self.amount {
            return Err("insufficient balance".into());
        }

        let allow_same_token_name = manager.state_db.must_get(&keys::ChainParameter::AllowSameTokenName) != 0;
        let maybe_asset = if allow_same_token_name {
            let token_id = self.asset_name.parse().map_err(|_| "invalid asset name")?;
            state_db.get(&keys::Asset(token_id)).map_err(|_| "db query error")?
        } else {
            find_asset_by_name(manager, &self.asset_name)
        };
        if maybe_asset.is_none() {
            return Err(format!("asset name {} not found", self.asset_name));
        }
        let asset = maybe_asset.unwrap();

        if to_address.as_bytes() != &*asset.owner_address {
            return Err(format!("asset {} is not issued by {}", asset.id, to_address));
        }

        // exchange feasibility check
        let now = manager.latest_block_timestamp();
        if now >= asset.end_time || now < asset.start_time {
            return Err("asset is not in issuing period".into());
        }

        let exchange_amount = self
            .amount
            .checked_mul(asset.num as i64)
            .ok_or("math overflow")?
            .checked_div(asset.trx_num as i64)
            .ok_or("math overflow")?;
        if exchange_amount < 0 {
            return Err("math overflow, cannot process the exchange".into());
        }

        // NOTE: asset implies account, this might be useless.
        let maybe_to_acct = state_db
            .get(&keys::Account(to_address))
            .map_err(|_| "error while querying db")?;
        if maybe_to_acct.is_none() {
            return Err("to account is not on chain".into());
        }
        let to_acct = maybe_to_acct.unwrap();

        if to_acct.token_balance.get(&asset.id).copied().unwrap_or(0) < exchange_amount {
            return Err("insufficient balance of target asset".into());
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_address = Address::try_from(&self.owner_address).unwrap();
        let to_address = Address::try_from(&self.to_address).unwrap();

        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_address));
        let mut to_acct = manager.state_db.must_get(&keys::Account(to_address));

        // TODO: might be optimized via ctx, to avoid re-calculation
        let allow_same_token_name = manager.state_db.must_get(&keys::ChainParameter::AllowSameTokenName) != 0;
        let asset = if allow_same_token_name {
            let token_id = self.asset_name.parse().map_err(|_| "invalid asset name")?;
            manager
                .state_db
                .get(&keys::Asset(token_id))
                .map_err(|_| "db query error")?
                .unwrap()
        } else {
            find_asset_by_name(manager, &self.asset_name).unwrap()
        };
        let exchange_amount = self.amount * asset.num as i64 / asset.trx_num as i64;

        owner_acct.adjust_balance(-self.amount).unwrap();
        to_acct.adjust_balance(self.amount).unwrap();

        owner_acct.adjust_token_balance(asset.id, exchange_amount).unwrap();
        to_acct.adjust_token_balance(asset.id, -exchange_amount).unwrap();

        manager
            .state_db
            .put_key(keys::Account(owner_address), owner_acct)
            .map_err(|e| e.to_string())?;
        manager
            .state_db
            .put_key(keys::Account(to_address), to_acct)
            .map_err(|e| e.to_string())?;

        Ok(TransactionResult::success())
    }
}

// Update an asset' url, description, per-account free bw limit, global free bw limit.
impl BuiltinContractExecutorExt for contract_pb::UpdateAssetContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        let maybe_owner_acct = state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "error while querying db")?;
        if maybe_owner_acct.is_none() {
            return Err("owner account is not on chain".into());
        }
        let owner_acct = maybe_owner_acct.unwrap();
        if owner_acct.issued_asset_id == 0 {
            return Err("account has not issued any asset".into());
        }

        // TODO: is this needless?
        let maybe_asset = state_db
            .get(&keys::Asset(owner_acct.issued_asset_id))
            .map_err(|_| "db query error")?;
        if maybe_asset.is_none() {
            return Err(format!(
                "asset for id {} is not found in state-db",
                owner_acct.issued_asset_id
            ));
        }

        // validUrl
        if self.url.is_empty() || self.url.len() > 256 {
            return Err("invalid asset url".into());
        }

        // validAssetDescription
        if self.description.len() > 200 {
            return Err("invalid asset description, too long".into());
        }

        if self.new_limit < 0 || self.new_limit >= constants::MAX_FREE_BANDWIDTH_IN_ASSET_ISSUE {
            return Err("invalid free_asset_bandwidth_limit".into());
        }

        if self.new_public_limit < 0 || self.new_public_limit >= constants::MAX_FREE_BANDWIDTH_IN_ASSET_ISSUE {
            return Err("invalid public_free_asset_bandwidth_limit".into());
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, _ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_address = Address::try_from(&self.owner_address).unwrap();
        let owner_acct = manager.state_db.must_get(&keys::Account(owner_address));
        let mut asset = manager.state_db.must_get(&keys::Asset(owner_acct.issued_asset_id));

        asset.url = self.url.clone();
        asset.description = self.description.clone();
        asset.free_asset_bandwidth_limit = self.new_limit;
        asset.public_free_asset_bandwidth_limit = self.new_public_limit;

        manager
            .state_db
            .put_key(keys::Asset(owner_acct.issued_asset_id), asset)
            .map_err(|_| "db insert error")?;

        Ok(TransactionResult::success())
    }
}

// Unfreeze an asset's frozen_supply.
impl BuiltinContractExecutorExt for contract_pb::UnfreezeAssetContract {
    fn validate(&self, manager: &Manager, _ctx: &mut TransactionContext) -> Result<(), String> {
        let state_db = &manager.state_db;

        let owner_address = Address::try_from(&self.owner_address).map_err(|_| "invalid owner_address")?;

        let maybe_owner_acct = state_db
            .get(&keys::Account(owner_address))
            .map_err(|_| "error while querying db")?;
        if maybe_owner_acct.is_none() {
            return Err("owner account is not on chain".into());
        }
        let owner_acct = maybe_owner_acct.unwrap();
        if owner_acct.issued_asset_id == 0 {
            return Err("account has not issued any asset".into());
        }

        let maybe_asset = state_db
            .get(&keys::Asset(owner_acct.issued_asset_id))
            .map_err(|_| "db query error")?;
        if maybe_asset.is_none() {
            return Err(format!(
                "asset for id {} is not found in state-db",
                owner_acct.issued_asset_id
            ));
        }
        let asset = maybe_asset.unwrap();
        if asset.frozen_supply.is_empty() {
            return Err("no frozen supply".into());
        }

        log::debug!("frozen_sup => {:?}", asset.frozen_supply);
        let now = manager.latest_block_timestamp();
        if asset
            .frozen_supply
            .iter()
            .find(|sup| !sup.is_unfrozen && sup.frozen_expiry_timestamp <= now)
            .is_none()
        {
            return Err("no frozen supply to unfreeze".into());
        }

        Ok(())
    }

    fn execute(&self, manager: &mut Manager, ctx: &mut TransactionContext) -> Result<TransactionResult, String> {
        let owner_address = Address::try_from(&self.owner_address).unwrap();
        let mut owner_acct = manager.state_db.must_get(&keys::Account(owner_address));
        let mut asset = manager.state_db.must_get(&keys::Asset(owner_acct.issued_asset_id));

        let now = manager.latest_block_timestamp();

        let mut unfrozen_amount = 0_i64;
        for sup in asset.frozen_supply.iter_mut() {
            if !sup.is_unfrozen && sup.frozen_expiry_timestamp <= now {
                unfrozen_amount += sup.frozen_amount;
                sup.is_unfrozen = true;
            }
        }

        ctx.unfrozen_amount = unfrozen_amount;
        owner_acct
            .adjust_token_balance(owner_acct.issued_asset_id, unfrozen_amount)
            .unwrap();

        manager
            .state_db
            .put_key(keys::Asset(owner_acct.issued_asset_id), asset)
            .map_err(|_| "db insert error")?;
        manager
            .state_db
            .put_key(keys::Account(owner_address), owner_acct)
            .map_err(|_| "db insert error")?;

        Ok(TransactionResult::success())
    }
}

// Asset name to asset id cache.
lazy_static! {
    static ref ASSET_ID_CACHE: Mutex<HashMap<String, i64>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}

/// Find asset from state-db by its name. This is a legacy feature.
/// So no need to implement any reverse index against token names.
/// A cache hashmap is enough to speed up legacy assets.
///
/// Should only be used before `AllowSameTokenName` is ON.
///
/// NOTE: This is a design flaw. Actually, one should use an asset's abbr instead of name.
/// Never mind, use asset id(token id) solves.
pub fn find_asset_by_name(manager: &Manager, asset_name: &str) -> Option<Asset> {
    let mut map = ASSET_ID_CACHE.lock().unwrap();
    if let Some(&token_id) = map.get(asset_name) {
        manager.state_db.get(&keys::Asset(token_id)).expect("db query error")
    } else {
        let mut found: Option<Asset> = None;
        {
            let found = &mut found;
            manager.state_db.for_each(move |_key: &keys::Asset, asset: &Asset| {
                if asset.name == asset_name {
                    map.insert(asset_name.to_owned(), asset.id);
                    *found = Some(asset.clone());
                }
            });
        }
        found
    }
}
