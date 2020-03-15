pub mod common {
    include!(concat!(env!("OUT_DIR"), "/proto.common.rs"));
}

pub mod chain {
    include!(concat!(env!("OUT_DIR"), "/proto.chain.rs"));
}

pub mod discover {
    include!(concat!(env!("OUT_DIR"), "/proto.discover.rs"));
}

pub mod channel {
    include!(concat!(env!("OUT_DIR"), "/proto.channel.rs"));

    pub use crate::chain::{Block, Transaction};
}

pub mod contract {
    include!(concat!(env!("OUT_DIR"), "/proto.contract.rs"));
}
