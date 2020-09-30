
use async_graphql::SimpleObject;

/// Running node info.
#[derive(SimpleObject)]
pub struct NodeInfo {
    /// Running code version.
    pub code_version: &'static str,
    /// Is node syncing.
    pub syncing: bool,
    /// Number of active(outgoing) connections.
    pub num_active_connections: u32,
    /// Number of passive(incoming) connections.
    pub num_passive_connections: u32,
    /// Number of currently running compactions.
    pub num_running_compactions: i32,
    /// Number of currently running flushes.
    pub num_running_flushes: i32,
    /// Number of immutable memtables that have not yet been flushed.
    pub num_immutable_mem_table: i32,
    /// If write has been stopped.
    pub is_write_stopped: bool,
    /// Total size (bytes) of all SST files belong to the latest LSM tree.
    pub total_size: i64,
}
