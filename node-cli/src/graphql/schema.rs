use chrono::{DateTime, TimeZone, Utc};
use juniper::meta::MetaType;
use juniper::{GraphQLType, Registry, ScalarValue};
use keys::Address;
use primitives::H256;
use std::convert::TryFrom;
use std::sync::Arc;

use crate::context::AppContext;

#[derive(juniper::GraphQLObject)]
struct RawTransaction {
    expiration: DateTime<Utc>,
    timestamp: DateTime<Utc>,
    ref_block_byte: String,
    ref_block_hash: String,
    // le 1000_000_000, i32 is ok
    fee_limit: i32,
    data: String,
}

#[derive(juniper::GraphQLObject)]
struct Transaction {
    id: String,
    signatures: Vec<String>,
    raw: String,
    result: String,
}

#[derive(juniper::GraphQLObject)]
pub struct Block {
    id: String,
    number: i32,
    timestamp: DateTime<Utc>,
    witness: String,
    parent_hash: String,
    version: i32,
    witness_signature: String,
    transactions: Vec<Transaction>,
}

#[derive(Clone)]
pub(crate) struct Context {
    pub app: Arc<AppContext>,
}

impl<S> GraphQLType<S> for Context
where
    S: ScalarValue,
{
    type Context = Self;
    type TypeInfo = ();

    fn name(_: &()) -> Option<&str> {
        Some("_Context")
    }

    fn meta<'r>(_: &(), registry: &mut Registry<'r, S>) -> MetaType<'r, S>
    where
        S: 'r,
    {
        registry.build_object_type::<Self>(&(), &[]).into_meta()
    }
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

#[derive(juniper::GraphQLObject)]
/// Misc node info
struct NodeInfo {
    /// Running code version
    code_version: String,
}

pub(crate) struct Query;

#[juniper::graphql_object(Context = Context)]
impl Query {
    /// Current API version
    fn api_version() -> &'static str {
        "1.0"
    }

    /// Current Node info
    fn node_info() -> NodeInfo {
        NodeInfo {
            code_version: "0.1.0".to_owned(),
        }
    }

    /// Get a block
    #[graphql(arguments(id(description = "hash of the block"), num(description = "block height")))]
    fn block(ctx: &Context, id: Option<String>, num: Option<i32>) -> Option<Block> {
        let blk = match (id, num) {
            (Some(_), Some(_)) => return None,
            (Some(id), _) => {
                let block_id = H256::from_slice(&hex::decode(&id).ok()?);
                ctx.app.db.get_block_by_hash(&block_id)?
            }
            (_, Some(num)) => ctx.app.db.get_block_by_number(num as _)?,
            (None, None) => ctx.app.db.highest_block()?,
        };

        let hdr = &blk.header;
        Some(Block {
            id: hex::encode(blk.hash().as_bytes()),
            number: blk.number() as _,
            timestamp: Utc.timestamp(hdr.raw.raw_data.as_ref().unwrap().timestamp / 1_000, 0),
            witness: Address::try_from(&hdr.raw.raw_data.as_ref().unwrap().witness_address)
                .unwrap()
                .to_string(),
            parent_hash: hex::encode(&hdr.raw.raw_data.as_ref().unwrap().parent_hash),
            version: hdr.raw.raw_data.as_ref().unwrap().version,
            witness_signature: hex::encode(&hdr.raw.witness_signature),
            transactions: vec![],
        })
    }
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub(crate) type Schema =
    juniper::RootNode<'static, Query, juniper::EmptyMutation<Context>, juniper::EmptySubscription<Context>>;
