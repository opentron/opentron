//! Resource processors.

use ::keys::Address;
use chain::IndexedTransaction;
use log::{debug, warn};
use prost::Message;
use proto2::chain::ContractType;
use proto2::contract::TransferAssetContract;
use proto2::state::Account;
use state::keys;

use super::actuators::BuiltinContractExt;
use super::executor::TransactionContext;
use super::Manager;

pub struct BandwidthProcessor<'m> {
    manager: &'m mut Manager,
}

impl<'m> BandwidthProcessor<'m> {
    pub fn new<'a>(manager: &'a mut Manager) -> BandwidthProcessor<'a> {
        BandwidthProcessor { manager }
    }

    pub fn consume<C: BuiltinContractExt>(
        &mut self,
        txn: &IndexedTransaction,
        cntr: &C,
        ctx: &mut TransactionContext,
    ) -> Result<(), String> {
        // NOTE: only first result is used.
        if !txn.raw.result.is_empty() && txn.raw.result[0].encoded_len() > constants::MAX_TRANSACTION_RESULT_SIZE {
            return Err("transaction result is too big".into());
        }

        let byte_size = if self.manager.state_db.must_get(&keys::ChainParameter::AllowTvm) == 1 {
            if txn.raw.result.is_empty() {
                txn.raw.encoded_len() + constants::MAX_TRANSACTION_RESULT_SIZE
            } else {
                let mut txn_without_ret = txn.raw.clone();
                txn_without_ret.result.clear();
                txn_without_ret.encoded_len() + constants::MAX_TRANSACTION_RESULT_SIZE
            }
        } else {
            txn.raw.encoded_len()
        };
        let byte_size = byte_size as i64;

        ctx.bandwidth_usage = byte_size;

        let now = self.manager.get_head_slot();

        let owner_address = *Address::from_bytes(cntr.owner_address());

        let owner_acct = self.manager.state_db.must_get(&keys::Account(owner_address));

        if ctx.new_account_created {
            if self.consume_bandwidth_for_new_account_creation(owner_address, &owner_acct, byte_size, now) ||
                self.consume_fee_for_new_account_creation(&owner_address, &owner_acct, ctx)
            {
                // covers all bw expense
                debug!("created new account!");
                return Ok(());
            } else {
                return Err("insufficient balance to create new account".into());
            }
        }

        // NOTE: Since Rust has no simple downcast support, use unsafe here.
        if cntr.type_code() == ContractType::TransferAssetContract &&
            self.consume_asset_bandwidth(unsafe { std::mem::transmute(cntr) })
        {
            return Ok(());
        }

        // NOTE: first use frozen bw, then free bw

        if self.consume_frozen_bandwidth(owner_address, &owner_acct, byte_size, now, ctx) {
            return Ok(());
        }

        if self.consume_free_bandwidth(owner_address, &owner_acct, byte_size, now, ctx) {
            return Ok(());
        }

        // burn for bandwidth
        if self.consume_burnt_bandwidth(owner_address, &owner_acct, byte_size, ctx) {
            return Ok(());
        }

        Err("insufficient bandwidth".into())
    }

    // Renamed: useTransactionFee
    fn consume_burnt_bandwidth(
        &mut self,
        addr: Address,
        acct: &Account,
        nbytes: i64,
        ctx: &mut TransactionContext,
    ) -> bool {
        let bw_fee = self.manager.state_db.must_get(&keys::ChainParameter::BandwidthFee) * nbytes;
        let mut new_acct = acct.clone();
        if new_acct.adjust_balance(-bw_fee).is_err() {
            return false;
        }

        self.manager.state_db.put_key(keys::Account(addr), new_acct).unwrap();
        ctx.bandwidth_fee = bw_fee;
        true
    }

    // Renamed: useAccountNet
    fn consume_frozen_bandwidth(
        &mut self,
        addr: Address,
        acct: &Account,
        nbytes: i64,
        now: i64,
        _ctx: &mut TransactionContext,
    ) -> bool {
        let bw_usage = acct.resource.as_ref().unwrap().frozen_bandwidth_used;
        let bw_latest_ts = acct.resource.as_ref().unwrap().frozen_bandwidth_latest_timestamp;
        let bw_limit = self.calculate_global_bandwidth_limit(acct);

        let mut new_bw_usage = adjust_usage(bw_usage, 0, bw_latest_ts, now);

        if nbytes > bw_limit - new_bw_usage {
            debug!("frozen bandwidth is insufficient, will try use free bandwidth");
            return false;
        }

        // consume frozen/delegated bw
        let latest_op_ts = self
            .manager
            .state_db
            .must_get(&keys::DynamicProperty::LatestBlockTimestamp);
        new_bw_usage = adjust_usage(new_bw_usage, nbytes, now, now);

        let mut acct = acct.clone();
        acct.latest_operation_timestamp = latest_op_ts;
        acct.resource_mut().frozen_bandwidth_used = new_bw_usage;
        acct.resource_mut().frozen_bandwidth_latest_timestamp = now;

        debug!("account bw: {}/{}", new_bw_usage, bw_limit);
        self.manager.state_db.put_key(keys::Account(addr), acct).unwrap();
        true
    }

    // Renamed: useFreeNet.
    fn consume_free_bandwidth(
        &mut self,
        addr: Address,
        acct: &Account,
        nbytes: i64,
        now: i64,
        _ctx: &mut TransactionContext,
    ) -> bool {
        let free_bw_limit = constants::FREE_BANDWIDTH;
        let free_bw_usage = acct.resource.as_ref().unwrap().free_bandwidth_used;
        let mut free_bw_latest_ts = acct.resource.as_ref().unwrap().free_bandwidth_latest_timestamp;

        let mut new_free_bw_usage = adjust_usage(free_bw_usage, 0, free_bw_latest_ts, now);
        if nbytes > free_bw_limit - new_free_bw_usage {
            debug!("free bandwidth is insufficient");
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
        let mut g_bw_latest_ts = self
            .manager
            .state_db
            .must_get(&keys::DynamicProperty::GlobalFreeBandwidthLatestTimestamp);

        let mut new_g_bw_usage = adjust_usage(g_bw_usage, 0, g_bw_latest_ts, now);
        if nbytes > g_bw_limit - new_g_bw_usage {
            debug!("global free bandwidth is insufficient");
            return false;
        }

        free_bw_latest_ts = now;
        g_bw_latest_ts = now;
        // FIXME: Is getHeadBlockTimeStamp current block?
        let lastes_op_ts = self.manager.latest_block_timestamp();
        new_free_bw_usage = adjust_usage(new_free_bw_usage, nbytes, free_bw_latest_ts, now);
        new_g_bw_usage = adjust_usage(new_g_bw_usage, nbytes, g_bw_latest_ts, now);

        let mut acct = acct.clone();
        {
            let mut resource = if acct.resource.is_none() {
                acct.resource = Some(Default::default());
                acct.resource.as_mut().unwrap()
            } else {
                acct.resource.as_mut().unwrap()
            };

            resource.free_bandwidth_used = new_free_bw_usage;
            resource.free_bandwidth_latest_timestamp = free_bw_latest_ts;
        }
        acct.latest_operation_timestamp = lastes_op_ts;
        debug!("account free bw: {}/{}", new_free_bw_usage, 5000);

        self.manager.state_db.put_key(keys::Account(addr), acct).unwrap();

        self.manager
            .state_db
            .put_key(keys::DynamicProperty::GlobalFreeBandwidthUsed, new_g_bw_usage)
            .unwrap();
        self.manager
            .state_db
            .put_key(
                keys::DynamicProperty::GlobalFreeBandwidthLatestTimestamp,
                g_bw_latest_ts,
            )
            .unwrap();

        true
    }

    // useAssetAccountNet
    fn consume_asset_bandwidth(&self, cntr: &TransferAssetContract) -> bool {
        warn!("TODO: useAssetAccountNet");
        debug!("transfer asset => {:?}", cntr);
        unimplemented!()
    }

    fn consume_fee_for_new_account_creation(
        &mut self,
        addr: &Address,
        acct: &Account,
        ctx: &mut TransactionContext,
    ) -> bool {
        // NOTE: distinguish `AccountCreateFee` from `CreateNewAccountFeeInSystemContract`
        let creation_fee = self.manager.state_db.must_get(&keys::ChainParameter::AccountCreateFee);
        // consumeFee
        if acct.balance >= creation_fee {
            // Reset bandwidth usage, account creation fee covers normal bandwidth.
            let mut acct = acct.clone();
            assert!(acct.adjust_balance(-creation_fee).is_ok());
            self.manager.state_db.put_key(keys::Account(*addr), acct).unwrap();
            ctx.bandwidth_fee = creation_fee;
            ctx.bandwidth_usage = 0;
            true
        } else {
            false
        }
    }

    // When an account has frozen enough bandwidth, it can create account freely.
    fn consume_bandwidth_for_new_account_creation(
        &mut self,
        addr: Address,
        acct: &Account,
        nbytes: i64,
        now: i64,
    ) -> bool {
        let new_acct_bw_ratio = self
            .manager
            .state_db
            .must_get(&keys::ChainParameter::CreateNewAccountBandwidthRate);

        // prost use optional fields for sub field.
        let res = acct.resource.as_ref().cloned().unwrap_or_default();

        let bw_usage = res.frozen_bandwidth_used;
        let bw_latest_ts = res.frozen_bandwidth_latest_timestamp;
        let bw_limit = self.calculate_global_bandwidth_limit(acct);

        let mut new_bw_usage = adjust_usage(bw_usage, 0, bw_latest_ts, now);

        // if freeze bw is enough
        if nbytes * new_acct_bw_ratio <= bw_limit - new_bw_usage {
            debug!(
                "create account with frozen bw: {}/{}",
                nbytes * new_acct_bw_ratio,
                bw_limit - new_bw_usage
            );
            let latest_op_ts = self
                .manager
                .state_db
                .must_get(&keys::DynamicProperty::LatestBlockTimestamp);
            new_bw_usage = adjust_usage(new_bw_usage, nbytes * new_acct_bw_ratio, now, now);

            let mut acct = acct.clone();
            acct.latest_operation_timestamp = latest_op_ts;
            acct.resource_mut().frozen_bandwidth_latest_timestamp = now;
            acct.resource_mut().frozen_bandwidth_used = new_bw_usage;

            self.manager.state_db.put_key(keys::Account(addr), acct).unwrap();
            true
        } else {
            false
        }
    }

    fn calculate_global_bandwidth_limit(&self, acct: &Account) -> i64 {
        let amount_for_bw = acct.amount_for_bandwidth();
        if amount_for_bw < 1_000_000 {
            return 0;
        }
        let bw_weight = amount_for_bw / 1_000_000;
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
fn adjust_usage(latest_usage: i64, new_usage: i64, latest_ts: i64, new_ts: i64) -> i64 {
    const WINDOW_SIZE: i64 = constants::RESOURCE_WINDOW_SIZE / constants::BLOCK_PRODUCING_INTERVAL;
    const PRECISION: i64 = constants::RESOURCE_PRECISION;

    let mut average_latest_usage = divide_ceil(latest_usage * PRECISION, WINDOW_SIZE);
    let average_new_usage = divide_ceil(new_usage * PRECISION, WINDOW_SIZE);

    if latest_ts != new_ts {
        assert!(new_ts > latest_ts);
        if latest_ts + WINDOW_SIZE > new_ts {
            let delta = new_ts - latest_ts;
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
