pub use std::net::Ipv4Addr;
use std::fmt;

use super::UdpDatagram;


#[derive(Clone)]
pub struct Ipv4Packet {
    /// When false, if a router can't transfer the packet 
    /// in one fragment, it is discarded.
    pub allow_fragmentation: bool,
    /// If true this packet is a segment, except for the
    /// last packet of a bundle.
    pub is_fragment: bool,
    /// Number of fragment of a packet.
    pub fragment_identifier: u16,
    /// Position of the packet from the first one of a 
    /// fragment bundle.
    pub fragment_offset: u16,
    /// Decremented by each traversed router, when 0 the
    /// packet is discarded and an ICMP packet is sent for
    /// notification.
    pub ttl: u8,
    /// Source IP address.
    pub src: Ipv4Addr,
    /// Destination IP address.
    pub dst: Ipv4Addr,
    /// Payload.
    pub payload: Ipv4Payload,
}

impl Ipv4Packet {

    pub fn new(src: Ipv4Addr, dst: Ipv4Addr, payload: Ipv4Payload) -> Self {
        Self {
            allow_fragmentation: true,
            is_fragment: false,
            fragment_identifier: 0,
            fragment_offset: 0,
            ttl: 32,
            src,
            dst,
            payload
        }
    }

}


#[derive(Debug, Clone)]
pub enum Ipv4Payload {
    Custom(Vec<u8>),
    Udp(UdpDatagram),
}


impl fmt::Debug for Ipv4Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ipv4Packet")
            .field("allow_frag", &self.allow_fragmentation)
            .field("is_frag", &self.is_fragment)
            .field("frag_id", &self.fragment_identifier)
            .field("frag_off", &self.fragment_offset)
            .field("ttl", &self.ttl)
            .field("src", &format_args!("{}", self.src))
            .field("dst", &format_args!("{}", self.dst))
            .field("payload", &self.payload)
            .finish()
    }
}
