//! A schema consists of two types: a query object and a mutation object.

use juniper::graphql_value;
use juniper::{FieldError, FieldResult};

use super::model::{Block, Context, NodeInfo, Transaction};

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
    /// Broadcast a transaction with its signatures.
    fn broadcast(_ctx: &Context, raw: String, signatures: Vec<String>) -> FieldResult<Transaction> {
        use chain::IndexedTransaction;
        use prost::Message;
        use proto2::chain::{transaction::Raw as RawTransaction, Transaction};

        let raw = hex::decode(&raw).map_err(|e| {
            FieldError::new(
                "fail to parse raw transaction as hex",
                graphql_value!({
                    "internal_error": (e.to_string())
                }),
            )
        })?;

        let buf = &raw[..];

        let raw_txn = RawTransaction::decode(buf).map_err(|e| {
            FieldError::new(
                "fail to parse raw transaction as protobuf",
                graphql_value!({
                    "internal_error": (e.to_string())
                }),
            )
        })?;

        let txn = Transaction {
            raw_data: Some(raw_txn),
            signatures: signatures
                .iter()
                .map(|sig| hex::decode(sig))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    FieldError::new(
                        "fail to parse signatures",
                        graphql_value!({
                            "internal_error": (e.to_string())
                        }),
                    )
                })?,
            ..Default::default()
        };
        let txn = IndexedTransaction::from_raw(txn);
        // TODO: broadcast
        Ok(txn.into())
    }
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub(crate) type Schema = juniper::RootNode<'static, Query, Mutation, juniper::EmptySubscription<Context>>;
