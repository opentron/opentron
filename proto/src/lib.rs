#[rustfmt::skip]
#[path = "proto.common.rs"]
mod common_inner;

pub mod common {
    pub use crate::common_inner::*;
    use byteorder::{ByteOrder, BE};

    impl From<Vec<u8>> for BlockId {
        fn from(block_hash: Vec<u8>) -> Self {
            assert_eq!(block_hash.len(), 32);
            let block_number = BE::read_u64(&block_hash[..8]);
            BlockId {
                hash: block_hash,
                number: block_number as i64,
            }
        }
    }

    impl ::std::fmt::Display for BlockId {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("BlockId")
                .field("number", &self.number)
                .field("hash", &hex::encode(&self.hash))
                .finish()
        }
    }
}

#[rustfmt::skip]
#[path = "proto.chain.rs"]
mod chain_inner;

pub mod chain {
    pub use crate::chain_inner::*;

    impl Block {
        pub fn number(&self) -> i64 {
            let raw_header = &self.block_header.as_ref().unwrap().raw_data.as_ref().unwrap();
            raw_header.number
        }
    }

    impl ::std::fmt::Display for Block {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            let raw_header = &self.block_header.as_ref().unwrap().raw_data.as_ref().unwrap();
            f.debug_struct("Block")
                .field("number", &raw_header.number)
                .field("timestamp", &raw_header.timestamp)
                .field("txns", &self.transactions.len())
                .finish()
        }
    }

    impl transaction::Result {
        pub fn success() -> Self {
            use self::transaction::result::ContractStatus;

            transaction::Result {
                contract_status: ContractStatus::Success as i32,
                ..Default::default()
            }
        }
    }
}

#[rustfmt::skip]
#[path = "proto.discovery.rs"]
pub mod discovery;

#[rustfmt::skip]
#[path = "proto.channel.rs"]
mod channel_inner;

pub mod channel {
    pub use crate::chain::{Block, Transaction};
    pub use crate::channel_inner::*;

    impl ::std::fmt::Display for ReasonCode {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }
}

#[rustfmt::skip]
#[path = "proto.contract.rs"]
pub mod contract;
mod contract_ext;
pub use contract_ext::ContractExt;

#[rustfmt::skip]
#[path = "proto.state.rs"]
mod state_inner;

pub mod state {
    pub use crate::common::{AccountType, ResourceCode, SmartContract};
    pub use crate::state_inner::*;

    use self::proposal::State as ProposalState;

    impl Account {
        pub fn new(block_timestamp: i64) -> Self {
            Account {
                creation_time: block_timestamp,
                resource: Some(Default::default()),
                ..Default::default()
            }
        }

        pub fn new_contract_account(block_timestamp: i64) -> Self {
            Account {
                creation_time: block_timestamp,
                r#type: AccountType::Contract as i32,
                ..Default::default()
            }
        }

        pub fn adjust_balance(&mut self, diff: i64) -> Result<(), ()> {
            if let Some(new_balance) = self.balance.checked_add(diff) {
                // When self.balance is negative, this is a blackhole.
                if self.balance < 0 || new_balance >= 0 {
                    self.balance = new_balance;
                    return Ok(());
                }
            }
            Err(())
        }

        pub fn adjust_allowance(&mut self, diff: i64) -> Result<(), ()> {
            if let Some(new_allowance) = self.allowance.checked_add(diff) {
                if new_allowance >= 0 {
                    self.allowance = new_allowance;
                    return Ok(());
                }
            }
            Err(())
        }

        pub fn adjust_token_balance(&mut self, token_id: i64, diff: i64) -> Result<(), ()> {
            if let Some(balance) = self.token_balance.get_mut(&token_id) {
                if let Some(new_balance) = balance.checked_add(diff) {
                    if new_balance >= 0 {
                        *balance = new_balance;
                        return Ok(());
                    }
                }
            } else if diff >= 0 {
                self.token_balance.insert(token_id, diff);
                return Ok(());
            }
            Err(())
        }

        pub fn tron_power(&self) -> i64 {
            (self.frozen_amount_for_bandwidth + self.frozen_amount_for_energy + self.delegated_out_amount) / 1_000_000
        }

        pub fn amount_for_bandwidth(&self) -> i64 {
            self.frozen_amount_for_bandwidth + self.delegated_frozen_amount_for_bandwidth
        }

        pub fn amount_for_energy(&self) -> i64 {
            self.frozen_amount_for_energy + self.delegated_frozen_amount_for_energy
        }

        pub fn resource(&self) -> &AccountResource {
            self.resource.as_ref().unwrap()
        }

        pub fn resource_mut(&mut self) -> &mut AccountResource {
            if self.resource.is_none() {
                self.resource = Some(Default::default());
            }
            self.resource.as_mut().unwrap()
        }

        pub fn delegated_amount_for_resource(&self, res: ResourceCode) -> i64 {
            match res {
                ResourceCode::Bandwidth => self.delegated_frozen_amount_for_bandwidth,
                ResourceCode::Energy => self.delegated_frozen_amount_for_energy,
            }
        }
    }

    impl Proposal {
        pub fn is_processed(&self) -> bool {
            if self.state == ProposalState::Disapproved as i32 || self.state == ProposalState::Approved as i32 {
                true
            } else {
                false
            }
        }

        pub fn is_cancelled(&self) -> bool {
            if self.state == ProposalState::Cancelled as i32 {
                true
            } else {
                false
            }
        }
    }

    impl SmartContract {
        pub fn new_inner() -> Self {
            SmartContract {
                name: "CreatedByContract".into(),
                consume_user_energy_percent: 100,
                origin_energy_limit: 0,
                ..Default::default()
            }
        }
    }

    impl ResourceDelegation {
        #[inline]
        pub fn is_empty(&self) -> bool {
            self.amount_for_energy == 0 && self.amount_for_bandwidth == 0
        }

        #[inline]
        pub fn amount_for_resource(&self, res: ResourceCode) -> i64 {
            match res {
                ResourceCode::Bandwidth => self.amount_for_bandwidth,
                ResourceCode::Energy => self.amount_for_energy,
            }
        }
        #[inline]
        pub fn expiration_timestamp_for_resource(&self, res: ResourceCode) -> i64 {
            match res {
                ResourceCode::Bandwidth => self.expiration_timestamp_for_bandwidth,
                ResourceCode::Energy => self.expiration_timestamp_for_energy,
            }
        }

        #[inline]
        pub fn reset_resource(&mut self, res: ResourceCode) {
            match res {
                ResourceCode::Bandwidth => {
                    self.amount_for_bandwidth = 0;
                    self.expiration_timestamp_for_bandwidth = 0;
                }
                ResourceCode::Energy => {
                    self.amount_for_energy = 0;
                    self.expiration_timestamp_for_energy = 0;
                }
            }
        }
    }
}
