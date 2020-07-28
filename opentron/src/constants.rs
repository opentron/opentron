pub const MAX_NUM_OF_ACTIVE_WITNESSES: usize = 27;
pub const MAX_NUM_OF_STANDBY_WITNESSES: usize = 127;

// 27 * 70% = 18.9, so a solid block is one verified by 19 witnesses.
pub const SOLID_THRESHOLD_PERCENT: usize = 70;

// 2MiB
pub const MAX_BLOCK_SIZE: usize = 2_000_000;

pub const CURRENT_BLOCK_VERSION: usize = 16;

pub const NUM_OF_SKIPPED_SLOTS_IN_MAINTENANCE: usize = 2;
