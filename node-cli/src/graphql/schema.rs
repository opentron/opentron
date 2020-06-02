//! A schema consists of two types: a query object and a mutation object.

use chrono::{Duration, Utc};
use juniper::FieldResult;

use super::contract::{Contract, TransferContract};
use super::model::{Block, Context, NodeInfo, RawTransaction, Transaction, UnsignedTransaction};

pub(crate) struct Query;

#[juniper::graphql_object(Context = Context)]
impl Query {
    /// Current API version
    fn api_version() -> &'static str {
        "1.0"
    }

    /// Current Node info
    fn node_info(ctx: &Context) -> NodeInfo {
        ctx.get_node_info()
    }

    /// Get a block
    #[graphql(arguments(id(description = "hash of the block"), num(description = "block height")))]
    fn block(ctx: &Context, id: Option<String>, num: Option<i32>) -> FieldResult<Block> {
        ctx.get_block(id, num)
    }

    /// Get a transaction
    #[graphql(arguments(id(description = "transaction hash")))]
    fn transaction(ctx: &Context, id: String) -> FieldResult<Transaction> {
        ctx.get_transaction(id)
    }
}

#[derive(juniper::GraphQLInputObject)]
struct ContractOptions {
    memo: Option<String>,
    permission_id: Option<i32>,
    fee_limit: Option<i32>,
}

pub(crate) struct Mutation;

#[juniper::graphql_object(Context = Context)]
impl Mutation {
    fn transfer(
        ctx: &Context,
        owner: String,
        to: String,
        amount: f64,
        mut option: Option<ContractOptions>,
    ) -> FieldResult<UnsignedTransaction> {
        let contract = TransferContract {
            owner_address: owner,
            to_address: to,
            amount,
        };

        let ref_block_id = ctx.app.db.highest_block()?.block_id();
        let memo = option.as_mut().and_then(|opt| opt.memo.take());
        let permission_id = option
            .as_mut()
            .and_then(|opt| opt.permission_id.take())
            .unwrap_or_default();
        let fee_limit = option.as_mut().and_then(|opt| opt.fee_limit.take()).unwrap_or_default();

        let raw_txn = RawTransaction {
            contract: Contract::TransferContract(contract),
            timestamp: Some(Utc::now()),
            expiration: Utc::now() + Duration::minutes(10),
            ref_block_bytes: hex::encode(&ref_block_id.hash[6..8]),
            ref_block_hash: hex::encode(&ref_block_id.hash[8..16]),
            permission_id,
            fee_limit,
            memo,
        };

        Ok(UnsignedTransaction {
            id: Default::default(),
            inner: raw_txn,
        })
    }
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub(crate) type Schema = juniper::RootNode<'static, Query, Mutation, juniper::EmptySubscription<Context>>;
