pub mod common {
    use byteorder::{ByteOrder, BE};

    include!(concat!(env!("OUT_DIR"), "/proto.common.rs"));

    impl From<Vec<u8>> for BlockId {
        fn from(block_hash: Vec<u8>) -> Self {
            assert_eq!(block_hash.len(), 32);
            let block_number = BE::read_u64(&block_hash[..8]);
            BlockId {
                hash: block_hash,
                number: block_number as i64
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

pub mod chain {
    include!(concat!(env!("OUT_DIR"), "/proto.chain.rs"));

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

pub mod state;
