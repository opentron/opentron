use juniper::FieldResult;

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
    fn block(ctx: &Context, id: Option<String>, num: Option<i32>) -> Option<Block> {
        ctx.get_block(id, num)
    }

    /// Get a transaction
    fn transaction(ctx: &Context, id: String) -> FieldResult<Transaction> {
        ctx.get_transaction(id)
    }
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub(crate) type Schema =
    juniper::RootNode<'static, Query, juniper::EmptyMutation<Context>, juniper::EmptySubscription<Context>>;
