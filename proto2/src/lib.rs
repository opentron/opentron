pub mod common {
    include!(concat!(env!("OUT_DIR"), "/proto.common.rs"));

    impl ::std::fmt::Display for BlockId {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("BlockId")
                .field("number", &self.number)
                .field("hash", &hex::encode(&self.hash))
                .finish()
        }
    }
}

pub mod chain {
    include!(concat!(env!("OUT_DIR"), "/proto.chain.rs"));

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
}

pub mod discovery {
    include!(concat!(env!("OUT_DIR"), "/proto.discovery.rs"));
}

pub mod channel {
    include!(concat!(env!("OUT_DIR"), "/proto.channel.rs"));

    pub use crate::chain::{Block, Transaction};

    impl ::std::fmt::Display for ReasonCode {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }
}

pub mod contract {
    include!(concat!(env!("OUT_DIR"), "/proto.contract.rs"));
}
