//! Resource processors.

use ::keys::Address;
use chain::IndexedTransaction;
use log::debug;
use prost::Message;
use proto2::chain::ContractType;
use proto2::contract::TransferAssetContract;
use proto2::state::Account;
use state::keys;

use super::actuators::BuiltinContractExt;
use super::executor::TransactionContext;
use super::Manager;

/// Bandwidth processor, `BandwidthProcessor.java`.
pub struct BandwidthProcessor<'a, C> {
    manager: &'a mut Manager,
    txn: &'a IndexedTransaction,
    cntr: &'a C,
    addr: Address,
    acct: Account,
}

impl<C> Drop for BandwidthProcessor<'_, C> {
    fn drop(&mut self) {
        self.manager
            .state_db
            .put_key(keys::Account(self.addr), self.acct.clone())
            .expect("error while saving bandwidth");
    }
}

impl<C: BuiltinContractExt> BandwidthProcessor<'_, C> {
    pub fn new<'a>(
        manager: &'a mut Manager,
        txn: &'a IndexedTransaction,
        cntr: &'a C,
    ) -> Result<BandwidthProcessor<'a, C>, String> {
        // NOTE: only first result is used.
        if !txn.raw.result.is_empty() && txn.raw.result[0].encoded_len() > constants::MAX_TRANSACTION_RESULT_SIZE {
            return Err("transaction result is too big".into());
        }
        let owner_address = *Address::from_bytes(cntr.owner_address());
        let owner_acct = manager.state_db.must_get(&keys::Account(owner_address));

        Ok(BandwidthProcessor {
            manager,
            txn,
            cntr,
            addr: owner_address,
            acct: owner_acct,
        })
    }

    pub fn consume(mut self, ctx: &mut TransactionContext) -> Result<(), String> {
        let byte_size = if self.manager.state_db.must_get(&keys::ChainParameter::AllowTvm) == 1 {
            if self.txn.raw.result.is_empty() {
                self.txn.raw.encoded_len() + constants::MAX_TRANSACTION_RESULT_SIZE
            } else {
                let mut txn_without_ret = self.txn.raw.clone();
                txn_without_ret.result.clear();
                txn_without_ret.encoded_len() + constants::MAX_TRANSACTION_RESULT_SIZE
            }
        } else {
            self.txn.raw.encoded_len()
        };
        let byte_size = byte_size as i64;
        ctx.bandwidth_usage = byte_size;

        // NOTE: multisig_fee is consumed in BandwidthProcessor
        if ctx.multisig_fee != 0 {
            debug!("consume multisig fee");
            self.acct
                .adjust_balance(-ctx.multisig_fee)
                .map_err(|_| "insufficient balance to multisig")?;
        }

        // NOTE: `now` is not a timestamp, it's a `slot`.
        let now = self.manager.get_head_slot();

        if ctx.new_account_created {
            // consumeForCreateNewAccount
            if self.consume_frozen_bandwidth_for_new_account_creation(byte_size, now) ||
                self.consume_fee_for_new_account_creation(ctx)
            {
                // covers all bw expense
                return Ok(());
            } else {
                return Err("insufficient balance to create new account".into());
            }
        }

        // NOTE: Since Rust has no simple downcast support, use unsafe here.
        if self.cntr.type_code() == ContractType::TransferAssetContract &&
            self.consume_asset_bandwidth(unsafe { std::mem::transmute(self.cntr) }, byte_size, now, ctx)
        {
            return Ok(());
        }

        // NOTE: first use frozen bw, then free bw

        if self.consume_frozen_bandwidth(byte_size, now, ctx) {
            return Ok(());
        }

        if self.consume_free_bandwidth(byte_size, now, ctx) {
            return Ok(());
        }

        // burn for bandwidth
        if self.consume_burnt_bandwidth(byte_size, ctx) {
            return Ok(());
        }

        Err("insufficient bandwidth".into())
    }

    // Renamed: useTransactionFee
    fn consume_burnt_bandwidth(&mut self, nbytes: i64, ctx: &mut TransactionContext) -> bool {
        let bw_fee = self.manager.state_db.must_get(&keys::ChainParameter::BandwidthFee) * nbytes;
        if self.acct.adjust_balance(-bw_fee).is_err() {
            return false;
        }

        ctx.bandwidth_fee = bw_fee;
        true
    }

    // Renamed: useAccountNet
    fn consume_frozen_bandwidth(&mut self, nbytes: i64, now: i64, _ctx: &mut TransactionContext) -> bool {
        let bw_usage = self.acct.resource().frozen_bandwidth_used;
        let bw_latest_slot = self.acct.resource().frozen_bandwidth_latest_slot;
        let bw_limit = self.calculate_global_bandwidth_limit(&self.acct);

        let mut new_bw_usage = adjust_usage(bw_usage, 0, bw_latest_slot, now);

        if nbytes > bw_limit - new_bw_usage {
            if bw_limit != 0 {
                // only log when there's freeze
                debug!(
                    "frozen bandwidth is insufficient {}/{}, requires={}",
                    new_bw_usage, bw_limit, nbytes
                );
            }
            return false;
        }

        // consume frozen/delegated bw
        let latest_op_ts = self
            .manager
            .state_db
            .must_get(&keys::DynamicProperty::LatestBlockTimestamp);
        new_bw_usage = adjust_usage(new_bw_usage, nbytes, now, now);

        self.acct.latest_operation_timestamp = latest_op_ts;
        self.acct.resource_mut().frozen_bandwidth_used = new_bw_usage;
        self.acct.resource_mut().frozen_bandwidth_latest_slot = now;

        debug!("frozen BW usage: {}/{} (+{})", new_bw_usage, bw_limit, nbytes);
        true
    }

    // Renamed: useFreeNet.
    fn consume_free_bandwidth(&mut self, nbytes: i64, now: i64, _ctx: &mut TransactionContext) -> bool {
        let free_bw_limit = constants::FREE_BANDWIDTH;
        let free_bw_usage = self.acct.resource().free_bandwidth_used;
        let mut free_bw_latest_slot = self.acct.resource().free_bandwidth_latest_slot;

        let mut new_free_bw_usage = adjust_usage(free_bw_usage, 0, free_bw_latest_slot, now);
        if nbytes > free_bw_limit - new_free_bw_usage {
            debug!(
                "free BW is insufficient {}/{}, require {}, will burn",
                new_free_bw_usage, free_bw_limit, nbytes
            );
            return false;
        }

        // global free bandwidth
        let g_bw_limit = self
            .manager
            .state_db
            .must_get(&keys::DynamicProperty::GlobalFreeBandwidthLimit);
        let g_bw_usage = self
            .manager
            .state_db
            .must_get(&keys::DynamicProperty::GlobalFreeBandwidthUsed);
        let mut g_bw_latest_slot = self
            .manager
            .state_db
            .must_get(&keys::DynamicProperty::GlobalFreeBandwidthLatestSlot);

        let mut new_g_bw_usage = adjust_usage(g_bw_usage, 0, g_bw_latest_slot, now);
        if nbytes > g_bw_limit - new_g_bw_usage {
            debug!("global free BW is insufficient");
            return false;
        }

        free_bw_latest_slot = now;
        g_bw_latest_slot = now;
        // FIXME: Is getHeadBlockTimeStamp current block?
        let lastes_op_ts = self.manager.latest_block_timestamp();
        new_free_bw_usage = adjust_usage(new_free_bw_usage, nbytes, free_bw_latest_slot, now);
        new_g_bw_usage = adjust_usage(new_g_bw_usage, nbytes, g_bw_latest_slot, now);

        self.acct.resource_mut().free_bandwidth_used = new_free_bw_usage;
        self.acct.resource_mut().free_bandwidth_latest_slot = free_bw_latest_slot;
        self.acct.latest_operation_timestamp = lastes_op_ts;

        debug!("free BW usage: {}/{} (+{})", new_free_bw_usage, free_bw_limit, nbytes);

        self.manager
            .state_db
            .put_key(keys::DynamicProperty::GlobalFreeBandwidthUsed, new_g_bw_usage)
            .unwrap();
        self.manager
            .state_db
            .put_key(keys::DynamicProperty::GlobalFreeBandwidthLatestSlot, g_bw_latest_slot)
            .unwrap();

        true
    }

    // useAssetAccountNet
    fn consume_asset_bandwidth(
        &mut self,
        cntr: &TransferAssetContract,
        nbytes: i64,
        now: i64,
        _ctx: &mut TransactionContext,
    ) -> bool {
        let allow_same_token_name = self
            .manager
            .state_db
            .must_get(&keys::ChainParameter::AllowSameTokenName) !=
            0;
        let mut asset = if allow_same_token_name {
            let token_id = cntr.asset_name.parse().unwrap();
            self.manager.state_db.must_get(&keys::Asset(token_id))
        } else {
            super::actuators::asset::find_asset_by_name(self.manager, &cntr.asset_name)
                .expect("must find by asset name")
        };
        let token_id = asset.id;

        if self.addr.as_bytes() == &*asset.owner_address {
            // NOTE: Different logic from java-tron.
            // Return a false, then automatically fallthrough to next `consume_frozen_bandwidth` in caller `consume`.
            // Avoid calling `consume_frozen_bandwidth` twice.
            //
            // return self.consume_frozen_bandwidth(addr, acct, nbytes, now, ctx);
            return false;
        }

        // check public limit
        let new_public_free_asset_bw_usage = adjust_usage(
            asset.public_free_asset_bandwidth_used,
            0,
            asset.public_free_asset_bandwidth_last_slot,
            now,
        );
        if nbytes > asset.public_free_asset_bandwidth_limit - new_public_free_asset_bw_usage {
            debug!("asset {} public free BW is insufficient", token_id);
            return false;
        }

        // check pre-account-limit
        let free_asset_bw_usage = self
            .acct
            .resource()
            .asset_bandwidth_used
            .get(&token_id)
            .copied()
            .unwrap_or(0);
        let latest_asset_op_slot = self
            .acct
            .resource()
            .asset_bandwidth_latest_slot
            .get(&token_id)
            .copied()
            .unwrap_or(0);

        let new_free_asset_bw_usage = adjust_usage(free_asset_bw_usage, 0, latest_asset_op_slot, now);

        if nbytes > asset.free_asset_bandwidth_limit - new_free_asset_bw_usage {
            debug!("asset {} free BW is insufficient", token_id);
            return false;
        }

        // check issuer's frozen bw
        let issuer_addr = *Address::from_bytes(&asset.owner_address);
        let mut issuer_acct = self.manager.state_db.must_get(&keys::Account(issuer_addr));
        let issuer_bw_limit = self.calculate_global_bandwidth_limit(&issuer_acct);

        let new_issuer_bw_usage = adjust_usage(
            issuer_acct.resource().frozen_bandwidth_used,
            0,
            issuer_acct.resource().frozen_bandwidth_latest_slot,
            now,
        );

        if nbytes > issuer_bw_limit - new_issuer_bw_usage {
            debug!("asset {} issuer bandwidth is insufficient", token_id);
            return false;
        }

        // now consume
        let latest_op_ts = self.manager.latest_block_timestamp();

        let new_issuer_bw_usage = adjust_usage(new_issuer_bw_usage, nbytes, now, now);
        let new_free_asset_bw_usage = adjust_usage(new_free_asset_bw_usage, nbytes, now, now);
        let new_public_free_asset_bw_usage = adjust_usage(new_public_free_asset_bw_usage, nbytes, now, now);

        debug!(
            "asset #{} issuer BW {}/{} (+{}), user limit {}/{}, public limit {}/{}",
            token_id, new_issuer_bw_usage, issuer_bw_limit, nbytes,
            new_free_asset_bw_usage, asset.free_asset_bandwidth_limit,
            new_public_free_asset_bw_usage, asset.public_free_asset_bandwidth_limit
        );

        issuer_acct.resource_mut().frozen_bandwidth_used = new_issuer_bw_usage;
        issuer_acct.resource_mut().frozen_bandwidth_latest_slot = now;

        asset.public_free_asset_bandwidth_used = new_public_free_asset_bw_usage;
        asset.public_free_asset_bandwidth_last_slot = now;

        self.acct.latest_operation_timestamp = latest_op_ts;
        self.acct
            .resource_mut()
            .asset_bandwidth_latest_slot
            .insert(token_id, now);
        self.acct
            .resource_mut()
            .asset_bandwidth_used
            .insert(token_id, new_free_asset_bw_usage);

        // now save
        self.manager
            .state_db
            .put_key(keys::Account(issuer_addr), issuer_acct)
            .unwrap();
        self.manager.state_db.put_key(keys::Asset(token_id), asset).unwrap();

        true
    }

    /// `consumeFeeForCreateNewAccount`
    fn consume_fee_for_new_account_creation(&mut self, ctx: &mut TransactionContext) -> bool {
        // NOTE: distinguish `AccountCreateFee` from `CreateNewAccountFeeInSystemContract`
        let creation_fee = self.manager.state_db.must_get(&keys::ChainParameter::AccountCreateFee);
        // consumeFee
        if self.acct.balance >= creation_fee {
            debug!("create account by BW fee");
            // Reset bandwidth usage, account creation fee covers normal bandwidth.
            assert!(self.acct.adjust_balance(-creation_fee).is_ok());
            ctx.bandwidth_fee = creation_fee;
            ctx.bandwidth_usage = 0;
            true
        } else {
            false
        }
    }

    /// `consumeBandwidthForCreateNewAccount`
    ///
    /// When an account has frozen enough bandwidth, it can create account freely.
    fn consume_frozen_bandwidth_for_new_account_creation(&mut self, nbytes: i64, now: i64) -> bool {
        let new_acct_bw_ratio = self
            .manager
            .state_db
            .must_get(&keys::ChainParameter::CreateNewAccountBandwidthRate);

        // prost use optional fields for sub field.

        let bw_usage = self.acct.resource().frozen_bandwidth_used;
        let bw_latest_slot = self.acct.resource().frozen_bandwidth_latest_slot;
        let bw_limit = self.calculate_global_bandwidth_limit(&self.acct);

        let mut new_bw_usage = adjust_usage(bw_usage, 0, bw_latest_slot, now);

        // if freeze bw is enough to create account
        if nbytes * new_acct_bw_ratio <= bw_limit - new_bw_usage {
            let latest_op_ts = self
                .manager
                .state_db
                .must_get(&keys::DynamicProperty::LatestBlockTimestamp);
            new_bw_usage = adjust_usage(new_bw_usage, nbytes * new_acct_bw_ratio, now, now);

            debug!(
                "create account by frozen BW: {}/{} (+{})",
                new_bw_usage,
                bw_limit,
                nbytes * new_acct_bw_ratio,
            );

            self.acct.latest_operation_timestamp = latest_op_ts;
            self.acct.resource_mut().frozen_bandwidth_latest_slot = now;
            self.acct.resource_mut().frozen_bandwidth_used = new_bw_usage;

            true
        } else {
            false
        }
    }

    /// `calculateGlobalNetLimit`
    fn calculate_global_bandwidth_limit(&self, acct: &Account) -> i64 {
        let amount_for_bw = acct.amount_for_bandwidth();
        if amount_for_bw < 1_000_000 {
            return 0;
        }
        let bw_weight = amount_for_bw / 1_000_000;
        // NOTE: Although resource weight values update as new freeze and unfreeze transactions handled,
        // new weight values is used when doing resource calculations of current block.
        //
        // Take block #43004 of mainnet as an example. This is an edge case with 3 transactions.
        // First is a FreezeBalanceContract of 5_000_000_TRX, last one is a TransferContract all balance to create a
        // new account(with 3 TRX frozen, enough BW to create account free of charge).
        // Freezing so much TRX causes weight to increase, so bandwidth acquired from previous freezing is decreased.
        //
        // For better handling of this situation, block producer should reorder transactions.
        let total_bw_limit = self
            .manager
            .state_db
            .must_get(&keys::DynamicProperty::TotalBandwidthLimit);

        let total_bw_weight = self
            .manager
            .state_db
            .must_get(&keys::DynamicProperty::TotalBandwidthWeight);

        if total_bw_weight == 0 {
            return 0;
        }
        return (bw_weight as f64 * (total_bw_limit as f64 / total_bw_weight as f64)) as i64;
    }
}

#[inline]
fn divide_ceil(numerator: i64, denominator: i64) -> i64 {
    (numerator / denominator) + ((numerator % denominator) > 0) as i64
}

// Renamed: increase.
fn adjust_usage(latest_usage: i64, new_usage: i64, latest_slot: i64, new_slot: i64) -> i64 {
    const WINDOW_SIZE: i64 = constants::RESOURCE_WINDOW_SIZE / constants::BLOCK_PRODUCING_INTERVAL;
    const PRECISION: i64 = constants::RESOURCE_PRECISION;

    let mut average_latest_usage = divide_ceil(latest_usage * PRECISION, WINDOW_SIZE);
    let average_new_usage = divide_ceil(new_usage * PRECISION, WINDOW_SIZE);

    if latest_slot != new_slot {
        assert!(new_slot > latest_slot);
        if latest_slot + WINDOW_SIZE > new_slot {
            let delta = new_slot - latest_slot;
            let decay: f64 = (WINDOW_SIZE - delta) as f64 / WINDOW_SIZE as f64;
            average_latest_usage = (average_latest_usage as f64 * decay).round() as _;
        } else {
            average_latest_usage = 0;
        }
    }

    average_latest_usage += average_new_usage;
    // getUsage
    average_latest_usage * WINDOW_SIZE / PRECISION
}
