use chrono::{DateTime, TimeZone, Utc};
use juniper::meta::MetaType;
use juniper::FieldResult;
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
/// A transaction of blockchain.
struct Transaction {
    /// Transaction hash.
    id: String,
    /// Signature of the transaction,
    signatures: Vec<String>,
    // raw: String,
    // result: String,
}

#[derive(juniper::GraphQLObject)]
/// A block, on the block chain.
pub struct Block {
    /// Block hash.
    id: String,
    /// Block number.
    number: i32,
    /// Block timestamp.
    timestamp: DateTime<Utc>,
    /// Parent hash.
    parent_hash: String,
    /// Hash of all the hashes of all the transactions.
    merkle_root_hash: String,
    /// Block version.
    version: i32,
    /// Witness address of the block.
    witness: String,
    /// Signature of the witness.
    witness_signature: String,
    /// The transactions in this block.
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
    /// Running code version.
    code_version: String,
    /// Number of currently running compactions.
    num_running_compactions: i32,
    /// Number of currently running flushes.
    num_running_flushes: i32,
    /// Number of immutable memtables that have not yet been flushed.
    num_immutable_mem_table: i32,
    /// If write has been stopped.
    is_write_stopped: bool,
    /// Total size (bytes) of all SST files belong to the latest LSM tree.
    total_size: i32,
}

pub(crate) struct Query;

#[juniper::graphql_object(Context = Context)]
impl Query {
    /// Current API version
    fn api_version() -> &'static str {
        "1.0"
    }

    /// Current Node info
    fn node_info(ctx: &Context) -> NodeInfo {
        NodeInfo {
            code_version: "0.1.0".to_owned(),
            num_running_compactions: ctx.app.db.get_db_property("rocksdb.num-running-compactions") as _,
            num_running_flushes: ctx.app.db.get_db_property("rocksdb.num-running-flushes") as _,
            num_immutable_mem_table: ctx.app.db.get_accumulated_db_property("rocksdb.num-immutable-mem-table") as _,
            is_write_stopped: ctx.app.db.get_accumulated_db_property("rocksdb.is-write-stopped") > 0,
            total_size: ctx.app.db.get_accumulated_db_property("rocksdb.live-sst-files-size") as _,
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

        let header = &blk.header;
        let raw_header = header.raw.raw_data.as_ref().unwrap();
        Some(Block {
            id: hex::encode(blk.hash().as_bytes()),
            number: blk.number() as _,
            timestamp: Utc.timestamp(raw_header.timestamp / 1_000, 0),
            witness: Address::try_from(&raw_header.witness_address)
                .map(|addr| addr.to_string())
                .unwrap_or_else(|_| String::from_utf8(raw_header.witness_address.clone()).unwrap()),
            parent_hash: hex::encode(&raw_header.parent_hash),
            merkle_root_hash: hex::encode(&raw_header.merkle_root_hash),
            version: raw_header.version,
            witness_signature: hex::encode(&header.raw.witness_signature),
            transactions: vec![],
        })
    }

    /// Get a transaction
    fn transaction(ctx: &Context, id: String) -> FieldResult<Transaction> {
        Err("unimplemented".into())
    }
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub(crate) type Schema =
    juniper::RootNode<'static, Query, juniper::EmptyMutation<Context>, juniper::EmptySubscription<Context>>;
