use std::fmt;

use super::{
    Ipv4Packet, ArpIpv4Packet, Ipv4Addr,
    Ipv6Addr,
};


#[derive(Clone)]
pub struct EthFrame {
    pub src: MacAddr,
    pub dst: MacAddr,
    pub payload: EthPayload,
}

#[derive(Debug, Clone)]
pub enum EthPayload {
    Custom(Vec<u8>),
    Vlan {
        /// VLan identifier.
        vlan_id: u16,
        /// The inner packet can't be a Vlan variant.
        inner: Box<EthPayload>,
    },
    Arp(Box<ArpIpv4Packet>),
    Ipv4(Box<Ipv4Packet>),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacAddr(pub [u8; 6]);

impl MacAddr {

    pub const BROADCAST: Self = Self([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    
    pub const ZERO: Self = Self([0, 0, 0, 0, 0, 0]);

    /// Create a new MAC address based on a multicast IPv4 address.
    pub const fn from_multicast_ipv4(ip: Ipv4Addr) -> Self {
        let o = ip.octets();
        Self([0x01, 0x00, 0x5E, o[1] & 0x7F, o[2], o[3]])
    }

    /// Create a new MAC address based on a multicast IPv6 address.
    pub const fn from_multicast_ipv6(ip: Ipv6Addr) -> Self {
        let o = ip.octets();
        Self([0x33, 0x33, o[12], o[13], o[14], o[15]])
    }

    pub const fn is_unicast(self) -> bool {
        self.0[0] & 0b01 == 0
    }

    pub const fn is_multicast(self) -> bool {
        !self.is_unicast()
    }

    pub const fn is_unique(self) -> bool {
        self.0[0] & 0b10 == 0
    }

    pub const fn is_universal(self) -> bool {
        !self.is_unique()
    }

}

impl fmt::Debug for EthFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EthFrame")
            .field("src", &format_args!("{}", self.src))
            .field("dst", &format_args!("{}", self.dst))
            .field("payload", &self.payload)
            .finish()
    }
}

impl fmt::Display for MacAddr {
    fn fmt(&self, f_: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c, d, e, f] = self.0;
        f_.write_fmt(format_args!("{a:02X}:{b:02X}:{c:02X}:{d:02X}:{e:02X}:{f:02X}"))
    }
}
