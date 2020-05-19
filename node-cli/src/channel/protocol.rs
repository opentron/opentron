//! The channel protocol.

use bytes::{Buf, BufMut, BytesMut};
use prost::Message;
use proto2::chain::Block;
use proto2::channel::{
    inventory::Type as InventoryType, BlockInventory, ChainInventory, HandshakeDisconnect, HandshakeHello, Inventory,
    Transactions,
};
use std::convert::TryFrom;
use std::io::{self, Cursor};
use tokio::prelude::*;
use tokio_util::codec::{Decoder, Encoder, Framed, FramedRead, FramedWrite};

/// Channel message variations.
pub enum ChannelMessage {
    Block(Block),
    Transactions(Transactions),

    // original: Inventory
    // type=TRX
    TransactionInventory(Inventory),
    // type=BLOCK
    BlockInventory(Inventory),

    // original: FetchInventoryData
    // type=TRX
    FetchTransactionInventory(Inventory),
    // type=BLOCK
    FetchBlockInventory(Inventory),

    SyncBlockchain(BlockInventory),
    BlockchainInventory(ChainInventory),

    HandshakeHello(HandshakeHello),
    HandshakeDisconnect(HandshakeDisconnect),

    Ping,
    Pong,
}

impl ChannelMessage {
    pub fn type_code(&self) -> u8 {
        use ChannelMessage::*;

        match *self {
            Block(_) => 0x02,
            Transactions(_) => 0x03,
            TransactionInventory(_) => 0x06,
            BlockInventory(_) => 0x06,
            FetchTransactionInventory(_) => 0x07,
            FetchBlockInventory(_) => 0x07,
            SyncBlockchain(_) => 0x08,
            BlockchainInventory(_) => 0x09,

            HandshakeHello(_) => 0x20,
            HandshakeDisconnect(_) => 0x21,

            Ping => 0x22,
            Pong => 0x23,
        }
    }

    pub fn encode_to<T: BufMut>(&self, dst: &mut T) -> Result<(), io::Error> {
        use ChannelMessage::*;

        dst.put_u8(self.type_code());
        let ret = match *self {
            Ping | Pong => {
                dst.put_u8(0xC0);
                Ok(())
            }
            Block(ref block) => block.encode(dst),
            Transactions(ref trxs) => trxs.encode(dst),
            TransactionInventory(ref inv) |
            BlockInventory(ref inv) |
            FetchTransactionInventory(ref inv) |
            FetchBlockInventory(ref inv) => inv.encode(dst),
            SyncBlockchain(ref block_inv) => block_inv.encode(dst),
            BlockchainInventory(ref chain_inv) => chain_inv.encode(dst),
            HandshakeHello(ref hello) => hello.encode(dst),
            HandshakeDisconnect(ref disconnect) => disconnect.encode(dst),
        };
        ret.map_err(From::from)
    }

    pub fn encoded_len(&self) -> usize {
        use ChannelMessage::*;

        let pb_len = match *self {
            Ping | Pong => 1,
            Block(ref block) => block.encoded_len(),
            Transactions(ref trxs) => trxs.encoded_len(),
            TransactionInventory(ref inv) |
            BlockInventory(ref inv) |
            FetchTransactionInventory(ref inv) |
            FetchBlockInventory(ref inv) => inv.encoded_len(),
            SyncBlockchain(ref block_inv) => block_inv.encoded_len(),
            BlockchainInventory(ref chain_inv) => chain_inv.encoded_len(),
            HandshakeHello(ref hello) => hello.encoded_len(),
            HandshakeDisconnect(ref disconnect) => disconnect.encoded_len(),
        };
        pb_len + 1
    }
}

impl From<HandshakeHello> for ChannelMessage {
    fn from(inner: HandshakeHello) -> ChannelMessage {
        ChannelMessage::HandshakeHello(inner)
    }
}

impl From<HandshakeDisconnect> for ChannelMessage {
    fn from(inner: HandshakeDisconnect) -> ChannelMessage {
        ChannelMessage::HandshakeDisconnect(inner)
    }
}

impl ::std::fmt::Debug for ChannelMessage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        use ChannelMessage::*;

        match *self {
            Ping => write!(f, "Ping"),
            Pong => write!(f, "Pong"),
            Block(ref block) => write!(
                f,
                "Block(number={}, |trxs|={})",
                block.block_header.as_ref().unwrap().raw_data.as_ref().unwrap().number,
                block.transactions.len()
            ),
            Transactions(ref trxs) => write!(f, "Transactions(|trxs|={})", trxs.transactions.len()),
            TransactionInventory(ref inv) => write!(f, "TransactionInventory(|ids|={})", inv.ids.len()),
            BlockInventory(ref inv) => write!(f, "BlockInventory(|ids|={})", inv.ids.len()),
            FetchTransactionInventory(ref inv) => write!(f, "FetchTransactionInventory(|ids|={})", inv.ids.len()),
            FetchBlockInventory(ref inv) => write!(f, "FetchBlockInventory(|ids|={})", inv.ids.len()),
            SyncBlockchain(ref block_inv) => write!(f, "SyncBlockchain(|ids|={})", block_inv.ids.len()),
            BlockchainInventory(ref chain_inv) => write!(
                f,
                "BlockchainInventory(|ids|={}, remain_num={})",
                chain_inv.ids.len(),
                chain_inv.remain_num
            ),
            HandshakeHello(ref hello) => write!(
                f,
                "HandshakeHello(from=\"{}...{}\", version={}, genesis={:?}, solid={}, head={}, timestamp={})",
                hex::encode(&hello.from.as_ref().unwrap().node_id[..4]),
                hex::encode(&hello.from.as_ref().unwrap().node_id[60..]),
                hello.version,
                hex::encode(&hello.genesis_block_id.as_ref().unwrap().hash),
                hello.solid_block_id.as_ref().unwrap().number,
                hello.head_block_id.as_ref().unwrap().number,
                hello.timestamp,
            ),
            HandshakeDisconnect(ref disconnect) => write!(f, "HandshakeDisconnect(reason={})", disconnect.reason),
        }
    }
}

impl TryFrom<&[u8]> for ChannelMessage {
    type Error = io::Error;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        if buf.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid data"));
        }

        match buf[0] {
            0x02 => Ok(ChannelMessage::Block(Message::decode(&buf[1..])?)),
            0x03 => Ok(ChannelMessage::Transactions(Message::decode(&buf[1..])?)),
            0x06 => {
                let inv = Inventory::decode(&buf[1..])?;
                if inv.r#type == InventoryType::Block as i32 {
                    Ok(ChannelMessage::BlockInventory(inv))
                } else {
                    Ok(ChannelMessage::TransactionInventory(inv))
                }
            }
            0x07 => {
                let inv = Inventory::decode(&buf[1..])?;
                if inv.r#type == InventoryType::Block as i32 {
                    Ok(ChannelMessage::FetchBlockInventory(inv))
                } else {
                    Ok(ChannelMessage::FetchTransactionInventory(inv))
                }
            }
            0x08 => Ok(ChannelMessage::SyncBlockchain(Message::decode(&buf[1..])?)),
            0x09 => Ok(ChannelMessage::BlockchainInventory(Message::decode(&buf[1..])?)),

            0x20 => Ok(ChannelMessage::HandshakeHello(Message::decode(&buf[1..])?)),
            0x21 => Ok(ChannelMessage::HandshakeDisconnect(Message::decode(&buf[1..])?)),
            0x22 => {
                assert!(buf[1] == 0xC0);
                Ok(ChannelMessage::Ping)
            }
            0x23 => {
                assert!(buf[1] == 0xC0);
                Ok(ChannelMessage::Pong)
            }
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "invalid data")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DecodeState {
    Head,
    Data(usize),
}

pub struct ChannelMessageCodec {
    // Read state
    state: DecodeState,
}

impl ChannelMessageCodec {
    pub fn new() -> Self {
        Self {
            state: DecodeState::Head,
        }
    }

    pub fn new_read<T>(upstream: T) -> FramedRead<T, ChannelMessageCodec>
    where
        T: AsyncRead,
    {
        FramedRead::new(upstream, Self::new())
    }

    pub fn new_write<T>(inner: T) -> FramedWrite<T, ChannelMessageCodec>
    where
        T: AsyncWrite,
    {
        FramedWrite::new(inner, Self::new())
    }

    pub fn new_framed<T>(inner: T) -> Framed<T, ChannelMessageCodec>
    where
        T: AsyncRead + AsyncWrite,
    {
        Framed::new(inner, Self::new())
    }

    fn decode_head(&mut self, src: &mut BytesMut) -> io::Result<Option<usize>> {
        let mut len = 0_usize;
        let mut num_skip = 0_usize;

        {
            let mut src = Cursor::new(&mut *src);
            for i in 0..9_u8 {
                if src.remaining() == 0 {
                    // Not enough data
                    return Ok(None);
                }

                let b = src.get_u8();
                len += ((b & 0x7f) as usize) << (7 * i);
                if b >> 7 == 0 {
                    num_skip = (i + 1) as usize;
                    break;
                }
                // overflows u32
                if i > 5 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "provided length would overflow u32",
                    ));
                }
            }
        }

        src.advance(num_skip);
        src.reserve(len);

        return Ok(Some(len));
    }

    fn decode_data(&self, n: usize, src: &mut BytesMut) -> io::Result<Option<ChannelMessage>> {
        // At this point, the buffer has already had the required capacity
        // reserved. All there is to do is read.
        if src.len() < n {
            return Ok(None);
        }

        Ok(Some(ChannelMessage::try_from(&*src.split_to(n))?))
    }
}

impl Decoder for ChannelMessageCodec {
    type Item = ChannelMessage;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<ChannelMessage>> {
        let n = match self.state {
            DecodeState::Head => match self.decode_head(src)? {
                Some(n) => {
                    self.state = DecodeState::Data(n);
                    n
                }
                None => return Ok(None),
            },
            DecodeState::Data(n) => n,
        };

        match self.decode_data(n, src)? {
            Some(data) => {
                // Update the decode state
                self.state = DecodeState::Head;

                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.decode(buf)? {
            Some(frame) => Ok(Some(frame)),
            None => {
                if buf.is_empty() {
                    Ok(None)
                } else {
                    // println!("remain => {:?}", hex::encode(buf));
                    Err(io::Error::new(io::ErrorKind::Other, "bytes remaining on stream").into())
                }
            }
        }
    }
}

impl Encoder<ChannelMessage> for ChannelMessageCodec {
    type Error = io::Error;

    fn encode(&mut self, data: ChannelMessage, dst: &mut BytesMut) -> Result<(), io::Error> {
        const ESTIMATED_PACKET_PREFIX_LEN: usize = 5;

        let n = data.encoded_len();

        // Reserve capacity in the destination buffer to fit the frame and length field
        dst.reserve(ESTIMATED_PACKET_PREFIX_LEN + n);

        prost::encode_length_delimiter(n, dst)?;

        // Write the frame to the buffer
        data.encode_to(dst)?;

        Ok(())
    }
}

impl Default for ChannelMessageCodec {
    fn default() -> Self {
        Self::new()
    }
}
