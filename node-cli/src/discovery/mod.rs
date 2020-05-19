//! The UDP discovery protocol.

use bytes::{BufMut, BytesMut};
use futures::sink::Sink;
use prost::Message;
use proto2::common::Endpoint;
use proto2::discovery::{FindPeers, Peers, Ping, Pong};
use std::convert::TryFrom;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::net::UdpSocket;
use tokio::stream::Stream;
use futures::ready;

pub mod server;

pub enum DiscoveryMessage {
    Ping(Ping),
    Pong(Pong),
    FindPeers(FindPeers),
    Peers(Peers),
}

impl DiscoveryMessage {
    pub fn type_code(&self) -> u8 {
        use DiscoveryMessage::*;

        match *self {
            Ping(_) => 0x01,
            Pong(_) => 0x02,
            FindPeers(_) => 0x03,
            Peers(_) => 0x04,
        }
    }

    pub fn encode_to<T: BufMut>(&self, dst: &mut T) -> Result<(), io::Error> {
        use DiscoveryMessage::*;

        dst.put_u8(self.type_code());
        let ret = match *self {
            Ping(ref ping) => ping.encode(dst),
            Pong(ref pong) => pong.encode(dst),
            FindPeers(ref find) => find.encode(dst),
            Peers(ref peers) => peers.encode(dst),
        };
        ret.map_err(From::from)
    }
}

impl TryFrom<&[u8]> for DiscoveryMessage {
    type Error = io::Error;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        match buf.first() {
            Some(0x01) => Ok(DiscoveryMessage::Ping(Ping::decode(&buf[1..])?)),
            Some(0x02) => Ok(DiscoveryMessage::Pong(Pong::decode(&buf[1..])?)),
            Some(0x03) => Ok(DiscoveryMessage::FindPeers(FindPeers::decode(&buf[1..])?)),
            Some(0x04) => Ok(DiscoveryMessage::Peers(Peers::decode(&buf[1..])?)),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "invalid packet data")),
        }
    }
}

impl From<Ping> for DiscoveryMessage {
    fn from(ping: Ping) -> Self {
        DiscoveryMessage::Ping(ping)
    }
}

impl From<Pong> for DiscoveryMessage {
    fn from(pong: Pong) -> Self {
        DiscoveryMessage::Pong(pong)
    }
}

impl From<FindPeers> for DiscoveryMessage {
    fn from(find: FindPeers) -> Self {
        DiscoveryMessage::FindPeers(find)
    }
}

impl From<Peers> for DiscoveryMessage {
    fn from(peers: Peers) -> Self {
        DiscoveryMessage::Peers(peers)
    }
}

impl ::std::fmt::Debug for DiscoveryMessage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        use DiscoveryMessage::*;

        match *self {
            Ping(ref ping) => f
                .debug_struct("Ping")
                .field("version", &ping.version)
                .field("from", &format_endpoint(ping.from.as_ref().unwrap()))
                .field("to", &format_endpoint(ping.to.as_ref().unwrap()))
                .finish(),
            Pong(ref pong) => f
                .debug_struct("Pong")
                .field("echo", &pong.echo_version)
                .field("from", &format_endpoint(pong.from.as_ref().unwrap()))
                .finish(),
            FindPeers(ref find) => f
                .debug_struct("FindPeers")
                .field("target", &format_node_id(&find.target_id))
                .field("from", &format_endpoint(find.from.as_ref().unwrap()))
                .finish(),
            Peers(ref peers) => f
                .debug_struct("Peers")
                .field("from", &format_endpoint(peers.from.as_ref().unwrap()))
                .field("peers", &format_peers(&peers.peers))
                .finish(),
        }
    }
}

fn format_node_id(node_id: &[u8]) -> String {
    if node_id.len() != 64 {
        eprintln!("!! error, wrong node id={:?}", hex::encode(&node_id));
    }
    format!("{}...", hex::encode(&node_id[..8]))
}

fn format_endpoint(ep: &Endpoint) -> String {
    format!("{}:{}", ep.address, ep.port)
}

fn format_peers(eps: &[Endpoint]) -> Vec<String> {
    eps.iter()
        .map(|ep| format!("{}:{}", ep.address, ep.port))
        .collect::<Vec<_>>()
}

pub struct DiscoveryMessageTransport {
    socket: UdpSocket,
    flushed: bool,
    // rd: BytesMut,
    wr: BytesMut,
    out_addr: SocketAddr,
}

impl DiscoveryMessageTransport {
    pub fn new(socket: UdpSocket) -> Self {
        // MTU: 1500
        // The recommended default maximum packet size is 1350 bytes for IPv6 and 1370 bytes for IPv4
        // ref: https://gist.github.com/jj1bdx/1adac3e305d0fb6dee90dd5b909513ed
        // const INITIAL_RD_CAPACITY: usize = 1500;
        const INITIAL_WR_CAPACITY: usize = 1500;
        DiscoveryMessageTransport {
            socket,
            flushed: true,
            // rd: BytesMut::with_capacity(INITIAL_RD_CAPACITY),
            wr: BytesMut::with_capacity(INITIAL_WR_CAPACITY),
            out_addr: "0.0.0.0:0".parse().expect("won't fail; qed"),
        }
    }
}

impl Stream for DiscoveryMessageTransport {
    type Item = Result<(DiscoveryMessage, SocketAddr), io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let pin = self.get_mut();

        // Note: poll_recv_from is an undocument fn
        let mut buf = [0u8; 1500];
        let (n, addr) = ready!(Pin::new(&mut pin.socket).poll_recv_from(cx, &mut buf[..]))?;
        {
            let buf = &buf[..n];
            Poll::Ready(Some(DiscoveryMessage::try_from(buf).map(|frame| (frame, addr))))
        }
    }
}

impl Sink<(DiscoveryMessage, SocketAddr)> for DiscoveryMessageTransport {
    type Error = io::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if !self.flushed {
            match self.poll_flush(cx)? {
                Poll::Ready(()) => {}
                Poll::Pending => return Poll::Pending,
            }
        }
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: (DiscoveryMessage, SocketAddr)) -> Result<(), Self::Error> {
        let (frame, out_addr) = item;

        let pin = self.get_mut();
        frame.encode_to(&mut pin.wr)?;
        pin.out_addr = out_addr;
        pin.flushed = false;

        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.flushed {
            return Poll::Ready(Ok(()));
        }

        let Self {
            ref mut socket,
            ref mut out_addr,
            ref mut wr,
            ..
        } = *self;

        let n = ready!(socket.poll_send_to(cx, &wr, &out_addr))?;

        let wrote_all = n == self.wr.len();
        self.wr.clear();
        self.flushed = true;

        let res = if wrote_all {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to write entire datagram to socket",
            ))
        };

        Poll::Ready(res)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.poll_flush(cx))?;
        Poll::Ready(Ok(()))
    }
}
